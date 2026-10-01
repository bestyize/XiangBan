package xyz.xiaban.server.stats

import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.checkin.CheckinRepository
import xyz.xiaban.server.checkin.CheckinRow
import xyz.xiaban.server.checkin.WorkDay
import xyz.xiaban.server.checkin.consecutiveRun
import xyz.xiaban.server.users.User
import xyz.xiaban.shared.StatsView
import xyz.xiaban.shared.TitleCode
import java.time.DayOfWeek
import java.time.Duration
import java.time.Instant
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.temporal.TemporalAdjusters

data class StatsSnapshot(
    val today: LocalDate,
    val totalDays: Int,
    val currentStreak: Int,
    val lastStreak: Int?,
    val bestStreak: Int,
    val todayOnDutyMinutes: Int?,
    val todayIsWeekLongest: Boolean,
    val weekAvgOffMinute: Int?,
    val weekCount: Int,
    val monthOvertimeMinutes: Int,
    val monthCount: Int,
    val titleCounts: Map<TitleCode, Int>,
) {
    /** Minute of the 下班日 back to a clock time, e.g. 974 → "21:14". */
    val weekAvgOffTime: String? get() = weekAvgOffMinute?.let { WorkDay.BOUNDARY.plusMinutes(it.toLong()).toString() }

    fun toView() = StatsView(
        totalDays, currentStreak, lastStreak, bestStreak, todayOnDutyMinutes, weekAvgOffTime,
        weekCount, monthOvertimeMinutes, monthCount, titleCounts,
    )
}

/** Personal numbers (PRD 5.5, 5.9): 相伴天数 never resets; a broken streak is reported, not punished. */
class StatsService(private val checkins: CheckinRepository, private val calendar: HolidayCalendar) {

    suspend fun snapshot(user: User, now: Instant): StatsSnapshot {
        val rows = checkins.allActive(user.id)
        val today = WorkDay.of(LocalDateTime.ofInstant(now, user.timezone))
        val days = rows.map { it.workDay to it.title }

        val latest = rows.firstOrNull()
        val latestRun = latest?.let { consecutiveRun(it.workDay, it.title, days.drop(1), calendar) { true } } ?: 0
        val alive = latest != null && noWorkdayMissed(latest.workDay, today)

        val weekStart = today.with(TemporalAdjusters.previousOrSame(DayOfWeek.MONDAY))
        val week = rows.filter { it.workDay >= weekStart && it.workDay <= today }
        val month = rows.filter { it.workDay.year == today.year && it.workDay.month == today.month }
        val todayRow = rows.firstOrNull { it.workDay == today }
        val todayDuty = todayRow?.let { onDuty(it, user) }

        return StatsSnapshot(
            today = today,
            totalDays = rows.size,
            currentStreak = if (alive) latestRun else 0,
            lastStreak = if (!alive && latest != null) latestRun else null,
            bestStreak = bestStreak(rows),
            todayOnDutyMinutes = todayDuty,
            todayIsWeekLongest = todayDuty != null && week.size >= 2 && week.all { (onDuty(it, user) ?: 0) <= todayDuty },
            weekAvgOffMinute = week.takeIf { it.isNotEmpty() }?.map { minuteOf(it, user) }?.average()?.toInt(),
            weekCount = week.size,
            monthOvertimeMinutes = month.sumOf { overtime(it, user) },
            monthCount = month.size,
            titleCounts = rows.groupingBy { it.title }.eachCount(),
        )
    }

    private fun noWorkdayMissed(last: LocalDate, today: LocalDate): Boolean =
        last >= today || generateSequence(last.plusDays(1)) { it.plusDays(1) }.takeWhile { it < today }.none(calendar::isWorkday)

    private fun bestStreak(rowsDesc: List<CheckinRow>): Int {
        var best = 0
        var run = 0
        var previous: LocalDate? = null
        for (row in rowsDesc.asReversed()) {
            run = if (previous != null && noWorkdayMissed(previous, row.workDay)) run + 1 else 1
            best = maxOf(best, run)
            previous = row.workDay
        }
        return best
    }

    private fun minuteOf(row: CheckinRow, user: User) = WorkDay.minuteOf(LocalDateTime.ofInstant(row.checkedAt, user.timezone))

    private fun onDuty(row: CheckinRow, user: User): Int? {
        val start = WorkDay.startOfWork(row.workDay, user.workStart)
        val minutes = Duration.between(start, LocalDateTime.ofInstant(row.checkedAt, user.timezone)).toMinutes()
        return minutes.toInt().takeIf { it > 0 }
    }

    private fun overtime(row: CheckinRow, user: User): Int {
        val end = WorkDay.endOfWork(row.workDay, user.workEnd)
        return Duration.between(end, LocalDateTime.ofInstant(row.checkedAt, user.timezone)).toMinutes().toInt().coerceAtLeast(0)
    }

    companion object {
        fun formatDuration(minutes: Int): String {
            val h = minutes / 60
            val m = minutes % 60
            return when {
                h == 0 -> "$m 分钟"
                m == 0 -> "$h 小时"
                else -> "$h 小时 $m 分"
            }
        }
    }
}
