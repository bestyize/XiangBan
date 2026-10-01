package xyz.xiaban.server.checkin

import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.llm.ReplySanitizer
import xyz.xiaban.shared.TitleCode
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class RunsAndSanitizerTest {
    private val calendar = HolidayCalendar.load()
    private fun d(s: String) = LocalDate.parse(s)
    private val late = TitleCode.NightOwl

    @Test
    fun `weekends without check-ins do not break a run`() {
        // Thu, Fri, then Monday
        val history = listOf(d("2026-10-16") to late, d("2026-10-15") to late)
        assertEquals(3, consecutiveRun(d("2026-10-19"), late, history, calendar) { it in LATE_NIGHT_TITLES })
    }

    @Test
    fun `a missed workday breaks a run`() {
        // Tuesday missing between Monday and Wednesday
        val history = listOf(d("2026-10-19") to late)
        assertEquals(1, consecutiveRun(d("2026-10-21"), late, history, calendar) { it in LATE_NIGHT_TITLES })
    }

    @Test
    fun `the national day break is skipped as rest days`() {
        val history = listOf(d("2026-09-30") to late, d("2026-09-29") to late)
        assertEquals(3, consecutiveRun(d("2026-10-08"), late, history, calendar) { it in LATE_NIGHT_TITLES })
    }

    @Test
    fun `run is zero when today does not qualify`() {
        assertEquals(0, consecutiveRun(d("2026-10-19"), TitleCode.OnTime, emptyList(), calendar) { it in OVERTIME_TITLES })
    }

    @Test
    fun `model output is trimmed to one clean line`() {
        assertEquals("今天辛苦了，快回家吃饭。", ReplySanitizer.clean("「今天辛苦了，快回家吃饭。」\n解释：……"))
        assertEquals("路上慢点。", ReplySanitizer.clean("  \"路上慢点。\" "))
    }

    @Test
    fun `model output that breaks the rules is dropped`() {
        assertNull(ReplySanitizer.clean(null))
        assertNull(ReplySanitizer.clean("好"))
        assertNull(ReplySanitizer.clean("这句话实在是太长了".repeat(6)))
        assertNull(ReplySanitizer.clean("加班就去罢工吧，别忍了"))
        assertNull(ReplySanitizer.clean("看看 https://example.com"))
    }
}
