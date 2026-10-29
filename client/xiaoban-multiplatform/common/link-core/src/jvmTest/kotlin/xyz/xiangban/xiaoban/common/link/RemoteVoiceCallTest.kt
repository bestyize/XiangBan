package xyz.xiangban.xiaoban.common.link

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger
import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import kotlin.math.PI
import kotlin.math.sin
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import xyz.xiangban.xiaoban.common.community.RecognizerEngine
import xyz.xiangban.xiaoban.common.community.SpokenCall
import xyz.xiangban.xiaoban.common.community.SynthesizerEngine
import xyz.xiangban.xiaoban.common.community.VoicePlatform

private class FakeMic : VoicePlatform {
    val played = AtomicInteger()
    override suspend fun streamRecording(onFrame: (ByteArray) -> Unit): (() -> Unit)? {
        val running = AtomicBoolean(true)
        Thread {
            var n = 0
            while (running.get()) {
                val amplitude = if (n in 20 until 70) 3000 else 0
                val frame = ByteArray(640)
                for (i in 0 until 320) {
                    val v = if (amplitude == 0) ((i * 7 + n) % 40 - 20) else (amplitude * sin(2 * PI * (i + n * 320) * 220 / 16000)).toInt()
                    frame[2 * i] = v.toByte(); frame[2 * i + 1] = (v shr 8).toByte()
                }
                onFrame(frame)
                n++
                Thread.sleep(1)
            }
        }.also { it.isDaemon = true }.start()
        return { running.set(false) }
    }

    override fun playAudio(bytes: ByteArray, mime: String, onDone: () -> Unit): Boolean {
        played.incrementAndGet()
        Thread { Thread.sleep(5); onDone() }.also { it.isDaemon = true }.start()
        return true
    }
}

class RemoteVoiceCallTest {
    private suspend fun waitFor(ms: Long = 15_000, condition: () -> Boolean) {
        withTimeout(ms) { while (!condition()) delay(20) }
    }

    /** 控制器在另一个线程上监听事件，刚创建时发的事件可能赶不上，所以一直发到它看到为止。 */
    private suspend fun bringOnline(link: PhoneLink, controller: RemoteChatController) {
        withTimeout(10_000) {
            while (!controller.state.value.isPeerOnline) {
                link.emitEventForTest(LinkEvent.PeerOnline("dsk_1", true))
                delay(50)
            }
        }
    }

    private class Rig(val scope: CoroutineScope, val link: PhoneLink, val controller: RemoteChatController, val sent: CopyOnWriteArrayList<String>)

    private suspend fun rig(selectSession: Boolean): Rig {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val storage = InMemoryLinkStorage(LinkState(installId = "test-install-id-0123456789", device = DeviceIdentity("phn_1", "手机")))
        val link = PhoneLink(scope, HttpClient(MockEngine { respond("{}", HttpStatusCode.OK) }), storage, { null }, "手机")
        link.seedPeersForTest(listOf(PeerSummary("dsk_1", "我的电脑", false)))
        val sent = CopyOnWriteArrayList<String>()
        link.sendObserverForTest = { _, data -> sent += data.decodeToString() }
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        bringOnline(link, controller)
        if (selectSession) {
            delay(200)
            link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"sessions.page","id":"1","items":[{"id":"s_1","title":"聊天","updated_at":1,"state":"idle"}]}""".encodeToByteArray()))
            waitFor { controller.state.value.sessions.isNotEmpty() }
            controller.selectSession("s_1")
            waitFor { controller.state.value.selectedSessionId == "s_1" }
        }
        return Rig(scope, link, controller, sent)
    }

    @Test
    fun `speech is recognized and spoken on the phone while the computer only thinks`() = runBlocking {
        val r = rig(selectSession = true)
        val spoken = CopyOnWriteArrayList<String>()
        val platform = FakeMic()
        val call = SpokenCall(
            r.scope, platform, RemoteAssistant(r.controller),
            listOf(object : RecognizerEngine { override val name = "手机识别"; override suspend fun recognize(pcm: ByteArray) = "今天天气怎么样" }),
            listOf(object : SynthesizerEngine { override val name = "手机合成"; override suspend fun synthesize(text: String): ByteArray { spoken += text; return byteArrayOf(1) } }),
        )
        call.start()
        // 文字发给电脑，不带录音，也不要求电脑念
        waitFor { r.sent.any { it.contains("\"t\":\"message.send\"") && it.contains("今天天气怎么样") } }
        assertTrue(r.sent.none { it.contains("voice.chunk") || it.contains("\"speak\"") })
        // 回答流回来，手机一句一句念
        r.link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"3","session":"s_1","kind":"text.delta","data":{"text":"今天晴，"}}""".encodeToByteArray()))
        r.link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"4","session":"s_1","kind":"text.delta","data":{"text":"气温二十度。"}}""".encodeToByteArray()))
        r.link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"event","id":"5","session":"s_1","kind":"turn.done","data":{}}""".encodeToByteArray()))
        waitFor { platform.played.get() >= 1 }
        assertTrue(spoken.joinToString("").contains("气温二十度"))
        // 播完接着听
        waitFor { call.state.value.phase.name == "Listening" || call.state.value.phase.name == "Hearing" }
        call.stop()
        waitFor { call.state.value.ended }
        r.scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @OptIn(ExperimentalEncodingApi::class)
    @Test
    fun `when the phone cannot recognize the recording goes to the computer`() = runBlocking {
        val r = rig(selectSession = false)
        val phoneEngine = object : RecognizerEngine {
            override val name = "手机识别"
            override suspend fun recognize(pcm: ByteArray): String = throw xyz.xiangban.xiaoban.common.community.EngineUnavailable("模型没下载")
        }
        val call = SpokenCall(r.scope, FakeMic(), RemoteAssistant(r.controller), listOf(phoneEngine, DesktopRecognizer(r.controller)), emptyList())
        call.start()
        // 第一句：手机识别报“用不了”，提示一下，请再说一次
        waitFor { call.state.value.notice?.contains("改用") == true }
        // 第二句：录音分片发给电脑
        waitFor { r.sent.any { it.contains("\"t\":\"voice.chunk\"") && it.contains("\"last\":true") } }
        val voiceId = Regex("\"voice_id\":\"([^\"]+)\"").find(r.sent.first { it.contains("voice.chunk") })!!.groupValues[1]
        r.link.emitEventForTest(LinkEvent.Message("dsk_1", """{"t":"voice.text","id":"2","voice_id":"$voiceId","ok":true,"text":"打开下载文件夹"}""".encodeToByteArray()))
        waitFor { r.sent.any { it.contains("\"t\":\"message.send\"") && it.contains("打开下载文件夹") } }
        call.stop()
        waitFor { call.state.value.ended }
        r.scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `an offline computer ends the call right away`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val storage = InMemoryLinkStorage(LinkState(installId = "test-install-id-0123456789", device = DeviceIdentity("phn_1", "手机")))
        val link = PhoneLink(scope, HttpClient(MockEngine { respond("{}", HttpStatusCode.OK) }), storage, { null }, "手机")
        link.seedPeersForTest(listOf(PeerSummary("dsk_1", "我的电脑", false)))
        val controller = RemoteChatController(link, scope)
        controller.refreshPeers()
        val call = SpokenCall(scope, FakeMic(), RemoteAssistant(controller), emptyList(), emptyList())
        call.start()
        waitFor { call.state.value.ended }
        assertEquals("设备不在线，没法通话", call.state.value.notice)
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }
}
