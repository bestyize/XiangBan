package xyz.xiangban.xiaoban.common.link

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.runTest
import xyz.xiangban.xiaoban.common.community.ToolStatus
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonArray

class RemoteChatTest {
    private val scope = CoroutineScope(Dispatchers.Unconfined)
    private val mockClient = HttpClient(MockEngine {
        respond("{}", HttpStatusCode.OK)
    })

    private suspend fun createLink(): PhoneLink {
        val storage = InMemoryLinkStorage(LinkState(installId = "test-install-id-0123456789", device = DeviceIdentity("phn_1", "手机")))
        val link = PhoneLink(scope, mockClient, storage, { null }, "手机")
        link.seedPeersForTest(listOf(PeerSummary("dsk_1", "我的Mac电脑", false)))
        return link
    }

    @Test
    fun `remote chat controller handles sessions, streaming text, tools and approvals`() = runTest {
        val link = createLink()
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()

        assertEquals(1, controller.state.value.peers.size)
        assertEquals("dsk_1", controller.state.value.selectedPeerId)
        assertEquals("我的Mac电脑", controller.state.value.selectedPeer?.name)

        // 模拟对端上线
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        assertTrue(controller.state.value.isPeerOnline)

        // 模拟对端发来会话列表
        val sessionJson = """{"t":"sessions.page","id":"1","items":[{"id":"s_1","title":"研究互联协议","updated_at":1000,"state":"idle"}]}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", sessionJson.encodeToByteArray()))

        assertEquals(1, controller.state.value.sessions.size)
        assertEquals("s_1", controller.state.value.sessions[0].id)
        assertEquals("s_1", controller.state.value.selectedSessionId)

        // 模拟对端发来快照消息
        val snapshotJson = """{"t":"session.snapshot","id":"2","session":"s_1","messages":[{"role":"user","content":"你好"},{"role":"assistant","content":"我是小伴"}]}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", snapshotJson.encodeToByteArray()))
        assertEquals(2, controller.state.value.messages.size)
        assertEquals("你好", controller.state.value.messages[0].text)
        assertEquals("我是小伴", controller.state.value.messages[1].text)

        // 模拟发送消息与实时流式增量
        controller.send("执行代码测试")
        assertTrue(controller.state.value.sending)
        assertEquals(4, controller.state.value.messages.size)
        assertEquals("执行代码测试", controller.state.value.messages[2].text)

        // 工具进度 tool.start
        val toolStart = """{"t":"event","id":"3","session":"s_1","kind":"tool.start","data":{"id":"t_1","name":"shell"}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", toolStart.encodeToByteArray()))
        val tools = { controller.state.value.messages.last().tools }
        assertEquals(1, tools().size)
        assertEquals("shell", tools()[0].name)
        assertEquals(ToolStatus.Running, tools()[0].status)
        // 带参数的工具调用：命令、路径会被提炼成一行说明
        val toolWithArgs = """{"t":"event","id":"3b","session":"s_1","kind":"tool.start","data":{"id":"t_2","name":"bash","arguments":{"command":"npm test -- login","timeout":30}}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", toolWithArgs.encodeToByteArray()))
        assertEquals("""{"command":"npm test -- login","timeout":30}""", tools()[1].arguments)
        assertEquals("src/a.vue", summarizeArgs(kotlinx.serialization.json.Json.parseToJsonElement("""{"file_path":"src/a.vue"}""")))
        assertEquals("rm -rf build", summarizeArgs(kotlinx.serialization.json.Json.parseToJsonElement("""{"anything":"rm -rf build"}""")))
        assertEquals("", summarizeArgs(kotlinx.serialization.json.Json.parseToJsonElement("{}")))

        // 文字增量 text.delta
        val textDelta = """{"t":"event","id":"4","session":"s_1","kind":"text.delta","data":{"text":"正在执行"}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", textDelta.encodeToByteArray()))
        assertEquals("正在执行", controller.state.value.messages.last().text)
        assertTrue(controller.state.value.messages.last().streaming)

        // 工具结束 tool.end
        val toolEnd = """{"t":"event","id":"5","session":"s_1","kind":"tool.end","data":{"id":"t_1","is_error":false}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", toolEnd.encodeToByteArray()))
        assertEquals(ToolStatus.Done, tools()[0].status)
        assertEquals(ToolStatus.Running, tools()[1].status)

        // 回复完成 turn.done
        val turnDone = """{"t":"event","id":"6","session":"s_1","kind":"turn.done","data":{}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", turnDone.encodeToByteArray()))
        assertFalse(controller.state.value.sending)
        assertFalse(controller.state.value.messages.last().streaming)

        // 审批卡片推送与处理
        val approvalReq = """{"t":"approval.request","id":"7","approval_id":"a_1","session":"s_1","title":"允许执行 rm？","risk":"high","options":[{"id":"once","label":"允许一次"},{"id":"deny","label":"拒绝"}]}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", approvalReq.encodeToByteArray()))
        val pending = controller.state.value.pendingApproval
        assertNotNull(pending)
        assertEquals("a_1", pending.approvalId)
        assertTrue(pending.isHighRisk)
        assertEquals("允许执行 rm？", pending.title)

        // 处理审批
        controller.answerApproval("a_1", "deny")
        assertNull(controller.state.value.pendingApproval)

        // 审批广播已解决
        val resolved = """{"t":"approval.resolved","id":"8","approval_id":"a_1","by":"desktop"}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", resolved.encodeToByteArray()))
        assertNull(controller.state.value.pendingApproval)
    }

    @Test
    fun `paged snapshot appends pages, requests the next one and settles stale approvals on the last page`() = runTest {
        val link = createLink()
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"sessions.page","id":"1","items":[{"id":"s_1","title":"长对话","updated_at":1,"state":"idle"}]}""".encodeToByteArray()))

        // 一张过期的审批卡片还挂在界面上
        val stale = """{"t":"approval.request","id":"2","approval_id":"a_old","session":"s_1","title":"允许执行 ls？","risk":"low","options":[{"id":"once","label":"允许一次"}]}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", stale.encodeToByteArray()))
        assertNotNull(controller.state.value.pendingApproval)

        val page1 = """{"t":"session.snapshot","id":"3","session":"s_1","after":0,"cursor":2,"has_more":true,"messages":[{"role":"user","parts":[{"type":"text","text":"一"}]},{"role":"assistant","parts":[{"type":"text","text":"二"}]}]}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", page1.encodeToByteArray()))
        assertEquals(listOf("一", "二"), controller.state.value.messages.map { it.text })
        assertNotNull(controller.state.value.pendingApproval, "还有后续页时不提前改动审批卡片")

        val page2 = """{"t":"session.snapshot","id":"4","session":"s_1","after":2,"cursor":3,"has_more":false,"messages":[{"role":"user","parts":[{"type":"text","text":"三"}]}],"pending":{}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", page2.encodeToByteArray()))
        assertEquals(listOf("一", "二", "三"), controller.state.value.messages.map { it.text })
        assertEquals(listOf("snap-0", "snap-1", "snap-2"), controller.state.value.messages.map { it.id })
        assertNull(controller.state.value.pendingApproval, "最后一页没有待审批，过期卡片应被收起")
    }

    @Test
    fun `expired approval reply clears the card`() = runTest {
        val link = createLink()
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        val req = """{"t":"approval.request","id":"2","approval_id":"a_9","session":"s_1","title":"允许？","risk":"low","options":[]}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", req.encodeToByteArray()))
        val expired = """{"t":"approval.resolved","id":"3","approval_id":"a_9","by":"expired","reason":"这个问题需要在电脑上处理"}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", expired.encodeToByteArray()))
        assertNull(controller.state.value.pendingApproval)
    }

    @Test
    fun `sidebar groups conversations by project folder and keeps loose ones apart`() {
        val sessions = listOf(
            RemoteSessionSummary("s1", "修复登录页的测试", 300, "running", "D:\\work\\xiaoban-web"),
            RemoteSessionSummary("s2", "整理本周周报", 100, "idle", "D:\\docs\\weekly"),
            RemoteSessionSummary("s3", "给项目加深色模式", 200, "idle", "D:\\work\\xiaoban-web\\"),
            RemoteSessionSummary("s4", "查端口占用", 50, "idle", ""),
        )
        val bar = buildSidebar(sessions, added = listOf("D:\\work\\xiaoban-docs", "D:\\docs\\weekly\\"))
        assertEquals(listOf("xiaoban-web", "weekly", "xiaoban-docs"), bar.projects.map { it.name })
        assertEquals(listOf("s1", "s3"), bar.projects[0].sessions.map { it.id }, "新的在前，末尾多一个分隔符算同一个项目")
        assertTrue(bar.projects[0].running)
        assertEquals(emptyList(), bar.projects[2].sessions, "刚添加的项目没有对话也要显示")
        assertEquals(1, bar.projects.count { it.name == "weekly" }, "已有对话的项目不会因为被添加过而重复")
        assertEquals(listOf("s4"), bar.loose.map { it.id })
        assertEquals("xiaoban-web", folderName("/Users/me/xiaoban-web/"))
        assertEquals("C:", folderName("C:\\"))
    }

    @Test
    fun `a new chat in a project sends the folder once and the first list opens only the latest chat`() = runTest {
        val link = createLink()
        val sent = mutableListOf<String>()
        link.sendObserverForTest = { _, data -> sent += data.decodeToString() }
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        assertTrue(sent.any { it.contains("\"projects.list\"") }, "上线后要向电脑要最近的项目")

        val page = """{"t":"sessions.page","id":"1","items":[
            {"id":"old","title":"旧的","updated_at":10,"state":"idle","workspace":"D:\\a"},
            {"id":"new","title":"新的","updated_at":20,"state":"idle","workspace":"D:\\b"}]}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", page.encodeToByteArray()))
        assertEquals("new", controller.state.value.selectedSessionId, "进设备先打开最近的对话")
        assertEquals("D:\\b", controller.state.value.currentWorkspace)

        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"projects.page","id":"2","recent":[{"path":"D:\\a","name":"a"},{"path":"D:\\c"}]}""".encodeToByteArray()))
        assertEquals(listOf("a", "c"), controller.state.value.recentProjects.map { it.name })

        controller.addProject("D:\\c")
        assertNull(controller.state.value.selectedSessionId)
        assertEquals("D:\\c", controller.state.value.currentWorkspace)
        assertTrue(controller.state.value.sidebar.projects.any { it.name == "c" })

        // 重新拉列表不能把用户从新对话里拽回旧对话
        link.emitEventForTest(LinkEvent.Message("dsk_1", page.encodeToByteArray()))
        assertNull(controller.state.value.selectedSessionId)

        sent.clear()
        controller.send("在 c 里做点事")
        val frame = sent.single { it.contains("\"message.send\"") }
        assertTrue(frame.contains("\"workspace\":\"D:\\\\c\""), frame)
        assertFalse(frame.contains("\"session\""), frame)

        // 已有对话里发消息，不再带文件夹
        controller.selectSession("old")
        sent.clear()
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"3","session":"old","kind":"turn.done","data":{}}""".encodeToByteArray()))
        controller.send("继续")
        val second = sent.single { it.contains("\"message.send\"") }
        assertTrue(second.contains("\"session\":\"old\"") && !second.contains("workspace"), second)
    }

    @Test
    fun `full trust is sent with the first message of a new chat and switched live in an open one`() = runTest {
        val link = createLink()
        val sent = mutableListOf<String>()
        link.sendObserverForTest = { _, data -> sent += data.decodeToString() }
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))

        controller.newSession("D:\\a")
        assertTrue(controller.state.value.fullTrust, "开发阶段：新对话默认完全信任")
        sent.clear()
        controller.send("干活")
        val first = sent.single { it.contains("\"message.send\"") }
        assertTrue(first.contains("\"permission\":\"auto\""), first)
        controller.newSession("D:\\a")
        controller.setPermission("ask")
        sent.clear()
        controller.send("小心点")
        assertTrue(sent.single { it.contains("\"message.send\"") }.contains("\"permission\":\"ask\""))

        // 已有对话：让电脑切换，电脑确认后才算数；电脑也可以主动通知模式变了。
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"message.accepted","id":"1","client_id":"c","session":"s_9","message_id":"m"}""".encodeToByteArray()))
        sent.clear()
        controller.setPermission("auto")
        assertTrue(sent.single().contains("\"session.permission\"") && sent.single().contains("\"mode\":\"auto\""), sent.toString())
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"2","session":"s_9","kind":"permission","data":{"mode":"ask"}}""".encodeToByteArray()))
        assertFalse(controller.state.value.fullTrust, "桌面上改成默认权限，手机跟着变")

        // 电脑拒绝切换（比如对话已被删除）：回到实际模式并报错。
        controller.setPermission("auto")
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"2b","session":"s_9","kind":"permission","data":{"mode":"ask"}}""".encodeToByteArray()))
        controller.setPermission("auto")
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"session.permission","id":"3","session":"s_9","ok":false,"error":"这个对话已经不存在"}""".encodeToByteArray()))
        assertFalse(controller.state.value.fullTrust)
        assertEquals("这个对话已经不存在", controller.state.value.error)

        // 打开对话时快照带着当前模式。
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"session.snapshot","id":"4","session":"s_9","permission":"auto","messages":[],"after":0,"cursor":0,"has_more":false}""".encodeToByteArray()))
        assertTrue(controller.state.value.fullTrust)
    }

    private fun parse(text: String): JsonObject = Json.parseToJsonElement(text).jsonObject

    @Test
    fun `history folds assistant messages and tool results into one reply with tool cards`() {
        val page1 = listOf(
            """{"role":"user","parts":[{"type":"text","text":"跑测试"}]}""",
            """{"role":"assistant","parts":[{"type":"text","text":"好的，我先看看。"}],"reasoning":["先读文件"],"tool_calls":[{"id":"c1","name":"read_file","arguments":"{\"path\":\"a.vue\"}"}]}""",
            """{"role":"tool","tool_call_id":"c1","parts":[{"type":"text","text":"文件内容"}]}""",
            """{"role":"assistant","parts":[],"tool_calls":[{"id":"c2","name":"shell","arguments":"{\"command\":\"npm test\"}"}]}""",
        ).map(::parse)
        val first = foldSnapshot(emptyList(), page1, 0)
        assertEquals(listOf("user", "assistant"), first.map { it.role })
        assertEquals(listOf("snap-0", "snap-1"), first.map { it.id })
        val reply = first[1]
        assertEquals("好的，我先看看。", reply.text)
        assertEquals("先读文件", reply.reasoning)
        assertEquals(listOf("read_file", "shell"), reply.tools.map { it.name })
        assertEquals("文件内容", reply.tools[0].output)
        assertEquals(ToolStatus.Done, reply.tools[0].status)
        assertEquals(ToolStatus.Running, reply.tools[1].status, "还没有结果的调用")

        // 一轮回答跨页：下一页接着折进同一条回复
        val page2 = listOf(
            """{"role":"tool","tool_call_id":"c2","is_error":true,"parts":[{"type":"text","text":"退出码 1"}]}""",
            """{"role":"assistant","parts":[{"type":"text","text":"有一个用例失败。"}]}""",
            """{"role":"user","parts":[{"type":"text","text":"修一下"}]}""",
        ).map(::parse)
        val second = foldSnapshot(first, page2, 4)
        assertEquals(3, second.size)
        assertEquals("好的，我先看看。\n\n有一个用例失败。", second[1].text)
        assertEquals(ToolStatus.Error, second[1].tools[1].status)
        assertEquals("退出码 1", second[1].tools[1].output)
        assertEquals("snap-6", second[2].id)
    }

    @Test
    fun `a question becomes a choice card and the answer is kept in the conversation`() = runTest {
        val link = createLink()
        val sent = mutableListOf<String>()
        link.sendObserverForTest = { _, data -> sent += data.decodeToString() }
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"sessions.page","id":"1","items":[{"id":"s_1","title":"整理","updated_at":1,"state":"idle"}]}""".encodeToByteArray()))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"2","session":"s_1","kind":"text.delta","data":{"text":"我先看了一下。"}}""".encodeToByteArray()))

        val request = """{"t":"approval.request","id":"3","approval_id":"q_1","session":"s_1","title":"重复文件怎么处理？","detail":{},"risk":"low",
            "options":[{"id":"移到文件夹","label":"移到文件夹"},{"id":"__dismiss__","label":"跳过"}],"kind":"question","question":"重复文件怎么处理？","choices":["移到文件夹","直接删除"],"free_text":true}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", request.encodeToByteArray()))
        val pending = controller.state.value.pendingApproval!!
        assertTrue(pending.isQuestion)
        assertEquals(listOf("移到文件夹", "直接删除"), pending.choices)
        assertTrue(pending.freeText)

        sent.clear()
        controller.answerApproval("q_1", "直接删除")
        assertTrue(sent.single().contains("\"choice\":\"直接删除\""))
        assertEquals(null, controller.state.value.pendingApproval)
        val record = controller.state.value.messages.last().records.single()
        assertEquals("answer", record.kind)
        assertEquals("重复文件怎么处理？", record.title)
        assertEquals("直接删除", record.result)

        // 跳过：留下“已跳过”
        link.emitEventForTest(LinkEvent.Message("dsk_1", request.replace("q_1", "q_2").encodeToByteArray()))
        controller.answerApproval("q_2", "__dismiss__")
        assertEquals(null, controller.state.value.messages.last().records.last().result)
    }

    @Test
    fun `an approval carries what the card needs and desktop handling leaves a record`() = runTest {
        val link = createLink()
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"sessions.page","id":"0","items":[{"id":"s_1","title":"整理","updated_at":1,"state":"idle"}]}""".encodeToByteArray()))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"1","session":"s_1","kind":"text.delta","data":{"text":"好"}}""".encodeToByteArray()))
        val request = """{"t":"approval.request","id":"2","approval_id":"a_1","session":"s_1","title":"允许执行 shell？","detail":{"command":"rm -rf build"},"risk":"high",
            "options":[{"id":"once","label":"允许一次"},{"id":"deny","label":"拒绝"}],"tool":"shell","risk_kind":"exec","external_path":false,"tainted":true,"danger":"删除多个文件"}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", request.encodeToByteArray()))
        val a = controller.state.value.pendingApproval!!
        assertEquals("shell", a.tool)
        assertEquals("exec", a.riskKind)
        assertTrue(a.tainted && a.isHighRisk)
        assertEquals("删除多个文件", a.danger)
        assertEquals("rm -rf build", a.summary)
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"approval.resolved","id":"3","approval_id":"a_1","by":"desktop"}""".encodeToByteArray()))
        assertEquals(null, controller.state.value.pendingApproval)
        assertEquals("已在电脑上处理", controller.state.value.messages.last().records.single().result)
    }

    @Test
    fun `stopping a turn asks the desktop to cancel and the reply is marked stopped`() = runTest {
        val link = createLink()
        val sent = mutableListOf<String>()
        link.sendObserverForTest = { _, data -> sent += data.decodeToString() }
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"sessions.page","id":"1","items":[{"id":"s_1","title":"整理","updated_at":1,"state":"idle"}]}""".encodeToByteArray()))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"2","session":"s_1","kind":"text.delta","data":{"text":"先看看"}}""".encodeToByteArray()))
        sent.clear()
        controller.cancel()
        assertTrue(sent.single().contains("\"t\":\"message.cancel\"") && sent.single().contains("\"session\":\"s_1\""))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"3","session":"s_1","kind":"state","data":{"state":"cancelled"}}""".encodeToByteArray()))
        assertEquals(TurnCancelled, controller.state.value.messages.last().error)
    }

    @Test
    fun `revising the plan stops the run first and then sends the new steps`() = runTest {
        val link = createLink()
        val sent = mutableListOf<String>()
        link.sendObserverForTest = { _, data -> sent += data.decodeToString() }
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"sessions.page","id":"1","items":[{"id":"s_1","title":"整理","updated_at":1,"state":"running"}]}""".encodeToByteArray()))
        controller.send("开始整理")
        sent.clear()
        controller.revisePlan("后续步骤改为：\n1. 只整理图片")
        assertTrue(sent.any { it.contains("message.cancel") })
        assertTrue(sent.none { it.contains("message.send") }, "还没停稳不能发")
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"3","session":"s_1","kind":"state","data":{"state":"cancelled"}}""".encodeToByteArray()))
        val revision = sent.first { it.contains("message.send") }
        assertTrue(revision.contains("只整理图片") && revision.contains("todo"), revision)
    }

    @Test
    fun `usage, truncation, undo, models and saving to the library`() = runTest {
        val link = createLink()
        val sent = mutableListOf<String>()
        link.sendObserverForTest = { _, data -> sent += data.decodeToString() }
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"sessions.page","id":"1","items":[{"id":"s_1","title":"整理","updated_at":1,"state":"idle"}]}""".encodeToByteArray()))
        suspend fun event(kind: String, data: String) = link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"e","session":"s_1","kind":"$kind","data":$data}""".encodeToByteArray()))
        event("text.delta", """{"text":"写好了"}""")
        event("usage", """{"prompt":1200,"completion":300}""")
        event("usage", """{"prompt":100,"completion":40}""")
        event("truncated", "{}")
        event("turn.done", "{}")
        event("checkpoint", """{"run_id":"r1","files":["a.txt","b.txt"],"count":2}""")
        var turn = controller.state.value.messages.last()
        assertEquals(1300 to 340, turn.promptTokens to turn.completionTokens)
        assertTrue(turn.truncated)
        assertEquals(RunCheckpoint("r1", 2, listOf("a.txt", "b.txt")), turn.checkpoint)

        sent.clear()
        controller.undoRun("r1")
        assertEquals("undoing", controller.state.value.messages.last().checkpoint?.state)
        assertTrue(sent.single().contains("\"t\":\"run.undo\"") && sent.single().contains("\"run_id\":\"r1\""))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"run.undo","id":"2","ok":true,"session":"s_1","run_id":"r1","restored":["a.txt"],"removed":["b.txt"],"failed":[]}""".encodeToByteArray()))
        turn = controller.state.value.messages.last()
        assertEquals("undone", turn.checkpoint?.state)
        assertTrue(turn.checkpoint!!.message.contains("恢复 1 个") && turn.checkpoint!!.message.contains("删除 1 个"))

        controller.fetchModels()
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"models.list","id":"3","ok":true,"models":[{"provider":"p1","provider_name":"服务一","id":"m1","name":"模型一"},{"provider":"p1","id":"m2"}],"current":{"provider":"p1","model":"m1"}}""".encodeToByteArray()))
        assertEquals(listOf("m1", "m2"), controller.state.value.models.map { it.id })
        assertEquals("m1", controller.state.value.currentModel)
        controller.setModel("p1", "m2")
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"session.model","id":"4","ok":true,"session":"s_1","provider":"p1","model":"m2"}""".encodeToByteArray()))
        assertEquals("m2", controller.state.value.currentModel)
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"session.model","id":"5","ok":false,"error":"这个对话还在回答"}""".encodeToByteArray()))
        assertEquals("这个对话还在回答", controller.state.value.error)

        sent.clear()
        controller.saveToLibrary(controller.state.value.messages.last())
        val request = sent.single()
        assertTrue(request.contains("\"t\":\"library.save\"") && request.contains("写好了"), request)
        val ref = Regex("\"id\":\"([^\"]+)\"").find(request)!!.groupValues[1]
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"library.save","id":"6","ref":"$ref","ok":true,"item":"x"}""".encodeToByteArray()))
        assertTrue(controller.state.value.messages.last().id in controller.state.value.savedTurns)
    }

    @OptIn(kotlin.io.encoding.ExperimentalEncodingApi::class)
    @Test
    fun `a recording goes up in pieces and the recognized text lands in the box or is sent and spoken`() = runTest {
        val link = createLink()
        val sent = mutableListOf<String>()
        link.sendObserverForTest = { _, data -> sent += data.decodeToString() }
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"sessions.page","id":"1","items":[{"id":"s_1","title":"整理","updated_at":1,"state":"idle"}]}""".encodeToByteArray()))

        sent.clear()
        controller.sendVoice(ByteArray(200_000) { 1 }, speakReply = false)
        assertEquals("recognizing", controller.state.value.voicePhase)
        assertEquals(3, sent.size)
        assertTrue(sent.all { it.contains("\"t\":\"voice.chunk\"") })
        assertTrue(sent[0].contains("\"seq\":0") && sent[0].contains("\"last\":false") && sent[2].contains("\"last\":true"))
        val voiceId = Regex("\"voice_id\":\"([^\"]+)\"").find(sent[0])!!.groupValues[1]
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"voice.text","id":"2","voice_id":"$voiceId","ok":true,"text":" 帮我整理下载文件夹 "}""".encodeToByteArray()))
        assertEquals("帮我整理下载文件夹", controller.state.value.voiceText)
        assertEquals("idle", controller.state.value.voicePhase)
        controller.consumeVoiceText()
        assertEquals(null, controller.state.value.voiceText)

        // 对话模式：识别完自动发出去，并要求电脑把回答念出来
        sent.clear()
        controller.sendVoice(ByteArray(40_000) { 1 }, speakReply = true)
        val second = Regex("\"voice_id\":\"([^\"]+)\"").find(sent[0])!!.groupValues[1]
        sent.clear()
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"voice.text","id":"3","voice_id":"$second","ok":true,"text":"现在几点"}""".encodeToByteArray()))
        assertTrue(sent.single().contains("\"t\":\"message.send\"") && sent.single().contains("\"speak\":true") && sent.single().contains("现在几点"), sent.toString())

        // 电脑把回答念成语音分片发回来
        val a = kotlin.io.encoding.Base64.encode(byteArrayOf(1, 2, 3))
        val b = kotlin.io.encoding.Base64.encode(byteArrayOf(4, 5))
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"voice.audio","id":"4","session":"s_1","audio_id":"a1","ok":true,"seq":0,"last":false,"mime":"audio/mpeg","data":"$a"}""".encodeToByteArray()))
        assertEquals(null, controller.state.value.voiceClip)
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"voice.audio","id":"5","session":"s_1","audio_id":"a1","ok":true,"seq":1,"last":true,"mime":"audio/mpeg","data":"$b"}""".encodeToByteArray()))
        assertEquals(listOf<Byte>(1, 2, 3, 4, 5), controller.state.value.voiceClip!!.bytes.toList())
        controller.consumeVoiceClip()
        assertEquals(null, controller.state.value.voiceClip)

        // 识别失败：回到空闲并说明原因
        controller.sendVoice(ByteArray(40_000) { 1 }, speakReply = false)
        link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"voice.text","id":"6","voice_id":"x","ok":false,"error":"电脑上还没有开启语音识别"}""".encodeToByteArray()))
        assertEquals("idle", controller.state.value.voicePhase)
        assertEquals("电脑上还没有开启语音识别", controller.state.value.error)
    }
}
