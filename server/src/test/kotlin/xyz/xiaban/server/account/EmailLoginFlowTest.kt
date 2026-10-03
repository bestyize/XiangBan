package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import io.ktor.client.request.*
import io.ktor.http.*
import kotlinx.coroutines.future.await
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.support.*
import xyz.xiaban.server.users.SessionService
import xyz.xiaban.shared.*
import kotlin.test.*

class EmailLoginFlowTest {
    @Test fun `new email registers and logout removes device association and revokes bearer`() = apiTest {
        val guest = session()
        val registered = emailLogin(guest, "new@example.com")
        assertTrue(registered.profile.onboarded)
        assertEquals("ne***@example.com", registered.profile.email)
        assertEquals(HttpStatusCode.OK, post(ApiPaths.LOGOUT, registered.token, LogoutRequest(guest.deviceId)).status)
        assertEquals(HttpStatusCode.Unauthorized, get(ApiPaths.PROFILE, registered.token).status)
        val reopened = session(guest.deviceId)
        assertNull(reopened.profile.email)
        assertNotEquals(registered.profile.inviteCode, reopened.profile.inviteCode)
    }

    @Test fun `login finds existing account without automatically merging guest checkins`() = apiTest {
        val original = session()
        val registered = emailLogin(original, "me@example.com")
        services.redis.cmd.del("xb:code:cool:${SessionService.sha256("me@example.com")}").await()
        val guest = session()
        post(ApiPaths.CHECKIN, guest.token)
        val restored = emailLogin(guest, "me@example.com")
        assertEquals(registered.profile.inviteCode, restored.profile.inviteCode)
        assertFalse(restored.merged)
        assertEquals(0, get(ApiPaths.STATS, restored.token).body<StatsView>().totalDays)
        val renewed = client.post(ApiPaths.SESSION) {
            bearerAuth(restored.token); contentType(ContentType.Application.Json); setBody(SessionRequest(guest.deviceId))
        }.body<SessionResponse>()
        assertEquals(restored.profile.inviteCode, renewed.profile.inviteCode)
        assertEquals("registered", renewed.profile.accountType)
    }

    @Test fun `wrong guesses lock code and verified codes cannot be replayed or used for another purpose`() = apiTest {
        val guest = session()
        val code = loginCode(guest, "codes@example.com")
        assertEquals(HttpStatusCode.BadRequest, post(ApiPaths.BIND, guest.token, BindEmailRequest("codes@example.com", code)).status)
        val wrong = if (code == "000000") "111111" else "000000"
        repeat(5) { assertEquals(HttpStatusCode.BadRequest, post(ApiPaths.EMAIL_LOGIN, guest.token, EmailLoginRequest("codes@example.com", wrong, guest.deviceId)).status) }
        assertEquals(HttpStatusCode.BadRequest, post(ApiPaths.EMAIL_LOGIN, guest.token, EmailLoginRequest("codes@example.com", code, guest.deviceId)).status)
        services.redis.cmd.del("xb:code:cool:${SessionService.sha256("codes@example.com")}").await()
        val next = loginCode(guest, "codes@example.com")
        assertEquals(HttpStatusCode.OK, post(ApiPaths.EMAIL_LOGIN, guest.token, EmailLoginRequest("codes@example.com", next, guest.deviceId, username = "CodeUser")).status)
        assertEquals(HttpStatusCode.Unauthorized, post(ApiPaths.EMAIL_LOGIN, guest.token, EmailLoginRequest("codes@example.com", next, guest.deviceId)).status)
    }

    @Test fun `logging in can explicitly merge guest records but cannot overwrite a bound email`() = apiTest {
        val original = session()
        val registered = emailLogin(original, "merge@example.com")
        services.redis.cmd.del("xb:code:cool:${SessionService.sha256("merge@example.com")}").await()
        val guest = session()
        post(ApiPaths.CHECKIN, guest.token)
        val merged = emailLogin(guest, "merge@example.com", merge = true)
        assertTrue(merged.merged)
        assertEquals(1, get(ApiPaths.STATS, merged.token).body<StatsView>().totalDays)
        assertEquals(HttpStatusCode.Conflict, post(ApiPaths.BIND_CODE, registered.token, EmailCodeRequest("other@example.com")).status)
    }

    @Test fun `unauthenticated visitor does not insert database user and can register directly`() = apiTest {
        val userCountBefore = services.db.read { c -> c.queryOne("select count(*) from users") { it.getLong(1) } }!!
        val visitorSession = client.post(ApiPaths.SESSION) {
            contentType(ContentType.Application.Json); setBody(SessionRequest("visitor-device-id-1234", "Asia/Shanghai", materialize = false))
        }.body<SessionResponse>()
        assertEquals("", visitorSession.token)
        assertEquals("guest", visitorSession.profile.accountType)
        assertEquals(0L, visitorSession.profile.userId)
        val userCountAfterVisit = services.db.read { c -> c.queryOne("select count(*) from users") { it.getLong(1) } }!!
        assertEquals(userCountBefore, userCountAfterVisit)

        val todayRes = client.get(ApiPaths.CHECKIN_TODAY).body<TodayResponse>()
        assertNull(todayRes.checkin)
        assertNotNull(todayRes.greeting)

        val visitorCheckin = client.post(ApiPaths.CHECKIN).body<CheckinView>()
        assertNotNull(visitorCheckin.title)
        val userCountAfterCheckin = services.db.read { c -> c.queryOne("select count(*) from users") { it.getLong(1) } }!!
        assertEquals(userCountBefore, userCountAfterCheckin)

        val codeRes = client.post(ApiPaths.EMAIL_LOGIN_CODE) {
            contentType(ContentType.Application.Json); setBody(EmailCodeRequest("visitor-direct@example.com"))
        }
        assertEquals(HttpStatusCode.OK, codeRes.status)
        val mail = TestEnv.awaitMail { it.to == "visitor-direct@example.com" }
        val code = Regex("\\d{6}").find(mail.subject)!!.value

        val regRes = client.post(ApiPaths.EMAIL_LOGIN) {
            contentType(ContentType.Application.Json)
            setBody(EmailLoginRequest("visitor-direct@example.com", code, "visitor-device-id-1234", username = "DirectRegUser"))
        }
        assertEquals(HttpStatusCode.OK, regRes.status)
        val registered = regRes.body<BindResponse>()
        assertTrue(registered.token.isNotEmpty())
        assertEquals("DirectRegUser", registered.profile.nickname)
        assertEquals("registered", registered.profile.accountType)
        assertTrue(registered.profile.userId > 0L)
        val userCountAfterReg = services.db.read { c -> c.queryOne("select count(*) from users") { it.getLong(1) } }!!
        assertEquals(userCountBefore + 1, userCountAfterReg)
    }
}

private suspend fun ApiScope.loginCode(guest: SessionResponse, email: String): String {
    put(ApiPaths.CONSENT, guest.token, ConsentUpdate("0.1", false))
    assertEquals(HttpStatusCode.OK, post(ApiPaths.EMAIL_LOGIN_CODE, guest.token, EmailCodeRequest(email)).status)
    val mail = TestEnv.awaitMail { it.to == email }
    synchronized(TestEnv.mailer.sent) { TestEnv.mailer.sent.remove(mail) }
    return Regex("\\d{6}").find(mail.subject)!!.value
}

private suspend fun ApiScope.emailLogin(guest: SessionResponse, email: String, merge: Boolean = false): BindResponse =
    post(ApiPaths.EMAIL_LOGIN, guest.token, EmailLoginRequest(email, loginCode(guest, email), guest.deviceId, merge, "User${guest.profile.userId}")).body()
