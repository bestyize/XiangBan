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
    /** 每段声音开始播放时调用，测试用它看通话状态。 */
    var onPlay: (() -> Unit)? = null

    override suspend fun streamRecording(onFrame: (ByteArray) -> Unit): (() -> Unit)? {
        recordings++
        val running = java.util.concurrent.atomic.AtomicBoolean(true)
        Thread {
            var n = 0
            while (running.get()) {
                val amplitude = if (n % 200 in 20 until 70) 3000 else 0
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

    val clips: MutableList<String> = java.util.Collections.synchronizedList(mutableListOf())

    override fun playAudio(bytes: ByteArray, mime: String, onDone: () -> Unit): Boolean {
        played.incrementAndGet()
        clips += bytes.decodeToString()
        onPlay?.invoke()
        Thread { Thread.sleep(5); onDone() }.also { it.isDaemon = true }.start()
        return true
    }
}

/**
 * 假的手机：麦克风平时安静，[speak] 一次才出一段声音；播放 5 毫秒就结束，播完调 [afterPlay]。
 * 每 [frameMs] 毫秒送一帧（一帧算 20 毫秒的声音）：要看判停和识别谁先谁后时用 20，按真实的快慢来。
 */
internal class BurstPlatform(private val frameMs: Long = 1) : VoicePlatform {
    private val burst = AtomicInteger()
    val played = AtomicInteger()
    var afterPlay: ((Int) -> Unit)? = null

    fun speak(frames: Int) = burst.set(frames)

    override suspend fun streamRecording(onFrame: (ByteArray) -> Unit): (() -> Unit) {
        val running = java.util.concurrent.atomic.AtomicBoolean(true)
        Thread {
            var n = 0
            while (running.get()) {
                val loud = burst.getAndUpdate { if (it > 0) it - 1 else 0 } > 0
                val frame = ByteArray(640)
                for (i in 0 until 320) {
                    val v = if (loud) (3000 * sin(2 * PI * (i + n * 320) * 220 / 16000)).toInt() else ((i * 7 + n) % 40 - 20)
                    frame[2 * i] = v.toByte(); frame[2 * i + 1] = (v shr 8).toByte()
                }
                onFrame(frame)
                n++
                Thread.sleep(frameMs)
            }
        }.also { it.isDaemon = true }.start()
        return { running.set(false) }
    }

    override fun playAudio(bytes: ByteArray, mime: String, onDone: () -> Unit): Boolean {
        val count = played.incrementAndGet()
        Thread { Thread.sleep(5); onDone(); afterPlay?.invoke(count) }.also { it.isDaemon = true }.start()
        return true
    }
}

class VoiceCallTest {
    private fun sse(text: String) = """data: {"choices":[{"index":0,"delta":{"content":"$text"},"finish_reason":"stop"}]}""" + "\n\ndata: [DONE]\n\n"

    private suspend fun waitFor(what: String, ms: Long = 15_000, condition: () -> Boolean) {
        withTimeout(ms) { while (!condition()) delay(20) }
    }

    @Test
    fun `voice context reaches the model but is hidden from the visible message`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        var requestBody = ""
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath == "/v1/chat/completions") {
                requestBody = (request.body as io.ktor.http.content.TextContent).text
                respond(sse("好的。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
            } else respond("{}", HttpStatusCode.OK)
        })
        val chat = CloudChatController(AiGatewayApi(client, CommunityConfig()) { "tok" }, scope)
        chat.sendVoice("继续说", voiceInterruptionNote("刚才听到的内容"))
        waitFor("response") { !chat.state.value.sending }
        assertEquals("继续说", chat.state.value.messages.first().text)
        assertTrue(requestBody.contains("语音打断"))
        assertTrue(requestBody.contains("刚才听到的内容"))
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `a quiet call remains active until the user ends it`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val platform = object : VoicePlatform {
            override suspend fun streamRecording(onFrame: (ByteArray) -> Unit): (() -> Unit) {
                val active = java.util.concurrent.atomic.AtomicBoolean(true)
                Thread {
                    while (active.get()) { onFrame(ByteArray(640)); Thread.sleep(1) }
                }.also { it.isDaemon = true }.start()
                return { active.set(false) }
            }
        }
        val api = AiGatewayApi(HttpClient(MockEngine { respond("{}") }), CommunityConfig()) { "tok" }
        val call = SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(CloudRecognizer(api)), emptyList())
        call.start()
        waitFor("listening") { call.state.value.phase == CallPhase.Listening }
        delay(2_200) // 加速麦克风已超过三个原来的 10 秒等待周期。
        assertTrue(!call.state.value.ended)
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
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
        val lit = java.util.Collections.synchronizedList(mutableListOf<String>())
        platform.onPlay = { call.state.value.let { s -> s.reading?.let { lit += s.replyText.substring(it.first, it.last + 1) } } }
        call.start()
        waitFor("heard") { call.state.value.userText == "现在几点" }
        waitFor("answer") { call.state.value.replyText.contains("下午三点") }
        waitFor("spoken") { platform.played.get() >= 2 }
        // 念到哪句，通话页就点亮哪句。
        assertEquals(listOf("现在是下午三点。", "先到这里。"), lit.toList())
        waitFor("reading cleared") { call.state.value.reading == null }
        assertEquals("现在几点", chat.state.value.messages.first { it.role == "user" }.text)
        call.stop()
        waitFor("ended") { call.state.value.ended }
        assertTrue(asked.get() >= 1)
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    private fun callWith(heard: String, asked: AtomicInteger, scope: CoroutineScope, platform: FakeVoicePlatform): SpokenCall {
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/audio/transcriptions" -> respond("""{"text":"$heard"}""", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
                "/v1/chat/completions" -> { asked.incrementAndGet(); respond(sse("现在是下午三点。先到这里。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream")) }
                "/v1/audio/speech" -> respond(byteArrayOf(1, 2, 3), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "audio/mpeg"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        return SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(CloudRecognizer(api)), listOf(CloudSynthesizer(api)))
    }

    @Test
    fun `a speech engine that gives up with a timeout does not freeze the call`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val asked = AtomicInteger()
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/audio/transcriptions" -> respond("""{"text":"现在几点"}""", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
                "/v1/chat/completions" -> { asked.incrementAndGet(); respond(sse("现在是下午三点。先到这里。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream")) }
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        // withTimeout 超时抛的是取消，不是普通错误；以前它会让合成那边悄悄结束，播放那边一直等，通话卡在“正在说”。
        val stuck = object : SynthesizerEngine {
            override val name = "卡住的合成"
            override suspend fun synthesize(text: String): ByteArray = throw kotlinx.coroutines.CancellationException("timed out")
        }
        val call = SpokenCall(scope, FakeVoicePlatform(), CloudAssistant(CloudChatController(api, scope)), listOf(CloudRecognizer(api)), listOf(stuck))
        call.setBargeIn(false)
        call.start()
        waitFor("answer") { call.state.value.replyText.contains("先到这里") }
        // 回答完接着听下一句：假麦克风会再说一句，小伴就会被问第二次。
        waitFor("next turn", ms = 8_000) { asked.get() >= 2 }
        assertTrue(!call.state.value.ended)
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `with smart turn the sentence is recognised during the pause, not after it`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val judging = java.util.concurrent.atomic.AtomicBoolean(false)
        val judged = java.util.concurrent.atomic.AtomicBoolean(false)
        val whileJudging = java.util.concurrent.atomic.AtomicBoolean(false)
        val recognitionStarted = kotlinx.coroutines.CompletableDeferred<Unit>()
        val judgeStarted = kotlinx.coroutines.CompletableDeferred<Unit>()
        val afterJudging = AtomicInteger()
        val lateWhenAsked = AtomicInteger(-1)
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/chat/completions" -> { lateWhenAsked.compareAndSet(-1, afterJudging.get()); respond(sse("现在是下午三点。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream")) }
                "/v1/audio/speech" -> respond(byteArrayOf(1, 2, 3), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "audio/mpeg"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val local = object : RecognizerEngine {
            override val name = "本地识别"
            override val local = true
            override suspend fun recognize(pcm: ByteArray): String {
                recognitionStarted.complete(Unit)
                judgeStarted.await()
                if (judging.get()) whileJudging.set(true)
                if (judged.get()) afterJudging.incrementAndGet()
                delay(200)
                return "现在几点"
            }
        }
        val judge: suspend (ByteArray) -> Boolean? = {
            judging.set(true)
            judgeStarted.complete(Unit)
            // Both jobs must start independently. A sequential implementation now times out,
            // while executor scheduling order cannot cause a false failure.
            recognitionStarted.await()
            delay(300)
            judging.set(false); judged.set(true); true
        }
        val platform = BurstPlatform(frameMs = 20)
        // 0.6 秒的一句话：短于 1 秒，不会先出中间结果，只能靠停顿时的识别。
        platform.speak(30)
        val call = SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(local), listOf(CloudSynthesizer(api)), judge)
        call.start()
        waitFor("asked") { lateWhenAsked.get() >= 0 }
        assertTrue(whileJudging.get(), "识别应该和判停同时开始")
        assertEquals(0, lateWhenAsked.get(), "判定说完以后不该再识别一遍")
        assertEquals("现在几点", call.state.value.userText)
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `later sentences are synthesised while the first is playing, and played in order`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val sentences = listOf("第一句话说完了。", "第二句话说完了。", "第三句话说完了。", "第四句话说完了。", "第五句话说完了。")
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/audio/transcriptions" -> respond("""{"text":"现在几点"}""", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
                "/v1/chat/completions" -> respond(sse(sentences.joinToString("")), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val inFlight = AtomicInteger()
        val most = AtomicInteger()
        val slow = object : SynthesizerEngine {
            override val name = "慢合成"
            override suspend fun synthesize(text: String): ByteArray {
                most.accumulateAndGet(inFlight.incrementAndGet(), ::maxOf)
                try { delay(150); return text.encodeToByteArray() } finally { inFlight.decrementAndGet() }
            }
        }
        val platform = FakeVoicePlatform()
        val call = SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(CloudRecognizer(api)), listOf(slow))
        call.setBargeIn(false)
        call.start()
        waitFor("five sentences played") { platform.clips.size >= 5 }
        assertEquals(sentences, platform.clips.take(5))
        assertTrue(most.get() in 2..3, "同时在合成的句子数：${most.get()}")
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `its own voice heard right after it stops talking is not taken as a question`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val asked = AtomicInteger()
        val echoHeard = AtomicInteger()
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/chat/completions" -> { asked.incrementAndGet(); respond(sse("现在是下午三点。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream")) }
                "/v1/audio/speech" -> respond(byteArrayOf(1, 2, 3), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "audio/mpeg"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        // 第一句是你问的；之后听到的都是小伴自己那句回答的尾音。
        val local = object : RecognizerEngine {
            override val name = "本地识别"
            override val local = true
            override suspend fun recognize(pcm: ByteArray): String =
                if (asked.get() == 0) "现在几点" else { echoHeard.incrementAndGet(); "现在是下午三点" }
        }
        val platform = BurstPlatform()
        // 回答念完的一瞬间，麦克风录到喇叭的尾音。
        platform.afterPlay = { platform.speak(300) }
        platform.speak(30)
        val call = SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(local), listOf(CloudSynthesizer(api)))
        call.setBargeIn(false)
        call.start()
        waitFor("answered and spoken") { asked.get() == 1 && platform.played.get() >= 1 }
        waitFor("echo recognised") { echoHeard.get() > 0 }
        delay(1_500)
        assertEquals(1, asked.get(), "回声不该被当成第二个问题发出去")
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }


    private fun fiveSentenceCall(scope: CoroutineScope, platform: FakeVoicePlatform, engine: SynthesizerEngine, sentences: List<String>): SpokenCall {
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/audio/transcriptions" -> respond("""{"text":"现在几点"}""", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
                "/v1/chat/completions" -> respond(sse(sentences.joinToString("")), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        return SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(CloudRecognizer(api)), listOf(engine)).also { it.setBargeIn(false) }
    }

    @Test
    fun `one failed sentence uses the phone's voice and the next one tries the online voice again`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val sentences = listOf("第一句话说完了。", "第二句话说完了。", "第三句话说完了。", "第四句话说完了。", "第五句话说完了。")
        val flaky = object : SynthesizerEngine {
            override val name = "Edge 语音合成"
            override suspend fun synthesize(text: String): ByteArray =
                if (text == sentences[1]) throw EngineUnavailable("无法连接语音服务，请检查网络") else text.encodeToByteArray()
        }
        val platform = FakeVoicePlatform()
        val call = fiveSentenceCall(scope, platform, flaky, sentences)
        call.start()
        waitFor("all five read") { platform.clips.size + platform.spoken.size >= 5 }
        assertEquals(listOf(sentences[1]), platform.spoken.toList(), "只有没合成出来的那句用手机自带的声音")
        assertEquals(sentences - sentences[1], platform.clips.take(4), "后面的句子还是在线的声音")
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `an online voice that keeps failing is given up after a few sentences`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val sentences = (1..8).map { "第${it}句话说完了。" }
        val calls = AtomicInteger()
        val down = object : SynthesizerEngine {
            override val name = "Edge 语音合成"
            override suspend fun synthesize(text: String): ByteArray { calls.incrementAndGet(); throw EngineUnavailable("语音服务连接超时") }
        }
        val platform = FakeVoicePlatform()
        val call = fiveSentenceCall(scope, platform, down, sentences)
        call.start()
        waitFor("all read by the phone") { platform.spoken.size >= 8 }
        assertEquals(sentences, platform.spoken.take(8))
        // 连着四句失败就不再用它；那时已经开始合成的最多还有两句。
        assertTrue(calls.get() in 4..6, "在线合成被试了 ${calls.get()} 次")
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `saying hang up ends the call without asking the assistant`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val asked = AtomicInteger()
        val call = callWith("好，挂断吧。", asked, scope, FakeVoicePlatform())
        call.start()
        waitFor("ended by voice") { call.state.value.ended }
        assertEquals("好，挂断吧。", call.state.value.userText)
        assertEquals(0, asked.get())
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `with reading turned off the answer is shown but not spoken`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val asked = AtomicInteger()
        val platform = FakeVoicePlatform()
        val call = callWith("现在几点", asked, scope, platform)
        call.setSilent(true)
        call.start()
        waitFor("answer") { call.state.value.replyText.contains("先到这里") }
        // 回答完回到听：回答文字还留着，一个字也没念。
        waitFor("listening again") { asked.get() >= 1 && call.state.value.phase == CallPhase.Listening && call.state.value.replyText.isNotEmpty() }
        assertEquals(0, platform.played.get())
        assertTrue(platform.spoken.isEmpty())
        assertEquals(null, call.state.value.reading)
        call.stop()
        waitFor("ended") { call.state.value.ended }
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

    /** 按需出声的假麦克风：平时是安静的，[speak] 之后接着来一段人声。 */
    private class ScriptedMic : VoicePlatform {
        private val voiced = AtomicInteger()
        val playing = java.util.concurrent.atomic.AtomicBoolean(false)
        val stoppedAudio = AtomicInteger()
        fun speak(frames: Int = 60) { voiced.addAndGet(frames) }

        override suspend fun streamRecording(onFrame: (ByteArray) -> Unit): (() -> Unit)? {
            val running = java.util.concurrent.atomic.AtomicBoolean(true)
            Thread {
                var n = 0
                while (running.get()) {
                    val loud = voiced.get() > 0 && n >= 15
                    if (loud) voiced.decrementAndGet()
                    val frame = ByteArray(640)
                    for (i in 0 until 320) {
                        val v = if (loud) (3000 * sin(2 * PI * (i + n * 320) * 220 / 16000)).toInt() else ((i * 7 + n) % 40 - 20)
                        frame[2 * i] = v.toByte(); frame[2 * i + 1] = (v shr 8).toByte()
                    }
                    onFrame(frame)
                    n++
                    Thread.sleep(1)
                }
            }.also { it.isDaemon = true }.start()
            return { running.set(false) }
        }

        override fun playAudio(bytes: ByteArray, mime: String, onDone: () -> Unit): Boolean { playing.set(true); return true }
        override fun stopAudio() { if (playing.getAndSet(false)) stoppedAudio.incrementAndGet() }
    }

    @Test
    fun `talking over the assistant interrupts it and the new sentence is heard`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val requests = java.util.concurrent.CopyOnWriteArrayList<String>()
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/chat/completions" -> {
                    requests += (request.body as io.ktor.http.content.TextContent).text
                    respond(sse("这是一句很长的回答，我会说很久很久。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
                }
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val chat = CloudChatController(api, scope)
        val heard = java.util.concurrent.CopyOnWriteArrayList<String>()
        val phoneAsr = object : RecognizerEngine {
            override val name = "手机识别"
            override suspend fun recognize(pcm: ByteArray): String { val text = if (heard.isEmpty()) "第一句" else "第二句"; heard += text; return text }
        }
        val tts = object : SynthesizerEngine { override val name = "合成"; override suspend fun synthesize(text: String) = byteArrayOf(1) }
        val platform = ScriptedMic()
        val call = SpokenCall(scope, platform, CloudAssistant(chat), listOf(phoneAsr), listOf(tts))
        call.start()
        waitFor("listening") { call.state.value.phase == CallPhase.Listening }
        platform.speak()
        waitFor("first") { call.state.value.userText == "第一句" }
        waitFor("speaking") { platform.playing.get() }
        // 小伴正在“说”（播放没结束）：用户开口
        platform.speak()
        waitFor("interrupted") { platform.stoppedAudio.get() >= 1 }
        waitFor("second") { call.state.value.userText == "第二句" }
        waitFor("second request") { requests.size >= 2 }
        assertEquals(listOf("第一句", "第二句"), heard.toList())
        assertTrue(requests.last().contains("语音打断"))
        assertTrue(requests.last().contains("这是一句很长的回答"))
        assertEquals("第二句", chat.state.value.messages.last { it.role == "user" }.text)
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `with barge in turned off the assistant keeps talking`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/chat/completions" -> respond(sse("这是一句很长的回答，我会说很久很久。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val phoneAsr = object : RecognizerEngine { override val name = "手机识别"; override suspend fun recognize(pcm: ByteArray) = "第一句" }
        val tts = object : SynthesizerEngine { override val name = "合成"; override suspend fun synthesize(text: String) = byteArrayOf(1) }
        val platform = ScriptedMic()
        val call = SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(phoneAsr), listOf(tts))
        call.setBargeIn(false)
        call.start()
        waitFor("listening") { call.state.value.phase == CallPhase.Listening }
        platform.speak()
        waitFor("speaking") { platform.playing.get() }
        platform.speak()
        delay(800)
        assertTrue(platform.playing.get(), "不该被打断")
        assertEquals(0, platform.stoppedAudio.get())
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `a confirmation during a call is shown on the call and answered without hanging up`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val asr = object : RecognizerEngine { override val name = "手机识别"; override suspend fun recognize(pcm: ByteArray) = "把临时文件删掉" }
        val tts = object : SynthesizerEngine { override val name = "合成"; override suspend fun synthesize(text: String) = byteArrayOf(1) }
        val assistant = object : CallAssistant {
            @Volatile var answered: String? = null
            override fun unavailableReason(): String? = null
            override fun ask(text: String) {}
            override fun cancel() {}
            override fun reply() = AssistantReply(if (answered == null) "" else "好的，已经处理。", busy = answered == null)
            override fun prompt() = if (answered != null) null else CallPrompt(
                "小伴想要运行命令：bash", "rm -rf ./tmp", null, false,
                listOf(CallPromptOption("once", "允许一次", CallPromptOption.Role.Allow), CallPromptOption("deny", "拒绝", CallPromptOption.Role.Deny)),
            )
            override fun answerPrompt(id: String) { answered = id }
        }
        val platform = ScriptedMic()
        val call = SpokenCall(scope, platform, assistant, listOf(asr), listOf(tts))
        call.start()
        waitFor("listening") { call.state.value.phase == CallPhase.Listening }
        platform.speak()
        waitFor("prompt shown") { call.state.value.prompt != null }
        assertEquals(listOf("once", "deny"), call.state.value.prompt!!.options.map { it.id })
        assertEquals(null, call.state.value.notice, "能在通话里回答的确认，不该再提示“请挂断”")
        call.answerPrompt("once")
        waitFor("reply after answer") { call.state.value.replyText.contains("已经处理") }
        assertEquals("once", assistant.answered)
        assertEquals(null, call.state.value.prompt)
        assertTrue(!call.state.value.ended, "回答之后通话还在")
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `while muted nothing is heard and after unmuting the next sentence is`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/chat/completions" -> respond(sse("好的。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val heard = AtomicInteger()
        val asr = object : RecognizerEngine { override val name = "手机识别"; override suspend fun recognize(pcm: ByteArray): String { heard.incrementAndGet(); return "你好" } }
        val tts = object : SynthesizerEngine { override val name = "合成"; override suspend fun synthesize(text: String) = byteArrayOf(1) }
        val platform = ScriptedMic()
        val call = SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(asr), listOf(tts))
        call.start()
        waitFor("listening") { call.state.value.phase == CallPhase.Listening }
        call.setMuted(true)
        assertTrue(call.state.value.muted)
        platform.speak()
        delay(1500)
        assertEquals(0, heard.get(), "静音时不该识别到任何话")
        assertEquals("", call.state.value.userText)
        call.setMuted(false)
        delay(300) // 取消静音后先有一小段安静，识别才会把接下来的声音当成人声
        platform.speak()
        waitFor("heard after unmute") { call.state.value.userText == "你好" }
        assertTrue(!call.state.value.muted)
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `a local recognizer judges whether the sentence is finished from its punctuation`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/chat/completions" -> respond(sse("好的。"), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "text/event-stream"))
                else -> respond("{}", HttpStatusCode.OK)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val calls = AtomicInteger()
        val local = object : RecognizerEngine {
            override val name = "手机识别"
            override val local = true
            override suspend fun recognize(pcm: ByteArray): String { calls.incrementAndGet(); return "现在几点了？" }
        }
        val platform = ScriptedMic()
        val call = SpokenCall(scope, platform, CloudAssistant(CloudChatController(api, scope)), listOf(local), listOf(object : SynthesizerEngine {
            override val name = "合成"
            override suspend fun synthesize(text: String) = byteArrayOf(1)
        }))
        call.start()
        waitFor("listening") { call.state.value.phase == CallPhase.Listening }
        platform.speak()
        waitFor("heard") { call.state.value.userText == "现在几点了？" }
        // 停顿时问了一次“说完了吗”（识别一次），收尾后又识别一次拿最终文字。
        assertTrue(calls.get() >= 2, "${calls.get()}")
        call.stop()
        waitFor("ended") { call.state.value.ended }
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }

    @Test
    fun `barge in is on by default even without echo cancellation`() = runBlocking {
        val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
        val api = AiGatewayApi(HttpClient(MockEngine { respond("{}", HttpStatusCode.OK) }), CommunityConfig()) { "tok" }
        val noEcho = object : VoicePlatform { override fun bargeInSafe() = false }
        val call = SpokenCall(scope, noEcho, CloudAssistant(CloudChatController(api, scope)), emptyList(), emptyList())
        assertTrue(call.state.value.bargeIn)
        assertEquals(null, call.state.value.notice)
        scope.coroutineContext[kotlinx.coroutines.Job]?.cancel()
        Unit
    }
}
