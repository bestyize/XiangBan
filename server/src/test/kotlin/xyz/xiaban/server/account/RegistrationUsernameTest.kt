package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.http.HttpStatusCode
import xyz.xiaban.server.support.*
import xyz.xiaban.shared.*
import kotlin.test.*

class RegistrationUsernameTest {
    @Test fun `registration requires a valid username and allows retry with the same code`() = apiTest {
        val guest = session()
        put(ApiPaths.CONSENT, guest.token, ConsentUpdate("0.2", false))
        post(ApiPaths.EMAIL_LOGIN_CODE, guest.token, EmailCodeRequest("required@example.com"))
        val mail = TestEnv.awaitMail { it.to == "required@example.com" }
        val code = Regex("\\d{6}").find(mail.subject)!!.value
        fun request(name: String? = null) = EmailLoginRequest("required@example.com", code, guest.deviceId, username = name)
        val missing = post(ApiPaths.EMAIL_LOGIN, guest.token, request())
        assertEquals(HttpStatusCode.BadRequest, missing.status)
        assertEquals("username_required", missing.body<ApiError>().error)
        assertEquals("guest", get(ApiPaths.PROFILE, guest.token).body<ProfileView>().accountType)
        assertEquals(HttpStatusCode.BadRequest, post(ApiPaths.EMAIL_LOGIN, guest.token, request("1234")).status)
        val registered = post(ApiPaths.EMAIL_LOGIN, guest.token, request("NewUser")).body<BindResponse>()
        assertEquals("NewUser", registered.profile.nickname)
        assertEquals(guest.profile.userId, registered.profile.userId)
        assertEquals("registered", registered.profile.accountType)
    }

    @Test fun `duplicate usernames preserve the code and existing-email login needs no new name`() = apiTest {
        val owner = session()
        val existing = bind(owner.token, "existing@example.com")
        val guest = session()
        put(ApiPaths.CONSENT, guest.token, ConsentUpdate("0.2", false))
        post(ApiPaths.BIND_CODE, guest.token, EmailCodeRequest("new@example.com"))
        val code = Regex("\\d{6}").find(TestEnv.awaitMail { it.to == "new@example.com" }.subject)!!.value
        assertEquals(HttpStatusCode.Conflict, post(ApiPaths.BIND, guest.token, BindEmailRequest("new@example.com", code, existing.profile.nickname)).status)
        assertEquals("guest", get(ApiPaths.PROFILE, guest.token).body<ProfileView>().accountType)
        val new = post(ApiPaths.BIND, guest.token, BindEmailRequest("new@example.com", code, "AnotherUser")).body<BindResponse>()
        assertEquals("AnotherUser", new.profile.nickname)
        val returning = session()
        put(ApiPaths.CONSENT, returning.token, ConsentUpdate("0.2", false))
        post(ApiPaths.EMAIL_LOGIN_CODE, returning.token, EmailCodeRequest("existing@example.com"))
        val loginCode = Regex("\\d{6}").find(TestEnv.awaitMail { it.to == "existing@example.com" }.subject)!!.value
        val logged = post(ApiPaths.EMAIL_LOGIN, returning.token, EmailLoginRequest("existing@example.com", loginCode, returning.deviceId)).body<BindResponse>()
        assertEquals(existing.profile.userId, logged.profile.userId)
        assertEquals(existing.profile.nickname, logged.profile.nickname)
    }
}
