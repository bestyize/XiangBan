package xyz.xiaban.server.redis

import io.lettuce.core.Consumer
import io.lettuce.core.RedisCommandExecutionException
import io.lettuce.core.RedisCommandTimeoutException
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
import java.lang.management.ManagementFactory

/**
 * A Redis Stream with one consumer group: jobs survive restarts and spread across instances (PRD 9.1).
 * Each job is acknowledged after its handler returns, whether it succeeded or not; retries are the
 * handler's business.
 */
class StreamQueue(private val redis: Redis, private val stream: String, private val group: String) {

    suspend fun add(fields: Map<String, String>) {
        redis.cmd.xadd(stream, XAddArgs.Builder.maxlen(10_000).approximateTrimming(), fields).await()
    }

    fun start(scope: CoroutineScope, concurrency: Int, handle: suspend (Map<String, String>) -> Unit): Job =
        scope.launch(Dispatchers.IO) {
            val connection = redis.dedicatedConnection()
            try {
                val blocking = connection.async()
                ensureGroup(blocking)
                val consumer = Consumer.from(group, ManagementFactory.getRuntimeMXBean().name)
                val slots = Semaphore(concurrency)
                coroutineScope {
                    while (isActive) {
                        for (message in readBatch(blocking, consumer)) {
                            slots.acquire()
                            launch {
                                try {
                                    handle(message.body)
                                } catch (e: Exception) {
                                    log.warn("{} job {} failed: {}", stream, message.id, e.message)
                                } finally {
                                    slots.release()
                                    runCatching { redis.cmd.xack(stream, group, message.id).await() }
                                }
                            }
                        }
                    }
                }
            } finally {
                connection.close()
            }
        }

    private suspend fun readBatch(blocking: RedisAsyncCommands<String, String>, consumer: Consumer<String>) = try {
        blocking.xreadgroup(consumer, XReadArgs.Builder.block(2000).count(16), XReadArgs.StreamOffset.lastConsumed(stream)).await()
    } catch (_: RedisCommandTimeoutException) {
        emptyList()
    } catch (e: RedisCommandExecutionException) {
        // The group vanishes when Redis is flushed; recreate it and carry on.
        log.warn("Reading {} failed: {}", stream, e.message)
        runCatching { ensureGroup(blocking) }
        delay(1000)
        emptyList()
    }

    private suspend fun ensureGroup(blocking: RedisAsyncCommands<String, String>) {
        try {
            // From "0" so jobs added before the group existed are still delivered.
            blocking.xgroupCreate(XReadArgs.StreamOffset.from(stream, "0"), group, XGroupCreateArgs.Builder.mkstream(true)).await()
        } catch (e: RedisCommandExecutionException) {
            if (e.message?.contains("BUSYGROUP") != true) throw e
        }
    }

    private companion object {
        val log = LoggerFactory.getLogger(StreamQueue::class.java)
    }
}
