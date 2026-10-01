package xyz.xiaban.server.stats

import io.ktor.client.call.body
import xyz.xiaban.server.support.FakeWeather
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.CheckinView
import xyz.xiaban.shared.PercentileScope
import xyz.xiaban.shared.ProfileUpdate
import xyz.xiaban.shared.StatsView
import xyz.xiaban.shared.TipKind
import xyz.xiaban.shared.TodayResponse
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class StatsFlowTest {
    private val clock = TestEnv.clock

    @Test
    fun `personal numbers follow PRD definitions`() = apiTest {
        val token = session().token
        listOf("2026-10-12T21:00", "2026-10-13T19:30", "2026-10-14T18:10", "2026-10-16T22:00").forEach {
            clock.setBeijing(it)
            post(ApiPaths.CHECKIN, token)
        }
        clock.setBeijing("2026-10-16T22:05")
        val stats = get(ApiPaths.STATS, token).body<StatsView>()
        assertEquals(4, stats.totalDays)
        assertEquals(1, stats.currentStreak, "Thursday was missed")
        assertEquals(3, stats.bestStreak)
        assertEquals("20:10", stats.weekAvgOffTime)
        assertEquals(13 * 60, stats.todayOnDutyMinutes)
        assertEquals(180 + 90 + 10 + 240, stats.monthOvertimeMinutes)

        clock.setBeijing("2026-10-19T12:00")
        assertEquals(1, get(ApiPaths.STATS, token).body<StatsView>().currentStreak, "the weekend does not break it")

        clock.setBeijing("2026-10-21T12:00")
        val broken = get(ApiPaths.STATS, token).body<StatsView>()
        assertEquals(0, broken.currentStreak)
        assertEquals(1, broken.lastStreak)
        assertEquals(4, broken.totalDays, "相伴天数 never resets")
    }

    @Test
    fun `tips put rain first and fill up with personal data`() = apiTest {
        val token = session().token
        put(ApiPaths.PROFILE, token, ProfileUpdate(cityCode = "beijing", payday = 15))
        TestEnv.weather.forecast = FakeWeather.forecast("2026-10-12T21:00", wetAt = setOf(22))
        clock.setBeijing("2026-10-12T21:30")
        post(ApiPaths.CHECKIN, token)

        val tips = get(ApiPaths.CHECKIN_TODAY, token).body<TodayResponse>().tips
        assertEquals(listOf(TipKind.Rain, TipKind.Duty, TipKind.Month, TipKind.Payday), tips.map { it.kind })
        assertEquals("22 点有小雨，带伞，打车老板可不报销", tips[0].text)
        assertEquals("今天在公司 12 小时 30 分", tips[1].text)
        assertEquals("距离发薪还有 3 天", tips[3].text)
        assertEquals(1, TestEnv.weather.fetches.get(), "forecast is cached per city")
    }

    @Test
    fun `make-up workday tomorrow is called out`() = apiTest {
        val token = session().token
        clock.setBeijing("2026-10-09T19:00")
        post(ApiPaths.CHECKIN, token)
        val tips = get(ApiPaths.CHECKIN_TODAY, token).body<TodayResponse>().tips
        assertEquals("明天要补班，闹钟别关", tips.first { it.kind == TipKind.Makeup }.text)
    }

    @Test
    fun `percentile appears once the reference window has enough people`() = apiTest {
        clock.setBeijing("2026-11-20T20:00")
        assertNull(post(ApiPaths.CHECKIN, session().token).body<CheckinView>().stats.percentile, "no reference data that week")

        // 60 people left between 15:00 and 24:50 the day before; 29 of them later than 20:00
        repeat(60) { i -> services.percentiles.record(LocalDate.parse("2026-10-11"), 600 + i * 10, null) }
        clock.setBeijing("2026-10-12T20:00")
        val checkin = post(ApiPaths.CHECKIN, session().token).body<CheckinView>()
        assertEquals(PercentileScope.All, checkin.stats.percentile?.scope)
        assertEquals(48, checkin.stats.percentile?.percent)
        assertTrue(checkin.stats.totalDays == 1 && checkin.stats.streak == 1)
    }
}
