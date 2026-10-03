package xyz.xiaban.server.copy

import io.ktor.client.call.body
import io.ktor.client.request.header
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.CheckinView
import xyz.xiaban.shared.FlagsUpdate
import xyz.xiaban.shared.FlagsView
import xyz.xiaban.shared.GreetingKind
import xyz.xiaban.shared.Mood
import xyz.xiaban.shared.ReplySource
import xyz.xiaban.shared.ReportRequest
import xyz.xiaban.shared.TodayResponse
import java.time.Duration
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class CopyFlowTest {
    private val clock = TestEnv.clock
    private val llm = TestEnv.llm

    @Test
    fun `library line comes back instantly and the rewrite replaces it`() = apiTest {
        services.flags.update(FlagsUpdate(llmEnabled = true))
        val token = session().token
        clock.setBeijing("2026-09-29T21:37")
        val checkin = post(ApiPaths.CHECKIN, token).body<CheckinView>()
        assertEquals(ReplySource.Library, checkin.reply.source)
        assertTrue(checkin.reply.pending)
        assertTrue(services.copyLibrary.find(services.checkinRepository.find(checkin.id.toLong())!!.copyId) != null)

        val today = get("${ApiPaths.CHECKIN_TODAY}?wait=3000", token).body<TodayResponse>()
        assertEquals(ReplySource.Llm, today.checkin!!.reply.source)
        assertEquals("改写过的一句话，路上慢点。", today.checkin!!.reply.text)
        assertFalse(today.checkin!!.reply.pending)
        assertContains(llm.lastUserPrompt!!, "加班侠")
    }

    @Test
    fun `switch off, model failure and quota all keep the library line`() = apiTest {
        val token = session().token
        clock.setBeijing("2026-09-29T19:00")
        assertFalse(post(ApiPaths.CHECKIN, token).body<CheckinView>().reply.pending, "switch is off by default")

        services.flags.update(FlagsUpdate(llmEnabled = true))
        llm.reply = null
        val other = session().token
        post(ApiPaths.CHECKIN, other)
        val failed = get("${ApiPaths.CHECKIN_TODAY}?wait=3000", other).body<TodayResponse>().checkin!!
        assertEquals(ReplySource.Library, failed.reply.source)

        llm.reply = "改写过的一句话，路上慢点。"
        clock.advance(Duration.ofMinutes(1))
        delete(ApiPaths.CHECKIN_TODAY, other)
        val redo = post(ApiPaths.CHECKIN, other).body<CheckinView>()
        assertFalse(redo.reply.pending, "the daily quota (1 in tests) is used up")
        assertEquals(1, llm.calls.get())
    }

    @Test
    fun `three late nights in a row switch to care mode`() = apiTest {
        val token = session().token
        listOf("2026-10-12T23:00", "2026-10-13T23:30", "2026-10-14T23:10").forEach { at ->
            clock.setBeijing(at)
            post(ApiPaths.CHECKIN, token)
        }
        val third = get(ApiPaths.CHECKIN_TODAY, token).body<TodayResponse>().checkin!!
        assertTrue(third.careMode)
        assertEquals(Mood.Care, third.mood)
    }

    @Test
    fun `greeting welcomes the user back after missed workdays`() = apiTest {
        val token = session().token
        clock.setBeijing("2026-10-12T20:00")
        assertEquals(GreetingKind.Idle, get(ApiPaths.CHECKIN_TODAY, token).body<TodayResponse>().greeting?.kind)
        post(ApiPaths.CHECKIN, token)
        clock.setBeijing("2026-10-16T12:00")
        val back = get(ApiPaths.CHECKIN_TODAY, token).body<TodayResponse>().greeting!!
        assertEquals(GreetingKind.Return, back.kind)
        assertEquals(Mood.Welcome, back.mood)
    }

    @Test
    fun `reporting a line takes it down`() = apiTest {
        val token = session().token
        clock.setBeijing("2026-09-29T20:00")
        val checkin = post(ApiPaths.CHECKIN, token).body<CheckinView>()
        assertEquals(HttpStatusCode.OK, post(ApiPaths.REPORTS, token, ReportRequest(checkin.id, "不好笑")).status)
        val copyId = services.checkinRepository.find(checkin.id.toLong())!!.copyId
        assertTrue(copyId in services.moderation.disabled())
        assertEquals(HttpStatusCode.NotFound, post(ApiPaths.REPORTS, session().token, ReportRequest(checkin.id)).status, "not your check-in")
    }

    @Test
    fun `admin switch needs the token`() = apiTest {
        assertEquals(HttpStatusCode.Unauthorized, get("/api/admin/flags", null).status)
        val updated = client.put("/api/admin/flags") {
            header("X-Admin-Token", TestEnv.ADMIN_TOKEN)
            contentType(ContentType.Application.Json)
            setBody(FlagsUpdate(llmEnabled = true))
        }.body<FlagsView>()
        assertTrue(updated.llmEnabled)
        assertTrue(services.flags.llmEnabled())
    }
}
