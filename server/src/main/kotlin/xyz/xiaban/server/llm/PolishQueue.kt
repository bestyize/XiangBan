package xyz.xiaban.server.llm

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.future.await
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.redis.StreamQueue
import java.time.Clock
import java.time.Duration

/** Model calls run off the request path through a Redis Stream (PRD 9.1), so check-in never waits on them. */
class PolishQueue(private val redis: Redis, private val clock: Clock) {
    private val queue = StreamQueue(redis, "xb:polish", "polishers")

    suspend fun enqueue(checkinId: Long) {
        redis.cmd.setex(statusKey(checkinId), 60, PENDING).await()
        queue.add(mapOf("checkin" to checkinId.toString(), "at" to clock.millis().toString()))
    }

    suspend fun isPending(checkinId: Long): Boolean = redis.cmd.get(statusKey(checkinId)).await() == PENDING

    /** Long-poll helper for GET /api/checkin/today?wait=… */
    suspend fun awaitDone(checkinId: Long, maxWait: Duration) {
        val end = System.nanoTime() + maxWait.toNanos()
        while (isPending(checkinId) && System.nanoTime() < end) delay(120)
    }

    /** [handle] gets the check-in id and the enqueue time; the status flips to done whatever happens. */
    fun startWorker(scope: CoroutineScope, handle: suspend (checkinId: Long, enqueuedAt: Long) -> Unit): Job =
        queue.start(scope, concurrency = 8) { body ->
            val id = body["checkin"]?.toLongOrNull() ?: return@start
            try {
                handle(id, body["at"]?.toLongOrNull() ?: 0L)
            } finally {
                redis.cmd.setex(statusKey(id), 120, DONE).await()
            }
        }

    private companion object {
        const val PENDING = "pending"
        const val DONE = "done"

        fun statusKey(checkinId: Long) = "xb:polish:status:$checkinId"
    }
}
