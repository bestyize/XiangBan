package xyz.xiaban.server.account

import io.ktor.http.HttpStatusCode
import kotlin.test.*
import kotlinx.coroutines.future.await
import xyz.xiaban.contracts.account.consent.ConsentApiPaths
import xyz.xiaban.contracts.account.consent.ConsentUpdate
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.mail.RuntimeMailer
import xyz.xiaban.server.support.*

class MailFailureFlowTest {
    @Test fun `unconfigured mail and SMTP failures do not claim to send a usable code`() = apiTest {
        val guest = session()
        put(ConsentApiPaths.CONSENT, guest.token, ConsentUpdate("0.1", false))
        assertFalse(RuntimeMailer(services.emailSettings).available())
        val userId = services.tokens.userIdOrNull(guest.token)!!
        val unavailable = EmailBinding(services.redis, services.users, services.profiles, services.tokens,
            AccountMerge(services.db), { false }, { fail("must not deliver") }, {})
        val error = assertFailsWith<ApiException> { unavailable.sendCode(userId, "unavailable@example.com") }
        assertEquals(HttpStatusCode.ServiceUnavailable, error.status)
        val failing = EmailBinding(services.redis, services.users, services.profiles, services.tokens,
            AccountMerge(services.db), { true }, { error("SMTP rejected delivery") }, {})
        assertEquals("mail_send_failed", assertFailsWith<ApiException> { failing.sendCode(userId, "failure@example.com") }.code)
        assertTrue(services.redis.cmd.keys("xb:code:bind:*").await().isEmpty())
    }
}
