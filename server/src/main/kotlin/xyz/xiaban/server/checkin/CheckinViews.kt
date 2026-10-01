package xyz.xiaban.server.checkin

import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.stats.PercentileService
import xyz.xiaban.server.stats.StatsSnapshot
import xyz.xiaban.server.users.HHMM
import xyz.xiaban.server.users.User
import xyz.xiaban.shared.CheckinStats
import xyz.xiaban.shared.CheckinView
import xyz.xiaban.shared.HolidayTag
import xyz.xiaban.shared.Mood
import xyz.xiaban.shared.ReplyView
import xyz.xiaban.shared.TitleView
import java.time.Duration
import java.time.Instant
import java.time.LocalDateTime

/** Builds the check-in payload shared by POST /api/checkin and GET /api/checkin/today. */
class CheckinViews(
    private val checkins: CheckinRepository,
    private val percentiles: PercentileService,
    private val calendar: HolidayCalendar,
) {

    suspend fun view(row: CheckinRow, user: User, now: Instant, stats: StatsSnapshot, pending: Boolean): CheckinView {
        val local = LocalDateTime.ofInstant(row.checkedAt, user.timezone)
        val undoDeadline = row.checkedAt.plus(UNDO_WINDOW)
        val canUndo = now.isBefore(undoDeadline) && !checkins.undoUsed(row.userId, row.workDay)
        return CheckinView(
            id = row.id.toString(),
            workDay = row.workDay.toString(),
            checkedAt = row.checkedAt.toEpochMilli(),
            localTime = local.format(HHMM),
            title = TitleView(row.title, row.title.label, row.title.rare, row.title.mood),
            mood = if (row.careMode) Mood.Care else row.title.mood,
            reply = ReplyView(row.reply, row.replySource, pending),
            careMode = row.careMode,
            undoUntil = if (canUndo) undoDeadline.toEpochMilli() else null,
            stats = CheckinStats(
                streak = stats.currentStreak,
                totalDays = stats.totalDays,
                percentile = percentiles.percentile(row.workDay, WorkDay.minuteOf(local), user.cityCode),
            ),
            holiday = calendar.holidayOn(row.workDay)?.let { HolidayTag(it.code, it.name) },
        )
    }

    companion object {
        val UNDO_WINDOW: Duration = Duration.ofMinutes(10)
    }
}
