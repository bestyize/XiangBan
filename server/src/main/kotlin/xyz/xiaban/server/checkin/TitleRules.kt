package xyz.xiaban.server.checkin

import xyz.xiaban.server.calendar.DayKind
import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.shared.TitleCode
import java.time.Duration
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.LocalTime

/** PRD 5.3: special titles in priority order first, then the overtime ladder. */
object TitleRules {
    fun compute(checkedAt: LocalDateTime, workDay: LocalDate, workEnd: LocalTime, calendar: HolidayCalendar): TitleCode {
        val overtime = Duration.between(WorkDay.endOfWork(workDay, workEnd), checkedAt).seconds

        if (overtime in 0 until 60) return TitleCode.ExactOnTime
        when (calendar.kind(workDay)) {
            DayKind.RestDay, DayKind.Holiday -> return TitleCode.WeekendWork
            DayKind.MakeupWorkday -> return TitleCode.BorrowedWeekend
            DayKind.Workday -> if (calendar.holidayAfter(workDay) != null) return TitleCode.HolidayCountdown
        }

        return when {
            overtime < 0 -> TitleCode.EarlyLeaver
            checkedAt.toLocalDate().isAfter(workDay) -> TitleCode.Immortal
            overtime < 30 * 60 -> TitleCode.OnTime
            overtime < 2 * 3600 -> TitleCode.NormalHuman
            overtime < 4 * 3600 -> TitleCode.OvertimeHero
            else -> TitleCode.NightOwl
        }
    }

    /** Used when no library line matches; the copy library normally supplies the reply. */
    fun fallbackLine(title: TitleCode): String = when (title) {
        TitleCode.ExactOnTime -> "一分不差，你是懂准点的。"
        TitleCode.WeekendWork -> "今天本来该在家躺着的，辛苦了。"
        TitleCode.BorrowedWeekend -> "周末被借走了，记得找它要回来。"
        TitleCode.HolidayCountdown -> "明天就放假了，今天先撑一下。"
        TitleCode.EarlyLeaver -> "溜得漂亮，我替你保密。"
        TitleCode.OnTime -> "准点下班，今天的你很会生活。"
        TitleCode.NormalHuman -> "今天也平平安安下班了，回家吧。"
        TitleCode.OvertimeHero -> "又加班了，路上慢点，我等你。"
        TitleCode.NightOwl -> "这么晚了，快回来，我给你留了灯。"
        TitleCode.Immortal -> "都这个点了……到家跟我说一声。"
    }
}
