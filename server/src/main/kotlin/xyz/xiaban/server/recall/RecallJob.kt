package xyz.xiaban.server.recall

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import org.slf4j.LoggerFactory
import xyz.xiaban.server.copy.CopyMode
import xyz.xiaban.server.copy.CopySelector
import xyz.xiaban.server.copy.Daypart
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.query
import xyz.xiaban.server.mail.MailJob
import xyz.xiaban.server.mail.MailQueue
import xyz.xiaban.server.users.UserRepository
import java.time.Clock
import java.time.Duration
import java.time.LocalDateTime
import java.util.Base64
import javax.crypto.Mac
import javax.crypto.spec.SecretKeySpec

/**
 * Opt-in "come back" emails (PRD 5.9): after 3 quiet days, then after 7, and never more than twice per
 * lapse; a check-in resets the count. Sent only between 10:00 and 20:00 in the user's time zone.
 */
class RecallJob(
    private val db: Db,
    private val users: UserRepository,
    private val selector: CopySelector,
    private val mail: MailQueue,
    private val links: UnsubscribeLinks,
    private val publicBaseUrl: String,
    private val clock: Clock,
) {
    fun start(scope: CoroutineScope) = scope.launch {
        delay(60_000)
        while (isActive) {
            runCatching { run() }.onFailure { log.warn("Recall run failed: {}", it.message) }
            delay(30 * 60_000)
        }
    }

    suspend fun run(): Int {
        val now = clock.instant()
        val candidates = db.read { c ->
            c.query(
                """
                select u.id, u.recall_count
                from users u join checkins ch on ch.user_id = u.id and ch.revoked_at is null
                where u.recall_opt_in and u.email_verified_at is not null and u.recall_count < 2
                  and (u.last_recall_at is null or u.last_recall_at < ?)
                group by u.id
                having max(ch.checked_at) < case when u.recall_count = 0 then ? else ? end
                limit 200
                """,
                now.minus(Duration.ofDays(2)), now.minus(Duration.ofDays(3)), now.minus(Duration.ofDays(7)),
            ) { it.getLong(1) to it.getInt(2) }
        }
        var sent = 0
        for ((userId, count) in candidates) {
            val user = users.find(userId) ?: continue
            val email = user.email ?: continue
            if (LocalDateTime.ofInstant(now, user.timezone).hour !in 10..19) continue
            val line = selector.pickGreeting(user.personality, CopyMode.Return, Daypart.Day, userId + count, emptySet())
                ?.let { CopySelector.render(it.text, user.nickname, 0) } ?: "好几天没见你了，最近还好吗？"
            mail.send(
                MailJob(
                    kind = "recall",
                    to = email,
                    subject = "${user.companionName}有点想你",
                    text = "$line\n\n回来打个卡：$publicBaseUrl\n\n不想再收到这类邮件，点这里退订：${links.link(userId)}\n— 下班了么",
                ),
            )
            db.read { c -> c.execute("update users set recall_count = recall_count + 1, last_recall_at = ? where id = ?", now, userId) }
            sent++
        }
        return sent
    }

    private companion object {
        val log = LoggerFactory.getLogger(RecallJob::class.java)
    }
}

/** Signed one-click unsubscribe links, so the link works without logging in. */
class UnsubscribeLinks(secret: String, private val apiBaseUrl: String) {
    private val key = SecretKeySpec(secret.toByteArray(), "HmacSHA256")

    fun token(userId: Long): String {
        val mac = Mac.getInstance("HmacSHA256").apply { init(key) }
        return Base64.getUrlEncoder().withoutPadding().encodeToString(mac.doFinal("unsubscribe:$userId".toByteArray())).take(22)
    }

    fun link(userId: Long) = "$apiBaseUrl/api/mail/unsubscribe?u=$userId&t=${token(userId)}"

    fun verify(userId: Long, token: String): Boolean = java.security.MessageDigest.isEqual(token(userId).toByteArray(), token.toByteArray())
}
