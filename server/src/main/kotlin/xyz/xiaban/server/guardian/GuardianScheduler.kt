package xyz.xiaban.server.guardian

import io.lettuce.core.Limit
import io.lettuce.core.Range
import io.lettuce.core.SetArgs
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.future.await
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import org.slf4j.LoggerFactory
import xyz.xiaban.server.mail.MailJob
import xyz.xiaban.server.mail.MailQueue
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.users.UserRepository
import java.time.Clock
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

/**
 * Every [TICK_MS] takes due entries off the sorted set (PRD 5.10). A Redis lock keeps two instances off the
 * same entry and guardian_alerts makes each email go out at most once per deadline.
 */
class GuardianScheduler(
    private val service: GuardianService,
    private val guardians: GuardianRepository,
    private val users: UserRepository,
    private val mail: MailQueue,
    private val redis: Redis,
    private val publicBaseUrl: String,
    private val clock: Clock,
) {
    fun start(scope: CoroutineScope) {
        mail.onOutcome { ref, delivered ->
            ref.removePrefix(REF_PREFIX).toLongOrNull()?.takeIf { ref.startsWith(REF_PREFIX) }
                ?.let { guardians.markAlert(it, if (delivered) "sent" else "failed") }
        }
        scope.launch {
            runCatching { service.rebuildQueue() }.onFailure { log.warn("Rebuilding guardian queue failed: {}", it.message) }
            while (isActive) {
                runCatching { tick() }.onFailure { log.warn("Guardian tick failed: {}", it.message) }
                delay(TICK_MS)
            }
        }
    }

    suspend fun tick() {
        val due = redis.cmd.zrangebyscore(GuardianService.QUEUE, Range.create(0.0, clock.millis().toDouble()), Limit.create(0, 100)).await()
        for (member in due) {
            val lock = "xb:lock:guardian:$member"
            if (redis.cmd.set(lock, "1", SetArgs.Builder.nx().ex(120)).await() != "OK") continue
            try {
                process(member)
            } finally {
                redis.cmd.del(lock).await()
            }
        }
    }

    private suspend fun process(member: String) {
        val (kind, id) = member.split(":").let { it[0] to it[1].toLong() }
        val g = guardians.find(id)
        val dueAt = if (kind == "remind") g?.nextRemindAt else g?.nextNotifyAt
        if (g == null || g.paused || dueAt == null || g.nextNotifyAt == null) {
            redis.cmd.zrem(GuardianService.QUEUE, member).await()
            return
        }
        if (dueAt.toEpochMilli() > clock.millis()) {
            // The database moved on (a check-in) but the queue entry is stale: put it back at the right time.
            redis.cmd.zadd(GuardianService.QUEUE, dueAt.toEpochMilli().toDouble(), member).await()
            return
        }
        val user = users.find(id)
        val alertId = guardians.claimAlert(id, kind, g.nextNotifyAt)
        if (alertId != null && user != null) {
            val deadline = LocalDateTime.ofInstant(g.nextNotifyAt, user.timezone)
            when (kind) {
                "remind" -> user.email?.let { mail.send(remindMail(it, user.nickname, g, deadline, alertId)) }
                "notify" -> mail.send(notifyMail(g, alertId))
            }
        }
        redis.cmd.zrem(GuardianService.QUEUE, member).await()
        if (kind == "notify") guardians.setSchedule(id, null, null)
    }

    private fun remindMail(to: String, nickname: String?, g: GuardianRow, deadline: LocalDateTime, alertId: Long) = MailJob(
        kind = "guardian_remind",
        to = to,
        ref = "$REF_PREFIX$alertId",
        subject = "小伴有点担心你",
        text = """
            ${nickname ?: "你"}，已经 ${g.thresholdDays} 个工作日没看到你打卡了。

            如果一切都好，今天 ${deadline.format(HHMM)} 前打开 $publicBaseUrl 按一下“我下班了”就行。
            到时还没有打卡，小伴会按约定发邮件告诉${g.contactName}。

            在休假的话，可以在「我的 → 平安守护」里暂停。

            本服务不是急救服务，紧急情况请拨打 120 或 110。
            — 下班了么
        """.trimIndent(),
    )

    private fun notifyMail(g: GuardianRow, alertId: Long) = MailJob(
        kind = "guardian_notify",
        to = g.contactEmail,
        ref = "$REF_PREFIX$alertId",
        subject = "${g.selfName}已经 ${g.thresholdDays} 个工作日没有在下班了么报平安",
        text = """
            ${g.contactName}你好：

            ${g.selfName}在「下班了么」里开启了平安守护，并把你设为联系人。
            ta 已经连续 ${g.thresholdDays} 个工作日没有打卡了。可能只是忘了，也可能在休假。
            方便的话，给 ta 打个电话或发条消息问问。

            本服务不是急救服务，紧急情况请拨打 120 或 110。
            — 下班了么 xiaban.xyz
        """.trimIndent(),
    )

    private companion object {
        const val TICK_MS = 30_000L
        const val REF_PREFIX = "guardian:"
        val HHMM: DateTimeFormatter = DateTimeFormatter.ofPattern("HH:mm")
        val log = LoggerFactory.getLogger(GuardianScheduler::class.java)
    }
}
