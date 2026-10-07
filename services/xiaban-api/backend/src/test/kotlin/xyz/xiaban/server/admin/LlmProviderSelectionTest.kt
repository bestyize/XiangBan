package xyz.xiaban.server.admin

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import xyz.xiaban.server.support.FakeLlm
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertSame

class LlmProviderSelectionTest {
    private val configured = LlmProviderConfig("primary", "Configured provider", "https://example.com/v1", "test-key", "test-model")

    @Test fun `paused service cannot fall back to environment providers when settings are empty`() {
        HttpClient(MockEngine { error("No request should be made") }).use { http ->
            assertNull(resolveLlmProvider(LlmSettings(enabled = false), listOf(FakeLlm()), http))
        }
    }

    @Test fun `paused service cannot fall back when a selected provider has no key`() {
        HttpClient(MockEngine { error("No request should be made") }).use { http ->
            val settings = LlmSettings(enabled = false, providers = listOf(configured.copy(apiKey = "")))
            assertNull(resolveLlmProvider(settings, listOf(FakeLlm()), http))
        }
    }

    @Test fun `paused configured provider cannot be selected`() {
        HttpClient(MockEngine { error("No request should be made") }).use { http ->
            assertNull(resolveLlmProvider(LlmSettings(enabled = false, providers = listOf(configured)), listOf(FakeLlm()), http))
        }
    }

    @Test fun `enabled service selects the configured active provider`() {
        HttpClient(MockEngine { error("No request should be made") }).use { http ->
            val other = configured.copy(id = "other", name = "Other provider")
            val settings = LlmSettings(enabled = true, activeProviderId = other.id, providers = listOf(configured, other))
            assertEquals(other.name, resolveLlmProvider(settings, listOf(FakeLlm()), http)?.name)
        }
    }

    @Test fun `enabled service retains environment fallback and handles no providers`() {
        HttpClient(MockEngine { error("No request should be made") }).use { http ->
            val fallback = FakeLlm()
            assertSame(fallback, resolveLlmProvider(LlmSettings(enabled = true), listOf(fallback), http))
            assertNull(resolveLlmProvider(LlmSettings(enabled = true), emptyList(), http))
        }
    }
}
