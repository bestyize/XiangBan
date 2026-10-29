package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
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
            respond(text, HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
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
            respond(text, HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
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
