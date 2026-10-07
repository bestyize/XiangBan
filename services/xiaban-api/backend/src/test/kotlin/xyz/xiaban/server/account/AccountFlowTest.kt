package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlinx.coroutines.future.await
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.contracts.account.auth.BindEmailRequest
import xyz.xiaban.contracts.account.auth.BindResponse
import xyz.xiaban.contracts.account.auth.EmailCodeRequest
import xyz.xiaban.contracts.account.consent.ConsentApiPaths
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.account.profile.SubscriptionUpdate
import xyz.xiaban.contracts.afterwork.checkin.CheckinApiPaths
import xyz.xiaban.contracts.afterwork.stats.StatsApiPaths
import xyz.xiaban.contracts.afterwork.stats.StatsView
import xyz.xiaban.server.support.ApiScope
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.server.users.SessionService

class AccountFlowTest {
    private val clock = TestEnv.clock

    @Test
    fun `binding an email that already has an account moves this device's data into it`() = apiTest {
        val phone = session().token
        clock.setBeijing("2026-10-12T20:00")
        post(CheckinApiPaths.CHECKIN, phone)
        val bound = bind(phone, "Me@Example.com")
        assertFalse(bound.merged)
        assertEquals("m***@example.com", bound.profile.email)

        val laptop = session().token
        clock.setBeijing("2026-10-12T20:30")
        post(CheckinApiPaths.CHECKIN, laptop) // same day as the phone: the phone's check-in wins
        clock.setBeijing("2026-10-13T19:00")
        post(CheckinApiPaths.CHECKIN, laptop)
        val merged = bind(laptop, "me@example.com")
        assertTrue(merged.merged)

        val stats = get(StatsApiPaths.STATS, merged.token).body<StatsView>()
        assertEquals(2, stats.totalDays)
        assertEquals(HttpStatusCode.Unauthorized, get(ProfileApiPaths.PROFILE, laptop).status, "the anonymous account is gone")
    }

    @Test
    fun `wrong codes are refused and recall emails need a bound email`() = apiTest {
        val token = session().token
        put(ConsentApiPaths.CONSENT, token, xyz.xiaban.contracts.account.consent.ConsentUpdate("0.1", false))
        post(AccountApiPaths.BIND_CODE, token, EmailCodeRequest("me@example.com"))
        assertEquals(HttpStatusCode.BadRequest, post(AccountApiPaths.BIND, token, BindEmailRequest("me@example.com", "000000x")).status)
        assertEquals(HttpStatusCode.TooManyRequests, post(AccountApiPaths.BIND_CODE, token, EmailCodeRequest("me@example.com")).status)

        val other = session().token
        assertEquals(HttpStatusCode.Conflict, put(ProfileApiPaths.SUBSCRIPTION, other, SubscriptionUpdate(true)).status)
        val boundOther = bind(other, "other@example.com")
        assertTrue(put(ProfileApiPaths.SUBSCRIPTION, boundOther.token, SubscriptionUpdate(true)).body<ProfileView>().recallOptIn)
    }
}

/** Requests a code, reads it from the logged mail and binds. */
suspend fun ApiScope.bind(token: String, email: String): BindResponse {
    put(ConsentApiPaths.CONSENT, token, xyz.xiaban.contracts.account.consent.ConsentUpdate("0.1", false))
    post(AccountApiPaths.BIND_CODE, token, EmailCodeRequest(email))
    val mail = TestEnv.awaitMail { it.to == email.lowercase() && it.subject.startsWith("下班啦验证码") }
    val code = Regex("\\d{6}").find(mail.subject)!!.value
    synchronized(TestEnv.mailer.sent) { TestEnv.mailer.sent.remove(mail) }
    // Lift the one-minute resend cooldown so a test can bind the same address again right away.
    services.redis.cmd.del("xb:code:cool:${SessionService.sha256(email.lowercase())}").await()
    return post(AccountApiPaths.BIND, token, BindEmailRequest(email, code, "User${services.tokens.userIdOrNull(token)}")).body()
}
