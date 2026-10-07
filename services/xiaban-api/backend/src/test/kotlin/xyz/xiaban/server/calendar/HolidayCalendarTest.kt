package xyz.xiaban.server.calendar

import xyz.xiaban.server.checkin.WorkDay
import java.time.LocalDate
import java.time.LocalDateTime
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class HolidayCalendarTest {
    private val calendar = HolidayCalendar.load()
    private fun d(s: String) = LocalDate.parse(s)

    @Test
    fun `classifies holidays, make-up days and weekends`() {
        assertEquals(DayKind.Holiday, calendar.kind(d("2026-10-01")))
        assertEquals(DayKind.Holiday, calendar.kind(d("2026-02-17")))
        assertEquals(DayKind.MakeupWorkday, calendar.kind(d("2026-10-10")))
        assertEquals(DayKind.MakeupWorkday, calendar.kind(d("2026-09-20")))
        assertEquals(DayKind.RestDay, calendar.kind(d("2026-10-11")))
        assertEquals(DayKind.Workday, calendar.kind(d("2026-10-12")))
    }

    @Test
    fun `finds the holiday after its last workday`() {
        assertEquals("national_day", calendar.holidayAfter(d("2026-09-30"))?.code)
        assertEquals("mid_autumn", calendar.holidayAfter(d("2026-09-24"))?.code)
        assertEquals("qingming", calendar.holidayAfter(d("2026-04-03"))?.code)
        assertEquals("spring_festival", calendar.holidayAfter(d("2026-02-14"))?.code)
        assertNull(calendar.holidayAfter(d("2026-10-16")), "ordinary friday")
        assertNull(calendar.holidayAfter(d("2026-10-03")), "holidays themselves are not workdays")
    }

    @Test
    fun `walks workdays across holidays`() {
        assertEquals(d("2026-10-08"), calendar.nextWorkday(d("2026-09-30")))
        assertEquals(d("2026-10-10"), calendar.nextWorkday(d("2026-10-09")))
        assertEquals(d("2026-09-30"), calendar.previousWorkday(d("2026-10-08")))
        assertEquals(d("2026-10-10"), calendar.nextMakeupDay(d("2026-10-05"), 6))
        assertNull(calendar.nextMakeupDay(d("2026-10-11"), 6))
    }

    @Test
    fun `work day boundary is 5am`() {
        assertEquals(d("2026-10-01"), WorkDay.of(LocalDateTime.parse("2026-10-02T04:59")))
        assertEquals(d("2026-10-02"), WorkDay.of(LocalDateTime.parse("2026-10-02T05:00")))
        assertEquals(0, WorkDay.minuteOf(LocalDateTime.parse("2026-10-02T05:00")))
        assertEquals(1439, WorkDay.minuteOf(LocalDateTime.parse("2026-10-03T04:59")))
    }
}
