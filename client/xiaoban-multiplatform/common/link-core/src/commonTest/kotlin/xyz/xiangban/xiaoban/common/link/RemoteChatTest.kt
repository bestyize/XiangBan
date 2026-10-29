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
        assertEquals(1, controller.state.value.toolProgress.size)
        assertEquals("shell", controller.state.value.toolProgress[0].name)
        assertTrue(controller.state.value.toolProgress[0].running)
        // 带参数的工具调用：命令、路径会被提炼成一行说明
        val toolWithArgs = """{"t":"event","id":"3b","session":"s_1","kind":"tool.start","data":{"id":"t_2","name":"bash","arguments":{"command":"npm test -- login","timeout":30}}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", toolWithArgs.encodeToByteArray()))
        assertEquals("npm test -- login", controller.state.value.toolProgress[1].detail)
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
        assertFalse(controller.state.value.toolProgress[0].running)
        assertTrue(controller.state.value.toolProgress[1].running)

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
}
