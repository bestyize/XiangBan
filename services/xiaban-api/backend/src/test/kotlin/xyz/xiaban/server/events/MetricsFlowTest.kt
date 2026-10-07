package xyz.xiaban.server.events

import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.http.HttpStatusCode
import java.time.LocalDate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import xyz.xiaban.contracts.afterwork.checkin.CheckinApiPaths
import xyz.xiaban.contracts.telemetry.EventBatch
import xyz.xiaban.contracts.telemetry.EventIn
import xyz.xiaban.contracts.telemetry.TelemetryApiPaths
import xyz.xiaban.server.db.query
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest

class MetricsFlowTest {
    private val clock = TestEnv.clock

    private fun MetricsResponse.row(key: String, day: String) =
        (visits + growth + product + games + quizzes).first { it.key == key }.points.first { it.day == day }

    @Test
    fun `guests report anonymously and are counted as people per 05-00 day`() = apiTest {
        val guestA = newDeviceId()
        val guestB = newDeviceId()
        val member = session().token

        clock.setBeijing("2026-10-12T21:00")
        post(TelemetryApiPaths.EVENTS, null, EventBatch(listOf(EventIn("page_view", props = mapOf("path" to "/")), EventIn("page_view", props = mapOf("path" to "/tests"))), guestA))
        post(TelemetryApiPaths.EVENTS, null, EventBatch(listOf(
            EventIn("game_start", props = mapOf("game" to "merge-2048", "secret" to "dropped")),
            EventIn("card_save", props = mapOf("via" to "download")),
            EventIn("checkin_undo"), // not something a guest can do
        ), guestA))
        post(TelemetryApiPaths.EVENTS, null, EventBatch(listOf(EventIn("quiz_start", props = mapOf("quiz" to "mbti", "mode" to "new"))), guestB))
        post(TelemetryApiPaths.EVENTS, member, EventBatch(listOf(EventIn("page_view", props = mapOf("path" to "/")), EventIn("card_share", props = mapOf("via" to "web_share")))))
        // Guest check-in is not stored but counts once the client sends its device id.
        client.post("${CheckinApiPaths.CHECKIN}?deviceId=$guestB")
        // 02:00 the next calendar day still belongs to the 12th.
        clock.setBeijing("2026-10-13T02:00")
        post(TelemetryApiPaths.EVENTS, null, EventBatch(listOf(EventIn("page_view", props = mapOf("path" to "/"))), guestB))
        // A batch without a token or device id is ignored instead of failing.
        assertEquals(HttpStatusCode.OK, post(TelemetryApiPaths.EVENTS, null, EventBatch(listOf(EventIn("page_view")))).status)

        val stored = services.db.read { c ->
            c.query("select actor, user_type, name, props::text from events where user_type = 'guest' order by id") {
                listOf(it.getString(1), it.getString(2), it.getString(3), it.getString(4))
            }
        }
        assertTrue(stored.all { it[0].startsWith("d:") && it[0].length == 66 }, "guests stored only as a hashed device id")
        assertFalse(stored.any { guestA in it[0] || guestB in it[0] }, "raw device ids never stored")
        assertFalse(stored.any { it[2] == "checkin_undo" }, "guests cannot report registered-only events")
        assertEquals("""{"game": "merge-2048"}""", stored.first { it[2] == "game_start" }[3], "undeclared parameters are dropped")

        clock.setBeijing("2026-10-13T09:00")
        val metrics = client.get("/api/admin/metrics?days=7") { header("X-Admin-Token", TestEnv.ADMIN_TOKEN) }.body<MetricsResponse>()
        val day = "2026-10-12"
        assertEquals(MetricPoint(day, 3, 1, 2), metrics.row("uv", day), "guest A, guest B (02:00) and the member")
        assertEquals(4L, metrics.row("pv", day).total)
        assertEquals(MetricPoint(day, 1, 0, 1), metrics.row("checkin", day))
        assertEquals(MetricPoint(day, 2, 1, 1), metrics.row("share", day), "a save and a share both count")
        assertEquals(1L, metrics.row("card_save", day).guest)
        assertEquals(1L, metrics.row("card_share", day).registered)
        assertEquals(1L, metrics.row("game:merge-2048", day).total)
        assertEquals(0L, metrics.row("game:water-sort", day).total)
        assertEquals(1L, metrics.row("quiz:mbti", day).total)
        assertEquals(7, metrics.days.size)
        assertEquals("2026-10-13", metrics.days.last())
    }

    @Test
    fun `new registrations are counted on the day the email was verified`() = apiTest {
        services.db.read { c ->
            c.query("insert into users (timezone, invite_code, email, email_verified_at) values ('Asia/Shanghai', 'metricsa01', 'a@example.com', '2026-10-12T22:00:00+08:00'), ('Asia/Shanghai', 'metricsb01', 'b@example.com', '2026-10-13T03:00:00+08:00') returning id") { it.getLong(1) }
        }
        clock.setBeijing("2026-10-13T09:00")
        val metrics = services.metrics.daily(3, null, emptyMap(), emptyMap())
        assertEquals(2L, metrics.row("signup", "2026-10-12").total, "03:00 on the 13th is still the 12th")
        assertEquals(0L, metrics.row("signup", "2026-10-13").total)
    }

    @Test
    fun `the end parameter pages the window into the past, clamped to today`() = apiTest {
        clock.setBeijing("2026-10-05T09:00")
        val latest = services.metrics.daily(3, null, emptyMap(), emptyMap())
        assertEquals(listOf("2026-10-03", "2026-10-04", "2026-10-05"), latest.days)
        assertTrue(latest.isLatest)

        val previousPage = services.metrics.daily(3, LocalDate.parse("2026-10-02"), emptyMap(), emptyMap())
        assertEquals(listOf("2026-09-30", "2026-10-01", "2026-10-02"), previousPage.days)
        assertFalse(previousPage.isLatest, "an older page is not the latest")

        // Paging past today lands back on today instead of showing a window into the future.
        val future = services.metrics.daily(3, LocalDate.parse("2026-12-25"), emptyMap(), emptyMap())
        assertEquals(latest.days, future.days)
        assertTrue(future.isLatest)
    }

    @Test
    fun `guest events expire after 180 days`() = apiTest {
        clock.setBeijing("2026-01-01T12:00")
        services.events.record(Actor.guest(newDeviceId())!!, listOf(EventIn("page_view")), null)
        services.db.read { c -> c.queryOne("update events set created_at = now() - interval '200 days' returning id") { it.getLong(1) } }
        clock.now = java.time.Instant.now()
        assertEquals(1, services.events.purgeGuestEvents())
        assertEquals(LocalDate.parse("2026-01-01"), EventService.metricsDay(java.time.OffsetDateTime.parse("2026-01-02T04:59:00+08:00").toInstant()))
    }
}
