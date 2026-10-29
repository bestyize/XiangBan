package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.math.PI
import kotlin.math.sin
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class VoiceActivityTest {
    /** 一帧 20ms（320 个采样）。amplitude 为 0 时是很小的底噪。 */
    private fun frame(amplitude: Int, phase: Int = 0): ByteArray {
        val bytes = ByteArray(640)
        for (i in 0 until 320) {
            val v = if (amplitude == 0) Random(i + phase).nextInt(-30, 30) else (amplitude * sin(2 * PI * (i + phase) * 220 / 16000)).toInt()
            bytes[2 * i] = v.toByte(); bytes[2 * i + 1] = (v shr 8).toByte()
        }
        return bytes
    }

    @Test
    fun `speaking immediately after opening the call is not calibrated as noise`() {
        val detector = EndpointDetector()
        var result: VadResult? = null
        repeat(50) { result = detector.feed(frame(3000, it)) }
        repeat(50) { if (result?.utterance == null) result = detector.feed(frame(0, it)) }
        assertNotNull(result?.utterance)
        detector.close()
    }

    @Test
    fun `a spoken sentence ends after a pause and keeps its beginning`() {
        val vad = EndpointDetector()
        var result: VadResult? = null
        var framesFed = 0
        fun feed(frame: ByteArray) { if (result?.utterance == null) { result = vad.feed(frame); framesFed++ } }
        repeat(15) { feed(frame(0, it)) }            // 0.3 秒安静
        repeat(50) { feed(frame(3000, it)) }          // 1 秒说话
        assertNull(result!!.utterance, "还在说")
        assertTrue(result!!.speaking)
        repeat(60) { feed(frame(0, it)) }             // 1.2 秒安静
        val utterance = assertNotNull(result!!.utterance)
        // 说话 1 秒 + 结尾约 0.9 秒安静 + 开头余量 0.3 秒
        val seconds = utterance.size / 32000.0
        assertTrue(seconds in 1.9..2.4, "$seconds")
    }

    @Test
    fun `a click is not speech and silence times out`() {
        val vad = EndpointDetector()
        var out: VadResult? = null
        repeat(15) { out = vad.feed(frame(0, it)) }
        repeat(5) { out = vad.feed(frame(4000, it)) }  // 0.1 秒的响声
        repeat(60) { out = vad.feed(frame(0, it)) }
        assertNull(out!!.utterance)
        // 一直没人说话
        val quiet = EndpointDetector(maxWaitMs = 1_000)
        var last: VadResult? = null
        for (i in 0 until 80) { last = quiet.feed(frame(0, i)); if (last.timedOut) break }
        assertTrue(last!!.timedOut)
    }

    @Test
    fun `noise that was there from the start does not count as speech`() {
        val vad = EndpointDetector()
        var out: VadResult? = null
        repeat(100) { out = vad.feed(frame(250, it)) }  // 稳定的环境声，低于下限
        assertTrue(out!!.utterance == null && !out!!.speaking)
    }

    @Test
    fun `with a judge a finished sentence ends after a short pause but an unfinished one is waited out`() {
        // 说完了：停顿约 0.35 秒时被问一次，回答“完整”后立刻结束。
        var vad = EndpointDetector(useJudge = true)
        var result: VadResult? = null
        var ask: VadResult? = null
        var silentFrames = 0
        repeat(15) { vad.feed(frame(0, it)) }
        repeat(50) { vad.feed(frame(3000, it)) }
        while (ask == null && silentFrames < 100) { result = vad.feed(frame(0, silentFrames)); silentFrames++; if (result.ask != null) ask = result }
        assertNotNull(ask, "停顿后该问一次")
        assertTrue(silentFrames * 20 in 340..400, "${silentFrames * 20}ms")
        vad.verdict(ask.askTicket, true)
        val ended = vad.feed(frame(0, 0))
        assertNotNull(ended.utterance)
        assertTrue(ended.utterance!!.size / 32000.0 in 1.2..1.8, "${ended.utterance!!.size}")

        // 没说完：回答“不完整”，不会结束，隔一会儿再问，直到停顿 1.6 秒。
        vad = EndpointDetector(useJudge = true)
        repeat(15) { vad.feed(frame(0, it)) }
        repeat(50) { vad.feed(frame(3000, it)) }
        var asks = 0
        var finished: VadResult? = null
        for (i in 0 until 200) {
            val r = vad.feed(frame(0, i))
            if (r.ask != null) { asks++; vad.verdict(r.askTicket, false) }
            if (r.utterance != null) { finished = r; break }
        }
        assertEquals(2, asks)
        assertNotNull(finished, "停顿太久还是要结束")
    }

    @Test
    fun `an old verdict does not count once the speaker talks again`() {
        val vad = EndpointDetector(useJudge = true)
        repeat(15) { vad.feed(frame(0, it)) }
        repeat(50) { vad.feed(frame(3000, it)) }
        var staleTicket = -1
        for (i in 0 until 40) { val r = vad.feed(frame(0, i)); if (r.ask != null) staleTicket = r.askTicket }
        assertTrue(staleTicket >= 0)
        repeat(10) { vad.feed(frame(3000, it)) }       // 又开口了
        vad.verdict(staleTicket, true)                  // 迟到的旧回答
        val r = vad.feed(frame(3000, 0))
        assertNull(r.utterance)
    }

    @Test
    fun `strict mode only accepts clearly loud voices`() {
        fun speaks(strict: Boolean): Boolean {
            val vad = EndpointDetector(strict = strict)
            var r: VadResult? = null
            repeat(15) { vad.feed(frame(0, it)) }
            repeat(40) { r = vad.feed(frame(700, it)) }
            return r!!.speaking
        }
        assertTrue(speaks(false))
        assertTrue(!speaks(true))
    }

    @Test
    fun `wav has a proper header`() {
        val wav = pcm16ToWav(ByteArray(100))
        assertEquals(144, wav.size)
        assertEquals("RIFF", wav.copyOfRange(0, 4).decodeToString())
        assertEquals("WAVEfmt ", wav.copyOfRange(8, 16).decodeToString())
        assertEquals("data", wav.copyOfRange(36, 40).decodeToString())
        assertEquals(36 + 100, (wav[4].toInt() and 0xFF) or ((wav[5].toInt() and 0xFF) shl 8))
        assertEquals(16000, (wav[24].toInt() and 0xFF) or ((wav[25].toInt() and 0xFF) shl 8))
    }

    @Test
    fun `a streaming answer is cut into sentences without code or markdown marks`() {
        val splitter = SpeechSplitter()
        val said = ArrayList<String>()
        val text = "## 结果\n好的，我先看了一下这个文件夹里的内容。**一共有三个**重复文件！\n```kotlin\nval x = 1\n```\n详见 [文档](https://a.b)，没有别的了"
        // 一次几个字地喂，模拟流式
        text.chunked(5).forEach { said += splitter.feed(it) }
        said += splitter.flush()
        assertEquals(listOf("结果", "好的，我先看了一下这个文件夹里的内容。", "一共有三个重复文件！", "详见 文档，没有别的了"), said)
    }

    @Test
    fun `very long answers stop being read`() {
        val splitter = SpeechSplitter(maxChars = 30)
        val said = splitter.feed("这是一句比较长的话用来测试上限。" .repeat(10)) + splitter.flush()
        assertTrue(said.sumOf { it.length } <= 60, "${said.sumOf { it.length }}")
        assertTrue(said.size < 10)
    }

    @Test
    fun `cloud recognition uploads a wav and speech comes back as audio`() = runTest {
        var uploaded = ""
        var spoken = ""
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/audio/transcriptions" -> {
                    uploaded = request.body.toString() + request.headers[HttpHeaders.Authorization]
                    respond("""{"text":" 帮我设个提醒 "}""", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
                }
                "/v1/audio/speech" -> {
                    spoken = request.headers[HttpHeaders.Authorization].orEmpty()
                    respond(byteArrayOf(1, 2, 3), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "audio/mpeg"))
                }
                else -> respond("{}", HttpStatusCode.NotFound)
            }
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        assertEquals("帮我设个提醒", api.transcribe(pcm16ToWav(ByteArray(3200))))
        assertTrue(uploaded.contains("Bearer tok"), uploaded)
        assertEquals(listOf<Byte>(1, 2, 3), api.speak("你好").toList())
        assertEquals("Bearer tok", spoken)
    }

    @Test
    fun `a gateway that has not opened voice says so with its code`() = runTest {
        val client = HttpClient(MockEngine {
            respond("""{"error":{"code":"transcribe_unavailable","message":"小伴 AI 语音识别暂未开放"}}""", HttpStatusCode.ServiceUnavailable, headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val api = AiGatewayApi(client, CommunityConfig()) { "tok" }
        val error = runCatching { api.transcribe(ByteArray(100)) }.exceptionOrNull() as ApiException
        assertEquals("transcribe_unavailable", error.code)
        val speech = runCatching { api.speak("你好") }.exceptionOrNull() as ApiException
        assertEquals(503, speech.status)
    }
}
