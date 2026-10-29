package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeoutConfig.Companion.INFINITE_TIMEOUT_MS
import io.ktor.client.plugins.timeout
import io.ktor.client.request.header
import io.ktor.client.request.prepareGet
import io.ktor.client.statement.bodyAsChannel
import io.ktor.http.HttpHeaders
import io.ktor.http.isSuccess
import io.ktor.utils.io.readAvailable
import kotlinx.coroutines.CancellationException

// 手机上本地语音识别要用的模型文件：和电脑端同一个 SenseVoice Small（INT8），同一个下载源，同样按大小和 SHA-256 校验。
// 文件清单要和电脑端 `src-tauri/src/asr/engine.rs` 保持一致。

/** 一个固定了大小和哈希的模型文件。 */
class ModelSpec(val name: String, val bytes: Long, val sha256: String)

/** 模型文件放在手机的什么地方（各平台自己实现）。写入先写到临时文件，校验通过才换成正式文件；下载中断后临时文件留着，下次接着下。 */
interface ModelFiles {
    /** 文件大小；不存在返回 -1。 */
    fun size(name: String): Long
    fun path(name: String): String
    /** 上次没下完留下的临时文件的大小；没有返回 0。 */
    fun partialSize(name: String): Long
    /** 把留下的临时文件从头读一遍（续传前要把它重新算进校验）。 */
    fun readPartial(name: String, onChunk: (ByteArray, Int) -> Unit)
    /** 开始写。[append] 为真就接着临时文件往后写，否则从头写。 */
    fun beginWrite(name: String, append: Boolean): ModelWriter
    fun delete(name: String)
    /** 丢掉没下完的临时文件。 */
    fun discardPartial(name: String)
}

interface ModelWriter {
    fun write(bytes: ByteArray, length: Int)
    /** 写完了且校验通过，把临时文件变成正式文件。 */
    fun commit()
    /** 中断了（断网、取消）：关上文件，临时文件留着以便续传。 */
    fun close()
    /** 内容不对：删掉临时文件。 */
    fun abort()
}

/** 模型下载失败（网络、校验不过）。 */
class ModelDownloadException(message: String) : Exception(message)

object SenseVoiceModel {
    /** 魔搭上的模型仓库，电脑端也从这里下载。 */
    const val Source = "https://modelscope.cn/api/v1/models/bestyize/xiaoban-asr/repo?Revision=master&FilePath="
    const val ModelName = "model.int8.onnx"
    const val TokensName = "tokens.txt"
    val Files = listOf(
        ModelSpec(ModelName, 239_233_841, "c71f0ce00bec95b07744e116345e33d8cbbe08cef896382cf907bf4b51a2cd51"),
        ModelSpec(TokensName, 315_894, "f449eb28dc567533d7fa59be34e2abca8784f771850c78a47fb731a31429a1dc"),
    )
    val TotalBytes: Long = Files.sumOf { it.bytes }

    /** SenseVoice 输出里带 `<|zh|><|NEUTRAL|>` 这样的标签，去掉只留文字。 */
    fun cleanText(text: String): String {
        val out = StringBuilder()
        var rest = text
        while (true) {
            val start = rest.indexOf("<|")
            if (start < 0) break
            out.append(rest, 0, start)
            val end = rest.indexOf("|>", start + 2)
            if (end < 0) { rest = rest.substring(start); break }
            rest = rest.substring(end + 2)
        }
        out.append(rest)
        return out.toString().trim()
    }
}

/** 平台提供的本地语音识别：模型文件放哪里，以及用它们识别的引擎。 */
interface LocalSpeech {
    val files: ModelFiles
    val recognizer: RecognizerEngine
    /** 未准备好时返回 null，通话改用原有的判停方式。 */
    suspend fun turnComplete(pcm: ByteArray): Boolean? = null
}

/** 手机上本地语音需要的全部模型：SenseVoice（识别）加 Silero VAD（判断有没有人在说话），和电脑端 `voice_turn` 用的是同一个 Silero 文件。 */
object VoiceModels {
    const val SileroName = "silero_vad.onnx"
    val Silero = ModelSpec(SileroName, 2_327_524, "1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3")
    const val SmartTurnName = "smart-turn-v3.2.onnx"
    val SmartTurn = ModelSpec(SmartTurnName, 8_679_182, "2bb026316b14a660486a75b1733cd3fbab8c2fd0314dc9af7be49f8cca967e4f")
    val Files: List<ModelSpec> = SenseVoiceModel.Files + Silero + SmartTurn
    val TotalBytes: Long = Files.sumOf { it.bytes }
}

/** 管理本地语音识别模型：是否齐了、下载并校验。 */
class SpeechModels(
    private val client: HttpClient,
    val files: ModelFiles,
    private val source: String = SenseVoiceModel.Source,
    private val specs: List<ModelSpec> = VoiceModels.Files,
) {
    val ready: Boolean get() = specs.all { files.size(it.name) == it.bytes }

    /** 下载缺的文件。[onProgress] 给（已下载字节, 总字节），按所有文件合计。失败时留下已完成的文件，下次接着下缺的。 */
    suspend fun download(onProgress: (Long, Long) -> Unit) {
        val total = specs.sumOf { it.bytes }
        var done = specs.filter { files.size(it.name) == it.bytes }.sumOf { it.bytes }
        onProgress(done, total)
        for (spec in specs) {
            if (files.size(spec.name) == spec.bytes) continue
            val base = done
            onProgress(base + files.partialSize(spec.name).coerceAtMost(spec.bytes), total)
            fetch(spec) { received -> onProgress(base + received, total) }
            done += spec.bytes
            onProgress(done, total)
        }
    }

    /**
     * 下载一个文件，支持断点续传：上次留下的临时文件先重新算进校验，再用 Range 请求剩下的部分；
     * 服务器不支持 Range（回 200）就从头来。断网或取消时临时文件留着，内容不对（太大、校验不过）才删掉。
     */
    private suspend fun fetch(spec: ModelSpec, progress: (Long) -> Unit) {
        var offset = files.partialSize(spec.name)
        if (offset > spec.bytes) { files.discardPartial(spec.name); offset = 0 }
        val hash = Sha256()
        if (offset > 0) files.readPartial(spec.name) { chunk, n -> hash.update(chunk, n) }
        var received = offset
        var writer: ModelWriter? = null
        var discard = false
        try {
            if (offset < spec.bytes) {
                client.prepareGet(source + spec.name) {
                    header(HttpHeaders.UserAgent, "XiaoBan/1.0")
                    // 模型有两百多兆，共用客户端的“整个请求 30 秒”会把下载在半路掐断：这里不限总时长，只要 60 秒内还有数据到就行。
                    timeout {
                        requestTimeoutMillis = INFINITE_TIMEOUT_MS
                        socketTimeoutMillis = 60_000
                        connectTimeoutMillis = 15_000
                    }
                    if (offset > 0) header(HttpHeaders.Range, "bytes=$offset-")
                }.execute { response ->
                    val code = response.status.value
                    when {
                        code == 206 -> {}
                        code == 416 -> { files.discardPartial(spec.name); throw ModelDownloadException("续传位置不对，已清掉重来，请再试一次") }
                        !response.status.isSuccess() -> throw ModelDownloadException("下载 ${spec.name} 失败（$code）")
                        else -> if (offset > 0) {
                            // 服务器没理会 Range，整个文件重新发来：丢掉旧的，从头算。
                            offset = 0; received = 0; hash.reset()
                        }
                    }
                    writer = files.beginWrite(spec.name, append = offset > 0)
                    val out = writer!!
                    val channel = response.bodyAsChannel()
                    val buffer = ByteArray(64 * 1024)
                    while (true) {
                        val n = channel.readAvailable(buffer, 0, buffer.size)
                        if (n < 0) break
                        if (n == 0) continue
                        received += n
                        if (received > spec.bytes) { discard = true; throw ModelDownloadException("${spec.name} 比预期的大，已放弃") }
                        hash.update(buffer, n)
                        out.write(buffer, n)
                        progress(received)
                    }
                }
            } else {
                writer = files.beginWrite(spec.name, append = true)
            }
            if (received != spec.bytes) {
                writer?.close()
                throw ModelDownloadException("${spec.name} 还没下载完（${received}/${spec.bytes}），再点一次会接着下")
            }
            if (hash.hex() != spec.sha256) {
                discard = true
                throw ModelDownloadException("${spec.name} 校验没通过，已清掉，请重试")
            }
            writer!!.commit()
        } catch (e: CancellationException) {
            writer?.close()
            throw e
        } catch (e: ModelDownloadException) {
            if (discard) writer?.abort() else writer?.close()
            throw e
        } catch (e: Exception) {
            writer?.close()
            throw ModelDownloadException("下载 ${spec.name} 中断了（已下 ${received / 1_000_000} MB，再点一次会接着下），请检查网络")
        }
    }
}

/** 标准 SHA-256，边下载边算。 */
class Sha256 {
    private val state = intArrayOf(0x6a09e667, 0xbb67ae85.toInt(), 0x3c6ef372, 0xa54ff53a.toInt(), 0x510e527f, 0x9b05688c.toInt(), 0x1f83d9ab, 0x5be0cd19)
    private val block = ByteArray(64)
    private var filled = 0
    private var length = 0L

    fun reset() {
        val fresh = Sha256()
        fresh.state.copyInto(state)
        filled = 0
        length = 0
    }

    fun update(bytes: ByteArray, count: Int = bytes.size) {
        for (i in 0 until count) {
            block[filled++] = bytes[i]
            if (filled == 64) { compress(); filled = 0 }
        }
        length += count
    }

    fun hex(): String {
        val bits = length * 8
        update(byteArrayOf(0x80.toByte()))
        while (filled != 56) update(byteArrayOf(0))
        val tail = ByteArray(8) { (bits ushr (56 - 8 * it)).toByte() }
        update(tail)
        return state.joinToString("") { it.toUInt().toString(16).padStart(8, '0') }
    }

    private fun compress() {
        val w = IntArray(64)
        for (i in 0 until 16) w[i] = ((block[4 * i].toInt() and 0xFF) shl 24) or ((block[4 * i + 1].toInt() and 0xFF) shl 16) or ((block[4 * i + 2].toInt() and 0xFF) shl 8) or (block[4 * i + 3].toInt() and 0xFF)
        for (i in 16 until 64) {
            val s0 = w[i - 15].rotateRight(7) xor w[i - 15].rotateRight(18) xor (w[i - 15] ushr 3)
            val s1 = w[i - 2].rotateRight(17) xor w[i - 2].rotateRight(19) xor (w[i - 2] ushr 10)
            w[i] = w[i - 16] + s0 + w[i - 7] + s1
        }
        var a = state[0]; var b = state[1]; var c = state[2]; var d = state[3]
        var e = state[4]; var f = state[5]; var g = state[6]; var h = state[7]
        for (i in 0 until 64) {
            val s1 = e.rotateRight(6) xor e.rotateRight(11) xor e.rotateRight(25)
            val ch = (e and f) xor (e.inv() and g)
            val t1 = h + s1 + ch + K[i] + w[i]
            val s0 = a.rotateRight(2) xor a.rotateRight(13) xor a.rotateRight(22)
            val maj = (a and b) xor (a and c) xor (b and c)
            val t2 = s0 + maj
            h = g; g = f; f = e; e = d + t1; d = c; c = b; b = a; a = t1 + t2
        }
        state[0] += a; state[1] += b; state[2] += c; state[3] += d; state[4] += e; state[5] += f; state[6] += g; state[7] += h
    }

    private companion object {
        val K = intArrayOf(
            0x428a2f98, 0x71374491, 0xb5c0fbcf.toInt(), 0xe9b5dba5.toInt(), 0x3956c25b, 0x59f111f1, 0x923f82a4.toInt(), 0xab1c5ed5.toInt(),
            0xd807aa98.toInt(), 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe.toInt(), 0x9bdc06a7.toInt(), 0xc19bf174.toInt(),
            0xe49b69c1.toInt(), 0xefbe4786.toInt(), 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
            0x983e5152.toInt(), 0xa831c66d.toInt(), 0xb00327c8.toInt(), 0xbf597fc7.toInt(), 0xc6e00bf3.toInt(), 0xd5a79147.toInt(), 0x06ca6351, 0x14292967,
            0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e.toInt(), 0x92722c85.toInt(),
            0xa2bfe8a1.toInt(), 0xa81a664b.toInt(), 0xc24b8b70.toInt(), 0xc76c51a3.toInt(), 0xd192e819.toInt(), 0xd6990624.toInt(), 0xf40e3585.toInt(), 0x106aa070,
            0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814.toInt(), 0x8cc70208.toInt(), 0x90befffa.toInt(), 0xa4506ceb.toInt(), 0xbef9a3f7.toInt(), 0xc67178f2.toInt(),
        )
    }
}
