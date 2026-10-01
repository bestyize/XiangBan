package xyz.xiaban.server.checkin

import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.shared.TitleCode
import java.time.LocalDate

val OVERTIME_TITLES = setOf(TitleCode.OvertimeHero, TitleCode.NightOwl, TitleCode.Immortal)
val LATE_NIGHT_TITLES = setOf(TitleCode.NightOwl, TitleCode.Immortal)

/**
 * Length of the run of check-ins ending at [today] that satisfy [predicate]. Only a missed workday
 * breaks a run; rest days without a check-in are skipped, so weekends never cost anyone a streak.
 *
 * @param history earlier check-ins, newest first, all before [today]
 */
fun consecutiveRun(
    today: LocalDate,
    todayTitle: TitleCode,
    history: List<Pair<LocalDate, TitleCode>>,
    calendar: HolidayCalendar,
    predicate: (TitleCode) -> Boolean,
): Int {
    if (!predicate(todayTitle)) return 0
    var count = 1
    var cursor = today
    for ((day, title) in history) {
        if (day < calendar.previousWorkday(cursor) || !predicate(title)) break
        count++
        cursor = day
    }
    return count
}
