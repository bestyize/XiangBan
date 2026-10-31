package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import kotlin.test.*

class MobileAgentExtensionsTest {
    private class Bridge : AgentBridge {
        val output = LinkedBlockingQueue<String>()
        val input = mutableListOf<JsonObject>()
        var run = ""
        override fun start(config: String): Long = 1L
        override fun poll(handle: Long, timeoutMs: Long): String? = output.poll(timeoutMs, TimeUnit.MILLISECONDS)
        override fun stop(handle: Long) = Unit
        override fun send(handle: Long, line: String) {
            val obj = Json.parseToJsonElement(line).jsonObject
            synchronized(input) { input += obj }
            when (obj["type"]?.jsonPrimitive?.content) {
                "run" -> {
                    run = obj.getValue("id").jsonPrimitive.content
                    output.put(buildJsonObject {
                        put("type", "question"); put("id", "q1"); put("session", obj.getValue("session"))
                        put("question", "用什么颜色？"); put("options", buildJsonArray { add(JsonPrimitive("红色")); add(JsonPrimitive("绿色")) })
                    }.toString())
                }
                "answer" -> output.put(buildJsonObject { put("type", "run_end"); put("run", run); put("ok", true); put("text", "已记录") }.toString())
            }
        }
    }

    @Test
    fun `image parts and question answers survive the Kotlin controller bridge`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val client = HttpClient(MockEngine { respond("{}") })
        val bridge = Bridge()
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, emptyList(), "test", "小伴", scope, utcOffsetMinutes = 480)
        val chat = CloudChatController(AiGatewayApi(client, CommunityConfig()) { "token" }, scope, agent = runtime, newSessionId = { "s1" })
        try {
            delay(50)
            val attachments = listOf(ChatAttachment("参考.png", imageDataUrl = "data:image/png;base64,AAAA"), ChatAttachment("规则.txt", text = "点击得分"))
            chat.sendWithAttachments("制作游戏", attachments)
            withTimeout(3_000) { while (chat.state.value.question == null) delay(10) }
            assertEquals(listOf("红色", "绿色"), chat.state.value.question?.options)
            val run = synchronized(bridge.input) { bridge.input.first { it["type"]?.jsonPrimitive?.content == "run" } }
            assertNull(run["input"], "stdio input and parts are mutually exclusive")
            assertEquals("image", run.getValue("parts").jsonArray.last().jsonObject["type"]?.jsonPrimitive?.content)
            assertTrue(run.toString().contains("点击得分"))
            chat.answerQuestion("绿色")
            withTimeout(3_000) { while (chat.state.value.sending) delay(10) }
            assertNull(chat.state.value.question)
            assertEquals("绿色", chat.state.value.messages.last().records.single().result)
            assertEquals(attachments, chat.state.value.messages.first().attachments)
        } finally { runtime.stop(); scope.cancel(); client.close() }
    }

    @Test
    fun `invalid and oversized attachments are rejected before sending`() {
        assertFailsWith<IllegalArgumentException> { validateChatAttachments(listOf(ChatAttachment("x", imageDataUrl = "https://example.org/x.png"))) }
        assertFailsWith<IllegalArgumentException> { validateChatAttachments(listOf(ChatAttachment("x", text = "a".repeat(256 * 1024 + 1)))) }
        assertFailsWith<IllegalArgumentException> { validateChatAttachments(List(4) { ChatAttachment("x", text = "ok") }) }
        assertFailsWith<IllegalArgumentException> { validateChatAttachments(listOf(ChatAttachment("x", text = "ok", imageDataUrl = "data:image/png;base64,AAAA"))) }
        assertFailsWith<IllegalArgumentException> { validateChatAttachments(listOf(ChatAttachment("x", imageDataUrl = "data:image/png;base64,AA=A"))) }
    }

    @Test
    fun `cloud fallback uses compatible image_url parts`() {
        val content = attachmentParts("参考它", listOf(ChatAttachment("ref.jpg", imageDataUrl = "data:image/jpeg;base64,AAAA")), chatCompletions = true)
        assertEquals("image_url", content.last().jsonObject.getValue("type").jsonPrimitive.content)
        assertEquals("data:image/jpeg;base64,AAAA", content.last().jsonObject.getValue("image_url").jsonObject.getValue("url").jsonPrimitive.content)
    }

    @Test
    fun `cancelled questions and late approvals cannot replace the next conversations prompt`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val client = HttpClient(MockEngine { respond("{}") })
        val bridge = Bridge()
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, emptyList(), "test", "小伴", scope)
        var session = 0
        val chat = CloudChatController(AiGatewayApi(client, CommunityConfig()) { "token" }, scope, agent = runtime, newSessionId = { "s${++session}" })
        try {
            delay(50)
            chat.send("第一轮")
            withTimeout(3_000) { while (chat.state.value.question == null) delay(10) }
            val oldSession = chat.state.value.conversationId
            val oldRun = bridge.run
            chat.clearConversation()
            chat.send("另一会话")
            withTimeout(3_000) { while (chat.state.value.question == null) delay(10) }
            bridge.output.put("""{"type":"question","id":"old","session":"$oldSession","question":"过期问题","options":[]}""")
            bridge.output.put("""{"type":"approval_request","id":"old","session":"$oldSession","tool":"write_file","risk":"write","arguments":{}}""")
            bridge.output.put("""{"type":"event","run":"$oldRun","event":{"type":"text_delta","text":"过期输出"}}""")
            delay(100)
            assertEquals("用什么颜色？", chat.state.value.question?.question)
            assertNull(chat.state.value.approval)
            assertFalse(chat.state.value.messages.last().text.contains("过期"))
            val assistant = CloudAssistant(chat)
            val prompt = assertNotNull(assistant.prompt())
            assertTrue(prompt.question)
            assistant.answerPrompt(prompt.options.first { it.label == "绿色" }.id)
            withTimeout(3_000) { while (chat.state.value.sending) delay(10) }
            assertEquals("绿色", chat.state.value.messages.last().records.single().result)
        } finally { runtime.stop(); scope.cancel(); client.close() }
    }

    @Test
    fun `a late tool result from a stopped engine never reaches the restarted engine`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val outputs = java.util.concurrent.ConcurrentHashMap<Long, LinkedBlockingQueue<String>>()
        val inputs = java.util.concurrent.CopyOnWriteArrayList<Pair<Long, JsonObject>>()
        val stops = java.util.concurrent.CopyOnWriteArrayList<Long>()
        var starts = 0L
        val bridge = object : AgentBridge {
            override fun start(config: String): Long = ++starts
            override fun send(handle: Long, line: String) { inputs += handle to Json.parseToJsonElement(line).jsonObject }
            override fun poll(handle: Long, timeoutMs: Long): String? = outputs.computeIfAbsent(handle) { LinkedBlockingQueue() }.poll(20, TimeUnit.MILLISECONDS)
            override fun stop(handle: Long) { stops += handle }
        }
        val entered = CompletableDeferred<Unit>()
        val release = CompletableDeferred<Unit>()
        val oldFinished = CompletableDeferred<Unit>()
        var calls = 0
        val tool = object : AgentTool {
            override val name = "slow"
            override val description = "test"
            override val parameters = buildJsonObject { put("type", "object") }
            override suspend fun run(arguments: JsonObject): AgentToolResult {
                if (++calls == 1) {
                    withContext(NonCancellable) { entered.complete(Unit); release.await() }
                    oldFinished.complete(Unit)
                    return AgentToolResult(true, "old")
                }
                return AgentToolResult(true, "new")
            }
        }
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, listOf(tool), "test", "小伴", scope)
        try {
            runtime.ensureStarted("")
            runtime.run("s1", "old", "r1")
            outputs.computeIfAbsent(1) { LinkedBlockingQueue() }.put("""{"type":"tool_request","id":"t1","name":"slow","arguments":{}}""")
            withTimeout(3_000) { entered.await() }
            runtime.stop()
            runtime.ensureStarted("")
            runtime.run("s1", "new", "r2")
            outputs.computeIfAbsent(2) { LinkedBlockingQueue() }.put("""{"type":"tool_request","id":"t1","name":"slow","arguments":{}}""")
            withTimeout(3_000) { while (inputs.none { it.first == 2L && it.second["type"]?.jsonPrimitive?.content == "tool_result" }) delay(10) }
            release.complete(Unit)
            withTimeout(3_000) { oldFinished.await() }
            delay(100)
            val results = inputs.filter { it.second["type"]?.jsonPrimitive?.content == "tool_result" }
            assertEquals(listOf("new"), results.map { it.second["output"]?.jsonPrimitive?.content })
            scope.cancel()
            withTimeout(3_000) { while (2L !in stops) delay(10) }
        } finally { release.complete(Unit); runtime.stop(); scope.cancel() }
    }

    @Test
    fun `cloud retry and following turns retain the selected image and text attachments`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val bodies = java.util.concurrent.CopyOnWriteArrayList<JsonObject>()
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath != "/v1/chat/completions") return@MockEngine respond("{}")
            bodies += Json.parseToJsonElement((request.body as TextContent).text).jsonObject
            if (bodies.size == 1) respond("""{"error":{"message":"暂时失败"}}""", HttpStatusCode.BadRequest)
            else respond("data: {\"choices\":[{\"delta\":{\"content\":\"收到\"}}]}\n\ndata: [DONE]\n\n")
        })
        val chat = CloudChatController(AiGatewayApi(client, CommunityConfig()) { "token" }, scope)
        val attachments = listOf(ChatAttachment("图.png", imageDataUrl = "data:image/png;base64,AAAA"), ChatAttachment("规则.txt", text = "点击得分"))
        try {
            chat.sendWithAttachments("参考这个", attachments)
            withTimeout(3_000) { while (chat.state.value.sending) delay(10) }
            assertNotNull(chat.state.value.messages.last().error)
            chat.retryLast()
            withTimeout(3_000) { while (chat.state.value.sending) delay(10) }
            assertNull(chat.state.value.messages.last().error)
            assertEquals(attachments, chat.state.value.messages.first().attachments)
            chat.send("继续")
            withTimeout(3_000) { while (chat.state.value.sending) delay(10) }
            assertEquals(3, bodies.size)
            for (body in bodies) {
                assertTrue(body.toString().contains("data:image/png;base64,AAAA"))
                assertTrue(body.toString().contains("点击得分"))
                assertTrue(body.toString().contains("image_url"))
            }
        } finally { scope.cancel(); client.close() }
    }
}
