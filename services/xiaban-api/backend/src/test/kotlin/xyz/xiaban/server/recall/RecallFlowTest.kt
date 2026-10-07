package xyz.xiaban.server.recall

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlin.test.Test
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlinx.coroutines.runBlocking
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.account.profile.SubscriptionUpdate
import xyz.xiaban.contracts.afterwork.checkin.CheckinApiPaths
import xyz.xiaban.server.account.bind
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest

class RecallFlowTest {
    private val clock = TestEnv.clock

    @Test
    fun `recall emails go out at most twice per lapse and can be unsubscribed`() = apiTest {
        val token = bind(session().token, "me@example.com").token
        put(ProfileApiPaths.SUBSCRIPTION, token, SubscriptionUpdate(true))
        clock.setBeijing("2026-10-12T20:00")
        post(CheckinApiPaths.CHECKIN, token)

        fun runAt(at: String): Int {
            clock.setBeijing(at)
            return runBlocking { services.recall.run() }
        }
        assertEquals(0, runAt("2026-10-14T12:00"), "only two quiet days")
        assertEquals(1, runAt("2026-10-16T12:00"))
        assertEquals(0, runAt("2026-10-17T12:00"), "not again so soon")
        assertEquals(0, runAt("2026-10-20T23:00"), "not at night")
        assertEquals(1, runAt("2026-10-20T12:00"))
        assertEquals(0, runAt("2026-10-30T12:00"), "two per lapse")

        val mail = TestEnv.awaitMail { it.subject.endsWith("有点想你") }
        assertContains(mail.text, "退订")
        val link = Regex("""/api/mail/unsubscribe\?\S+""").find(mail.text)!!.value
        assertEquals(HttpStatusCode.OK, get(link, null).status)
        assertFalse(get(ProfileApiPaths.PROFILE, token).body<ProfileView>().recallOptIn)
        assertEquals(HttpStatusCode.BadRequest, get("/api/mail/unsubscribe?u=1&t=forged", null).status)
    }
}
