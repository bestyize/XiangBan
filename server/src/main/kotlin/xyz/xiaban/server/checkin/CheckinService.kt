package xyz.xiaban.server.checkin

import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.http.conflict
import xyz.xiaban.server.memory.MemoryKind
import xyz.xiaban.server.memory.MemoryService
import xyz.xiaban.server.users.HHMM
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.User
import xyz.xiaban.shared.CheckinView
import xyz.xiaban.shared.Mood
import xyz.xiaban.shared.ReplyView
import xyz.xiaban.shared.TitleView
import xyz.xiaban.shared.TodayResponse
import java.time.Clock
import java.time.Duration
import java.time.Instant
import java.time.LocalDateTime

class CheckinService(
    private val profiles: ProfileService,
    private val checkins: CheckinRepository,
    private val calendar: HolidayCalendar,
    private val planner: ReplyPlanner,
    private val memories: MemoryService,
    private val polisher: Polisher,
    private val clock: Clock,
) {
    suspend fun checkIn(userId: Long): CheckinView {
        val user = profiles.user(userId)
        val now = clock.instant()
        val local = LocalDateTime.ofInstant(now, user.timezone)
        val workDay = WorkDay.of(local)

        // 当天重复点击直接返回已有结果，不重新计算
        checkins.active(userId, workDay)?.let { return view(it, user, now) }

        val title = TitleRules.compute(local, workDay, user.workEnd, calendar)
        val history = checkins.since(userId, workDay.minusDays(HISTORY_DAYS))
        val recentCopies = checkins.copyIdsSince(userId, workDay.minusDays(NO_REPEAT_DAYS))
        val plan = planner.plan(user, workDay, title, history, recentCopies, weather = null)
        val created = checkins.insert(NewCheckin(userId, workDay, now, title, plan.copyId, plan.text, plan.careMode))
            ?: return view(checkins.active(userId, workDay)!!, user, now)

        remember(created, plan, enteredCare = plan.careMode && history.firstOrNull()?.careMode != true)
        return view(created, user, now, pending = polisher.request(created))
    }

    /** With [waitMs] the call holds until the model rewrite lands or the wait runs out. */
    suspend fun today(userId: Long, waitMs: Long = 0): TodayResponse {
        val user = profiles.user(userId)
        val now = clock.instant()
        val local = LocalDateTime.ofInstant(now, user.timezone)
        val workDay = WorkDay.of(local)
        var row = checkins.active(userId, workDay)
            ?: return TodayResponse(workDay.toString(), null, planner.greeting(user, workDay, local, checkins.lastDayBefore(userId, workDay)))

        if (waitMs > 0 && polisher.isPending(row.id)) {
            polisher.await(row.id, Duration.ofMillis(waitMs.coerceAtMost(MAX_WAIT_MS)))
            row = checkins.find(row.id) ?: row
        }
        return TodayResponse(workDay.toString(), view(row, user, now, pending = polisher.isPending(row.id)))
    }

    suspend fun undo(userId: Long): TodayResponse {
        val user = profiles.user(userId)
        val now = clock.instant()
        val workDay = WorkDay.of(LocalDateTime.ofInstant(now, user.timezone))
        val active = checkins.active(userId, workDay) ?: conflict("今天还没打卡，没什么可撤销的", "nothing_to_undo")
        if (checkins.undoUsed(userId, workDay)) conflict("今天已经撤销过一次了", "undo_used")
        if (now.isAfter(active.checkedAt.plus(UNDO_WINDOW))) conflict("打卡超过 10 分钟，撤销不了了", "undo_expired")
        checkins.revoke(active.id, now)
        memories.forget(userId, sourceRef(active))
        return today(userId)
    }

    private suspend fun remember(row: CheckinRow, plan: ReplyPlan, enteredCare: Boolean) {
        val ref = sourceRef(row)
        if (checkins.isFirst(row.userId, row.id)) {
            memories.record(row.userId, MemoryKind.Event, "第一次在下班了么打卡，那天是「${row.title.label}」", SOURCE, ref)
        }
        if (row.title.rare) memories.record(row.userId, MemoryKind.Event, "拿到了稀有称号「${row.title.label}」", SOURCE, ref)
        when {
            enteredCare -> memories.record(row.userId, MemoryKind.Emotion, "连续 3 天很晚才下班，有点累", SOURCE, ref)
            plan.overtimeStreak == 3 -> memories.record(row.userId, MemoryKind.Emotion, "连续 3 天加班", SOURCE, ref)
        }
    }

    private suspend fun view(row: CheckinRow, user: User, now: Instant, pending: Boolean = false): CheckinView {
        val undoDeadline = row.checkedAt.plus(UNDO_WINDOW)
        val canUndo = now.isBefore(undoDeadline) && !checkins.undoUsed(row.userId, row.workDay)
        return CheckinView(
            id = row.id.toString(),
            workDay = row.workDay.toString(),
            checkedAt = row.checkedAt.toEpochMilli(),
            localTime = LocalDateTime.ofInstant(row.checkedAt, user.timezone).format(HHMM),
            title = TitleView(row.title, row.title.label, row.title.rare, row.title.mood),
            mood = if (row.careMode) Mood.Care else row.title.mood,
            reply = ReplyView(row.reply, row.replySource, pending),
            careMode = row.careMode,
            undoUntil = if (canUndo) undoDeadline.toEpochMilli() else null,
        )
    }

    private fun sourceRef(row: CheckinRow) = "checkin:${row.id}"

    companion object {
        val UNDO_WINDOW: Duration = Duration.ofMinutes(10)
        private const val SOURCE = "checkin"
        private const val HISTORY_DAYS = 45L
        private const val NO_REPEAT_DAYS = 30L
        private const val MAX_WAIT_MS = 3_500L
    }
}
