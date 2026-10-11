package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import java.util.UUID
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.contracts.account.auth.AccountCenterLoginRequest
import xyz.xiaban.contracts.account.auth.BindResponse
import xyz.xiaban.contracts.account.auth.LogoutRequest
import xyz.xiaban.contracts.account.consent.ConsentApiPaths
import xyz.xiaban.contracts.account.consent.ConsentUpdate
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.account.session.SessionApiPaths
import xyz.xiaban.contracts.account.session.SessionRequest
import xyz.xiaban.contracts.account.session.SessionResponse
import xyz.xiaban.contracts.afterwork.checkin.CheckinApiPaths
import xyz.xiaban.contracts.afterwork.stats.StatsApiPaths
import xyz.xiaban.contracts.afterwork.stats.StatsView
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.support.ApiScope
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest

class AccountCenterLoginFlowTest {
    @Test fun `guest becomes the registered account and takes the account center username`() = apiTest {
        val guest = session()
        val token = TestEnv.accountCenter.issue(email = "fresh@example.com", username = "Fresh_User")
        val result = login(guest, token, termsVersion = "0.3")

        assertEquals(guest.profile.userId, result.profile.userId)
        assertEquals("registered", result.profile.accountType)
        assertEquals("Fresh_User", result.profile.nickname)
        assertEquals("0.3", result.profile.termsVersion)
        assertEquals("fr***@example.com", result.profile.email)
        assertTrue(result.profile.onboarded)
        assertEquals("registered", get(ProfileApiPaths.PROFILE, result.token).body<ProfileView>().accountType)
        assertEquals(HttpStatusCode.Unauthorized, get(ProfileApiPaths.PROFILE, guest.token).status)
    }

    @Test fun `a visitor without a session creates no user row and can sign in directly`() = apiTest {
        val before = services.db.read { c -> c.queryOne("select count(*) from users") { it.getLong(1) } }!!
        val visitor = client.post(SessionApiPaths.SESSION) {
            contentType(ContentType.Application.Json); setBody(SessionRequest("visitor-device-id-1234", "Asia/Shanghai", materialize = false))
        }.body<SessionResponse>()
        assertEquals("guest", visitor.profile.accountType)
        assertEquals(0L, visitor.profile.userId)
        assertEquals(before, services.db.read { c -> c.queryOne("select count(*) from users") { it.getLong(1) } }!!)

        val response = client.post(AccountApiPaths.ACCOUNT_CENTER_LOGIN) {
            contentType(ContentType.Application.Json)
            setBody(AccountCenterLoginRequest(TestEnv.accountCenter.issue(username = "Direct_Reg"), "visitor-device-id-1234", termsVersion = "0.3"))
        }
        assertEquals(HttpStatusCode.OK, response.status)
        val registered = response.body<BindResponse>()
        assertEquals("Direct_Reg", registered.profile.nickname)
        assertEquals("registered", registered.profile.accountType)
        assertTrue(registered.profile.userId > 0L)
        assertEquals(before + 1, services.db.read { c -> c.queryOne("select count(*) from users") { it.getLong(1) } }!!)
    }

    @Test fun `the same account center user reaches the same account from another device`() = apiTest {
        val uid = UUID.randomUUID()
        val first = login(session(), TestEnv.accountCenter.issue(uid, "again@example.com", "Again_User"))
        val second = login(session(), TestEnv.accountCenter.issue(uid, "again@example.com", "Again_User"))
        assertEquals(first.profile.userId, second.profile.userId)
        assertEquals("Again_User", second.profile.nickname)
    }

    @Test fun `an administrator rename in the account center shows up at the next login`() = apiTest {
        val uid = UUID.randomUUID()
        val first = login(session(), TestEnv.accountCenter.issue(uid, username = "Old_Name"))
        val renamed = login(session(), TestEnv.accountCenter.issue(uid, username = "New_Name"))
        assertEquals(first.profile.userId, renamed.profile.userId)
        assertEquals("New_Name", renamed.profile.nickname)
    }

    @Test fun `an account without an email is registered and can use its token`() = apiTest {
        val result = login(session(), TestEnv.accountCenter.issue(email = null, username = "Phone_User"))
        assertEquals("registered", result.profile.accountType)
        assertNull(result.profile.email)
        assertEquals(HttpStatusCode.OK, get(ProfileApiPaths.PROFILE, result.token).status)
        assertEquals(true, services.users.authState(result.profile.userId))
    }

    @Test fun `merge guest moves the data into the existing account only when asked`() = apiTest {
        val uid = UUID.randomUUID()
        val existing = login(session(), TestEnv.accountCenter.issue(uid, "merge@example.com", "Merge_User"))

        val keep = session()
        post(CheckinApiPaths.CHECKIN, keep.token)
        val kept = login(keep, TestEnv.accountCenter.issue(uid, "merge@example.com", "Merge_User"), mergeGuest = false)
        assertEquals(existing.profile.userId, kept.profile.userId)
        assertEquals(false, kept.merged)
        assertTrue(services.users.exists(keep.profile.userId))
        assertEquals(0, get(StatsApiPaths.STATS, kept.token).body<StatsView>().totalDays)

        val merge = session()
        post(CheckinApiPaths.CHECKIN, merge.token)
        val merged = login(merge, TestEnv.accountCenter.issue(uid, "merge@example.com", "Merge_User"), mergeGuest = true)
        assertEquals(existing.profile.userId, merged.profile.userId)
        assertEquals(true, merged.merged)
        assertEquals(false, services.users.exists(merge.profile.userId))
        assertEquals(1, get(StatsApiPaths.STATS, merged.token).body<StatsView>().totalDays)
    }

    @Test fun `logout drops the device association and revokes the bearer`() = apiTest {
        val guest = session()
        val registered = login(guest, TestEnv.accountCenter.issue(email = "new@example.com", username = "Logout_User"))
        assertEquals(HttpStatusCode.OK, post(AccountApiPaths.LOGOUT, registered.token, LogoutRequest(guest.deviceId)).status)
        assertEquals(HttpStatusCode.Unauthorized, get(ProfileApiPaths.PROFILE, registered.token).status)
        val reopened = session(guest.deviceId)
        assertNull(reopened.profile.email)
        assertNotEquals(registered.profile.inviteCode, reopened.profile.inviteCode)
    }

    @Test fun `unknown tokens and bad requests are rejected`() = apiTest {
        val guest = session()
        put(ConsentApiPaths.CONSENT, guest.token, ConsentUpdate("0.1", false))
        assertEquals(HttpStatusCode.Unauthorized, post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, guest.token, request(guest, "forged-token")).status)
        assertEquals(HttpStatusCode.BadRequest, post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, guest.token,
            AccountCenterLoginRequest(TestEnv.accountCenter.issue(), "short")).status)
        assertEquals(HttpStatusCode.BadRequest, post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, guest.token,
            request(guest, TestEnv.accountCenter.issue(), termsVersion = "   ")).status)
        assertEquals("guest", services.profiles.view(guest.profile.userId).accountType)
    }

    @Test fun `a signed in account cannot silently switch to another account center user`() = apiTest {
        val first = login(session(), TestEnv.accountCenter.issue(email = "one@example.com", username = "User_One"))
        val response = post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, first.token,
            AccountCenterLoginRequest(TestEnv.accountCenter.issue(email = "two@example.com", username = "User_Two"), newDeviceId()))
        assertEquals(HttpStatusCode.Conflict, response.status)
    }

    private fun ApiScope.request(guest: SessionResponse, accountToken: String, mergeGuest: Boolean = false, termsVersion: String? = null) =
        AccountCenterLoginRequest(accountToken, guest.deviceId, mergeGuest, termsVersion)

    private suspend fun ApiScope.login(
        guest: SessionResponse,
        accountToken: String,
        mergeGuest: Boolean = false,
        termsVersion: String? = null,
    ): BindResponse {
        put(ConsentApiPaths.CONSENT, guest.token, ConsentUpdate("0.1", false))
        val response = post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, guest.token, request(guest, accountToken, mergeGuest, termsVersion))
        assertEquals(HttpStatusCode.OK, response.status)
        return response.body()
    }
}
