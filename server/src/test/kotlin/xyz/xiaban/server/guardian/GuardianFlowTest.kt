package xyz.xiaban.server.guardian

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.runBlocking
import xyz.xiaban.server.account.bind
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.GuardianPause
import xyz.xiaban.shared.GuardianUpdate
import xyz.xiaban.shared.GuardianView
import java.time.LocalDate
import java.time.LocalTime
import java.time.OffsetDateTime
import java.time.ZoneId
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertNull

class GuardianFlowTest {
    private val clock = TestEnv.clock
    private val zone = ZoneId.of("Asia/Shanghai")
    private fun beijing(s: String) = OffsetDateTime.parse("$s:00+08:00").toInstant().toEpochMilli()
    private val request = GuardianUpdate("妈妈", "mom@example.com", "小明", thresholdDays = 2, consent = true, informedContact = true)

    @Test
    fun `deadline counts workdays and lands in the evening`() {
        val service = TestEnv.services().guardian
        val friday = LocalDate.parse("2026-10-16")
        assertEquals(beijing("2026-10-20T22:00"), service.deadline(friday, 2, LocalTime.of(18, 0), zone).toEpochMilli())
        assertEquals(beijing("2026-10-08T23:00"), service.deadline(LocalDate.parse("2026-09-30"), 1, LocalTime.of(19, 0), zone).toEpochMilli())
        assertEquals(beijing("2026-10-19T23:30"), service.deadline(friday, 1, LocalTime.of(21, 0), zone).toEpochMilli())
    }

    @Test
    fun `setting up needs a bound email and consent`() = apiTest {
        val token = session().token
        assertEquals(HttpStatusCode.Conflict, put(ApiPaths.GUARDIAN, token, request).status)
        bind(token, "me@example.com")
        assertEquals(HttpStatusCode.BadRequest, put(ApiPaths.GUARDIAN, token, request.copy(informedContact = false)).status)
        assertEquals(HttpStatusCode.OK, put(ApiPaths.GUARDIAN, token, request).status)
    }

    @Test
    fun `reminds the user, then tells the contact once, and a check-in resets the clock`() = apiTest {
        val token = session().token
        bind(token, "me@example.com")
        clock.setBeijing("2026-10-12T20:00")
        post(ApiPaths.CHECKIN, token)
        val view = put(ApiPaths.GUARDIAN, token, request).body<GuardianView>()
        assertEquals(beijing("2026-10-14T22:00"), view.nextNotifyAt)
        assertEquals(beijing("2026-10-14T20:00"), view.nextRemindAt)

        clock.setBeijing("2026-10-14T20:01")
        runBlocking { services.guardianScheduler.tick() }
        assertContains(TestEnv.awaitMail { it.to == "me@example.com" && it.subject == "小伴有点担心你" }.text, "22:00 前")

        clock.setBeijing("2026-10-14T22:01")
        runBlocking { services.guardianScheduler.tick(); services.guardianScheduler.tick() }
        val notice = TestEnv.awaitMail { it.to == "mom@example.com" }
        assertContains(notice.text, "不是急救服务")
        Thread.sleep(300)
        assertEquals(1, TestEnv.sentMails { it.to == "mom@example.com" }.size, "sent once")
        assertNull(get(ApiPaths.GUARDIAN, token).body<GuardianView>().nextNotifyAt)

        clock.setBeijing("2026-10-15T19:00")
        post(ApiPaths.CHECKIN, token)
        assertEquals(beijing("2026-10-19T22:00"), get(ApiPaths.GUARDIAN, token).body<GuardianView>().nextNotifyAt)

        val paused = put(ApiPaths.GUARDIAN_PAUSE, token, GuardianPause(true)).body<GuardianView>()
        assertNull(paused.nextNotifyAt)
    }
}
