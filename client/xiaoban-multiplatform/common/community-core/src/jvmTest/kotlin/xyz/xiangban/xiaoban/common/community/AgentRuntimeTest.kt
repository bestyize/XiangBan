package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonObject

/** 接真正的 Rust 内核库（AGENT_MOBILE_LIB 没设就跳过）：手机侧的模型请求和工具由测试里的假实现应答。 */
class AgentRuntimeTest {
    private class JvmBridge : AgentBridge {
        override fun start(config: String) = AgentNative.nativeStart(config)
        override fun send(handle: Long, line: String) = AgentNative.nativeSend(handle, line)
        override fun poll(handle: Long, timeoutMs: Long) = AgentNative.nativePoll(handle, timeoutMs)
        override fun stop(handle: Long) = AgentNative.nativeStop(handle)
    }

    private class Reminder : AgentTool {
        val set = mutableListOf<String>()
        override val name = "set_reminder"
        override val description = "设一个提醒"
        override val parameters: JsonObject = buildJsonObject {
            put("type", "object")
            putJsonObject("properties") {
                putJsonObject("title") { put("type", "string") }
                putJsonObject("minutes") { put("type", "integer") }
            }
        }
        override suspend fun run(arguments: JsonObject): AgentToolResult {
            set += arguments["title"]?.jsonPrimitive?.contentOrNull.orEmpty()
            return AgentToolResult(true, "已设置")
        }
    }

    private fun lib() = System.getenv("AGENT_MOBILE_LIB")?.takeIf { it.isNotBlank() }

    @Test
    fun `a run asks, runs the phone tool after approval and streams the answer`() = runBlocking {
        if (lib() == null) return@runBlocking
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val tool = Reminder()
        val bodies = mutableListOf<String>()
        val model: suspend (String, suspend (String) -> Unit) -> ModelResult = { body, onLine ->
            bodies += body
            if (bodies.size == 1) {
                onLine("""data: {"choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"c1","type":"function","function":{"name":"set_reminder","arguments":"{\"title\":\"喝水\",\"minutes\":10}"}}]},"finish_reason":"tool_calls"}]}""")
                onLine("")
            } else {
                onLine("""data: {"choices":[{"index":0,"delta":{"content":"好的，"}}]}""")
                onLine("")
                onLine("""data: {"choices":[{"index":0,"delta":{"content":"10 分钟后叫你。"},"finish_reason":"stop"}]}""")
                onLine("")
            }
            onLine("data: [DONE]")
            onLine("")
            ModelResult(200)
        }
        val dir = java.nio.file.Files.createTempDirectory("agent-runtime").toString()
        val runtime = AgentRuntime(JvmBridge(), model, listOf(tool), dir, "你是小伴", scope)
        val seen = mutableListOf<AgentEvent>()
        scope.launch {
            runtime.events.collect { ev ->
                synchronized(seen) { seen += ev }
                if (ev is AgentEvent.Approval) runtime.answerApproval(ev.id, "once")
            }
        }
        delay(100)
        runtime.ensureStarted("standard")
        runtime.run("s1", "十分钟后提醒我喝水")
        withTimeout(20_000) { while (synchronized(seen) { seen.none { it is AgentEvent.RunEnd } }) delay(50) }
        runtime.stop()

        val events = synchronized(seen) { seen.toList() }
        val approval = events.filterIsInstance<AgentEvent.Approval>().single()
        assertEquals("set_reminder", approval.tool)
        assertEquals(listOf("喝水"), tool.set)
        val end = events.filterIsInstance<AgentEvent.RunEnd>().single()
        assertTrue(end.ok, "$end")
        assertEquals("好的，10 分钟后叫你。", events.filterIsInstance<AgentEvent.Text>().joinToString("") { it.text })
        assertEquals(2, bodies.size)
        assertTrue(bodies[1].contains("已设置"), "第二次请求带着工具结果")
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
    }

    @Test
    fun `a model failure ends the run with the server's message`() = runBlocking {
        if (lib() == null) return@runBlocking
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val model: suspend (String, suspend (String) -> Unit) -> ModelResult = { _, _ ->
            ModelResult(402, body = """{"error":{"message":"额度不足，请兑换","code":"insufficient_quota"}}""")
        }
        val dir = java.nio.file.Files.createTempDirectory("agent-runtime").toString()
        val runtime = AgentRuntime(JvmBridge(), model, emptyList(), dir, "你是小伴", scope)
        val seen = mutableListOf<AgentEvent>()
        scope.launch { runtime.events.collect { synchronized(seen) { seen += it } } }
        delay(100)
        runtime.ensureStarted("standard")
        runtime.run("s1", "你好")
        withTimeout(20_000) { while (synchronized(seen) { seen.none { it is AgentEvent.RunEnd } }) delay(50) }
        runtime.stop()
        val end = synchronized(seen) { seen.filterIsInstance<AgentEvent.RunEnd>().single() }
        assertTrue(!end.ok)
        assertTrue(end.error.orEmpty().contains("额度不足"), "$end")
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
    }
}
