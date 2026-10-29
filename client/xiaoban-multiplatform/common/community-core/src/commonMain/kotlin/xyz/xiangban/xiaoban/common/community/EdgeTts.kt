package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.plugins.websocket.webSocketSession
import io.ktor.client.request.header
import io.ktor.websocket.Frame
import io.ktor.websocket.close
import io.ktor.websocket.readBytes
import io.ktor.websocket.readText
import kotlin.random.Random
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.withTimeout

// 微软 Edge “大声朗读”的语音合成，手机直接连。协议、令牌、分句都和电脑端 `src-tauri/src/tts/` 是同一套，改动要两边一起改。

/** Edge 朗读的 Sec-MS-GEC（v2）令牌，从电脑端 `tts/drm.rs` 一行一行移植，结果必须和它一致。 */
internal object EdgeDrm {
    private const val WinEpoch = 11_644_473_600L
    private const val TrustedClientToken = "6A5AA1D4EAFF4E9FB37E23D68491D6F4"

    private val Sha256Initial = uintArrayOf(0x6A09E667u, 0xBB67AE85u, 0x3C6EF372u, 0xA54FF53Au, 0x510E527Fu, 0x9B05688Cu, 0x1F83D9ABu, 0x5BE0CD19u)
    private val Sha256State = uintArrayOf(0xFD9FBEA2u, 0x91243CE7u, 0xBE499094u, 0xCD2E1E0Au, 0x6C006F49u, 0x04BD35ABu, 0xFEB0A7E4u, 0x03EFFBD2u)
    private val Sha256Schedule = uintArrayOf(
        0xEB24BF9Eu, 0x1558691Fu, 0xA0C09EE2u, 0x0CFA32BAu, 0xA622EE8Cu, 0xD1DD9491u, 0xC0DD25A0u,
        0xF9E70AB6u, 0xF0C31573u, 0x7058CE4Eu, 0x7188E606u, 0x00E5AB33u, 0xD258B060u, 0x72766564u,
    )
    private val RoundConstants = uintArrayOf(
        0x428A2F98u, 0x71374491u, 0xB5C0FBCFu, 0xE9B5DBA5u, 0x3956C25Bu, 0x59F111F1u, 0x923F82A4u, 0xAB1C5ED5u,
        0xD807AA98u, 0x12835B01u, 0x243185BEu, 0x550C7DC3u, 0x72BE5D74u, 0x80DEB1FEu, 0x9BDC06A7u, 0xC19BF174u,
        0xE49B69C1u, 0xEFBE4786u, 0x0FC19DC6u, 0x240CA1CCu, 0x2DE92C6Fu, 0x4A7484AAu, 0x5CB0A9DCu, 0x76F988DAu,
        0x983E5152u, 0xA831C66Du, 0xB00327C8u, 0xBF597FC7u, 0xC6E00BF3u, 0xD5A79147u, 0x06CA6351u, 0x14292967u,
        0x27B70A85u, 0x2E1B2138u, 0x4D2C6DFCu, 0x53380D13u, 0x650A7354u, 0x766A0ABBu, 0x81C2C92Eu, 0x92722C85u,
        0xA2BFE8A1u, 0xA81A664Bu, 0xC24B8B70u, 0xC76C51A3u, 0xD192E819u, 0xD6990624u, 0xF40E3585u, 0x106AA070u,
        0x19A4C116u, 0x1E376C08u, 0x2748774Cu, 0x34B0BCB5u, 0x391C0CB3u, 0x4ED8AA4Au, 0x5B9CCA4Fu, 0x682E6FF3u,
        0x748F82EEu, 0x78A5636Fu, 0x84C87814u, 0x8CC70208u, 0x90BEFFFAu, 0xA4506CEBu, 0xBEF9A3F7u, 0xC67178F2u,
    )
    private const val Sha256Adjustment = 0xC9DC06A7u
    private const val Sigma0Adjustment = 0x059C0B80u
    private const val Sigma1Adjustment = 0x000B92C0u

    private val Cs64Coefficients = arrayOf(
        0xE82E91B1u to 0xB32700D0u, 0x44DE24C6u to 0x88DFF51Du, 0x3BEB65BCu to 0x29514038u, 0xC89C5933u to 0xB328CBDFu,
        0x8D1EC211u to 0xD405B4D5u, 0x314EF5D5u to 0x26E55F3Cu, 0xE92491B1u to 0x607801C6u, 0xF0A5B9D5u to 0x82B2C1FBu,
        0x06064CF3u to 0xFA51A3DAu, 0xDA79FA60u to 0x207CC464u, 0xBED20560u to 0x3C3C74F3u, 0x89F9C9F8u to 0xD2D14B61u,
        0x35FA38C6u to 0xBB04A2C1u, 0xCB49E3C6u to 0xC5626167u,
    )
    private const val Cs64Factor = 0x82893933u
    private const val Cs64Adjustment0 = 0xF2F90B1Au
    private const val Cs64Adjustment1 = 0x0B1A0000u

    private fun sigma0(x: UInt) = x.rotateRight(7) xor x.rotateRight(18) xor (x shr 3)
    private fun sigma1(x: UInt) = x.rotateRight(17) xor x.rotateRight(19) xor (x shr 10)

    private fun rounds(state: UIntArray, words: UIntArray, first: Int): UIntArray {
        var a = state[0]; var b = state[1]; var c = state[2]; var d = state[3]
        var e = state[4]; var f = state[5]; var g = state[6]; var h = state[7]
        for (i in first until 64) {
            val k = if (first == 14 && i == 14) Sha256Adjustment else RoundConstants[i]
            val s1 = e.rotateRight(6) xor e.rotateRight(11) xor e.rotateRight(25)
            val ch = g xor (e and (f xor g))
            val t1 = h + s1 + ch + k + words[i]
            val s0 = a.rotateRight(2) xor a.rotateRight(13) xor a.rotateRight(22)
            val maj = (a and b) or (c and (a or b))
            val t2 = s0 + maj
            h = g; g = f; f = e; e = d + t1; d = c; c = b; b = a; a = t1 + t2
        }
        return uintArrayOf(a, b, c, d, e, f, g, h)
    }

    private fun expand(words: UIntArray, first: Int) {
        for (i in first until 64) words[i] = sigma1(words[i - 2]) + words[i - 7] + sigma0(words[i - 15]) + words[i - 16]
    }

    private fun beWord(data: ByteArray, at: Int): UInt =
        ((data[at].toUInt() and 0xFFu) shl 24) or ((data[at + 1].toUInt() and 0xFFu) shl 16) or ((data[at + 2].toUInt() and 0xFFu) shl 8) or (data[at + 3].toUInt() and 0xFFu)

    private fun leWord(data: ByteArray, at: Int): UInt =
        (data[at].toUInt() and 0xFFu) or ((data[at + 1].toUInt() and 0xFFu) shl 8) or ((data[at + 2].toUInt() and 0xFFu) shl 16) or ((data[at + 3].toUInt() and 0xFFu) shl 24)

    private fun init(data: ByteArray): UIntArray {
        val words = UIntArray(64)
        words[14] = ((data[0].toUInt() and 0xFFu) shl 16) or ((data[1].toUInt() and 0xFFu) shl 8) or (data[2].toUInt() and 0xFFu)
        words[15] = beWord(data, 3)
        for (i in 16 until 30) {
            var value = Sha256Schedule[i - 16]
            if (i - 2 >= 14) {
                var s1 = sigma1(words[i - 2])
                if (i == 16) s1 = s1 xor Sigma1Adjustment
                value += s1
            }
            if (i - 7 >= 14) value += words[i - 7]
            if (i - 15 >= 14) {
                var s0 = sigma0(words[i - 15])
                if (i == 29) s0 = s0 xor Sigma0Adjustment
                value += s0
            }
            words[i] = value
        }
        words[30] = sigma1(words[28]) + words[23] + sigma0(words[15]) + words[14] + Sha256Adjustment - RoundConstants[14]
        expand(words, 31)
        val working = rounds(Sha256State, words, 14)
        return UIntArray(8) { Sha256Initial[it] + working[it] }
    }

    private fun digest(data: ByteArray): ByteArray {
        val length = 57 + data.size
        val zeros = (((55 - length) % 64) + 64) % 64
        val padded = ByteArray(data.size - 7 + 1 + zeros + 8)
        data.copyInto(padded, 0, 7, data.size)
        padded[data.size - 7] = 0x80.toByte()
        val bits = length.toLong() * 8
        for (i in 0 until 8) padded[padded.size - 1 - i] = (bits shr (8 * i)).toByte()
        var state = init(data.copyOfRange(0, 7))
        var at = 0
        while (at < padded.size) {
            val words = UIntArray(64)
            for (w in 0 until 16) words[w] = beWord(padded, at + 4 * w)
            expand(words, 16)
            val working = rounds(state, words, 0)
            state = UIntArray(8) { state[it] + working[it] }
            at += 64
        }
        val out = ByteArray(32)
        for (i in 0 until 8) for (j in 0 until 4) out[i * 4 + j] = (state[i] shr (24 - 8 * j)).toByte()
        return out
    }

    private fun wordsLe4Even(data: ByteArray): List<UInt> {
        val padded = data.copyOf(((data.size + 3) / 4) * 4)
        val words = ArrayList<UInt>()
        for (i in 0 until padded.size / 4) words.add(leWord(padded, i * 4))
        if (words.size % 2 != 0) words.add(0u)
        return words
    }

    private fun cs64(data: ByteArray, digest: ByteArray): Pair<UInt, UInt> {
        val keyA = leWord(digest, 0) or 1u
        val keyB = leWord(digest, 4) or 1u
        val factors = uintArrayOf(keyA * Cs64Factor, keyB * Cs64Factor)
        val terms = ArrayList<UInt>()
        val offsets = ArrayList<UInt>()
        Cs64Coefficients.forEachIndexed { i, (c, o) -> terms.add(c * factors[i % 2]); offsets.add(o) }
        val zeroPlusData = ByteArray(data.size + 1)
        data.copyInto(zeroPlusData, 1)
        wordsLe4Even(zeroPlusData).forEachIndexed { i, word ->
            var term = word * (if (i % 2 == 0) keyA else keyB)
            var offset = word.rotateLeft(16) * (if (i % 2 == 0) 0xEF0569FBu else 0xC31713DBu)
            if (i == 0) {
                term += Cs64Adjustment0 * factors[0]
                offset += Cs64Adjustment1
            }
            terms.add(term)
            offsets.add(offset)
        }
        var swapAcc = 0u
        var swapLast = 0u
        var state = 0u
        var acc = 0u
        var i = 0
        while (i < terms.size) {
            val x = terms[i]
            val y = terms[i + 1]
            var t = (offsets[i] + x).rotateLeft(16)
            var q = t * 0x76F50F34u
            swapAcc = swapAcc + q.rotateLeft(16) * 0xC3EFEA97u + q
            t = (offsets[i + 1] + y).rotateLeft(16)
            q = t * 0x1881CE3Cu
            swapLast = q.rotateLeft(16) * 0x35BD1EC9u + q
            swapAcc += swapLast
            state = state * keyA + x
            state = state.rotateLeft(16) * 0xCF98B111u
            state = state.rotateLeft(16) * 0x87085B9Fu
            state = state.rotateLeft(16) * 0x12CEB96Du
            state = state.rotateLeft(16) * 0x257E1D83u
            acc += state
            var u = state * keyB + y
            u = u.rotateLeft(16) * 0xA27416F5u
            u = u.rotateLeft(16) * 0xD38396FFu
            state = (u.rotateLeft(16) * 0x7C932B89u).rotateLeft(16) * 0xBFA49F69u
            acc += state
            i += 2
        }
        return (swapLast xor state) to (swapAcc xor acc)
    }

    private fun hex(bytes: ByteArray) = bytes.joinToString("") { (it.toInt() and 0xFF).toString(16).padStart(2, '0').uppercase() }

    private fun leBytes(word: UInt) = ByteArray(4) { (word shr (8 * it)).toByte() }

    fun secMsGec(unix: Long): String {
        val windows = unix + WinEpoch
        val ticks = (windows - windows % 300) * 10_000_000L
        val text = "$ticks$TrustedClientToken".encodeToByteArray()
        val sha = digest(text)
        val (w1, w2) = cs64(text, sha)
        return hex(leBytes(w1) + leBytes(w2)) + "-" + hex(sha)
    }
}

/** Edge 朗读的连接、配置、SSML 和响应帧；和电脑端 `tts/protocol.rs` 一一对应。 */
internal object EdgeProtocol {
    const val Host = "speech.platform.bing.com"
    const val Base = "/consumer/speech/synthesize/readaloud"
    const val Token = "6A5AA1D4EAFF4E9FB37E23D68491D6F4"
    const val GecVersion = "2-143.0.3650.75"
    const val Origin = "chrome-extension://jdiccldimpdaibmpdkjnbmckianbfold"
    const val UserAgent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36 Edg/143.0.0.0"
    const val MaxChars = 10_000
    const val MaxAudio = 16 * 1024 * 1024
    const val DefaultVoice = "zh-CN-XiaoxiaoNeural"

    fun randomId(): String = Random.nextBytes(16).joinToString("") { (it.toInt() and 0xFF).toString(16).padStart(2, '0').uppercase() }

    fun wsUrl(id: String, unix: Long) =
        "wss://$Host$Base/edge/v1?TrustedClientToken=$Token&ConnectionId=$id&Sec-MS-GEC=${EdgeDrm.secMsGec(unix)}&Sec-MS-GEC-Version=$GecVersion"

    private val Days = listOf("Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed")
    private val Months = listOf("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")

    /** `Tue Nov 14 2023 22:13:20 GMT+0000 (Coordinated Universal Time)`。 */
    fun timestamp(unix: Long): String {
        val days = unix.floorDiv(86_400L)
        val secs = unix.mod(86_400L).toInt()
        // 从 1970-01-01 的天数换算年月日（公历）。
        val z = days + 719_468
        val era = z.floorDiv(146_097L)
        val doe = (z - era * 146_097).toInt()
        val yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365
        val doy = doe - (365 * yoe + yoe / 4 - yoe / 100)
        val mp = (5 * doy + 2) / 153
        val day = doy - (153 * mp + 2) / 5 + 1
        val month = if (mp < 10) mp + 3 else mp - 9
        val year = (yoe + era * 400 + if (month <= 2) 1 else 0).toInt()
        val weekday = Days[days.mod(7L).toInt()]
        fun two(n: Int) = n.toString().padStart(2, '0')
        return "$weekday ${Months[month - 1]} ${two(day)} $year ${two(secs / 3600)}:${two(secs % 3600 / 60)}:${two(secs % 60)} GMT+0000 (Coordinated Universal Time)"
    }

    fun xml(text: String) = text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;").replace("\"", "&quot;").replace("'", "&apos;")

    fun voiceName(short: String): String {
        val parts = short.split('-')
        if (parts.size < 3 || !short.endsWith("Neural")) return short
        return "Microsoft Server Speech Text to Speech Voice (${parts.dropLast(1).joinToString("-")}, ${parts.last()})"
    }

    fun config(unix: Long) =
        "X-Timestamp:${timestamp(unix)}\r\nContent-Type:application/json; charset=utf-8\r\nPath:speech.config\r\n\r\n" +
            """{"context":{"synthesis":{"audio":{"metadataoptions":{"sentenceBoundaryEnabled":"false","wordBoundaryEnabled":"false"},"outputFormat":"audio-24khz-48kbitrate-mono-mp3"}}}}"""

    private fun signed(n: Int) = if (n >= 0) "+$n" else "$n"

    fun ssml(id: String, voice: String, rate: Int, pitch: Int, volume: Int, text: String, unix: Long) =
        "X-RequestId:$id\r\nContent-Type:application/ssml+xml\r\nX-Timestamp:${timestamp(unix)}Z\r\nPath:ssml\r\n\r\n" +
            "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='en-US'><voice name='${xml(voiceName(voice))}'>" +
            "<prosody pitch='${signed(pitch)}Hz' rate='${signed(rate)}%' volume='${signed(volume - 100)}%'>${xml(text)}</prosody></voice></speak>"

    /** 一次最多送 4096 个 UTF-8 字节，尽量在句末或空白处断开。 */
    fun chunks(text: String): List<String> {
        if (text.length > MaxChars) throw EngineUnavailable("回复较长，请分段朗读")
        var rest = text.filter { !it.isISOControl() || it == '\n' || it == '\r' || it == '\t' }.trim()
        val result = ArrayList<String>()
        while (rest.isNotEmpty()) {
            var end = 0
            var bytes = 0
            while (end < rest.length) {
                val step = if (rest[end].isHighSurrogate() && end + 1 < rest.length) 2 else 1
                val size = rest.substring(end, end + step).encodeToByteArray().size
                if (bytes + size > 4096) break
                bytes += size
                end += step
            }
            if (end < rest.length) {
                val cut = (end - 1 downTo end / 2).firstOrNull { rest[it].isWhitespace() || rest[it] in "。！？.!?" }
                if (cut != null) end = cut + 1
            }
            result += rest.substring(0, end).trim()
            rest = rest.substring(end).trimStart()
        }
        if (result.isEmpty()) throw EngineUnavailable("没有可以朗读的文本")
        return result
    }

    private fun headers(raw: String): Map<String, String> =
        raw.lines().mapNotNull { line -> line.indexOf(':').takeIf { it > 0 }?.let { line.substring(0, it).trim().lowercase() to line.substring(it + 1).trim() } }.toMap()

    fun textPath(frame: String): String {
        val head = frame.substringBefore("\r\n\r\n", missingDelimiterValue = "")
        if (head.isEmpty()) throw EngineUnavailable("语音响应缺少帧头")
        return headers(head)["path"] ?: throw EngineUnavailable("语音响应缺少 Path")
    }

    /** 二进制帧：前两个字节是头长度，后面是头，再后面是音频。 */
    fun audioFrame(frame: ByteArray): ByteArray {
        if (frame.size < 2) throw EngineUnavailable("语音响应帧不完整")
        val len = ((frame[0].toInt() and 0xFF) shl 8) or (frame[1].toInt() and 0xFF)
        if (len == 0 && frame.size == 2) return ByteArray(0)
        if (len < 4 || len + 2 > frame.size || frame[len] != '\r'.code.toByte() || frame[len + 1] != '\n'.code.toByte()) throw EngineUnavailable("语音响应帧长度无效")
        val head = frame.decodeToString(2, len)
        if (headers(head)["path"] != "audio") throw EngineUnavailable("收到非音频响应帧")
        return frame.copyOfRange(len + 2, frame.size)
    }
}

/** 手机直接连 Edge 朗读合成语音（和电脑上同一个服务、同一个默认音色）。网络不通、服务拒绝连接都当作“用不了”，通话会换下一个引擎。 */
@OptIn(ExperimentalTime::class)
class EdgeTtsSynthesizer(
    private val client: HttpClient,
    private val voice: String = EdgeProtocol.DefaultVoice,
    private val rate: Int = 0,
    private val now: () -> Long = { Clock.System.now().epochSeconds },
) : SynthesizerEngine {
    override val name = "Edge 语音合成"

    override suspend fun synthesize(text: String): ByteArray {
        val output = ArrayList<ByteArray>()
        var total = 0
        for (part in EdgeProtocol.chunks(text)) {
            val audio = one(part)
            total += audio.size
            if (total > EdgeProtocol.MaxAudio) throw EngineUnavailable("合成音频过长")
            output += audio
        }
        val all = ByteArray(total)
        var at = 0
        for (a in output) { a.copyInto(all, at); at += a.size }
        return all
    }

    private suspend fun one(text: String): ByteArray {
        val session = try {
            client.webSocketSession(EdgeProtocol.wsUrl(EdgeProtocol.randomId(), now())) {
                header("User-Agent", EdgeProtocol.UserAgent)
                header("Origin", EdgeProtocol.Origin)
                header("Accept", "*/*")
                header("Accept-Language", "en-US,en;q=0.9")
                header("Pragma", "no-cache")
                header("Cache-Control", "no-cache")
                header("Cookie", "muid=${EdgeProtocol.randomId()};")
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            throw EngineUnavailable("无法连接语音服务，请检查网络")
        }
        try {
            return withTimeout(60_000) {
                session.send(Frame.Text(EdgeProtocol.config(now())))
                session.send(Frame.Text(EdgeProtocol.ssml(EdgeProtocol.randomId(), voice, rate, 0, 100, text, now())))
                val audio = ArrayList<ByteArray>()
                var size = 0
                var ended = false
                for (frame in session.incoming) {
                    when (frame) {
                        is Frame.Text -> when (EdgeProtocol.textPath(frame.readText())) {
                            "turn.end" -> { ended = true }
                            "turn.start", "response", "audio.metadata" -> {}
                            else -> throw EngineUnavailable("语音服务响应协议已变化")
                        }
                        is Frame.Binary -> {
                            val bytes = EdgeProtocol.audioFrame(frame.readBytes())
                            size += bytes.size
                            if (size > EdgeProtocol.MaxAudio) throw EngineUnavailable("合成音频过长")
                            audio += bytes
                        }
                        is Frame.Close -> break
                        else -> {}
                    }
                    if (ended) break
                }
                if (!ended || size == 0) throw EngineUnavailable("语音服务未返回完整音频")
                val all = ByteArray(size)
                var at = 0
                for (a in audio) { a.copyInto(all, at); at += a.size }
                all
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: EngineUnavailable) {
            throw e
        } catch (e: Exception) {
            throw EngineUnavailable("语音合成连接中断")
        } finally {
            runCatching { session.close() }
        }
    }
}
