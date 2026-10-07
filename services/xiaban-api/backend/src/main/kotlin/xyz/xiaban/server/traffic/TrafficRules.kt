package xyz.xiaban.server.traffic

import kotlinx.serialization.Serializable
import xyz.xiaban.server.calendar.DayKind
import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.http.AppJson
import java.time.LocalDate

/**
 * Self-maintained tail-number restriction table (PRD 5.5). Only cities marked verified are used, so an
 * unconfirmed rotation is never shown to anyone. Holidays and 调休 make-up weekends are not restricted.
 */
class TrafficRules(cities: List<CityRules>, private val calendar: HolidayCalendar) {
    private val byCity = cities.filter { it.verified }.associateBy { it.city }

    fun restrictedTails(cityCode: String?, date: LocalDate): List<Int>? {
        val rules = byCity[cityCode] ?: return null
        if (calendar.kind(date) != DayKind.Workday) return null
        val day = date.toString()
        val period = rules.periods.firstOrNull { day >= it.from && day <= it.to } ?: return null
        return period.pairs[date.dayOfWeek.value.toString()]
    }

    @Serializable
    data class CityRules(val city: String, val verified: Boolean, val note: String? = null, val periods: List<Period>)

    @Serializable
    data class Period(val from: String, val to: String, val pairs: Map<String, List<Int>>)

    companion object {
        fun load(calendar: HolidayCalendar): TrafficRules {
            val text = TrafficRules::class.java.getResourceAsStream("/traffic.json")!!.bufferedReader().use { it.readText() }
            return TrafficRules(AppJson.decodeFromString<List<CityRules>>(text), calendar)
        }
    }
}
