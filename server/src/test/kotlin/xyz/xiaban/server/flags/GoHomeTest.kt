package xyz.xiaban.server.flags

import io.ktor.client.call.body
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.CheckinView
import xyz.xiaban.shared.FlagsUpdate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class GoHomeTest {
    private val clock = TestEnv.clock
    private val days = listOf("2026-10-12", "2026-10-13", "2026-10-14", "2026-10-15", "2026-10-16", "2026-10-19", "2026-10-20")

    @Test
    fun `day seven invites the cat home only when the switch is on`() = apiTest {
        val session = session()
        assertFalse(session.features.goHome, "off by default until phase three")
        val token = session.token
        val views = days.map { day ->
            clock.setBeijing("${day}T19:00")
            post(ApiPaths.CHECKIN, token).body<CheckinView>()
        }
        assertEquals(7, views.last().stats.streak)
        assertFalse(views.last().goHome)

        services.flags.update(FlagsUpdate(goHomeEnabled = true))
        val today = get(ApiPaths.CHECKIN_TODAY, token).body<xyz.xiaban.shared.TodayResponse>()
        assertTrue(today.checkin!!.goHome)
        assertTrue(session(session.deviceId).features.goHome)
        assertEquals("https://example.com/xiangban", session(session.deviceId).features.appDownloadUrl)
    }
}
