package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.headersOf
import java.nio.file.Files
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import kotlin.test.*

class MobileCancellationTest {
    @Test
    fun `a transport cancellation ends the native run and allows the next message`() = runBlocking {
        if (System.getenv("AGENT_MOBILE_LIB").isNullOrBlank()) return@runBlocking
        val directory = Files.createTempDirectory("mobile-model-cancellation").toFile()
        val scope = CoroutineScope(coroutineContext + SupervisorJob())
        val bridge = object : AgentBridge {
            override fun start(config: String) = AgentNative.nativeStart(config)
            override fun send(handle: Long, line: String) = AgentNative.nativeSend(handle, line)
            override fun poll(handle: Long, timeoutMs: Long) = AgentNative.nativePoll(handle, timeoutMs)
            override fun stop(handle: Long) = AgentNative.nativeStop(handle)
        }
        val connected = AtomicBoolean(false)
        val client = HttpClient(MockEngine { request ->
            if (!request.url.encodedPath.endsWith("/chat/completions")) return@MockEngine respond("{}")
            if (!connected.get()) throw CancellationException("model transport independently cancelled")
            respond("""{"choices":[{"message":{"content":"服务已恢复"},"finish_reason":"stop"}]}""",
                headers = headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val api = AiGatewayApi(client) { "token" }
        val runtime = AgentRuntime(bridge, api::postChatRaw, emptyList(), directory.path, "小伴", scope)
        val chat = CloudChatController(api, scope, agent = runtime)
        suspend fun idle() = withTimeout(15_000) { while (chat.state.value.sending) delay(10) }
        try {
            chat.send("测试请求")
            idle()
            val interrupted = chat.state.value.messages.last()
            assertTrue(interrupted.error.orEmpty().contains("模型请求已中断"))
            assertFalse(interrupted.cancelled)
            assertFalse(interrupted.streaming)
            connected.set(true)
            chat.send("再次询问")
            idle()
            assertEquals("服务已恢复", chat.state.value.messages.last().text)
        } finally {
            runtime.stop()
            scope.cancel()
            client.close()
            directory.deleteRecursively()
        }
    }

    @Test
    fun `stopping a run cancels its host coroutine and reports uncertain mutations`() = runBlocking {
        val queue = LinkedBlockingQueue<String>()
        val replies = LinkedBlockingQueue<JsonObject>()
        val entered = CompletableDeferred<Unit>()
        val ended = CompletableDeferred<AgentEvent.RunEnd>()
        var committed = false
        val bridge = object : AgentBridge {
            override fun start(config: String) = 1L
            override fun send(handle: Long, line: String) {
                val msg = CommunityJson.parseToJsonElement(line).jsonObject
                when (msg["type"]?.jsonPrimitive?.content) {
                    "run" -> queue.add("""{"type":"tool_request","id":"t1","session":"s1","name":"slow_write","arguments":{}}""")
                    "cancel" -> queue.add("""{"type":"tool_cancel","id":"t1","session":"s1"}""")
                    "tool_result" -> {
                        replies.add(msg)
                        queue.add("""{"type":"run_end","run":"r1","ok":false,"error":"run paused by continuation guard: tool_outcome_needs_review"}""")
                    }
                }
            }
            override fun poll(handle: Long, timeoutMs: Long): String? = queue.poll(timeoutMs, TimeUnit.MILLISECONDS)
            override fun stop(handle: Long) = Unit
        }
        val tool = object : AgentTool {
            override val name = "slow_write"
            override val description = "test"
            override val parameters = buildJsonObject { put("type","object") }
            override suspend fun run(arguments: JsonObject): AgentToolResult {
                entered.complete(Unit)
                delay(30_000)
                committed = true
                return AgentToolResult(true,"written")
            }
        }
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val runtime = AgentRuntime(bridge,{ _, _ -> ModelResult(200) },listOf(tool),"","",scope)
        try {
            val collector = scope.launch(start = CoroutineStart.UNDISPATCHED) {
                runtime.events.collect { if (it is AgentEvent.RunEnd) ended.complete(it) }
            }
            runtime.ensureStarted("tier")
            runtime.run("s1","write","r1")
            withTimeout(5000) { entered.await() }
            runtime.cancel("r1")
            withTimeout(5000) { ended.await() }
            val result = replies.poll(1,TimeUnit.SECONDS)
            assertNotNull(result)
            assertEquals(true,result["cancelled"]?.jsonPrimitive?.boolean)
            assertEquals(true,result["outcome_unknown"]?.jsonPrimitive?.boolean)
            assertFalse(committed)
            collector.cancel()
        } finally { runtime.stop(); scope.cancel() }
    }
}
