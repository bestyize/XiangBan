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
import kotlinx.coroutines.future.await
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.support.TestEnv
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.FlagsUpdate
import xyz.xiaban.shared.FlagsView
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlin.test.assertNull

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
            webSearchEnabled = true,
            webSearchBaseUrl = "http://43.155.99.113:50001",
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
        assertTrue(saved.webSearchEnabled)
        assertEquals("http://43.155.99.113:50001", saved.webSearchBaseUrl)
        assertEquals("", saved.providers[0].apiKey)
        assertTrue(saved.providers[0].apiKeyConfigured)
        assertTrue(services.flags.llmEnabled(), "saved admin settings must enable the chat and check-in gate")

        val fetched = client.get("/api/admin/llm") {
            header(HttpHeaders.Cookie, cookie)
        }.body<LlmSettings>()
        assertEquals("", fetched.providers[0].apiKey)
        assertTrue(fetched.providers[0].apiKeyConfigured)
        assertTrue(fetched.webSearchEnabled)
        assertEquals("http://43.155.99.113:50001", fetched.webSearchBaseUrl)

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

        val disabled = client.put("/api/admin/flags") {
            header(HttpHeaders.Cookie, cookie)
            contentType(ContentType.Application.Json)
            setBody(FlagsUpdate(llmEnabled = false))
        }.body<FlagsView>()
        assertFalse(disabled.llmEnabled)
        val paused = client.get("/api/admin/llm") { header(HttpHeaders.Cookie, cookie) }.body<LlmSettings>()
        assertFalse(paused.enabled)
        assertEquals("deepseek-reasoner", paused.providers[0].model)
        assertTrue(paused.providers[0].apiKeyConfigured)
        assertTrue(paused.webSearchEnabled)
        assertEquals("sk-test-secret-key-12345", services.llmSettings.current().providers[0].apiKey)
        assertNull(services.llmSettings.activeProvider())

        client.put("/api/admin/llm") {
            header(HttpHeaders.Cookie, cookie)
            contentType(ContentType.Application.Json)
            setBody(paused.copy(enabled = true))
        }
        assertTrue(services.flags.llmEnabled())
        assertEquals("sk-test-secret-key-12345", services.llmSettings.current().providers[0].apiKey)
    }

    @Test fun `admin enablement starts chat despite stale redis while disabling and reenabling take effect`() = apiTest {
        assertFalse(services.config.llm.enabledByDefault)
        services.redis.cmd.hset("xb:flags", "llm", "0").await()
        assertFalse(services.flags.llmEnabled())
        TestEnv.llm.reply = "收到，聊天已开启。"
        client.put("/api/admin/llm") {
            header("X-Admin-Token", TestEnv.ADMIN_TOKEN)
            contentType(ContentType.Application.Json)
            setBody(LlmSettings(enabled = true))
        }
        assertTrue(services.flags.llmEnabled())
        val reply = services.chat.stream(null, "你好") { }
        assertEquals("收到，聊天已开启。", reply)
        assertEquals(1, TestEnv.llm.calls.get())

        services.flags.update(FlagsUpdate(llmEnabled = false))
        assertFalse(services.llmSettings.current().enabled)
        assertTrue(services.chat.stream(null, "再聊聊") { }.contains("暂时关闭"))
        assertEquals(1, TestEnv.llm.calls.get(), "a paused service must make no model calls")

        services.llmSettings.save(services.llmSettings.current().copy(enabled = true))
        assertEquals("收到，聊天已开启。", services.chat.stream(null, "继续聊") { })
        assertEquals(2, TestEnv.llm.calls.get())
    }

    @Test fun `legacy redis switch bootstraps settings only until admin explicitly saves a switch`() = apiTest {
        services.redis.cmd.hset("xb:flags", "llm", "1").await()
        assertTrue(services.llmSettings.current().enabled)
        services.llmSettings.save(LlmSettings(enabled = false))
        assertFalse(services.llmSettings.current().enabled)
        assertFalse(services.flags.llmEnabled())
        assertNull(services.llmSettings.activeProvider())
    }

    @Test fun `unreadable saved settings disable calls instead of falling back to environment models`() = apiTest {
        services.db.tx { c -> c.execute("insert into runtime_settings(key,encrypted_value) values ('llm','invalid-encrypted-settings')") }
        assertFalse(services.llmSettings.current().enabled)
        assertFalse(services.flags.llmEnabled())
        assertNull(services.llmSettings.activeProvider())
    }
}
