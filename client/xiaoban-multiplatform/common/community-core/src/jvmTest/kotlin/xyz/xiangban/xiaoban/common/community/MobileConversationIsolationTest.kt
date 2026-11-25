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

    private class Harness(context: CoroutineContext, tools: List<AgentTool>, val client: HttpClient, creationStore: MobileGameStore?) {
        val scope = CoroutineScope(SupervisorJob() + context.minusKey(Job))
        val bridge = Bridge()
        val runtime = AgentRuntime(bridge, { _, _ -> ModelResult(200) }, tools, "", "", scope)
        val store = ChatHistory(InMemoryChatHistoryFiles(), scope)
        val chat = CloudChatController(AiGatewayApi(client) { "token" }, scope, agent = runtime, history = store, creationStore = creationStore)
        suspend fun runId(session: String): String {
            eventually { bridge.run(session) != null }
            return bridge.run(session)!!.getValue("id").jsonPrimitive.content
        }
    }

    private suspend fun withChat(
        tools: List<AgentTool> = emptyList(),
        client: HttpClient = HttpClient(MockEngine { respond("{}") }),
        creationStore: MobileGameStore? = null,
        test: suspend Harness.() -> Unit,
    ) {
        val harness = Harness(currentCoroutineContext(), tools, client, creationStore)
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
            assertTrue(chat.state.value.sending)
            assertEquals(br, bridge.run(b)?.get("id")?.jsonPrimitive?.content)
            bridge.emit("""{"type":"event","run":"$br","event":{"type":"tool_end","id":"tb","output":"实际已写入","is_error":false}}""")
            bridge.emit("""{"type":"run_end","run":"$br","ok":false,"error":"tool_outcome_needs_review"}""")
            eventually { chat.state.value.messages.first { it.id == stoppedReply }.error.orEmpty().contains("结果尚未确认") }
            eventually { bridge.run(b)?.get("id")?.jsonPrimitive?.content != br }
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

    @Test fun mainChatKeepsDraftThroughStatusRepliesAndValidatesAnEarlierInstallation() = runBlocking {
        val workspace = MemoryMobileWorkspace()
        val path = "drafts/ledger/index.html"
        workspace.write(path, "<h1>账本</h1>")
        workspace.write("drafts/ledger/icon.svg", "<svg/>")
        workspace.write("drafts/ledger/cover.svg", "<svg/>")
        val games = MobileGameStore(workspace)
        withChat(creationStore = games) {
            fun toolEvent(run: String, event: JsonObject) = bridge.emit(buildJsonObject {
                put("type", "event"); put("run", run); put("event", event)
            }.toString())
            fun toolStart(run: String, id: String, name: String, args: String) = toolEvent(run, buildJsonObject {
                put("type", "tool_start"); put("id", id); put("name", name); put("arguments", args)
            })
            fun toolEnd(run: String, id: String, output: String) = toolEvent(run, buildJsonObject {
                put("type", "tool_end"); put("id", id); put("output", output); put("is_error", false)
            })
            chat.send("做一个记账小程序")
            val session = chat.state.value.conversationId
            val draftRun = runId(session)
            toolStart(draftRun, "write", "write_file", """{"path":"$path"}""")
            toolEnd(draftRun, "write", """{"path":"$path","saved":true}""")
            eventually { chat.state.value.messages.last().tools.singleOrNull()?.status == ToolStatus.Done }
            chat.stop()
            assertEquals(setOf(path), chat.state.value.creationDeliveryProgress()?.files)
            chat.send("做好了吗")
            bridge.emit("""{"type":"run_end","run":"$draftRun","ok":false,"error":"cancelled"}""")
            eventually { bridge.messages("run").size == 2 }
            val followup = runId(session)
            bridge.emit("""{"type":"run_end","run":"$followup","ok":false,"error":"provider error: HTTP 400: 请求参数与模型不兼容"}""")
            eventually { !chat.state.value.sending }
            assertEquals(setOf(path), chat.state.value.creationDeliveryProgress()?.files)

            chat.send("做好了吗")
            eventually { bridge.messages("run").size == 3 }
            val statusRun = runId(session)
            bridge.emit("""{"type":"run_end","run":"$statusRun","ok":true,"text":"草稿已保存，作品尚未安装。","finish":"stop"}""")
            eventually { !chat.state.value.sending }
            chat.clearConversation()
            chat.openConversation(session)
            assertEquals(setOf(path), chat.state.value.creationDeliveryProgress()?.files)

            chat.resumeCreation()
            chat.resumeCreation()
            eventually { bridge.messages("run").size == 4 }
            val resume = bridge.run(session)!!
            assertTrue(resume.getValue("input").jsonPrimitive.content.contains("drafts/ledger 当前文件"))
            val run = resume.getValue("id").jsonPrimitive.content
            val installArgs = """{"id":"ledger","kind":"app","name":"账本","description":"记账","source_dir":"drafts/ledger"}"""
            val tools = mobileGameTools(games).associateBy { it.name }
            val install = tools.getValue("app_create").run(CommunityJson.parseToJsonElement(installArgs).jsonObject)
            assertTrue(install.ok, install.output)
            toolStart(run, "install", "app_create", installArgs)
            toolEnd(run, "install", install.output)
            bridge.emit("""{"type":"run_end","run":"$run","ok":true,"text":"作品已安装，等待校验。","finish":"stop"}""")
            eventually { !chat.state.value.sending }
            val pending = assertNotNull(chat.state.value.creationDeliveryProgress())
            assertEquals("ledger", pending.installed?.id)
            assertFalse(pending.validated)

            chat.resumeCreation()
            eventually { bridge.messages("run").size == 5 }
            val validationRun = runId(session)
            val validateArgs = """{"id":"ledger"}"""
            val validate = tools.getValue("app_validate").run(CommunityJson.parseToJsonElement(validateArgs).jsonObject)
            assertTrue(validate.ok, validate.output)
            toolStart(validationRun, "validate", "app_validate", validateArgs)
            toolEnd(validationRun, "validate", validate.output)
            bridge.emit("""{"type":"run_end","run":"$validationRun","ok":true,"text":"制作完成","finish":"stop"}""")
            eventually { !chat.state.value.sending }
            val delivery = assertNotNull(chat.state.value.creationDeliveryProgress())
            assertTrue(delivery.validated)
            assertEquals("ledger", delivery.installed?.id)
            assertEquals("<h1>账本</h1>", games.readFile("ledger", "index.html"))
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

    @Test fun deliveredCreationHandsOffPlaytestAndRecordsFeedbackWithoutRunningTheModelAgain() = runBlocking {
        val workspace = MemoryMobileWorkspace()
        workspace.write("drafts/fruit/index.html", "<h1>水果消消乐</h1>")
        workspace.write("drafts/fruit/icon.svg", "<svg/>")
        workspace.write("drafts/fruit/cover.svg", "<svg/>")
        val games = MobileGameStore(workspace)
        withChat(client = creationPlanner(), creationStore = games) {
            assertTrue(chat.startCreation("游戏 · Design and build", "做水果消消乐", "20 步"))
            eventually { chat.state.value.creationPlanReady }
            val session = chat.state.value.conversationId
            assertEquals("水果消消乐", chat.state.value.conversations.single().title)
            chat.confirmCreationPlan()
            val run = runId(session)
            assertNotNull(chat.state.value.creation?.startedAtMs)

            fun event(kind: String, id: String, field: String, value: String, args: String? = null) {
                bridge.emit(buildJsonObject {
                    put("type", "event"); put("run", run)
                    put("event", buildJsonObject {
                        put("type", kind); put("id", id); put(field, value)
                        args?.let { put("arguments", it) }
                    })
                }.toString())
            }
            fun start(id: String, name: String, args: String) = event("tool_start", id, "name", name, args)
            fun end(id: String, output: String) = event("tool_end", id, "output", output)
            val install = """{"id":"fruit","name":"水果消消乐","description":"水果三消","version":"1.0.2","source_dir":"drafts/fruit"}"""
            val work = mobileGameTools(games).first { it.name == "app_create" }.run(CommunityJson.parseToJsonElement(install).jsonObject)
            assertTrue(work.ok, work.output)
            start("install", "app_create", install)
            end("install", work.output)
            start("validate", "app_validate", """{"id":"fruit"}""")
            end("validate", work.output)
            start("open", "app_open", """{"id":"fruit"}""")
            end("open", "已请求打开作品")
            start("ask", "ask_user", """{"question":"试玩一局，情况如何？"}""")
            bridge.emit("""{"type":"question","id":"playtest","session":"$session","question":"试玩一局，情况如何？","options":[]}""")
            eventually { bridge.messages("cancel").size == 1 }
            assertNull(chat.state.value.question)
            bridge.emit("""{"type":"run_end","run":"$run","ok":false,"error":"cancelled"}""")
            eventually { !chat.state.value.sending }
            assertNull(chat.state.value.messages.last().error)
            val delivery = assertNotNull(chat.state.value.creationDeliveryProgress())
            val ref = assertNotNull(delivery.installRef)
            assertNull(delivery.playtest)
            assertTrue(chat.recordCreationPlaytest(ref, CreationPlaytestResult.KnownIssue, "首局上报待完善"))
            assertFalse(chat.recordCreationPlaytest(ref.copy(toolId = "another-version"), CreationPlaytestResult.Passed))
            assertEquals(1, bridge.messages("run").size)
            chat.clearConversation()
            chat.openConversation(session)
            assertEquals("首局上报待完善", chat.state.value.creationDeliveryProgress()?.playtest?.note)
        }
    }

    @Test fun draftGrantOnlyAppliesToWritesAndEditsUnderTheApprovedDirectoryInThisRun() = runBlocking {
        withChat(client = creationPlanner()) {
            assertTrue(chat.startCreation("游戏", "做水果消消乐", "20 步"))
            eventually { chat.state.value.creationPlanReady }
            chat.confirmCreationPlan()
            val session = chat.state.value.conversationId
            val run = runId(session)
            fun request(id: String, tool: String, path: String) {
                bridge.emit("""{"type":"approval_request","id":"$id","session":"$session","tool":"$tool","risk":"write","arguments":{"path":"$path"},"can_remember":true}""")
            }
            request("first", "write_file", "drafts/fruit/index.html")
            eventually { chat.state.value.approval?.draftDirectory == "drafts/fruit" }
            chat.answerApproval(ALLOW_CREATION_DRAFT)
            eventually { bridge.messages("approval").size == 1 }
            request("edit", "edit_file", "drafts/fruit/style.css")
            eventually { bridge.messages("approval").size == 2 }
            assertTrue(bridge.messages("approval").all { it["decision"]?.jsonPrimitive?.content == "once" })
            assertTrue(chat.state.value.grants.isEmpty())

            request("other", "write_file", "drafts/fruit-old/index.html")
            eventually { chat.state.value.approval?.draftDirectory == "drafts/fruit-old" }
            chat.answerApproval("deny")
            request("delete", "delete_file", "drafts/fruit/index.html")
            eventually { chat.state.value.approval?.tool == "delete_file" }
            assertNull(chat.state.value.approval?.draftDirectory)
            chat.answerApproval("deny")

            bridge.emit("""{"type":"event","run":"$run","event":{"type":"retrying","attempt":1,"message":"模型上下文超限"}}""")
            bridge.emit("""{"type":"event","run":"$run","event":{"type":"retrying","attempt":1}}""")
            eventually { chat.state.value.messages.last().creationTiming?.retries == 2 }
            assertEquals("正在重试（2）", chat.state.value.messages.last().activity)

            bridge.emit("""{"type":"run_end","run":"$run","ok":true}""")
            eventually { !chat.state.value.sending }
            chat.send("接着写")
            eventually { bridge.messages("run").size == 2 }
            request("new-run", "write_file", "drafts/fruit/index.html")
            eventually { chat.state.value.approval?.draftDirectory == "drafts/fruit" }
            assertEquals(4, bridge.messages("approval").size)
        }
    }

    private fun creationPlanner() = HttpClient(MockEngine { request ->
        if (request.url.encodedPath == "/v1/chat/completions") {
            respond("""{"choices":[{"message":{"content":"作品名称：水果消消乐"},"finish_reason":"stop"}]}""",
                headers = headersOf(HttpHeaders.ContentType, "application/json"))
        } else respond("{}")
    })

    companion object {
        private suspend fun eventually(condition: () -> Boolean) = withTimeout(5000) { while (!condition()) delay(5) }
    }
}
