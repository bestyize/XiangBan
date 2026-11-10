package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import io.ktor.http.content.TextContent
import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.cancel
import kotlin.test.assertFalse
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout

/** 整条链：对话控制器 → 智能体运行时 → 真正的 Rust 内核（JNI）→ 假的 server-ai（MockEngine）和假的提醒调度。 */
class AgentChatTest {
    @Test
    fun `creation retry and free text cannot start the native agent before confirmation`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val starts = AtomicInteger()
        val plans = AtomicInteger()
        val bridge = object : AgentBridge {
            override fun start(config: String) = starts.incrementAndGet().toLong()
            override fun send(handle: Long, line: String) = Unit
            override fun poll(handle: Long, timeoutMs: Long): String? { Thread.sleep(20); return null }
            override fun stop(handle: Long) = Unit
        }
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath != "/v1/chat/completions") return@MockEngine respond("{}")
            val body = (request.body as TextContent).text
            assertFalse(body.contains("\"tools\""))
            if (plans.incrementAndGet() == 1) return@MockEngine respond(
                """{"error":{"message":"暂不可用"}}""", HttpStatusCode.BadGateway,
            )
            respond(sse("""{"choices":[{"delta":{"content":"先做地图，再测试。"},"finish_reason":"stop"}]}"""),
                headers = headersOf(HttpHeaders.ContentType, "text/event-stream"))
        })
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, emptyList(), "", "", scope)
        val store = ChatHistory(InMemoryChatHistoryFiles(), scope)
        val chat = CloudChatController(AiGatewayApi(client) { "token" }, scope, agent = runtime, history = store)
        suspend fun idle() = withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
        try {
            assertTrue(chat.startCreation("潮汐灯塔", "制作潮汐灯塔", "三张地图"))
            idle()
            assertFalse(chat.state.value.creationPlanReady)
            chat.retryLast()
            idle()
            assertTrue(chat.state.value.creationPlanReady)
            val conversation = chat.state.value.conversationId
            chat.clearConversation()
            chat.openConversation(conversation)
            assertTrue(chat.state.value.planningCreation)
            chat.send("请接着上一次任务继续")
            idle()
            assertEquals(0, starts.get())
            assertEquals(3, plans.get())
            chat.confirmCreationPlan()
            withTimeout(5_000) { while (starts.get() == 0) delay(10) }
            assertTrue(chat.state.value.creation?.approved == true)
        } finally {
            chat.stop()
            runtime.stop()
            client.close()
            scope.cancel()
        }
    }

    @Test
    fun `an unexpectedly closed engine ends the turn and can be restarted`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val closed = java.util.concurrent.atomic.AtomicBoolean(false)
        val starts = java.util.concurrent.atomic.AtomicInteger()
        val bridge = object : AgentBridge {
            override fun start(config: String) = starts.incrementAndGet().toLong()
            override fun send(handle: Long, line: String) { closed.set(true) }
            override fun poll(handle: Long, timeoutMs: Long): String? {
                Thread.sleep(20)
                return if (closed.getAndSet(false)) """{"type":"closed"}""" else null
            }
            override fun stop(handle: Long) = Unit
        }
        val api = AiGatewayApi(HttpClient(MockEngine { respond("{}") }), CommunityConfig()) { "token" }
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, emptyList(), "", "", scope)
        val chat = CloudChatController(api, scope, agent = runtime)
        delay(100) // 等控制器开始接收引擎事件。
        chat.send("你好")
        withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
        assertTrue(chat.state.value.messages.last().error.orEmpty().contains("意外退出"))
        chat.retryLast()
        withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
        assertEquals(2, starts.get())
        runtime.stop()
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `a failed native send ends the turn with an error instead of waiting forever`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val bridge = object : AgentBridge {
            override fun start(config: String) = 1L
            override fun send(handle: Long, line: String) { throw IllegalStateException("引擎发送失败") }
            override fun poll(handle: Long, timeoutMs: Long): String? { Thread.sleep(20); return null }
            override fun stop(handle: Long) = Unit
        }
        val api = AiGatewayApi(HttpClient(MockEngine { respond("{}") }), CommunityConfig()) { "token" }
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, emptyList(), "", "", scope)
        val chat = CloudChatController(api, scope, agent = runtime)
        chat.send("你好")
        withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
        assertTrue(chat.state.value.messages.last().error.orEmpty().contains("引擎发送失败"))
        runtime.stop()
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `a protocol error clears the active run so retry does not deadlock`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val queue = java.util.concurrent.LinkedBlockingQueue<String>()
        val runs = java.util.concurrent.atomic.AtomicInteger()
        val lastSent = java.util.concurrent.atomic.AtomicReference("")
        val bridge = object : AgentBridge {
            override fun start(config: String) = 1L
            override fun send(handle: Long, line: String) {
                if (!line.contains("\"type\":\"run\"")) return
                lastSent.set(line)
                if (runs.incrementAndGet() == 1) {
                    queue.put("""{"type":"error","message":"invalid message: unknown field"}""")
                } else {
                    val runId = Regex("\"id\":\"([^\"]+)\"").find(line)!!.groupValues[1]
                    queue.put("""{"type":"run_end","run":"$runId","ok":true,"text":"已恢复","turns":1,"finish":"stop"}""")
                }
            }
            override fun poll(handle: Long, timeoutMs: Long): String? =
                queue.poll(timeoutMs, java.util.concurrent.TimeUnit.MILLISECONDS)
            override fun stop(handle: Long) = Unit
        }
        val api = AiGatewayApi(HttpClient(MockEngine { respond("{}") }), CommunityConfig()) { "token" }
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, emptyList(), "", "", scope)
        val chat = CloudChatController(api, scope, agent = runtime)
        delay(50)
        chat.send("你好")
        withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
        assertTrue(chat.state.value.messages.last().error.orEmpty().contains("invalid message"))
        chat.retryLast()
        withTimeout(5_000) { while (chat.state.value.sending) delay(10) }
        assertEquals(null, chat.state.value.messages.last().error)
        assertEquals("已恢复", chat.state.value.messages.last().text)
        assertTrue(lastSent.get().contains("\"input\":\"你好\""), "未真正启动的一轮重试时重新携带输入")
        runtime.stop()
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    private class JvmBridge : AgentBridge {
        override fun start(config: String) = AgentNative.nativeStart(config)
        override fun send(handle: Long, line: String) = AgentNative.nativeSend(handle, line)
        override fun poll(handle: Long, timeoutMs: Long) = AgentNative.nativePoll(handle, timeoutMs)
        override fun stop(handle: Long) = AgentNative.nativeStop(handle)
    }

    private fun sse(vararg chunks: String) = chunks.joinToString("\n\n") { "data: $it" } + "\n\ndata: [DONE]\n\n"

    @Test
    fun `chat asks before setting a reminder, runs it, and shows the tool and the answer`() = runBlocking {
        if (System.getenv("AGENT_MOBILE_LIB").isNullOrBlank()) return@runBlocking
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        var calls = 0
        val client = HttpClient(MockEngine { request ->
            // 对话结束后会顺手刷新余额，那不是模型请求。
            if (request.url.encodedPath != "/v1/chat/completions") return@MockEngine respond("{}", HttpStatusCode.OK)
            calls++
            val text = if (calls == 1) {
                sse("""{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"c1","type":"function","function":{"name":"set_reminder","arguments":"{\"title\":\"喝水\",\"minutes\":10}"}}]},"finish_reason":"tool_calls"}]}""")
            } else {
                sse("""{"choices":[{"index":0,"delta":{"content":"好的，10 分钟后叫你。"},"finish_reason":"stop"}]}""")
            }
            respond(text, HttpStatusCode.OK, headersOf(HttpHeaders.ContentType to listOf("text/event-stream"), "x-xiaoban-stream-terminal-version" to listOf("1")))
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "token" }
        val scheduled = mutableListOf<String>()
        val reminders = object : ReminderScheduler {
            override fun schedule(title: String, atEpochMs: Long): Boolean { scheduled += title; return true }
        }
        val dir = java.nio.file.Files.createTempDirectory("agent-chat").toString()
        val runtime = AgentRuntime(JvmBridge(), { body, onLine -> api.postChatLines(body, onLine) }, listOf(SetReminderTool(reminders) { 0L }), dir, "你是小伴", scope)
        val chat = CloudChatController(api, scope, agent = runtime)

        chat.send("十分钟后提醒我喝水")
        withTimeout(20_000) { while (chat.state.value.approval == null) delay(30) }
        val approval = assertNotNull(chat.state.value.approval)
        assertEquals("set_reminder", approval.tool)
        assertEquals("喝水 · 10 分钟后", approval.detail)
        assertEquals(emptyList(), scheduled, "同意之前不能执行")

        chat.answerApproval("once")
        withTimeout(20_000) { while (chat.state.value.sending) delay(30) }
        val state = chat.state.value
        assertEquals(listOf("喝水"), scheduled)
        assertEquals("好的，10 分钟后叫你。", state.messages.last().text)
        assertEquals(null, state.messages.last().error)
        val tool = state.messages.last().tools.single()
        assertEquals(ToolStatus.Done, tool.status, "$tool")
        assertEquals("set_reminder", tool.name)
        assertEquals(listOf("已允许"), state.messages.last().records.map { it.result })
        assertEquals(2, calls, "一次调用工具、一次给出最终回复")
        runtime.stop()
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
    }

    @Test
    fun `a refused reminder is not run`() = runBlocking {
        if (System.getenv("AGENT_MOBILE_LIB").isNullOrBlank()) return@runBlocking
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        var calls = 0
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath != "/v1/chat/completions") return@MockEngine respond("{}", HttpStatusCode.OK)
            calls++
            val text = if (calls == 1) {
                sse("""{"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"c1","type":"function","function":{"name":"set_reminder","arguments":"{\"title\":\"x\",\"minutes\":5}"}}]},"finish_reason":"tool_calls"}]}""")
            } else {
                sse("""{"choices":[{"index":0,"delta":{"content":"好的，没设。"},"finish_reason":"stop"}]}""")
            }
            respond(text, HttpStatusCode.OK, headersOf(HttpHeaders.ContentType to listOf("text/event-stream"), "x-xiaoban-stream-terminal-version" to listOf("1")))
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "token" }
        val scheduled = mutableListOf<String>()
        val reminders = object : ReminderScheduler {
            override fun schedule(title: String, atEpochMs: Long): Boolean { scheduled += title; return true }
        }
        val dir = java.nio.file.Files.createTempDirectory("agent-chat").toString()
        val runtime = AgentRuntime(JvmBridge(), { body, onLine -> api.postChatLines(body, onLine) }, listOf(SetReminderTool(reminders) { 0L }), dir, "你是小伴", scope)
        val chat = CloudChatController(api, scope, agent = runtime)
        chat.send("提醒我")
        withTimeout(20_000) { while (chat.state.value.approval == null) delay(30) }
        chat.answerApproval("deny")
        withTimeout(20_000) { while (chat.state.value.sending) delay(30) }
        assertEquals(emptyList(), scheduled)
        assertEquals("好的，没设。", chat.state.value.messages.last().text)
        runtime.stop()
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
    }
}
