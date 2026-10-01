package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.future.await
import xyz.xiaban.server.support.ApiScope
import xyz.xiaban.server.users.SessionService
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.BindEmailRequest
import xyz.xiaban.shared.BindResponse
import xyz.xiaban.shared.EmailCodeRequest
import xyz.xiaban.shared.ProfileView
import xyz.xiaban.shared.StatsView
import xyz.xiaban.shared.SubscriptionUpdate
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class AccountFlowTest {
    private val clock = TestEnv.clock

    @Test
    fun `binding an email that already has an account moves this device's data into it`() = apiTest {
        val phone = session().token
        clock.setBeijing("2026-10-12T20:00")
        post(ApiPaths.CHECKIN, phone)
        val bound = bind(phone, "Me@Example.com")
        assertFalse(bound.merged)
        assertEquals("m***@example.com", bound.profile.email)

        val laptop = session().token
        clock.setBeijing("2026-10-12T20:30")
        post(ApiPaths.CHECKIN, laptop) // same day as the phone: the phone's check-in wins
        clock.setBeijing("2026-10-13T19:00")
        post(ApiPaths.CHECKIN, laptop)
        val merged = bind(laptop, "me@example.com")
        assertTrue(merged.merged)

        val stats = get(ApiPaths.STATS, merged.token).body<StatsView>()
        assertEquals(2, stats.totalDays)
        assertEquals(HttpStatusCode.Unauthorized, get(ApiPaths.PROFILE, laptop).status, "the anonymous account is gone")
    }

    @Test
    fun `wrong codes are refused and recall emails need a bound email`() = apiTest {
        val token = session().token
        post(ApiPaths.BIND_CODE, token, EmailCodeRequest("me@example.com"))
        assertEquals(HttpStatusCode.BadRequest, post(ApiPaths.BIND, token, BindEmailRequest("me@example.com", "000000x")).status)
        assertEquals(HttpStatusCode.TooManyRequests, post(ApiPaths.BIND_CODE, token, EmailCodeRequest("me@example.com")).status)

        val other = session().token
        assertEquals(HttpStatusCode.Conflict, put(ApiPaths.SUBSCRIPTION, other, SubscriptionUpdate(true)).status)
        bind(other, "other@example.com")
        assertTrue(put(ApiPaths.SUBSCRIPTION, other, SubscriptionUpdate(true)).body<ProfileView>().recallOptIn)
    }
}

/** Requests a code, reads it from the logged mail and binds. */
suspend fun ApiScope.bind(token: String, email: String): BindResponse {
    post(ApiPaths.BIND_CODE, token, EmailCodeRequest(email))
    val mail = TestEnv.awaitMail { it.to == email.lowercase() && it.subject.startsWith("下班了么验证码") }
    val code = Regex("\\d{6}").find(mail.subject)!!.value
    synchronized(TestEnv.mailer.sent) { TestEnv.mailer.sent.remove(mail) }
    // Lift the one-minute resend cooldown so a test can bind the same address again right away.
    services.redis.cmd.del("xb:code:cool:${SessionService.sha256(email.lowercase())}").await()
    return post(ApiPaths.BIND, token, BindEmailRequest(email, code)).body()
}
