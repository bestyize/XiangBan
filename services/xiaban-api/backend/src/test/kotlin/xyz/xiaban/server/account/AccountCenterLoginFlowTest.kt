package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import java.util.UUID
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.contracts.account.auth.AccountCenterLoginRequest
import xyz.xiaban.contracts.account.auth.BindResponse
import xyz.xiaban.contracts.account.consent.ConsentApiPaths
import xyz.xiaban.contracts.account.consent.ConsentUpdate
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.account.session.SessionResponse
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.support.ApiScope
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest

class AccountCenterLoginFlowTest {
    @Test fun `guest becomes the registered account for a new account center user`() = apiTest {
        val guest = session()
        val token = TestEnv.accountCenter.issue(email = "fresh@example.com")
        val result = login(guest, token, username = "FreshUser", termsVersion = "0.3")

        assertEquals(guest.profile.userId, result.profile.userId)
        assertEquals("registered", result.profile.accountType)
        assertEquals("FreshUser", result.profile.nickname)
        assertEquals("0.3", result.profile.termsVersion)
        assertEquals("fr***@example.com", result.profile.email)
        assertEquals("registered", get(ProfileApiPaths.PROFILE, result.token).body<ProfileView>().accountType)
        assertEquals(HttpStatusCode.Unauthorized, get(ProfileApiPaths.PROFILE, guest.token).status)
    }

    @Test fun `same account center user reaches the same account from another device without fetching the email`() = apiTest {
        val uid = UUID.randomUUID()
        val first = login(session(), TestEnv.accountCenter.issue(uid, "again@example.com"))
        val lookups = TestEnv.accountCenter.emailLookups.get()

        val second = login(session(), TestEnv.accountCenter.issue(uid, "again@example.com"))
        assertEquals(first.profile.userId, second.profile.userId)
        assertEquals(lookups, TestEnv.accountCenter.emailLookups.get())
    }

    @Test fun `an older email account is linked automatically by its verified email`() = apiTest {
        val legacyId = services.users.claimEmail(0, "legacy@example.com", "LegacyUser") { }
        assertNull(services.profiles.user(legacyId).accountUid)

        val uid = UUID.randomUUID()
        val result = login(session(), TestEnv.accountCenter.issue(uid, "Legacy@Example.com"))
        assertEquals(legacyId, result.profile.userId)
        assertEquals("LegacyUser", result.profile.nickname)
        assertEquals(uid, services.profiles.user(legacyId).accountUid)
        assertEquals(1L, services.db.read { c -> c.queryOne("select count(*) from users where email = 'legacy@example.com'") { it.getLong(1) } })
    }

    @Test fun `a legacy email already linked to another account center user is refused`() = apiTest {
        val legacyId = services.users.claimEmail(0, "taken@example.com", null) { }
        login(session(), TestEnv.accountCenter.issue(UUID.randomUUID(), "taken@example.com"))
        assertEquals(legacyId, services.users.findIdByEmail("taken@example.com"))
        val other = session()
        val response = post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, other.token, request(other, TestEnv.accountCenter.issue(UUID.randomUUID(), "taken@example.com")))
        assertEquals(HttpStatusCode.Conflict, response.status)
    }

    @Test fun `an account without an email is registered and can use its token`() = apiTest {
        val result = login(session(), TestEnv.accountCenter.issue(email = null))
        assertEquals("registered", result.profile.accountType)
        assertNull(result.profile.email)
        assertEquals(HttpStatusCode.OK, get(ProfileApiPaths.PROFILE, result.token).status)
        assertEquals(true, services.users.authState(result.profile.userId))
    }

    @Test fun `merge guest moves the data into the existing account only when asked`() = apiTest {
        val uid = UUID.randomUUID()
        val existing = login(session(), TestEnv.accountCenter.issue(uid, "merge@example.com"))

        val keep = session()
        val kept = login(keep, TestEnv.accountCenter.issue(uid, "merge@example.com"), mergeGuest = false)
        assertEquals(existing.profile.userId, kept.profile.userId)
        assertEquals(false, kept.merged)
        assertTrue(services.users.exists(keep.profile.userId))

        val merge = session()
        val merged = login(merge, TestEnv.accountCenter.issue(uid, "merge@example.com"), mergeGuest = true)
        assertEquals(existing.profile.userId, merged.profile.userId)
        assertEquals(true, merged.merged)
        assertEquals(false, services.users.exists(merge.profile.userId))
    }

    @Test fun `a taken username falls back to a generated nickname instead of failing the login`() = apiTest {
        login(session(), TestEnv.accountCenter.issue(email = "first@example.com"), username = "SameName")
        val second = login(session(), TestEnv.accountCenter.issue(email = "second@example.com"), username = "SameName")
        assertNotEquals("SameName", second.profile.nickname)
        assertEquals("registered", second.profile.accountType)
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
        val first = login(session(), TestEnv.accountCenter.issue(email = "one@example.com"))
        val response = post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, first.token,
            AccountCenterLoginRequest(TestEnv.accountCenter.issue(email = "two@example.com"), newDeviceId()))
        assertEquals(HttpStatusCode.Conflict, response.status)
    }

    private fun ApiScope.request(guest: SessionResponse, accountToken: String, username: String? = null, mergeGuest: Boolean = false, termsVersion: String? = null) =
        AccountCenterLoginRequest(accountToken, guest.deviceId, mergeGuest, username, termsVersion)

    private suspend fun ApiScope.login(
        guest: SessionResponse,
        accountToken: String,
        username: String? = null,
        mergeGuest: Boolean = false,
        termsVersion: String? = null,
    ): BindResponse {
        put(ConsentApiPaths.CONSENT, guest.token, ConsentUpdate("0.1", false))
        val response = post(AccountApiPaths.ACCOUNT_CENTER_LOGIN, guest.token, request(guest, accountToken, username, mergeGuest, termsVersion))
        assertEquals(HttpStatusCode.OK, response.status)
        return response.body()
    }
}
