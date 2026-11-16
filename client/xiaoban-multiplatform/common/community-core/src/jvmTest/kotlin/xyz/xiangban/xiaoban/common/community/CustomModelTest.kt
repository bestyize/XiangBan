package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.delay
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class CustomModelTest {
    @Test fun `validates addresses and builds compatible paths`() {
        fun config(url: String) = CustomModelConfig(url, "secret", "model").validated()
        assertEquals("https://example.com/v1/chat/completions", config("https://example.com/").chatUrl())
        assertEquals("https://example.com/v1/chat/completions", config("https://example.com/v1").chatUrl())
        assertEquals("https://example.com/api/chat/completions", config("https://example.com/api/chat/completions").chatUrl())
        assertEquals("http://192.168.1.2:11434/v1/chat/completions", config("http://192.168.1.2:11434").chatUrl())
        assertEquals("https://provider.example/api/paas/v4/chat/completions", config("https://provider.example/api/paas/v4").chatUrl())
        listOf("ftp://example.com", "https://user:pass@example.com", "https://example.com/?key=x", "https://example.com/#x").forEach {
            assertFailsWith<IllegalArgumentException> { config(it) }
        }
    }

    @Test fun `responses and anthropic plain chats use their own wire format and authentication`() = runBlocking {
        val requests = mutableListOf<Triple<String, Map<String, String>, String>>()
        var protocol = "responses"
        val client = HttpClient(MockEngine { request ->
            requests += Triple(request.url.toString(), request.headers.entries().associate { it.key to it.value.joinToString() }, (request.body as TextContent).text)
            val events = if (protocol == "responses")
                "data: {\"type\":\"response.output_text.delta\",\"delta\":\"你好\"}\n\ndata: {\"type\":\"response.completed\",\"response\":{\"output\":[{\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"你好\"}]}]}}\n\n"
            else "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"你好\"}}\n\ndata: {\"type\":\"message_stop\"}\n\n"
            respond(events, headers = headersOf(HttpHeaders.ContentType, "text/event-stream"))
        })
        val api = AiGatewayApi(client) { error("official token must not be requested") }
        for (kind in listOf("responses", "anthropic")) {
            protocol = kind
            api.saveCustomModel(CustomModelConfig("https://third.example/v1", "third-key", "m1", protocol = kind, maxOutputTokens = 2048))
            assertEquals("你好", api.streamChat(listOf(ChatMessage("system", "system"), ChatMessage("user", "hello")), CUSTOM_MODEL_TIER))
        }
        val response = requests[0]
        assertEquals("https://third.example/v1/responses", response.first)
        assertEquals("Bearer third-key", response.second["Authorization"])
        assertTrue(response.third.contains("\"input\"")); assertTrue(response.third.contains("\"max_output_tokens\":2048"))
        val anthropic = requests[1]
        assertEquals("https://third.example/v1/messages", anthropic.first)
        assertEquals("third-key", anthropic.second["x-api-key"])
        assertEquals("2023-06-01", anthropic.second["anthropic-version"])
        assertTrue(anthropic.third.contains("\"system\":\"system\"")); assertTrue(anthropic.third.contains("\"max_tokens\":2048"))
        assertFalse(anthropic.second.containsKey("Authorization"))
        client.close()
    }

    @Test fun `ordinary chat and agent requests use custom endpoint and key without account token`() = runBlocking {
        val requests = mutableListOf<Triple<String, String?, String>>()
        val client = HttpClient(MockEngine { request ->
            requests += Triple(request.url.toString(), request.headers[HttpHeaders.Authorization], (request.body as TextContent).text)
            respond("""{"choices":[{"message":{"content":"你好"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val storage = InMemoryCustomModelStorage()
        val api = AiGatewayApi(client, customModels = storage) { error("official token must not be requested") }
        api.saveCustomModel(CustomModelConfig("https://third.example/v1", "third-key", "deepseek-chat"))
        assertEquals("你好", api.streamChat(listOf(ChatMessage("user", "hi")), CUSTOM_MODEL_TIER))
        val agent = api.postChatRaw("""{"model":"$CUSTOM_MODEL_TIER","messages":[],"tools":[]}""") { }
        assertEquals(200, agent.status)
        assertEquals(2, requests.size)
        requests.forEach { (url, authorization, body) ->
            assertEquals("https://third.example/v1/chat/completions", url)
            assertEquals("Bearer third-key", authorization)
            assertTrue(body.contains("\"model\":\"deepseek-chat\""))
            assertFalse(body.contains(CUSTOM_MODEL_TIER))
        }
        api.saveCustomModel(null)
        assertEquals(0, api.postChatRaw("""{"model":"$CUSTOM_MODEL_TIER"}""") { }.status)
        client.close()
    }

    @Test fun `third party redirects do not forward credentials`() = runBlocking {
        var count = 0
        val client = HttpClient(MockEngine {
            count++
            respond("", HttpStatusCode.TemporaryRedirect, headersOf(HttpHeaders.Location, "https://elsewhere.example/steal"))
        })
        val api = AiGatewayApi(client) { "official-token" }
        api.saveCustomModel(CustomModelConfig("https://third.example", "third-key", "model"))
        assertEquals(307, api.postChatRaw("""{"model":"$CUSTOM_MODEL_TIER"}""") { }.status)
        assertEquals(1, count)
        client.close()
    }

    @Test fun `models can be discovered without sending a chat or an account token`() = runBlocking {
        val client = HttpClient(MockEngine { request ->
            assertEquals("https://third.example/v1/models", request.url.toString())
            assertEquals("Bearer custom-key", request.headers[HttpHeaders.Authorization])
            assertEquals("GET", request.method.value)
            respond("""{"data":[{"id":"first"},{"id":"second"},{"id":"first"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val api = AiGatewayApi(client) { error("official token must not be requested") }
        assertEquals(listOf("first", "second"), api.fetchCustomModels(CustomModelConfig("https://third.example/v1", "custom-key", "")))
        client.close()
    }

    @Test fun `agent only trusts a complete third party event stream`() = runBlocking {
        var complete = true
        val client = HttpClient(MockEngine {
            respond("data: {\"choices\":[]}\n\n" + if (complete) "data: [DONE]\n\n" else "", headers = headersOf(HttpHeaders.ContentType, "text/event-stream"))
        })
        val api = AiGatewayApi(client) { "official-token" }
        api.saveCustomModel(CustomModelConfig("https://third.example", "third-key", "model"))
        val body = """{"model":"$CUSTOM_MODEL_TIER"}"""
        assertTrue(api.postChatRaw(body) { }.verifiedToolTerminal)
        complete = false
        assertFalse(api.postChatRaw(body) { }.verifiedToolTerminal)
        client.close()
    }

    @Test fun `signed out user can select save and use a local model`() = runBlocking {
        val client = HttpClient(MockEngine {
            respond("""{"choices":[{"message":{"content":"本地模型回复"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val session = SessionManager(AccountApi(client, CommunityConfig()), InMemoryTokenStorage(), now = { 0L })
        val api = AiGatewayApi(client, sessions = session) { null }
        val chat = CloudChatController(api, this)
        chat.useAccount(null)
        chat.saveCustomModel(CustomModelConfig("https://third.example", "third-key", "model"))
        chat.clearConversation()
        assertEquals(CUSTOM_MODEL_TIER, chat.state.value.selectedTier)
        chat.send("你好")
        withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
        assertEquals("本地模型回复", chat.state.value.messages.last().text)
        chat.saveCustomModel(null)
        assertEquals(null, chat.state.value.customModelName)
        client.close()
    }
}
