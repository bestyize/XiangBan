package xyz.xiaban.server.calendar

import kotlinx.serialization.Serializable
import org.slf4j.LoggerFactory
import xyz.xiaban.server.http.AppJson
import java.time.DayOfWeek
import java.time.LocalDate
import java.util.concurrent.ConcurrentHashMap

enum class DayKind { Workday, RestDay, Holiday, MakeupWorkday }

data class Holiday(val code: String, val name: String, val start: LocalDate, val end: LocalDate, val makeup: List<LocalDate>)

/** Statutory holidays and 调休 make-up workdays, entered once a year from the State Council notice. */
class HolidayCalendar(holidays: List<Holiday>, private val coveredYears: Set<Int>) {
    private val holidayDays = holidays.flatMap { h -> h.start.datesUntil(h.end.plusDays(1)).toList().map { it to h } }.toMap()
    private val makeupDays = holidays.flatMap { h -> h.makeup.map { it to h } }.toMap()
    private val warnedYears = ConcurrentHashMap.newKeySet<Int>()

    fun kind(date: LocalDate): DayKind {
        if (date.year !in coveredYears && warnedYears.add(date.year)) {
            log.warn("No holiday data for {}, falling back to plain weekends. Add it to holidays.json.", date.year)
        }
        return when {
            date in holidayDays -> DayKind.Holiday
            date in makeupDays -> DayKind.MakeupWorkday
            date.dayOfWeek == DayOfWeek.SATURDAY || date.dayOfWeek == DayOfWeek.SUNDAY -> DayKind.RestDay
            else -> DayKind.Workday
        }
    }

    fun isWorkday(date: LocalDate): Boolean = kind(date).let { it == DayKind.Workday || it == DayKind.MakeupWorkday }

    fun holidayOn(date: LocalDate): Holiday? = holidayDays[date]

    /** The holiday right after [date] when it is the last workday before that holiday starts. */
    fun holidayAfter(date: LocalDate): Holiday? {
        if (!isWorkday(date)) return null
        var day = date.plusDays(1)
        repeat(7) {
            if (isWorkday(day)) return null
            holidayDays[day]?.let { return it }
            day = day.plusDays(1)
        }
        return null
    }

    /** First make-up workday strictly after [from] and at most [days] days away. */
    fun nextMakeupDay(from: LocalDate, days: Long): LocalDate? =
        (1..days).map { from.plusDays(it) }.firstOrNull { it in makeupDays }

    fun nextWorkday(from: LocalDate): LocalDate = generateSequence(from.plusDays(1)) { it.plusDays(1) }.first(::isWorkday)

    fun previousWorkday(from: LocalDate): LocalDate = generateSequence(from.minusDays(1)) { it.minusDays(1) }.first(::isWorkday)

    companion object {
        private val log = LoggerFactory.getLogger(HolidayCalendar::class.java)

        fun load(): HolidayCalendar {
            val text = HolidayCalendar::class.java.getResourceAsStream("/holidays.json")!!.bufferedReader().use { it.readText() }
            val years = AppJson.decodeFromString<List<YearFile>>(text)
            val holidays = years.flatMap { y ->
                y.holidays.map { Holiday(it.code, it.name, LocalDate.parse(it.start), LocalDate.parse(it.end), it.makeup.map(LocalDate::parse)) }
            }
            return HolidayCalendar(holidays, years.map { it.year }.toSet())
        }
    }

    @Serializable
    private data class YearFile(val year: Int, val source: String, val holidays: List<HolidayEntry>)

    @Serializable
    private data class HolidayEntry(val code: String, val name: String, val start: String, val end: String, val makeup: List<String>)
}
