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

    private fun createLink(): PhoneLink {
        val storage = InMemoryLinkStorage(
            LinkState(
                relayBase = "https://link.test",
                device = DeviceIdentity("phn_1", "手机", "priv", "pub"),
                startCounter = 1,
                peers = mapOf(
                    "dsk_1" to PeerRecord("pair_1", "dsk_1", "我的Mac电脑", "pub_dsk", "prk_123"),
                ),
            ),
        )
        return PhoneLink(scope, mockClient, storage, { "tok" }, "手机")
    }

    @Test
    fun `remote chat controller handles sessions, streaming text, tools and approvals`() = runTest {
        val link = createLink()
        link.start()
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

        // 文字增量 text.delta
        val textDelta = """{"t":"event","id":"4","session":"s_1","kind":"text.delta","data":{"text":"正在执行"}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", textDelta.encodeToByteArray()))
        assertEquals("正在执行", controller.state.value.messages.last().text)
        assertTrue(controller.state.value.messages.last().streaming)

        // 工具结束 tool.end
        val toolEnd = """{"t":"event","id":"5","session":"s_1","kind":"tool.end","data":{"id":"t_1","is_error":false}}"""
        link.emitEventForTest(LinkEvent.Message("dsk_1", toolEnd.encodeToByteArray()))
        assertFalse(controller.state.value.toolProgress[0].running)

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
}
