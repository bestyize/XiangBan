package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class CustomModelCatalogTest {
    @Test fun `separate providers and models keep their own endpoint credentials and protocol`() = runBlocking {
        val requests = mutableListOf<Triple<String, String?, String>>()
        val client = HttpClient(MockEngine { request ->
            requests += Triple(request.url.toString(), request.headers[HttpHeaders.Authorization] ?: request.headers["x-api-key"], (request.body as TextContent).text)
            respond("""{"choices":[{"message":{"content":"ok"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val storage = InMemoryCustomModelStorage()
        val api = AiGatewayApi(client, customModels = storage) { error("official token must not be requested") }
        api.saveCustomProvider(CustomModelConfig("https://first.example/v1", "first-key", "same", id = "first", name = "First", models = listOf("same", "第二模型")))
        api.saveCustomProvider(CustomModelConfig("https://second.example/v1", "second-key", "same", protocol = "anthropic", id = "second", name = "Second", models = listOf("same")))
        val first = customModelTier("first", "第二模型")
        val second = customModelTier("second", "same")
        assertEquals(200, api.postChatRaw("""{"model":"$first","messages":[]}""") { }.status)
        assertEquals(200, api.postChatRaw("""{"model":"$second","messages":[]}""") { }.status)
        assertEquals("https://first.example/v1/chat/completions", requests[0].first)
        assertEquals("Bearer first-key", requests[0].second)
        assertTrue(requests[0].third.contains("第二模型"))
        assertEquals("https://second.example/v1/messages", requests[1].first)
        assertEquals("second-key", requests[1].second)
        assertTrue(requests[1].third.contains("\"model\":\"same\""))
        api.deleteCustomProvider("first")
        assertEquals(0, api.postChatRaw("""{"model":"$first"}""") { }.status)
        assertEquals(2, requests.size, "a removed selection must not reach another service or the gateway")
        client.close()
    }

    @Test fun `old configuration and old conversation selection survive migration and default changes`() {
        val preferences = mutableMapOf("custom_model_config" to """{"baseUrl":"https://old.example/v1","apiKey":"old-key","model":"old-model"}""")
        val storage = PreferencesCustomModelStorage(preferences::get) { key, value -> preferences[key] = value }
        val api = AiGatewayApi(HttpClient(MockEngine { respond("{}") }), customModels = storage) { null }
        assertEquals("old-model", api.customForTier(CUSTOM_MODEL_TIER)?.model)
        assertEquals("https://old.example/v1|old-model", CloudChatController(api, CoroutineScope(Dispatchers.Unconfined)).state.value.legacyCustomTier?.localSource)
        api.saveCustomProvider(CustomModelConfig("https://new.example/v1", "new-key", "new-model", id = "new", models = listOf("new-model")))
        val legacy = assertNotNull(api.customProvider("legacy"))
        api.saveCustomProvider(legacy.copy(model = "another", models = listOf("old-model", "another")))
        val restarted = AiGatewayApi(HttpClient(MockEngine { respond("{}") }), customModels = PreferencesCustomModelStorage(preferences::get) { k, v -> preferences[k] = v }) { null }
        assertEquals("old-model", restarted.customForTier(CUSTOM_MODEL_TIER)?.model)
        assertEquals("another", restarted.customForTier(customModelTier("legacy", "another"))?.model)
        assertEquals("old-key", restarted.customForTier(CUSTOM_MODEL_TIER)?.apiKey)
        restarted.deleteCustomProvider("legacy")
        assertNull(restarted.customForTier(CUSTOM_MODEL_TIER))
        assertEquals("new-model", restarted.customForTier(customModelTier("new", "new-model"))?.model)
        assertTrue(preferences.getValue("custom_model_config").contains("old-model"), "migration does not overwrite the old preference")
    }

    @Test fun `selection persists and remains scoped to its provider across new conversations`() = runBlocking {
        val prefs = mutableMapOf<String, String>()
        val storage = PreferencesCustomModelStorage(prefs::get) { key, value -> prefs[key] = value }
        val client = HttpClient(MockEngine { respond("{}") })
        val api = AiGatewayApi(client, customModels = storage) { null }
        val chat = CloudChatController(api, this)
        chat.useAccount(null)
        chat.saveCustomProvider(CustomModelConfig("https://one.example", "a", "a1", id = "one", models = listOf("a1", "a2")))
        chat.saveCustomProvider(CustomModelConfig("https://two.example", "b", "b1", id = "two", models = listOf("b1", "b2")))
        val selected = customModelTier("one", "a2")
        chat.selectTier(selected)
        chat.clearConversation()
        assertEquals(selected, chat.state.value.selectedTier)
        val resumed = CloudChatController(AiGatewayApi(client, customModels = storage) { null }, this)
        resumed.useAccount(null)
        assertEquals(selected, resumed.state.value.selectedTier)
        resumed.deleteCustomProvider("two")
        assertEquals(selected, resumed.state.value.selectedTier)
        resumed.deleteCustomProvider("one")
        assertNull(resumed.state.value.activeTier)
        client.close()
    }

    @Test fun `corrupt saved catalog is reported instead of replacing the old configuration`() {
        val prefs = mutableMapOf("custom_model_catalog_v2" to "{broken", "custom_model_config" to "original")
        val storage = PreferencesCustomModelStorage(prefs::get) { key, value -> prefs[key] = value }
        assertFailsWith<IllegalStateException> { storage.loadCatalog() }
        assertEquals("original", prefs["custom_model_config"])
    }

    @Test fun `bad local configuration does not prevent official chat initialization`() = runBlocking {
        val prefs = mutableMapOf("custom_model_catalog_v2" to "{broken")
        val storage = PreferencesCustomModelStorage(prefs::get) { key, value -> prefs[key] = value }
        val client = HttpClient(MockEngine { respond("{}") })
        val chat = CloudChatController(AiGatewayApi(client, customModels = storage) { null }, this)
        chat.useAccount("owner")
        assertNotNull(chat.state.value.customConfigError)
        chat.selectTier("official")
        assertEquals("official", chat.state.value.selectedTier)
        assertFailsWith<IllegalStateException> { chat.saveCustomProvider(CustomModelConfig("https://one.example", "key", "model", id = "one")) }
        assertEquals("{broken", prefs["custom_model_catalog_v2"])
        client.close()
    }

    @Test fun `each saved conversation restores its own provider and model`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val storage = InMemoryCustomModelStorage()
        val files = InMemoryChatHistoryFiles()
        val client = HttpClient(MockEngine { respond("""{"choices":[{"message":{"content":"ok"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json")) })
        val api = AiGatewayApi(client, customModels = storage) { null }
        val history = ChatHistory(files, scope)
        val chat = CloudChatController(api, scope, history = history)
        try {
            chat.useAccount("owner")
            chat.saveCustomProvider(CustomModelConfig("https://one.example", "key-a", "a1", id = "one", models = listOf("a1", "a2")))
            chat.saveCustomProvider(CustomModelConfig("https://two.example", "key-b", "b1", id = "two", models = listOf("b1")))
            val firstTier = customModelTier("one", "a2")
            chat.selectTier(firstTier)
            chat.send("first")
            withTimeout(5_000) { while (chat.state.value.sending || chat.state.value.messages.lastOrNull()?.text != "ok") delay(10) }
            val first = chat.state.value.conversationId
            assertTrue(history.awaitSaved(first))
            assertEquals(firstTier, history.loadSnapshot(first)?.options?.tier, "first conversation snapshot")
            chat.clearConversation()
            chat.selectTier(customModelTier("two", "b1"))
            chat.send("second")
            withTimeout(5_000) { while (chat.state.value.sending || chat.state.value.messages.lastOrNull()?.text != "ok") delay(10) }
            val second = chat.state.value.conversationId
            assertTrue(first != second)
            assertTrue(history.awaitSaved(second))
            chat.openConversation(first)
            assertEquals(firstTier, chat.state.value.selectedTier)
            chat.openConversation(second)
            assertEquals(customModelTier("two", "b1"), chat.state.value.selectedTier)
            chat.deleteCustomProvider("one")
            chat.openConversation(first)
            assertEquals(firstTier, chat.state.value.selectedTier)
            assertNull(chat.state.value.activeTier)
            assertEquals(false, chat.sendWithAttachments("hello", emptyList()), "a removed model cannot silently route to another provider")
            assertTrue(chat.state.value.inputError.orEmpty().contains("已删除"))
        } finally { chat.stop(); scope.cancel(); client.close() }
    }
}
