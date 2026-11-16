package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.headersOf
import kotlinx.coroutines.delay
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class OfficialAiSwitchTest {
    private fun catalogue(enabled: Boolean? = null): String =
        """{"version":1,"refreshAfterSeconds":300,${enabled?.let { "\"xiaobanAi\":{\"enabled\":$it}," }.orEmpty()}"providers":[]}"""

    @Test fun `missing switch defaults on and disabled value survives restart and failed refresh`() = runBlocking {
        var result = catalogue()
        val client = HttpClient(MockEngine { respond(result) })
        val storage = InMemoryCustomModelStorage()
        val api = AiGatewayApi(client, customModels = storage) { "account-token" }
        assertTrue(api.customCatalog().officialAiEnabled)
        api.refreshModelCatalogue(true)
        assertTrue(api.customCatalog().officialAiEnabled)
        result = catalogue(false)
        api.refreshModelCatalogue(true)
        assertFalse(api.customCatalog().officialAiEnabled)
        val restarted = AiGatewayApi(client, customModels = storage) { "account-token" }
        assertFalse(restarted.customCatalog().officialAiEnabled)
        result = "{broken"
        assertFailsWith<Exception> { restarted.refreshModelCatalogue(true) }
        assertFalse(restarted.customCatalog().officialAiEnabled)
        result = catalogue(true)
        restarted.refreshModelCatalogue(true)
        assertTrue(restarted.customCatalog().officialAiEnabled)
        client.close()
    }

    @Test fun `disabling official model blocks both chat routes without affecting third party`() = runBlocking {
        var enabled = false
        val chatHosts = mutableListOf<Pair<String, String?>>()
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath.endsWith("/ai/providers")) respond(catalogue(enabled))
            else {
                chatHosts += request.url.host to request.headers[HttpHeaders.Authorization]
                respond("""{"choices":[{"message":{"content":"ok"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
            }
        })
        val api = AiGatewayApi(client) { "account-token" }
        api.saveCustomProvider(CustomModelConfig("https://third.example/v1", "third-key", "third-model", id = "third"))
        api.refreshModelCatalogue(true)
        val official = api.postChatRaw("""{"model":"official","messages":[]}""") {}
        assertTrue(official.status !in 200..299)
        assertFailsWith<ApiException> { api.streamChat(listOf(ChatMessage("user", "hello")), "official") }
        assertTrue(chatHosts.isEmpty(), "disabled official requests must not reach the gateway")
        val tier = customModelTier("third", "third-model")
        assertEquals(200, api.postChatRaw("""{"model":"$tier","messages":[]}""") {}.status)
        assertEquals("ok", api.streamChat(listOf(ChatMessage("user", "hello")), tier))
        assertEquals(listOf<Pair<String, String?>>("third.example" to "Bearer third-key", "third.example" to "Bearer third-key"), chatHosts)
        enabled = true
        api.refreshModelCatalogue(true)
        assertEquals(200, api.postChatRaw("""{"model":"official","messages":[]}""") {}.status)
        assertEquals("Bearer account-token", chatHosts.last().second)
        client.close()
    }

    @Test fun `open official conversation is blocked after refresh and third party can be selected`() = runBlocking {
        var enabled = true
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath.endsWith("/ai/providers")) respond(catalogue(enabled))
            else respond("""{"choices":[{"message":{"content":"third-party reply"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val api = AiGatewayApi(client) { "account-token" }
        val chat = CloudChatController(api, this)
        chat.useAccount(null)
        chat.selectTier("official")
        chat.refreshModelCatalogue(true)
        assertTrue(chat.state.value.officialAiEnabled)
        enabled = false
        chat.refreshModelCatalogue(true)
        assertFalse(chat.state.value.officialAiEnabled)
        assertFalse(chat.sendWithAttachments("hello", emptyList()))
        assertTrue(chat.state.value.inputError?.isNotBlank() == true)
        chat.saveCustomProvider(CustomModelConfig("https://third.example/v1", "third-key", "model", id = "third"))
        chat.selectTier(customModelTier("third", "model"))
        assertTrue(chat.sendWithAttachments("hello", emptyList()))
        withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
        assertEquals("third-party reply", chat.state.value.messages.last().text)
        client.close()
    }

    @Test fun `disabling stops an active official chat but not an active third party chat`() = runBlocking {
        var enabled = true
        val officialStarted = CompletableDeferred<Unit>()
        val finishOfficial = CompletableDeferred<Unit>()
        val thirdStarted = CompletableDeferred<Unit>()
        val finishThird = CompletableDeferred<Unit>()
        val client = HttpClient(MockEngine { request ->
            when {
                request.url.encodedPath.endsWith("/ai/providers") -> respond(catalogue(enabled))
                request.url.host == "third.example" -> {
                    thirdStarted.complete(Unit)
                    finishThird.await()
                    respond("""{"choices":[{"message":{"content":"ok"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
                }
                else -> {
                    officialStarted.complete(Unit)
                    finishOfficial.await()
                    respond("""{"choices":[{"message":{"content":"official"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
                }
            }
        })
        val api = AiGatewayApi(client) { "account-token" }
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val chat = CloudChatController(api, scope)
        try {
            chat.useAccount(null)
            chat.saveCustomProvider(CustomModelConfig("https://third.example/v1", "third-key", "model", id = "third"))
            chat.selectTier("official")
            assertTrue(chat.sendWithAttachments("hello", emptyList()))
            withTimeout(5_000) { officialStarted.await() }
            enabled = false
            withTimeout(5_000) { chat.refreshModelCatalogue(true) }
            assertFalse(chat.state.value.sending)
            finishOfficial.complete(Unit)
            chat.selectTier(customModelTier("third", "model"))
            enabled = true
            withTimeout(5_000) { chat.refreshModelCatalogue(true) }
            assertTrue(chat.sendWithAttachments("hello third", emptyList()))
            withTimeout(5_000) { thirdStarted.await() }
            enabled = false
            withTimeout(5_000) { chat.refreshModelCatalogue(true) }
            assertTrue(chat.state.value.sending)
            finishThird.complete(Unit)
            withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
            assertEquals("ok", chat.state.value.messages.last().text)
        } finally { finishOfficial.complete(Unit); finishThird.complete(Unit); chat.stop(); scope.cancel(); client.close() }
    }

    @Test fun `disabling prevents starting official image generation`() = runBlocking {
        val client = HttpClient(MockEngine { request ->
            check(request.url.encodedPath.endsWith("/ai/providers")) { "image generation must not reach the gateway" }
            respond(catalogue(false))
        })
        val api = AiGatewayApi(client) { "account-token" }
        val chat = CloudChatController(api, this)
        chat.refreshModelCatalogue(true)
        assertFalse(chat.generateImages("hello", 1, null))
        assertFailsWith<ApiException> { api.generateImages("hello", 1, null) }
        client.close()
    }
}
