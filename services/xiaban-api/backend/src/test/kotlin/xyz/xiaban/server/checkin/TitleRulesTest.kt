package xyz.xiaban.server.checkin

import java.time.LocalDateTime
import java.time.LocalTime
import kotlin.test.Test
import kotlin.test.assertEquals
import xyz.xiaban.contracts.afterwork.checkin.TitleCode
import xyz.xiaban.server.calendar.HolidayCalendar

class TitleRulesTest {
    private val calendar = HolidayCalendar.load()

    private fun title(at: String, workEnd: String = "18:00"): TitleCode {
        val local = LocalDateTime.parse(at)
        return TitleRules.compute(local, WorkDay.of(local), LocalTime.parse(workEnd), calendar)
    }

    @Test
    fun `regular titles follow the overtime ladder on a plain workday`() {
        assertEquals(TitleCode.EarlyLeaver, title("2026-09-29T17:59:59"))
        assertEquals(TitleCode.ExactOnTime, title("2026-09-29T18:00:42"))
        assertEquals(TitleCode.OnTime, title("2026-09-29T18:01"))
        assertEquals(TitleCode.OnTime, title("2026-09-29T18:29:59"))
        assertEquals(TitleCode.NormalHuman, title("2026-09-29T18:30"))
        assertEquals(TitleCode.OvertimeHero, title("2026-09-29T20:00"))
        assertEquals(TitleCode.NightOwl, title("2026-09-29T22:00"))
        assertEquals(TitleCode.NightOwl, title("2026-09-29T23:59"))
        assertEquals(TitleCode.Immortal, title("2026-09-30T00:30"))
        assertEquals(TitleCode.Immortal, title("2026-09-30T04:59"))
    }

    @Test
    fun `special titles take priority in PRD order`() {
        assertEquals(TitleCode.WeekendWork, title("2026-10-03T15:00"), "national day holiday")
        assertEquals(TitleCode.WeekendWork, title("2026-10-17T20:00"), "plain saturday")
        assertEquals(TitleCode.BorrowedWeekend, title("2026-10-10T20:00"), "make-up saturday")
        assertEquals(TitleCode.HolidayCountdown, title("2026-09-30T21:00"), "last workday before national day")
        assertEquals(TitleCode.HolidayCountdown, title("2026-04-03T19:00"), "friday before qingming saturday")
        assertEquals(TitleCode.ExactOnTime, title("2026-10-10T18:00:05"), "exact minute beats make-up day")
    }

    @Test
    fun `after-midnight check-ins belong to the previous work day`() {
        // 02:00 on Saturday is still Friday's 下班日, a plain workday
        assertEquals(TitleCode.Immortal, title("2026-10-17T02:00"))
    }

    @Test
    fun `custom off-work time shifts the ladder`() {
        assertEquals(TitleCode.OnTime, title("2026-09-29T20:10", workEnd = "20:00"))
        assertEquals(TitleCode.EarlyLeaver, title("2026-09-29T19:00", workEnd = "20:00"))
        assertEquals(TitleCode.ExactOnTime, title("2026-09-30T01:30", workEnd = "01:30"))
    }
}
