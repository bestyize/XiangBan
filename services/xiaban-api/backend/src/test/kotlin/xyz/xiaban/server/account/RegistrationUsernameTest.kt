package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import kotlin.test.*
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.contracts.account.auth.BindEmailRequest
import xyz.xiaban.contracts.account.auth.BindResponse
import xyz.xiaban.contracts.account.auth.EmailCodeRequest
import xyz.xiaban.contracts.account.auth.EmailLoginRequest
import xyz.xiaban.contracts.account.consent.ConsentApiPaths
import xyz.xiaban.contracts.account.consent.ConsentUpdate
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.server.support.*

class RegistrationUsernameTest {
    @Test fun `registration requires a valid username and allows retry with the same code`() = apiTest {
        val guest = session()
        put(ConsentApiPaths.CONSENT, guest.token, ConsentUpdate("0.2", false))
        post(AccountApiPaths.EMAIL_LOGIN_CODE, guest.token, EmailCodeRequest("required@example.com"))
        val mail = TestEnv.awaitMail { it.to == "required@example.com" }
        val code = Regex("\\d{6}").find(mail.subject)!!.value
        fun request(name: String? = null) = EmailLoginRequest("required@example.com", code, guest.deviceId, username = name)
        assertEquals(HttpStatusCode.BadRequest, post(AccountApiPaths.EMAIL_LOGIN, guest.token, request("1234")).status)
        assertEquals("guest", get(ProfileApiPaths.PROFILE, guest.token).body<ProfileView>().accountType)
        val registered = post(AccountApiPaths.EMAIL_LOGIN, guest.token, request()).body<BindResponse>()
        assertTrue(registered.profile.nickname!!.isNotBlank())
        assertEquals(guest.profile.userId, registered.profile.userId)
        assertEquals("registered", registered.profile.accountType)
    }

    @Test fun `duplicate usernames preserve the code and existing-email login needs no new name`() = apiTest {
        val owner = session()
        val existing = bind(owner.token, "existing@example.com")
        val guest = session()
        put(ConsentApiPaths.CONSENT, guest.token, ConsentUpdate("0.2", false))
        post(AccountApiPaths.BIND_CODE, guest.token, EmailCodeRequest("new@example.com"))
        val code = Regex("\\d{6}").find(TestEnv.awaitMail { it.to == "new@example.com" }.subject)!!.value
        assertEquals(HttpStatusCode.Conflict, post(AccountApiPaths.BIND, guest.token, BindEmailRequest("new@example.com", code, existing.profile.nickname)).status)
        assertEquals("guest", get(ProfileApiPaths.PROFILE, guest.token).body<ProfileView>().accountType)
        val new = post(AccountApiPaths.BIND, guest.token, BindEmailRequest("new@example.com", code, "AnotherUser")).body<BindResponse>()
        assertEquals("AnotherUser", new.profile.nickname)
        val returning = session()
        put(ConsentApiPaths.CONSENT, returning.token, ConsentUpdate("0.2", false))
        post(AccountApiPaths.EMAIL_LOGIN_CODE, returning.token, EmailCodeRequest("existing@example.com"))
        val loginCode = Regex("\\d{6}").find(TestEnv.awaitMail { it.to == "existing@example.com" }.subject)!!.value
        val logged = post(AccountApiPaths.EMAIL_LOGIN, returning.token, EmailLoginRequest("existing@example.com", loginCode, returning.deviceId)).body<BindResponse>()
        assertEquals(existing.profile.userId, logged.profile.userId)
        assertEquals(existing.profile.nickname, logged.profile.nickname)
    }
}
