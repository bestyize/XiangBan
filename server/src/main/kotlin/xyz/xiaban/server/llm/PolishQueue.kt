package xyz.xiaban.server.llm

import io.lettuce.core.Consumer
import io.lettuce.core.RedisCommandExecutionException
import io.lettuce.core.RedisCommandTimeoutException
import io.lettuce.core.StreamMessage
import io.lettuce.core.XAddArgs
import io.lettuce.core.XGroupCreateArgs
import io.lettuce.core.XReadArgs
import io.lettuce.core.api.async.RedisAsyncCommands
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.future.await
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Semaphore
import org.slf4j.LoggerFactory
import xyz.xiaban.server.redis.Redis
import java.lang.management.ManagementFactory
import java.time.Clock
import java.time.Duration

/** Model calls run off the request path through a Redis Stream (PRD 9.1), so check-in never waits on them. */
class PolishQueue(private val redis: Redis, private val clock: Clock) {

    suspend fun enqueue(checkinId: Long) {
        redis.cmd.setex(statusKey(checkinId), 60, PENDING).await()
        redis.cmd.xadd(
            STREAM,
            XAddArgs.Builder.maxlen(10_000).approximateTrimming(),
            mapOf("checkin" to checkinId.toString(), "at" to clock.millis().toString()),
        ).await()
    }

    suspend fun isPending(checkinId: Long): Boolean = redis.cmd.get(statusKey(checkinId)).await() == PENDING

    suspend fun finish(checkinId: Long) {
        redis.cmd.setex(statusKey(checkinId), 120, DONE).await()
    }

    /** Long-poll helper for GET /api/checkin/today?wait=… */
    suspend fun awaitDone(checkinId: Long, maxWait: Duration) {
        val end = System.nanoTime() + maxWait.toNanos()
        while (isPending(checkinId) && System.nanoTime() < end) delay(120)
    }

    /** Consumes the stream until [scope] is cancelled; [handle] gets the check-in id and enqueue time. */
    fun startWorker(scope: CoroutineScope, handle: suspend (checkinId: Long, enqueuedAt: Long) -> Unit): Job =
        scope.launch(Dispatchers.IO) {
            val connection = redis.dedicatedConnection()
            try {
                val blocking = connection.async()
                ensureGroup(blocking)
                val consumer = Consumer.from(GROUP, ManagementFactory.getRuntimeMXBean().name)
                val slots = Semaphore(8)
                coroutineScope {
                    while (isActive) {
                        for (message in readBatch(blocking, consumer)) {
                            slots.acquire()
                            launch {
                                try {
                                    process(message, handle)
                                } finally {
                                    slots.release()
                                }
                            }
                        }
                    }
                }
            } finally {
                connection.close()
            }
        }

    private suspend fun process(message: StreamMessage<String, String>, handle: suspend (Long, Long) -> Unit) {
        val id = message.body["checkin"]?.toLongOrNull()
        try {
            if (id != null) handle(id, message.body["at"]?.toLongOrNull() ?: 0L)
        } catch (e: Exception) {
            log.warn("Polishing check-in {} failed: {}", id, e.message)
        } finally {
            if (id != null) runCatching { finish(id) }
            runCatching { redis.cmd.xack(STREAM, GROUP, message.id).await() }
        }
    }

    private suspend fun readBatch(blocking: RedisAsyncCommands<String, String>, consumer: Consumer<String>) = try {
        blocking.xreadgroup(consumer, XReadArgs.Builder.block(2000).count(16), XReadArgs.StreamOffset.lastConsumed(STREAM)).await()
    } catch (_: RedisCommandTimeoutException) {
        emptyList()
    } catch (e: RedisCommandExecutionException) {
        log.warn("Reading {} failed: {}", STREAM, e.message)
        delay(1000)
        emptyList()
    }

    private suspend fun ensureGroup(blocking: RedisAsyncCommands<String, String>) {
        try {
            // From "0" so jobs added before the group exists are still delivered; stale ones are skipped by age.
            blocking.xgroupCreate(XReadArgs.StreamOffset.from(STREAM, "0"), GROUP, XGroupCreateArgs.Builder.mkstream(true)).await()
        } catch (e: RedisCommandExecutionException) {
            if (e.message?.contains("BUSYGROUP") != true) throw e
        }
    }

    companion object {
        const val STREAM = "xb:polish"
        private const val GROUP = "polishers"
        private const val PENDING = "pending"
        private const val DONE = "done"
        private val log = LoggerFactory.getLogger(PolishQueue::class.java)

        private fun statusKey(checkinId: Long) = "xb:polish:status:$checkinId"
    }
}
