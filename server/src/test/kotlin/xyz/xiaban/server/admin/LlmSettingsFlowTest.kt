package xyz.xiaban.server.admin

import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class LlmSettingsFlowTest {

    @Test
    fun `llm settings require admin and persist encrypted with key redacted`() = apiTest {
        assertEquals(HttpStatusCode.Unauthorized, client.get("/api/admin/llm").status)

        val login = client.post("/api/admin/login") {
            contentType(ContentType.Application.Json)
            setBody(AdminLogin("testadmin", "test-admin-password-2026"))
        }
        assertEquals(HttpStatusCode.OK, login.status)
        val cookie = login.headers[HttpHeaders.SetCookie]!!.substringBefore(';')

        val initial = client.get("/api/admin/llm") {
            header(HttpHeaders.Cookie, cookie)
        }.body<LlmSettings>()
        // By default disabled in test environment unless enabled via settings
        assertFalse(initial.enabled)

        val newSettings = LlmSettings(
            enabled = true,
            activeProviderId = "deepseek",
            providers = listOf(
                LlmProviderConfig(
                    id = "deepseek",
                    name = "DeepSeek V3",
                    baseUrl = "https://api.deepseek.com/v1",
                    apiKey = "sk-test-secret-key-12345",
                    model = "deepseek-chat"
                )
            )
        )

        val saved = client.put("/api/admin/llm") {
            header(HttpHeaders.Cookie, cookie)
            contentType(ContentType.Application.Json)
            setBody(newSettings)
        }.body<LlmSettings>()

        assertEquals("deepseek", saved.activeProviderId)
        assertEquals("", saved.providers[0].apiKey)
        assertTrue(saved.providers[0].apiKeyConfigured)

        val fetched = client.get("/api/admin/llm") {
            header(HttpHeaders.Cookie, cookie)
        }.body<LlmSettings>()
        assertEquals("", fetched.providers[0].apiKey)
        assertTrue(fetched.providers[0].apiKeyConfigured)

        // Updating without sending apiKey preserves the existing key
        val updateWithoutKey = fetched.copy(
            providers = fetched.providers.map { it.copy(model = "deepseek-reasoner") }
        )
        val updated = client.put("/api/admin/llm") {
            header(HttpHeaders.Cookie, cookie)
            contentType(ContentType.Application.Json)
            setBody(updateWithoutKey)
        }.body<LlmSettings>()
        assertEquals("deepseek-reasoner", updated.providers[0].model)
        assertTrue(updated.providers[0].apiKeyConfigured)

        val internal = services.llmSettings.current()
        assertEquals("sk-test-secret-key-12345", internal.providers[0].apiKey)
    }
}
