package xyz.xiaban.server.mail

import io.lettuce.core.Limit
import io.lettuce.core.Range
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.future.await
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import org.slf4j.LoggerFactory
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.redis.StreamQueue
import java.time.Clock

@Serializable
data class MailJob(val kind: String, val to: String, val subject: String, val text: String, val ref: String? = null, val attempt: Int = 1)

/**
 * Outgoing mail through a Redis Stream. A failed send is retried up to [MAX_RETRIES] times with growing
 * delays (parked in a sorted set), and every attempt is written to mail_logs (PRD 5.10).
 */
class MailQueue(private val redis: Redis, private val db: Db, private val mailer: Mailer, private val clock: Clock) {
    private val stream = StreamQueue(redis, "xb:mail", "mailers")
    private val outcomeListeners = mutableListOf<suspend (ref: String, delivered: Boolean) -> Unit>()

    /** Called once per job when it is finally delivered or given up on. */
    fun onOutcome(listener: suspend (ref: String, delivered: Boolean) -> Unit) {
        outcomeListeners += listener
    }

    suspend fun send(job: MailJob) = stream.add(mapOf("job" to AppJson.encodeToString(MailJob.serializer(), job)))

    fun start(scope: CoroutineScope) {
        stream.start(scope, concurrency = 4) { body -> body["job"]?.let { deliver(AppJson.decodeFromString(MailJob.serializer(), it)) } }
        scope.launch {
            while (isActive) {
                runCatching { releaseDueRetries() }.onFailure { log.warn("Moving mail retries failed: {}", it.message) }
                delay(10_000)
            }
        }
    }

    private suspend fun deliver(job: MailJob) {
        val error = runCatching { mailer.send(Mail(job.to, job.subject, job.text)) }.exceptionOrNull()
        db.read { c ->
            c.execute(
                "insert into mail_logs (kind, ref, recipient, attempt, ok, error) values (?, ?, ?, ?, ?, ?)",
                job.kind, job.ref, maskEmail(job.to), job.attempt, error == null, error?.message?.take(500),
            )
        }
        when {
            error == null -> notify(job, delivered = true)
            job.attempt <= MAX_RETRIES -> {
                val next = job.copy(attempt = job.attempt + 1)
                val at = clock.millis() + BACKOFF_SECONDS[job.attempt - 1] * 1000
                redis.cmd.zadd(RETRY_KEY, at.toDouble(), AppJson.encodeToString(MailJob.serializer(), next)).await()
                log.warn("Mail {} attempt {} failed, retrying: {}", job.kind, job.attempt, error.message)
            }
            else -> {
                log.error("Mail {} failed after {} attempts: {}", job.kind, job.attempt, error.message)
                notify(job, delivered = false)
            }
        }
    }

    private suspend fun releaseDueRetries() {
        val due = redis.cmd.zrangebyscore(RETRY_KEY, Range.create(0.0, clock.millis().toDouble()), Limit.create(0, 50)).await()
        for (member in due) {
            if (redis.cmd.zrem(RETRY_KEY, member).await() == 1L) stream.add(mapOf("job" to member))
        }
    }

    private suspend fun notify(job: MailJob, delivered: Boolean) {
        val ref = job.ref ?: return
        outcomeListeners.forEach { runCatching { it(ref, delivered) } }
    }

    private companion object {
        const val RETRY_KEY = "xb:mail:retry"
        const val MAX_RETRIES = 3
        val BACKOFF_SECONDS = longArrayOf(60, 300, 900)
        val log = LoggerFactory.getLogger(MailQueue::class.java)
    }
}
