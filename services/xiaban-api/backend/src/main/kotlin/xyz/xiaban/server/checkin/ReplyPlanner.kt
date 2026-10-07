package xyz.xiaban.server.checkin

import java.time.LocalDate
import java.time.LocalDateTime
import xyz.xiaban.contracts.afterwork.checkin.GreetingKind
import xyz.xiaban.contracts.afterwork.checkin.GreetingView
import xyz.xiaban.contracts.afterwork.checkin.TitleCode
import xyz.xiaban.contracts.companion.personality.Mood
import xyz.xiaban.server.calendar.DayKind
import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.copy.CopyContext
import xyz.xiaban.server.copy.CopyMode
import xyz.xiaban.server.copy.CopyModeration
import xyz.xiaban.server.copy.CopySelector
import xyz.xiaban.server.copy.Daypart
import xyz.xiaban.server.users.User

data class ReplyPlan(val copyId: String, val text: String, val careMode: Boolean, val overtimeStreak: Int)

/** Turns today's situation into copy-library tags and picks the line (PRD 5.4, 5.9). */
class ReplyPlanner(
    private val selector: CopySelector,
    private val moderation: CopyModeration,
    private val calendar: HolidayCalendar,
) {
    suspend fun plan(
        user: User,
        workDay: LocalDate,
        title: TitleCode,
        history: List<CheckinRow>,
        recentCopyIds: Set<String>,
        weather: String?,
    ): ReplyPlan {
        val past = history.filter { it.workDay < workDay }.map { it.workDay to it.title }
        val overtimeStreak = consecutiveRun(workDay, title, past, calendar) { it in OVERTIME_TITLES }
        val careMode = consecutiveRun(workDay, title, past, calendar) { it in LATE_NIGHT_TITLES } >= 3
        val context = CopyContext(
            personality = user.personality,
            title = title,
            weather = weather,
            weekday = workDay.dayOfWeek.value,
            holidayTags = holidayTags(workDay),
            overtimeStreak = overtimeStreak,
            careMode = careMode,
        )
        val entry = selector.pickReply(context, recentCopyIds, moderation.disabled())
        return ReplyPlan(
            copyId = entry?.id ?: "fallback:${title.code}",
            text = entry?.let { CopySelector.render(it.text, user.nickname, overtimeStreak) } ?: TitleRules.fallbackLine(title),
            careMode = careMode,
            overtimeStreak = overtimeStreak,
        )
    }

    /** The bubble on the home screen before check-in; a welcome-back line after two or more missed workdays. */
    suspend fun greeting(user: User, workDay: LocalDate, local: LocalDateTime, lastCheckinDay: LocalDate?): GreetingView {
        val returning = lastCheckinDay != null && missedWorkdays(lastCheckinDay, workDay) >= 2
        val mode = if (returning) CopyMode.Return else CopyMode.Idle
        val daypart = when (local.hour) {
            in 5 until 17 -> Daypart.Day
            in 17 until 21 -> Daypart.Evening
            else -> Daypart.Night
        }
        val entry = selector.pickGreeting(user.personality, mode, daypart, seed = user.id * 7919 + workDay.toEpochDay(), moderation.disabled())
        val text = entry?.let { CopySelector.render(it.text, user.nickname, 0) } ?: "今天也辛苦了。下班了就按一下。"
        return if (returning) GreetingView(text, Mood.Welcome, GreetingKind.Return) else GreetingView(text, Mood.Idle, GreetingKind.Idle)
    }

    fun holidayTags(workDay: LocalDate): Set<String> = buildSet {
        calendar.holidayOn(workDay)?.let { add(it.code) }
        if (calendar.holidayAfter(workDay) != null) add("pre_holiday")
        if (calendar.kind(workDay) == DayKind.MakeupWorkday) add("makeup_day")
    }

    private fun missedWorkdays(last: LocalDate, today: LocalDate): Int =
        generateSequence(last.plusDays(1)) { it.plusDays(1) }.takeWhile { it < today }.count(calendar::isWorkday)
}
