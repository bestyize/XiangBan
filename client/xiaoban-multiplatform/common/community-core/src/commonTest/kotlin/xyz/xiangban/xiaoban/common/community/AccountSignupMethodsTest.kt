package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpMethod
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals

class AccountSignupMethodsTest {
    @Test
    fun `registration entries follow the account server configuration`() = runTest {
        val server = FakeServer()
        server.on("/api/auth/methods", body = """{"email":false,"phone":true}""")
        assertEquals(AccountSignupMethods(email = false, phone = true), AccountApi(server.client, CommunityConfig()).signupMethods())
        assertEquals(HttpMethod.Get, server.requests.single().method)
    }

    @Test
    fun `unavailable or malformed settings keep phone registration hidden`() = runTest {
        val offline = FakeServer()
        offline.onThrow("/api/auth/methods")
        assertEquals(AccountSignupMethods(), AccountApi(offline.client, CommunityConfig()).signupMethods())
        val malformed = FakeServer()
        malformed.on("/api/auth/methods", body = "not json")
        assertEquals(AccountSignupMethods(), AccountApi(malformed.client, CommunityConfig()).signupMethods())
    }
}
