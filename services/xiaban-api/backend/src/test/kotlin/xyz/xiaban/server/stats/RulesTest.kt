package xyz.xiaban.server.stats

import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.traffic.TrafficRules
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class RulesTest {
    private val calendar = HolidayCalendar.load()
    private fun d(s: String) = LocalDate.parse(s)

    @Test
    fun `percentile counts only people who left strictly later`() {
        val counts = IntArray(1440).also {
            it[700] = 10 // earlier
            it[800] = 5 // same minute
            it[900] = 5 // later
        }
        assertEquals(25, PercentileService.percentLater(counts, 800))
        assertEquals(0, PercentileService.percentLater(counts, 900))
        assertEquals(100, PercentileService.percentLater(counts, 100))
        assertEquals(0, PercentileService.percentLater(IntArray(1440), 100))
    }

    @Test
    fun `unverified restriction data is never used`() {
        val rules = TrafficRules.load(calendar)
        assertNull(rules.restrictedTails("beijing", d("2026-10-12")))
    }

    @Test
    fun `verified rules skip holidays and make-up weekends`() {
        val period = TrafficRules.Period("2026-09-28", "2026-12-27", mapOf("1" to listOf(5, 0), "6" to listOf(1, 6)))
        val rules = TrafficRules(listOf(TrafficRules.CityRules("beijing", verified = true, periods = listOf(period))), calendar)
        assertEquals(listOf(5, 0), rules.restrictedTails("beijing", d("2026-10-12")))
        assertNull(rules.restrictedTails("beijing", d("2026-10-05")), "national day holiday")
        assertNull(rules.restrictedTails("beijing", d("2026-10-10")), "make-up saturday")
        assertNull(rules.restrictedTails("shanghai", d("2026-10-12")))
    }

    @Test
    fun `durations read naturally`() {
        assertEquals("10 小时 24 分", StatsService.formatDuration(624))
        assertEquals("45 分钟", StatsService.formatDuration(45))
        assertEquals("2 小时", StatsService.formatDuration(120))
    }
}
