package xyz.xiaban.server.stats

import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.checkin.CheckinRow
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.server.traffic.TrafficRules
import xyz.xiaban.server.users.User
import xyz.xiaban.server.weather.WeatherInsights
import xyz.xiaban.server.weather.WeatherService
import xyz.xiaban.shared.TipKind
import xyz.xiaban.shared.TipView
import java.time.Instant
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.temporal.ChronoUnit
import java.time.temporal.IsoFields

/**
 * 3–4 things worth knowing tonight (PRD 5.5), in 小伴's voice, ordered rain/cooling > 限行 > 补班/节前 > personal.
 * Weather and 限行 only appear when the data is available; personal numbers fill the rest.
 */
class TipsService(
    private val weather: WeatherService,
    private val traffic: TrafficRules,
    private val calendar: HolidayCalendar,
    private val cities: Cities,
) {
    suspend fun tips(user: User, row: CheckinRow, stats: StatsSnapshot, now: Instant): List<TipView> {
        val tips = mutableListOf<TipView>()
        val forecast = cities.find(user.cityCode)?.let { weather.forecast(it) }
        val tomorrow = row.workDay.plusDays(1)

        forecast?.let { f ->
            WeatherInsights.rainSoon(f, now)?.let {
                val hour = LocalDateTime.ofInstant(Instant.ofEpochMilli(it.time), user.timezone).hour
                tips += TipView(TipKind.Rain, "$hour 点有${it.text}，带伞，打车老板可不报销")
            }
            WeatherInsights.tempDrop(f, row.workDay)?.let { tips += TipView(TipKind.Cold, "明天降温 $it°C，别逞强，多穿一件") }
        }
        traffic.restrictedTails(user.cityCode, tomorrow)?.let { tails ->
            tips += TipView(TipKind.Traffic, "明天尾号 ${tails.joinToString(" 和 ")} 限行")
        }
        holidayTip(row.workDay)?.let { tips += it }
        forecast?.let { f -> WeatherInsights.nextMorningTemp(f, now, user.timezone)?.let(::clothes)?.let { tips += TipView(TipKind.Morning, it) } }

        stats.todayOnDutyMinutes?.let {
            tips += TipView(TipKind.Duty, "今天在公司 ${StatsService.formatDuration(it)}${if (stats.todayIsWeekLongest) "，本周最长" else ""}")
        }
        stats.weekAvgOffTime?.takeIf { stats.weekCount >= 2 }?.let { tips += TipView(TipKind.Week, "本周平均 $it 下班") }
        stats.monthOvertimeMinutes.takeIf { it >= 60 }?.let { tips += TipView(TipKind.Month, "这个月已经加班 ${it / 60} 小时") }
        user.payday?.let { tips += paydayTip(it, row.workDay) }
        return tips.take(MAX_TIPS)
    }

    private fun holidayTip(workDay: LocalDate): TipView? {
        calendar.holidayAfter(workDay)?.let { return TipView(TipKind.Holiday, "明天开始放${it.name}，好好歇几天") }
        calendar.nextMakeupDay(workDay, 6)?.let { day ->
            val text = if (day == workDay.plusDays(1)) "明天要补班，闹钟别关" else "${relativeWeekday(day, workDay)}要补班，别忘了"
            return TipView(TipKind.Makeup, text)
        }
        calendar.nextHolidayStart(workDay, 7)?.let {
            return TipView(TipKind.Holiday, "还有 ${ChronoUnit.DAYS.between(workDay, it.start)} 天就放${it.name}了，再坚持一下")
        }
        return null
    }

    private fun relativeWeekday(day: LocalDate, from: LocalDate): String {
        val sameWeek = day.get(IsoFields.WEEK_OF_WEEK_BASED_YEAR) == from.get(IsoFields.WEEK_OF_WEEK_BASED_YEAR)
        return (if (sameWeek) "这周" else "下周") + "一二三四五六日"[day.dayOfWeek.value - 1]
    }

    private fun clothes(temp: Int): String? = when {
        temp <= 0 -> "明早 $temp°C，羽绒服安排上"
        temp <= 8 -> "明早 $temp°C，穿厚一点再出门"
        temp <= 15 -> "明早 $temp°C，加件外套"
        temp >= 30 -> "明早就 $temp°C 了，穿凉快点"
        else -> null
    }

    private fun paydayTip(payday: Int, today: LocalDate): TipView {
        fun inMonth(month: LocalDate) = month.withDayOfMonth(minOf(payday, month.lengthOfMonth()))
        val next = inMonth(today).let { if (it < today) inMonth(today.plusMonths(1).withDayOfMonth(1)) else it }
        val days = ChronoUnit.DAYS.between(today, next)
        return TipView(TipKind.Payday, if (days == 0L) "今天发薪，请自己吃顿好的" else "距离发薪还有 $days 天")
    }

    private companion object {
        const val MAX_TIPS = 4
    }
}
