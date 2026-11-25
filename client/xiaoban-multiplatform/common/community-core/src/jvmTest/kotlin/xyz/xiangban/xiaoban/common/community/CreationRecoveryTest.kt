package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.headersOf
import kotlinx.coroutines.*
import kotlinx.serialization.json.*
import java.nio.file.Files
import kotlin.test.*

class CreationRecoveryTest {
    private suspend fun MemoryMobileWorkspace.prepareDraft(id: String) {
        write("drafts/$id/index.html", "original")
        write("drafts/$id/icon.svg", "<svg/>")
        write("drafts/$id/cover.svg", "<svg/>")
    }

    private fun installationArgs(id: String, source: String = id, overwrite: Boolean = false) = buildJsonObject {
        put("id", id)
        put("kind", "app")
        put("name", id)
        put("description", "测试作品")
        put("source_dir", "drafts/$source")
        put("overwrite", overwrite)
    }

    private suspend fun AgentTool.receipt(arguments: JsonObject): ToolCallInfo {
        val result = run(arguments)
        assertTrue(result.ok, result.output)
        return ToolCallInfo(name, name, arguments.toString(), result.output, ToolStatus.Done)
    }

    @Test
    fun playtestBelongsToTheInstallationReceiptEvenWhenTheVersionIsReused() = runBlocking {
        val workspace = MemoryMobileWorkspace()
        workspace.prepareDraft("book")
        val tools = mobileGameTools(MobileGameStore(workspace)).associateBy { it.name }
        val install = tools.getValue("app_create").receipt(installationArgs("book"))
        val validate = tools.getValue("app_validate").receipt(buildJsonObject { put("id", "book") })
        val first = ChatTurn(1, "assistant", "", tools = listOf(install, validate),
            creationPlaytest = CreationPlaytest(install.id, CreationPlaytestResult.Passed))
        assertEquals(CreationPlaytestResult.Passed, creationProgress(listOf(first)).playtest?.result)

        val updated = tools.getValue("app_create").receipt(installationArgs("book", overwrite = true))
        val second = ChatTurn(2, "assistant", "", tools = listOf(updated, validate))
        val current = creationProgress(listOf(first, second))
        assertTrue(current.validated)
        assertEquals(CreationInstallRef(second.id, updated.id), current.installRef)
        assertNull(current.playtest)
    }

    @Test
    fun startupEvidenceAndFirstRoundScoreAreSeparateAndBelongToThisVersion() {
        val work = MobileGame("fruit", "水果消消乐", "水果三消", version = "1.0.2")
        val open = """{"type":"host.open","gameId":"fruit","version":"1.0.2"}"""
        val ready = """{"type":"game.ready","readyMs":74}"""
        val start = """{"type":"game.start"}"""
        val result = """{"type":"game.result","score":810}"""
        fun report(vararg lines: String) = mobileGameRuntimeReport(work, lines.joinToString("\n"))

        val loading = report(open, ready)
        assertEquals("ready", loading["startup"]?.jsonPrimitive?.content)
        assertEquals("no_start_recorded", loading["first_round"]?.jsonPrimitive?.content)
        assertEquals(0, loading["errors"]?.jsonPrimitive?.int)
        assertEquals(810, report(open, ready, start, result)["first_round_score"]?.jsonPrimitive?.int)
        assertEquals("no_result_recorded", report(open, ready, start, start, result)["first_round"]?.jsonPrimitive?.content)
        val stale = report(open, ready, start, result, open.replace("1.0.2", "1.0.1"), ready, start, result)
        assertEquals("not_opened", stale["startup"]?.jsonPrimitive?.content)
        assertNull(stale["first_round_score"])
    }

    @Test
    fun onlyConfirmedWritesSurviveFailedCallsAndDeletion() {
        fun write(name: String, receipt: String, status: ToolStatus) =
            ToolCallInfo(name, "write_file", """{"path":"drafts/book/$name"}""", receipt, status)
        val turns = listOf(ChatTurn(1, "assistant", "", tools = listOf(
            write("index.html", """{"path":"drafts/book/index.html","saved":true}""", ToolStatus.Done),
            write("unused.js", """{"path":"drafts/book/unused.js","saved":true}""", ToolStatus.Done),
            write("missing.css", "", ToolStatus.Error),
        )), ChatTurn(3, "assistant", "", tools = listOf(
            ToolCallInfo("d", "delete_file", output = """{"path":"drafts/book/unused.js","deleted":true}""", status = ToolStatus.Done),
        )))
        val progress = creationProgress(turns)
        assertEquals(setOf("drafts/book/index.html"), progress.files)
        assertEquals(setOf("drafts/book"), progress.directories)
        assertNull(progress.installed)
    }

    @Test
    fun recoveryProtectsSavedPageAndExistingInstallation() = runBlocking {
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        val tools = (mobileWorkspaceTools(workspace) + mobileGameTools(store)).associateBy { it.name }
        val page = "drafts/book/index.html"
        workspace.write(page, "original")
        workspace.write("drafts/book/icon.svg", "<svg/>")
        workspace.write("drafts/book/cover.svg", "<svg/>")
        val install = buildJsonObject { put("id", "book"); put("name", "账本"); put("description", "记录"); put("source_dir", "drafts/book") }
        val receipt = tools.getValue("app_create").run(install)
        assertTrue(receipt.ok)
        val progress = creationProgress(listOf(ChatTurn(1, "assistant", "", tools = listOf(
            ToolCallInfo("1", "write_file", output = """{"path":"$page","saved":true}""", status = ToolStatus.Done),
            ToolCallInfo("2", "app_create", install.toString(), receipt.output, ToolStatus.Done),
        ))))
        val input = progress.resumeInput(store, CreationDraft("账本", "记账", true), null)
        assertTrue(input.contains("沿用原作品 ID book"))
        assertTrue(input.contains("已通过文件校验"))
        withContext(AgentToolContext("test", "call", recovery = progress)) {
            val write = tools.getValue("write_file")
            assertTrue(write.run(buildJsonObject { put("path", page); put("content", "original") }).ok)
            assertFalse(write.run(buildJsonObject { put("path", page); put("content", "replaced") }).ok)
            // The icon exists on disk even though its original acknowledgement was lost.
            assertFalse(write.run(buildJsonObject { put("path", "drafts/book/icon.svg"); put("content", "replaced") }).ok)
            val duplicate = JsonObject(install + ("id" to JsonPrimitive("another-book")))
            assertFalse(tools.getValue("app_create").run(duplicate).ok)
        }
        assertEquals("original", workspace.read(page))
        assertEquals(listOf("book"), store.list().map { it.id })
    }

    @Test
    fun anEditedDraftNeedsReinstallationEvenIfTheOldVersionStillValidates() = runBlocking {
        val workspace = MemoryMobileWorkspace()
        workspace.prepareDraft("book")
        val store = MobileGameStore(workspace)
        val tools = (mobileWorkspaceTools(workspace) + mobileGameTools(store)).associateBy { it.name }
        val validateArgs = buildJsonObject { put("id", "book") }
        val original = ChatTurn(2, "assistant", "初版完成", tools = listOf(
            tools.getValue("app_create").receipt(installationArgs("book")),
            tools.getValue("app_validate").receipt(validateArgs),
        ))
        val edition = ChatTurn(4, "assistant", "草稿已更新", cancelled = true, tools = listOf(
            tools.getValue("edit_file").receipt(buildJsonObject {
                put("path", "drafts/book/index.html"); put("old_text", "original"); put("new_text", "updated")
            }),
        ))
        val state = CloudChatState(nativeAgentAvailable = true, creation = CreationDraft("账本", "记账", true),
            messages = listOf(ChatTurn(1, "user", "做一个账本"), original, ChatTurn(3, "user", "更新首页"), edition))
        val pending = assertNotNull(state.creationDeliveryProgress())
        assertTrue(pending.draftChangedSinceInstall)
        assertFalse(pending.validated)
        assertNull(pending.installed)
        assertEquals("book", pending.resumeId)
        assertTrue(pending.resumeInput(store, state.creation, null).contains("草稿更新前的版本"))
        assertEquals("original", store.readFile("book", "index.html"))

        val oldValidation = ChatTurn(5, "assistant", "旧版通过校验", tools = listOf(tools.getValue("app_validate").receipt(validateArgs)))
        val checked = state.copy(messages = state.messages + oldValidation)
        assertFalse(assertNotNull(checked.creationDeliveryProgress()).validated)

        val reinstalled = ChatTurn(6, "assistant", "已安装更新版", tools = listOf(
            tools.getValue("app_create").receipt(installationArgs("book", overwrite = true)),
        ))
        val replacement = checked.copy(messages = checked.messages + reinstalled)
        assertFalse(assertNotNull(replacement.creationDeliveryProgress()).validated)
        val newValidation = ChatTurn(7, "assistant", "更新版通过校验", tools = listOf(tools.getValue("app_validate").receipt(validateArgs)))
        val delivered = assertNotNull(replacement.copy(messages = replacement.messages + newValidation).creationDeliveryProgress())
        assertEquals("book", delivered.installed?.id)
        assertTrue(delivered.validated)
        assertEquals("updated", store.readFile("book", "index.html"))
    }

    @Test
    fun resumingTheSecondDraftKeepsItsOwnDirectoryAndCanUseItsOwnId() = runBlocking {
        val workspace = MemoryMobileWorkspace()
        workspace.prepareDraft("book")
        workspace.prepareDraft("calc")
        val store = MobileGameStore(workspace)
        val create = mobileGameTools(store).single { it.name == "app_create" }
        val first = ChatTurn(2, "assistant", "账本待校验", cancelled = true, tools = listOf(create.receipt(installationArgs("book"))))
        val second = ChatTurn(4, "assistant", "计算器草稿已保存", cancelled = true, tools = listOf(
            mobileWorkspaceTools(workspace).single { it.name == "write_file" }.receipt(buildJsonObject {
                put("path", "drafts/calc/index.html"); put("content", "calculator")
            }),
        ))
        val validate = mobileGameTools(store).single { it.name == "app_validate" }.receipt(buildJsonObject { put("id", "book") })
        val state = CloudChatState(nativeAgentAvailable = true, messages = listOf(
            ChatTurn(1, "user", "做一个账本"), first, ChatTurn(3, "user", "换一个任务，做计算器"), second,
            ChatTurn(5, "user", "检查一下账本"), ChatTurn(6, "assistant", "账本校验通过", tools = listOf(validate)),
        ))
        val pending = assertNotNull(state.creationDeliveryProgress())
        assertEquals(setOf("drafts/calc"), pending.directories)
        assertEquals(setOf("drafts/calc/index.html"), pending.files)
        assertNull(pending.resumeId)
        val firstPlan = ChatTurn(0, "assistant", "账本创作方案", creationPlan = true)
        val confirmedFirst = CreationDraft("账本", "第一件作品是记账工具", true)
        val fromApprovedChat = assertNotNull(state.copy(creation = confirmedFirst).creationDeliveryProgress())
        val input = fromApprovedChat.resumeInput(store, confirmedFirst, firstPlan)
        assertTrue(input.contains("本次续作的草稿目录是 drafts/calc"))
        assertFalse(input.contains(confirmedFirst.request))
        assertFalse(input.contains("conversation_read(message_id="))
        withContext(AgentToolContext("test", "call", recovery = pending)) {
            val wrongDraft = create.run(installationArgs("calc", source = "book"))
            assertFalse(wrongDraft.ok)
            assertTrue(wrongDraft.output.contains("草稿目录"), wrongDraft.output)
            create.receipt(installationArgs("calc"))
        }
        assertEquals("calculator", store.readFile("calc", "index.html"))
        assertEquals("original", store.readFile("book", "index.html"))
    }

    @Test
    fun resumeAfterModelDisconnectUsesExistingPageAndInstallsOneWork() = runBlocking {
        if (System.getenv("AGENT_MOBILE_LIB").isNullOrBlank()) return@runBlocking
        val directory = Files.createTempDirectory("creation-recovery").toFile()
        val scope = CoroutineScope(coroutineContext + SupervisorJob())
        val bridge = object : AgentBridge {
            override fun start(config: String) = AgentNative.nativeStart(config)
            override fun send(handle: Long, line: String) = AgentNative.nativeSend(handle, line)
            override fun poll(handle: Long, timeoutMs: Long) = AgentNative.nativePoll(handle, timeoutMs)
            override fun stop(handle: Long) = AgentNative.nativeStop(handle)
        }
        val indexPath = "drafts/book/index.html"
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        val installedIds = mutableListOf<String>()
        val modelCalls = listOf(
            "write_file" to """{"path":"$indexPath","content":"<h1>记账</h1>"}""",
            "write_file" to """{"path":"drafts/book/icon.svg","content":"<svg/>"}""",
            "write_file" to """{"path":"drafts/book/cover.svg","content":"<svg/>"}""",
            "app_create" to """{"source_dir":"drafts/book","id":"book","kind":"app","name":"账本","description":"收支记录"}""",
            "app_validate" to """{"id":"book"}""",
        )
        var step = 0
        var connected = false
        var resumeRequest = ""
        val runtime = AgentRuntime(bridge, { body, line ->
            if (step == 1 && !connected) ModelResult(400, error = "连接小伴 AI 服务失败，请检查网络后重试")
            else {
                if (step == 1) resumeRequest = body
                val call = modelCalls.getOrNull(step++)
                val delta = buildJsonObject {
                    if (call == null) put("content", "制作完成")
                    else putJsonArray("tool_calls") { add(buildJsonObject {
                        put("index", 0); put("id", "call$step"); put("type", "function")
                        putJsonObject("function") { put("name", call.first); put("arguments", call.second) }
                    }) }
                }
                val finish = if (call == null) "stop" else "tool_calls"
                line("""data: {"choices":[{"index":0,"delta":$delta,"finish_reason":"$finish"}]}""")
                line(""); line("data: [DONE]"); line("")
                ModelResult(200)
            }
        }, mobileWorkspaceTools(workspace) + mobileGameTools(store) { installedIds += it }, directory.path, "制作手机作品", scope)
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath.endsWith("/chat/completions")) respond(
                """{"choices":[{"message":{"content":"账本，记录收支，保存在本机。"},"finish_reason":"stop"}]}""",
                headers = headersOf(HttpHeaders.ContentType, "application/json"))
            else respond("{}")
        })
        val history = ChatHistory(InMemoryChatHistoryFiles(), scope)
        val chat = CloudChatController(AiGatewayApi(client) { "token" }, scope, agent = runtime, history = history, creationStore = store)
        suspend fun idle() = withTimeout(30_000) { while (chat.state.value.sending) delay(10) }
        try {
            chat.useAccount("tester")
            assertTrue(chat.startCreation("账本", "设计一个账本", "记账"))
            idle()
            assertTrue(chat.state.value.creationPlanReady)
            val id = chat.state.value.conversationId
            chat.clearConversation()
            chat.openConversation(id)
            assertEquals(0, step)
            chat.setPermission("auto")
            chat.confirmCreationPlan()
            idle()
            assertTrue(chat.state.value.messages.last().creationInterrupted(), chat.state.value.messages.last().toString())
            assertEquals(ChatTaskStatus.Interrupted, chat.state.value.conversations.single { it.id == id }.taskStatus)
            assertEquals(listOf(indexPath), workspace.list("drafts/book").map { it.path })
            chat.clearConversation()
            chat.openConversation(id)
            assertEquals(setOf(indexPath), creationProgress(chat.state.value.messages).files)
            connected = true
            chat.resumeCreation()
            chat.resumeCreation()
            idle()
            assertTrue(resumeRequest.contains("drafts/book 当前文件：$indexPath"), resumeRequest.takeLast(1000))
            assertNull(chat.state.value.messages.last().error)
            assertEquals(listOf("book"), installedIds)
            assertEquals(listOf("book"), store.list().map { it.id })
            assertEquals("<h1>记账</h1>", store.readFile("book","index.html"))
            val progress = creationProgress(chat.state.value.messages)
            assertEquals("账本", progress.installed?.name)
            assertTrue(progress.validated)
            assertEquals(3, chat.state.value.messages.flatMap { it.tools }.count { it.name == "write_file" })
        } finally { runtime.stop(); scope.cancel(); client.close(); directory.deleteRecursively() }
    }
}
