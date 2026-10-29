package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.request.get
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

/** 模型文件放在手机的什么地方（各平台自己实现）。写入先写到临时文件，校验通过才换成正式文件。 */
interface ModelFiles {
    /** 文件大小；不存在返回 -1。 */
    fun size(name: String): Long
    fun path(name: String): String
    fun beginWrite(name: String): ModelWriter
    fun delete(name: String)
}

interface ModelWriter {
    fun write(bytes: ByteArray, length: Int)
    /** 写完了，把临时文件变成正式文件。 */
    fun commit()
    /** 放弃，删掉临时文件。 */
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
}

/** 手机上本地语音需要的全部模型：SenseVoice（识别）加 Silero VAD（判断有没有人在说话），和电脑端 `voice_turn` 用的是同一个 Silero 文件。 */
object VoiceModels {
    const val SileroName = "silero_vad.onnx"
    val Silero = ModelSpec(SileroName, 2_327_524, "1a153a22f4509e292a94e67d6f9b85e8deb25b4988682b7e174c65279d8788e3")
    val Files: List<ModelSpec> = SenseVoiceModel.Files + Silero
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
            fetch(spec) { received -> onProgress(base + received, total) }
            done += spec.bytes
            onProgress(done, total)
        }
    }

    private suspend fun fetch(spec: ModelSpec, progress: (Long) -> Unit) {
        val writer = files.beginWrite(spec.name)
        try {
            val hash = Sha256()
            var received = 0L
            client.prepareGet(source + spec.name) { header(HttpHeaders.UserAgent, "XiaoBan/1.0") }.execute { response ->
                if (!response.status.isSuccess()) throw ModelDownloadException("下载 ${spec.name} 失败（${response.status.value}）")
                val channel = response.bodyAsChannel()
                val buffer = ByteArray(64 * 1024)
                while (true) {
                    val n = channel.readAvailable(buffer, 0, buffer.size)
                    if (n < 0) break
                    if (n == 0) continue
                    received += n
                    if (received > spec.bytes) throw ModelDownloadException("${spec.name} 比预期的大，已放弃")
                    hash.update(buffer, n)
                    writer.write(buffer, n)
                    progress(received)
                }
            }
            if (received != spec.bytes) throw ModelDownloadException("${spec.name} 没有下载完整（${received}/${spec.bytes}）")
            if (hash.hex() != spec.sha256) throw ModelDownloadException("${spec.name} 校验没通过，请重试")
            writer.commit()
        } catch (e: CancellationException) {
            writer.abort()
            throw e
        } catch (e: ModelDownloadException) {
            writer.abort()
            throw e
        } catch (e: Exception) {
            writer.abort()
            throw ModelDownloadException("下载 ${spec.name} 失败，请检查网络后重试")
        }
    }
}

/** 标准 SHA-256，边下载边算。 */
class Sha256 {
    private val state = intArrayOf(0x6a09e667, 0xbb67ae85.toInt(), 0x3c6ef372, 0xa54ff53a.toInt(), 0x510e527f, 0x9b05688c.toInt(), 0x1f83d9ab, 0x5be0cd19)
    private val block = ByteArray(64)
    private var filled = 0
    private var length = 0L

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
