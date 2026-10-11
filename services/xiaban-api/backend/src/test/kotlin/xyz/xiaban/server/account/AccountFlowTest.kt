package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import java.util.UUID
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.contracts.account.auth.AccountCenterLoginRequest
import xyz.xiaban.contracts.account.auth.BindResponse
import xyz.xiaban.contracts.account.consent.ConsentApiPaths
import xyz.xiaban.contracts.account.consent.ConsentUpdate
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.account.profile.SubscriptionUpdate
import xyz.xiaban.contracts.afterwork.checkin.CheckinApiPaths
import xyz.xiaban.contracts.afterwork.stats.StatsApiPaths
import xyz.xiaban.contracts.afterwork.stats.StatsView
import xyz.xiaban.server.support.ApiScope
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest

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
    fun `recall emails need a bound email`() = apiTest {
        val other = session().token
        assertEquals(HttpStatusCode.Conflict, put(ProfileApiPaths.SUBSCRIPTION, other, SubscriptionUpdate(true)).status)
        val boundOther = bind(other, "other@example.com")
        assertTrue(put(ProfileApiPaths.SUBSCRIPTION, boundOther.token, SubscriptionUpdate(true)).body<ProfileView>().recallOptIn)
    }
}

/**
 * Signs the caller in as the account-center account that owns [email] (the same email always maps to the same uid),
 * moving this device's guest data into it, so tests can get a registered user without going through the account center.
 */
suspend fun ApiScope.bind(token: String, email: String): BindResponse {
    put(ConsentApiPaths.CONSENT, token, ConsentUpdate("0.1", false))
    val normalized = email.trim().lowercase()
    val accountToken = TestEnv.accountCenter.issue(UUID.nameUUIDFromBytes(normalized.toByteArray()), normalized, normalized.substringBefore('@'))
    return post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, token, AccountCenterLoginRequest(accountToken, newDeviceId(), mergeGuest = true)).body()
}
