package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import java.util.concurrent.atomic.AtomicInteger
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

/** 假的手机：每次录音先安静 0.4 秒、说 1 秒、再一直安静；播放和朗读马上结束。 */
internal class FakeVoicePlatform : VoicePlatform {
    val played = AtomicInteger()
    val spoken = mutableListOf<String>()
    var systemHeard: String? = null
    var recordings = 0

    override suspend fun streamRecording(onFrame: (ByteArray) -> Unit): (() -> Unit)? {
        recordings++
        val running = java.util.concurrent.atomic.AtomicBoolean(true)
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

    override fun listenOnce(onPartial: (String) -> Unit, onResult: (String?, String?) -> Unit): (() -> Unit)? {
        val text = systemHeard ?: return null
        Thread { Thread.sleep(20); onPartial(text.take(1)); onResult(text, null) }.also { it.isDaemon = true }.start()
        return {}
    }

    override fun speakText(text: String, onDone: (Boolean) -> Unit): Boolean {
        spoken += text
        Thread { Thread.sleep(5); onDone(true) }.also { it.isDaemon = true }.start()
        return true
    }

    override fun playAudio(bytes: ByteArray, mime: String, onDone: () -> Unit): Boolean {
        played.incrementAndGet()
        Thread { Thread.sleep(5); onDone() }.also { it.isDaemon = true }.start()
        return true
    }
}

class VoiceCallTest {
    private fun sse(text: String) = """data: {"choices":[{"index":0,"delta":{"content":"$text"},"finish_reason":"stop"}]}""" + "\n\ndata: [DONE]\n\n"

    private suspend fun waitFor(what: String, ms: Long = 15_000, condition: () -> Boolean) {
        withTimeout(ms) { while (!condition()) delay(20) }
    }

    @Test
    fun `a call hears a sentence, asks the assistant and speaks the answer`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val asked = AtomicInteger()
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/audio/transcriptions" -> respond("""{"text":"现在几点"}""", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
                "/v1/chat/completions" -> { asked.incrementAndGet(); respond(sse("现在是下午三点。先到这里。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream")) }
                "/v1/audio/speech" -> respond(byteArrayOf(1, 2, 3), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "audio/mpeg"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val chat = CloudChatController(api, scope)
        val platform = FakeVoicePlatform()
        val call = SpokenCall(scope, platform, CloudAssistant(chat), listOf(CloudRecognizer(api)), listOf(CloudSynthesizer(api)))
        call.start()
        waitFor("heard") { call.state.value.userText == "现在几点" }
        waitFor("answer") { call.state.value.replyText.contains("下午三点") }
        waitFor("spoken") { platform.played.get() >= 2 }
        assertEquals("现在几点", chat.state.value.messages.first { it.role == "user" }.text)
        call.stop()
        waitFor("ended") { call.state.value.ended }
        assertTrue(asked.get() >= 1)
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `when cloud recognition and speech are not open the phone's own engines are used`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val unavailable = """{"error":{"code":"transcribe_unavailable","message":"暂未开放"}}"""
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/audio/transcriptions", "/v1/audio/speech" -> respond(unavailable, HttpStatusCode.ServiceUnavailable, headersOf(HttpHeaders.ContentType, "application/json"))
                "/v1/chat/completions" -> respond(sse("好的，收到了。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val chat = CloudChatController(api, scope)
        val platform = FakeVoicePlatform().also { it.systemHeard = "你好小伴" }
        val call = SpokenCall(scope, platform, CloudAssistant(chat), listOf(CloudRecognizer(api)), listOf(CloudSynthesizer(api)))
        call.start()
        // 第一句走云端识别，失败后提示改用手机自带的；第二句起用手机自带的识别和朗读。
        waitFor("notice") { call.state.value.notice?.contains("手机自带") == true }
        waitFor("system asr") { call.state.value.userText == "你好小伴" }
        waitFor("system tts") { platform.spoken.any { it.contains("收到了") } }
        assertEquals(0, platform.played.get())
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `a call without microphone permission ends with the reason`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val api = AiGatewayApi(HttpClient(MockEngine { respond("{}", HttpStatusCode.OK) }), CommunityConfig()) { "tok" }
        val call = SpokenCall(scope, object : VoicePlatform {}, CloudAssistant(CloudChatController(api, scope)), listOf(CloudRecognizer(api)), listOf(CloudSynthesizer(api)))
        call.start()
        waitFor("ended") { call.state.value.ended }
        assertEquals("没有麦克风权限，没法通话", call.state.value.notice)
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `an engine on the phone is used first and the cloud is not asked`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val cloudAsked = AtomicInteger()
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/audio/transcriptions", "/v1/audio/speech" -> { cloudAsked.incrementAndGet(); respond("{}", HttpStatusCode.InternalServerError) }
                "/v1/chat/completions" -> respond(sse("好的，已经记下了。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val chat = CloudChatController(api, scope)
        val heardPcm = AtomicInteger()
        val onPhoneAsr = object : RecognizerEngine {
            override val name = "手机上的识别"
            override suspend fun recognize(pcm: ByteArray): String { heardPcm.set(pcm.size); return "记一下明天开会" }
        }
        val spokenByPhone = java.util.concurrent.CopyOnWriteArrayList<String>()
        val onPhoneTts = object : SynthesizerEngine {
            override val name = "手机上的合成"
            override suspend fun synthesize(text: String): ByteArray { spokenByPhone += text; return byteArrayOf(9) }
        }
        val platform = FakeVoicePlatform()
        val call = SpokenCall(scope, platform, CloudAssistant(chat), listOf(onPhoneAsr, CloudRecognizer(api)), listOf(onPhoneTts, CloudSynthesizer(api)))
        call.start()
        waitFor("heard") { call.state.value.userText == "记一下明天开会" }
        waitFor("spoken") { platform.played.get() >= 1 }
        assertTrue(heardPcm.get() > 32_000, "录音至少一秒：${heardPcm.get()}")
        assertTrue(spokenByPhone.any { it.contains("记下了") })
        assertEquals(0, cloudAsked.get())
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }
}
