package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.headersOf
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import kotlin.coroutines.CoroutineContext
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

/** Real host runtime and controller, deterministic native protocol and HTTP doubles; no JNI/network required. */
class MobileConversationIsolationTest {
    private class Bridge : AgentBridge {
        val sent = CopyOnWriteArrayList<JsonObject>()
        val incoming = LinkedBlockingQueue<String>()
        var config = ""
        override fun start(config: String): Long { this.config = config; return 1 }
        override fun send(handle: Long, line: String) { sent += CommunityJson.parseToJsonElement(line).jsonObject }
        override fun poll(handle: Long, timeoutMs: Long): String? = incoming.poll(timeoutMs, TimeUnit.MILLISECONDS)
        override fun stop(handle: Long) = Unit
        fun emit(value: String) { incoming.put(value) }
        fun run(session: String) = sent.lastOrNull { it["type"]?.jsonPrimitive?.content == "run" && it["session"]?.jsonPrimitive?.content == session }
        fun messages(type: String) = sent.filter { it["type"]?.jsonPrimitive?.content == type }
    }

    private class Harness(context: CoroutineContext, tools: List<AgentTool>, val client: HttpClient) {
        val scope = CoroutineScope(SupervisorJob() + context.minusKey(Job))
        val bridge = Bridge()
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, tools, "", "", scope)
        val store = ChatHistory(InMemoryChatHistoryFiles(), scope)
        val chat = CloudChatController(AiGatewayApi(client) { "token" }, scope, agent = runtime, history = store)
        suspend fun runId(session: String): String {
            eventually { bridge.run(session) != null }
            return bridge.run(session)!!.getValue("id").jsonPrimitive.content
        }
    }

    private suspend fun withChat(
        tools: List<AgentTool> = emptyList(),
        client: HttpClient = HttpClient(MockEngine { respond("{}") }),
        test: suspend Harness.() -> Unit,
    ) {
        val harness = Harness(currentCoroutineContext(), tools, client)
        try { harness.test() }
        finally {
            harness.chat.reset()
            harness.runtime.stop()
            harness.scope.cancel()
            client.close()
        }
    }

    @Test fun switchingSessionsPreservesTasksAndRoutesPromptsResultsAndStop() = runBlocking {
        withChat {
            chat.send("任务A")
            val a = chat.state.value.conversationId
            val ar = runId(a)
            chat.clearConversation()
            chat.send("任务B")
            val b = chat.state.value.conversationId
            val br = runId(b)
            assertTrue(bridge.messages("cancel").isEmpty())
            assertEquals(setOf(a, b), chat.state.value.runningSessions)
            bridge.emit("""{"type":"question","session":"$a","id":"qa","question":"A需要哪一种？","options":["甲","乙"]}""")
            bridge.emit("""{"type":"approval_request","session":"$b","id":"ab","tool":"write_file","arguments":{},"risk":"write"}""")
            eventually { chat.state.value.waitingSessions == setOf(a, b) }
            assertNull(chat.state.value.question)
            assertNotNull(chat.state.value.approval)
            chat.answerQuestion("不能误答A")
            assertTrue(bridge.messages("answer").isEmpty())
            chat.openConversation(a)
            assertEquals("A需要哪一种？", chat.state.value.question?.question)
            assertNull(chat.state.value.approval)
            chat.answerQuestion("甲")
            assertEquals("qa", bridge.messages("answer").single().getValue("id").jsonPrimitive.content)
            bridge.emit("""{"type":"event","run":"$ar","event":{"type":"text_delta","text":"A的回复"}}""")
            bridge.emit("""{"type":"event","run":"$br","event":{"type":"text_delta","text":"B的回复"}}""")
            bridge.emit("""{"type":"event","run":"$br","event":{"type":"tool_start","id":"tb","name":"write_file","arguments":"{}"}}""")
            eventually { chat.state.value.messages.last().text == "A的回复" && bridge.incoming.isEmpty() }
            chat.openConversation(b)
            eventually { chat.state.value.messages.last().tools.isNotEmpty() }
            assertEquals("B的回复", chat.state.value.messages.last().text)
            assertNotNull(chat.state.value.approval)
            chat.answerApproval("once")
            assertEquals("ab", bridge.messages("approval").single().getValue("id").jsonPrimitive.content)
            val stoppedReply = chat.state.value.messages.last().id
            chat.stop()
            assertEquals(listOf(br), bridge.messages("cancel").map { it.getValue("id").jsonPrimitive.content })
            assertEquals(setOf(a), chat.state.value.runningSessions)
            // A user may type again before the native stop settles; a late result must not erase that new turn.
            chat.send("B的新请求")
            eventually { !chat.state.value.sending }
            bridge.emit("""{"type":"event","run":"$br","event":{"type":"tool_end","id":"tb","output":"实际已写入","is_error":false}}""")
            bridge.emit("""{"type":"run_end","run":"$br","ok":false,"error":"tool_outcome_needs_review"}""")
            eventually { chat.state.value.messages.first { it.id == stoppedReply }.error.orEmpty().contains("结果尚未确认") }
            assertTrue(chat.state.value.messages.any { it.text == "B的新请求" })
            assertEquals("实际已写入", chat.state.value.messages.first { it.id == stoppedReply }.tools.single().output)
            chat.openConversation(a)
            assertTrue(chat.state.value.sending)
            bridge.emit("""{"type":"run_end","run":"$ar","ok":true,"text":"A的回复","finish":"stop"}""")
            eventually { !chat.state.value.sending }
            assertEquals("A的回复", store.loadSnapshot(a)!!.messages.last().text)
            assertTrue(store.loadSnapshot(b)!!.messages.any { it.text == "B的新请求" })
        }
    }

    @Test fun accountChangeStopsEveryOwnedRunAndRejectsLateEvents() = runBlocking {
        withChat {
            chat.useAccount("first")
            chat.send("A")
            val a = chat.state.value.conversationId
            val ar = runId(a)
            chat.clearConversation()
            chat.send("B")
            val b = chat.state.value.conversationId
            val br = runId(b)
            chat.useAccount("second")
            assertEquals(setOf(ar, br), bridge.messages("cancel").map { it.getValue("id").jsonPrimitive.content }.toSet())
            bridge.emit("""{"type":"event","run":"$ar","event":{"type":"text_delta","text":"旧账号内容"}}""")
            bridge.emit("""{"type":"run_end","run":"$ar","ok":true,"text":"旧账号内容"}""")
            bridge.emit("""{"type":"run_end","run":"$br","ok":true,"text":"旧账号内容"}""")
            eventually { bridge.incoming.isEmpty() }
            delay(20)
            assertTrue(chat.state.value.messages.isEmpty())
            assertTrue(chat.state.value.runningSessions.isEmpty())
            assertTrue(chat.state.value.conversations.isEmpty())
            chat.useAccount("first")
            assertTrue(store.loadSnapshot(a)!!.messages.last().cancelled)
            assertTrue(store.loadSnapshot(b)!!.messages.last().cancelled)
        }
    }

    @Test fun partialToolImagesStayWithTheirSessionAndAreNotInsertedIntoModelText() = runBlocking {
        val image = "data:image/png;base64,YQ=="
        val tool = object : AgentTool {
            override val name = "image_generate"
            override val description = "test"
            override val parameters = JsonObject(emptyMap())
            override suspend fun run(arguments: JsonObject): AgentToolResult {
                assertFalse(currentCoroutineContext()[AgentToolContext]!!.supportsVision)
                return AgentToolResult(false, "已收到1张，第二张失败", parts = JsonArray(listOf(buildJsonObject {
                    put("type", "image"); put("url", image)
                })), generatedImages = listOf(image))
            }
        }
        withChat(tools = listOf(tool)) {
            chat.saveCustomModel(CustomModelConfig("https://third.example", "key", "text", supportsVision = false))
            chat.send("生成两张图")
            val a = chat.state.value.conversationId
            val ar = runId(a)
            chat.clearConversation()
            chat.send("另一任务")
            val b = chat.state.value.conversationId
            runId(b)
            bridge.emit("""{"type":"event","run":"$ar","event":{"type":"tool_start","id":"tc","name":"image_generate","arguments":"{}"}}""")
            bridge.emit("""{"type":"tool_request","session":"$a","id":"host-image","name":"image_generate","arguments":{}}""")
            eventually { bridge.messages("tool_result").isNotEmpty() }
            val result = bridge.messages("tool_result").single()
            assertFalse(result.toString().contains("base64"))
            bridge.emit("""{"type":"event","run":"$ar","event":{"type":"tool_end","id":"tc","output":"已收到1张，第二张失败","is_error":true}}""")
            bridge.emit("""{"type":"run_end","run":"$ar","ok":false,"error":"部分图片失败"}""")
            eventually { a !in chat.state.value.runningSessions }
            assertEquals(b, chat.state.value.conversationId)
            assertTrue(chat.state.value.messages.last().generatedImages.isEmpty())
            assertTrue(chat.state.value.sending)
            chat.openConversation(a)
            val generated = chat.state.value.messages.last().generatedImages.single()
            assertEquals(image, chat.resolveImage(a, generated))
            assertFalse(chat.state.value.messages.last().imageGeneration)
            assertTrue(store.loadSnapshot(a)!!.messages.last().generatedImages.isNotEmpty())
        }
    }

    @Test fun confirmedCreationCanReadOriginalCorrectionsAndAttachmentsAfterSwitchingAway() = runBlocking {
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath == "/v1/chat/completions") respond("""{"choices":[{"message":{"content":"制作方案"},"finish_reason":"stop"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
            else respond("{}")
        })
        withChat(client = client) {
            assertTrue(chat.startCreation("作品", "画板", "支持离线导出"))
            eventually { !chat.state.value.sending }
            val attachment = "完整资料".repeat(4000) + "末尾验收项"
            val image = "data:image/png;base64,YQ=="
            chat.sendWithAttachments("更正：蓝色背景", listOf(ChatAttachment("需求.txt", text = attachment), ChatAttachment("参考.png", imageDataUrl = image)))
            eventually { !chat.state.value.sending }
            val userId = chat.state.value.messages.last { it.role == "user" }.id
            assertTrue(chat.state.value.creationPlanReady)
            chat.confirmCreationPlan()
            val a = chat.state.value.conversationId
            runId(a)
            val input = bridge.run(a)!!.getValue("input").jsonPrimitive.content
            assertTrue(input.contains("支持离线导出"))
            assertTrue(input.contains("更正：蓝色背景"))
            assertTrue(input.contains("需求.txt"))
            assertTrue(input.contains("参考.png"))
            assertTrue(bridge.config.contains("conversation_read"))
            chat.clearConversation()
            bridge.emit("""{"type":"tool_request","session":"$a","id":"read-tail","name":"conversation_read","arguments":{"message_id":$userId,"attachment_index":0,"offset":16000}}""")
            eventually { bridge.messages("tool_result").any { it["id"]?.jsonPrimitive?.content == "read-tail" } }
            val text = bridge.messages("tool_result").first { it["id"]?.jsonPrimitive?.content == "read-tail" }.getValue("output").jsonPrimitive.content
            assertTrue(text.contains("末尾验收项"))
            bridge.emit("""{"type":"tool_request","session":"$a","id":"read-image","name":"conversation_read","arguments":{"message_id":$userId,"attachment_index":1}}""")
            eventually { bridge.messages("tool_result").any { it["id"]?.jsonPrimitive?.content == "read-image" } }
            val images = bridge.messages("tool_result").first { it["id"]?.jsonPrimitive?.content == "read-image" }.getValue("parts") as JsonArray
            assertEquals(image, images.single().jsonObject.getValue("url").jsonPrimitive.content)
            assertTrue(chat.state.value.messages.isEmpty())
        }
    }

    @Test fun truncatedCustomPlanCannotBeConfirmed() = runBlocking {
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath.endsWith("/messages")) respond("""{"stop_reason":"max_tokens","content":[{"type":"text","text":"未写完的方案"}]}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
            else respond("{}")
        })
        withChat(client = client) {
            chat.saveCustomModel(CustomModelConfig("https://third.example", "key", "model", protocol = "anthropic"))
            chat.startCreation("作品", "画板", "需求")
            eventually { !chat.state.value.sending }
            assertEquals("未写完的方案", chat.state.value.messages.last().text)
            assertTrue(chat.state.value.messages.last().truncated)
            assertNotNull(chat.state.value.messages.last().error)
            assertFalse(chat.state.value.creationPlanReady)
            chat.confirmCreationPlan()
            assertTrue(bridge.messages("run").isEmpty())
        }
    }

    companion object {
        private suspend fun eventually(condition: () -> Boolean) = withTimeout(5000) { while (!condition()) delay(5) }
    }
}
