package xyz.xiaban.server.guardian

import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.LocalTime
import java.time.ZoneId
import kotlinx.coroutines.future.await
import xyz.xiaban.contracts.account.guardian.GuardianUpdate
import xyz.xiaban.contracts.account.guardian.GuardianView
import xyz.xiaban.server.account.EmailBinding
import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.checkin.CheckinRepository
import xyz.xiaban.server.checkin.WorkDay
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.conflict
import xyz.xiaban.server.http.notFound
import xyz.xiaban.server.mail.maskEmail
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.users.TextRules
import xyz.xiaban.server.users.UserRepository

/**
 * 平安守护 (PRD 5.10). Counting only workdays, a contact is emailed when the user has not checked in for
 * the chosen number of workdays by the evening deadline; the user gets a heads-up two hours earlier.
 */
class GuardianService(
    private val guardians: GuardianRepository,
    private val users: UserRepository,
    private val checkins: CheckinRepository,
    private val calendar: HolidayCalendar,
    private val redis: Redis,
    private val clock: Clock,
) {
    suspend fun view(userId: Long): GuardianView? = guardians.find(userId)?.let(::toView)

    suspend fun set(userId: Long, req: GuardianUpdate): GuardianView {
        val user = users.find(userId) ?: notFound("找不到这个账号")
        if (user.email == null) conflict("先绑定你自己的邮箱，快到时间时小伴要先提醒你", "email_required")
        if (!req.consent || !req.informedContact) badRequest("需要勾选同意，并确认已经告诉联系人本人", "consent_required")
        if (req.thresholdDays !in 1..3) badRequest("可以选 1–3 天", "invalid_threshold")
        val contactEmail = EmailBinding.normalize(req.contactEmail)
        if (contactEmail == user.email) badRequest("联系人的邮箱不能是你自己的", "invalid_contact")
        guardians.upsert(
            userId = userId,
            contactName = TextRules.contactName(req.contactName) ?: badRequest("写一下联系人的称呼", "invalid_text"),
            contactEmail = contactEmail,
            selfName = TextRules.contactName(req.selfName) ?: badRequest("写一下联系人怎么称呼你", "invalid_text"),
            thresholdDays = req.thresholdDays,
            now = clock.instant(),
        )
        reschedule(userId)
        return view(userId)!!
    }

    suspend fun pause(userId: Long, paused: Boolean): GuardianView {
        guardians.find(userId) ?: notFound("还没有开启平安守护")
        guardians.setPaused(userId, paused, clock.instant())
        reschedule(userId)
        return view(userId)!!
    }

    suspend fun remove(userId: Long) {
        guardians.delete(userId)
        unschedule(userId)
    }

    /** Recomputes the deadline after a check-in, an undo, a settings change or an account merge. */
    suspend fun reschedule(userId: Long) {
        val g = guardians.find(userId) ?: return unschedule(userId)
        val user = users.find(userId) ?: return
        if (g.paused) {
            guardians.setSchedule(userId, null, null)
            return unschedule(userId)
        }
        val enabledDay = WorkDay.of(LocalDateTime.ofInstant(g.baseAt, user.timezone))
        val baseDay = maxOf(checkins.latestDay(userId) ?: enabledDay, enabledDay)
        val notifyAt = deadline(baseDay, g.thresholdDays, user.workEnd, user.timezone)
        val remindAt = notifyAt.minus(REMIND_BEFORE)
        guardians.setSchedule(userId, remindAt, notifyAt)
        redis.cmd.zadd(QUEUE, remindAt.toEpochMilli().toDouble(), "remind:$userId").await()
        redis.cmd.zadd(QUEUE, notifyAt.toEpochMilli().toDouble(), "notify:$userId").await()
    }

    /** Puts every active schedule back into Redis, e.g. after a restart with an empty Redis. */
    suspend fun rebuildQueue() {
        for (g in guardians.scheduled()) {
            g.nextRemindAt?.let { redis.cmd.zadd(QUEUE, it.toEpochMilli().toDouble(), "remind:${g.userId}").await() }
            g.nextNotifyAt?.let { redis.cmd.zadd(QUEUE, it.toEpochMilli().toDouble(), "notify:${g.userId}").await() }
        }
    }

    private suspend fun unschedule(userId: Long) {
        redis.cmd.zrem(QUEUE, "remind:$userId", "notify:$userId").await()
    }

    /**
     * The [threshold]-th workday after [baseDay], in the evening: at least 22:00 and four hours after the
     * usual off-work time, never later than 23:30, so nobody is emailed in the middle of the night.
     */
    fun deadline(baseDay: LocalDate, threshold: Int, workEnd: LocalTime, zone: ZoneId): Instant {
        var day = baseDay
        repeat(threshold) { day = calendar.nextWorkday(day) }
        val late = workEnd.hour * 60 + workEnd.minute + 4 * 60
        val minute = if (workEnd < WorkDay.BOUNDARY || late >= LATEST) LATEST else maxOf(late, EARLIEST)
        return day.atTime(LocalTime.of(minute / 60, minute % 60)).atZone(zone).toInstant()
    }

    private fun toView(g: GuardianRow) = GuardianView(
        contactName = g.contactName,
        contactEmailMasked = maskEmail(g.contactEmail),
        selfName = g.selfName,
        thresholdDays = g.thresholdDays,
        paused = g.paused,
        nextRemindAt = g.nextRemindAt?.toEpochMilli(),
        nextNotifyAt = g.nextNotifyAt?.toEpochMilli(),
    )

    companion object {
        const val QUEUE = "xb:guardian"
        val REMIND_BEFORE: Duration = Duration.ofHours(2)
        private const val EARLIEST = 22 * 60
        private const val LATEST = 23 * 60 + 30
    }
}
