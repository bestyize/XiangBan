package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import kotlin.test.*

class MobileReliabilityTest {
    private suspend fun await(condition: () -> Boolean) = withTimeout(5000) { while (!condition()) delay(10) }
    private fun gateway() = AiGatewayApi(HttpClient(MockEngine { request ->
        when (request.url.encodedPath) {
            "/api/ai/tiers", "/tiers" -> respond("""{"tiers":[]}""")
            else -> respond("data: {\"choices\":[{\"delta\":{\"content\":\"回答\"}}]}\n\ndata: [DONE]\n", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
        }
    }), CommunityConfig()) { "token" }

    @Test
    fun `retired tiers and explicit text only capability block unsupported input`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Unconfined)
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/api/ai/tiers" -> respond("""{"tiers":[{"tier":"text","display_name":"Text","supports_vision":false}]}""")
                "/api/ai/balance" -> respond("""{"total":1}""")
                else -> error("unsupported input reached the gateway")
            }
        })
        try {
            val store = ChatHistory(InMemoryChatHistoryFiles(), scope)
            val chat = CloudChatController(AiGatewayApi(client) { "token" }, scope, history = store)
            chat.useAccount("owner")
            store.save("old", listOf(ChatTurn(1, "user", "hi")), 1, ChatOptions(tier = "retired"))
            chat.loadMeta()
            await { chat.state.value.tiers.isNotEmpty() }
            chat.openConversation("old")
            assertEquals("retired", chat.state.value.selectedTier)
            assertFalse(chat.sendWithAttachments("hello", emptyList()))
            assertTrue(chat.state.value.inputError.orEmpty().contains("下架"))
            chat.selectTier("text")
            assertFalse(chat.sendWithAttachments("view", listOf(ChatAttachment("pic.png", imageDataUrl = "data:image/png;base64,AAAA"))))
            assertTrue(chat.state.value.inputError.orEmpty().contains("不支持图片"))
        } finally {
            scope.cancel()
            client.close()
        }
    }

    @Test
    fun `a late quiet balance response cannot replace the new accounts balance`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Unconfined)
        val started = CompletableDeferred<Unit>()
        val release = CompletableDeferred<Unit>()
        var token = "A"
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/api/ai/balance" -> {
                    if (request.headers[HttpHeaders.Authorization] == "Bearer A") {
                        started.complete(Unit)
                        withContext(NonCancellable) { release.await() }
                        respond("""{"total":111}""")
                    } else respond("""{"total":222}""")
                }
                "/api/ai/tiers" -> respond("""{"tiers":[]}""")
                else -> respond(
                    """data: {"choices":[{"delta":{"content":"ok"}}]}""" + "\n\ndata: [DONE]\n",
                    headers = headersOf(HttpHeaders.ContentType, "text/event-stream"),
                )
            }
        })
        try {
            val chat = CloudChatController(AiGatewayApi(client) { token }, scope)
            chat.useAccount("A")
            chat.send("balance")
            withTimeout(5_000) { started.await() }
            token = "B"
            chat.useAccount("B")
            chat.loadMeta()
            await { chat.state.value.balance?.total == 222L }
            release.complete(Unit)
            delay(30)
            assertEquals(222L, chat.state.value.balance?.total)
        } finally {
            release.complete(Unit)
            scope.cancel()
            client.close()
        }
    }

    @Test
    fun `model permission reasoning and budgets belong to each saved conversation`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Unconfined)
        val files = InMemoryChatHistoryFiles()
        try {
            val history = ChatHistory(files, scope)
            history.useAccount("owner")
            val a = ChatOptions("model-a", mapOf("model-a" to "high"), 128_000, "auto", listOf("write_file"), AgentExecutionBudget(maxTurns = 7))
            val b = ChatOptions("model-b", contextTokens = 64_000)
            history.save("a", listOf(ChatTurn(1,"user","a")),1,a)
            history.save("b", listOf(ChatTurn(1,"user","b")),2,b)
            await { files.read("chats/owner/b.json") != null }
            val restored = ChatHistory(files, scope).apply { useAccount("owner") }
            assertEquals(a, restored.loadSnapshot("a")?.options)
            assertEquals(b, restored.loadSnapshot("b")?.options)
            val chat = CloudChatController(gateway(), scope, history = restored)
            chat.useAccount("owner")
            chat.openConversation("a")
            assertEquals("model-a",chat.state.value.selectedTier)
            assertEquals("auto",chat.state.value.permissionMode)
            assertEquals("high",chat.state.value.reasoningByTier["model-a"])
            chat.openConversation("b")
            assertEquals("model-b",chat.state.value.selectedTier)
            assertEquals("ask",chat.state.value.permissionMode)
            assertTrue(chat.state.value.reasoningByTier.isEmpty())
            chat.clearConversation()
            assertEquals(AgentExecutionBudget(),chat.state.value.budget)
            assertEquals("ask",chat.state.value.permissionMode)
            assertTrue(chat.state.value.grants.isEmpty())
        } finally { scope.cancel() }
    }

    @Test
    fun `legacy conversations load with safe options`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Unconfined)
        try {
            val files = InMemoryChatHistoryFiles()
            files.write("chats/u/old.json", """{"id":"old","messages":[{"id":1,"role":"user","text":"旧对话"}]}""")
            val store = ChatHistory(files, scope).apply { useAccount("u") }
            assertEquals(ChatOptions(),store.loadSnapshot("old")?.options)
            assertEquals("旧对话",store.load("old")?.single()?.text)
        } finally { scope.cancel() }
    }

    @Test
    fun `switching accounts stops and saves the old run before changing its storage namespace`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Unconfined)
        val files = InMemoryChatHistoryFiles()
        val client = HttpClient(MockEngine { delay(30_000); respond("{}") })
        try {
            val store = ChatHistory(files,scope)
            val chat = CloudChatController(AiGatewayApi(client,CommunityConfig()) { "token" },scope,history=store)
            chat.useAccount("first")
            chat.send("first account private text")
            val id = chat.state.value.conversationId
            assertTrue(chat.state.value.sending)
            chat.useAccount("second")
            assertTrue(chat.state.value.conversations.isEmpty())
            await { files.read("chats/first/$id.json") != null }
            assertNull(files.read("chats/second/$id.json"))
            assertTrue(chat.state.value.messages.isEmpty())
        } finally { client.close(); scope.cancel() }
    }

    @Test
    fun `failed saves stay readable and retry the latest version`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Unconfined)
        var fail = true
        val disk = InMemoryChatHistoryFiles()
        val files = object : ChatHistoryFiles {
            override fun read(name: String) = disk.read(name)
            override fun write(name: String, text: String) { check(!fail) { "disk full" }; disk.write(name,text) }
            override fun delete(name: String) { disk.delete(name) }
        }
        try {
            val store = ChatHistory(files,scope).apply { useAccount("u") }
            store.save("one",listOf(ChatTurn(1,"user","first")),1)
            await { store.saveError.value != null }
            assertEquals("first",store.load("one")?.single()?.text)
            store.save("one",listOf(ChatTurn(1,"user","latest")),2)
            fail = false
            store.retryPending()
            await { disk.read("chats/u/one.json")?.contains("latest") == true && store.saveError.value == null }
            assertEquals("latest",ChatHistory(files,scope).apply { useAccount("u") }.load("one")?.single()?.text)
        } finally { scope.cancel() }
    }

    @Test
    fun `cloud requests fit the byte budget without modifying saved images`() {
        val image = "data:image/jpeg;base64," + "A".repeat(2_400_000)
        val original = buildList {
            add(ChatMessage("system","你是小伴"))
            repeat(4) { add(ChatMessage("user","图片",listOf(ChatAttachment("ref.jpg",imageDataUrl=image)))); add(ChatMessage("assistant","收到")) }
            add(ChatMessage("user","请继续检查最后这张图",listOf(ChatAttachment("current.jpg",imageDataUrl=image))))
        }
        val body = boundedChatBody(original,"model","high",256_000)
        assertTrue(body.encodeToByteArray().size <= CHAT_BODY_BYTES)
        val parsed = CommunityJson.parseToJsonElement(body).jsonObject
        assertEquals("high", parsed["reasoning_effort"]?.jsonPrimitive?.content)
        assertTrue(parsed["messages"].toString().contains("请继续检查最后这张图"))
        assertEquals(5,original.count { it.attachments.any { a -> a.imageDataUrl == image } })
        assertTrue(body.contains("已从本次请求省略"))
    }

    @Test
    fun `impossible current inputs fail clearly rather than silently disappearing`() {
        assertFailsWith<ApiException> {
            boundedChatBody(listOf(ChatMessage("user","中".repeat(40_000))),null,null,16_000)
        }
        assertFailsWith<IllegalArgumentException> { AgentExecutionBudget(maxTurns = 0).validate() }
        assertFailsWith<IllegalArgumentException> { ChatOptions(contextTokens = 0).validate() }
    }
}
