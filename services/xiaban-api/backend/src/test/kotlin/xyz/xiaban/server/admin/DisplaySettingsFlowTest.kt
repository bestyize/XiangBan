package xyz.xiaban.server.admin

import io.ktor.client.call.body
import io.ktor.client.request.*
import io.ktor.http.*
import kotlin.test.*
import xyz.xiaban.contracts.admin.settings.DisplayItem
import xyz.xiaban.contracts.admin.settings.DisplaySettings
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest

class DisplaySettingsFlowTest {
    @Test fun `admin-only writes persist and appear on the public endpoint with an audit entry`() = apiTest {
        val initial = client.get("/api/display")
        assertEquals(HttpStatusCode.OK, initial.status)
        assertEquals("no-store", initial.headers[HttpHeaders.CacheControl])
        val defaults = initial.body<DisplaySettings>()
        assertEquals(HttpStatusCode.Unauthorized, client.get("/api/admin/display").status)
        assertEquals(HttpStatusCode.Unauthorized, client.put("/api/admin/display") { contentType(ContentType.Application.Json); setBody(defaults) }.status)
        val changed = defaults.copy(chatTabVisible = false, games = defaults.games.reversed().map { it.copy(visible = it.id != "merge-2048") }, quizzes = defaults.quizzes.reversed().map { it.copy(visible = it.id != "mbti") })
        val saved = client.put("/api/admin/display") {
            header("X-Admin-Token", TestEnv.ADMIN_TOKEN); contentType(ContentType.Application.Json); setBody(changed)
        }.body<DisplaySettings>()
        assertEquals(changed, saved)
        assertEquals(saved, client.get("/api/display").body<DisplaySettings>())
        assertTrue(services.adminUsers.auditPage(1, 20).items.any { it.action == "display_settings" })
        val invalid = changed.copy(games = listOf(DisplayItem("unknown-pack", true)))
        assertEquals(HttpStatusCode.BadRequest, client.put("/api/admin/display") {
            header("X-Admin-Token", TestEnv.ADMIN_TOKEN); contentType(ContentType.Application.Json); setBody(invalid)
        }.status)
        assertEquals(saved, client.get("/api/display").body<DisplaySettings>())
    }
}
