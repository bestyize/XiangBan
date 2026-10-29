package xyz.xiangban.xiaoban

import android.content.Context
import ai.onnxruntime.OnnxTensor
import ai.onnxruntime.OrtEnvironment
import ai.onnxruntime.OrtSession
import java.nio.FloatBuffer
import com.k2fsa.sherpa.onnx.FeatureConfig
import com.k2fsa.sherpa.onnx.OfflineModelConfig
import com.k2fsa.sherpa.onnx.OfflineRecognizer
import com.k2fsa.sherpa.onnx.OfflineRecognizerConfig
import com.k2fsa.sherpa.onnx.OfflineSenseVoiceModelConfig
import java.io.File
import java.io.FileOutputStream
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.common.community.EngineUnavailable
import xyz.xiangban.xiaoban.common.community.LocalSpeech
import xyz.xiangban.xiaoban.common.community.ModelFiles
import xyz.xiangban.xiaoban.common.community.ModelWriter
import xyz.xiangban.xiaoban.common.community.RecognizerEngine
import xyz.xiangban.xiaoban.common.community.SenseVoiceModel
import xyz.xiangban.xiaoban.common.community.SpeechGate
import xyz.xiangban.xiaoban.common.community.SmartTurnFeatures
import xyz.xiangban.xiaoban.common.community.VoiceModels
import com.k2fsa.sherpa.onnx.SileroVadModelConfig
import com.k2fsa.sherpa.onnx.Vad
import com.k2fsa.sherpa.onnx.VadModelConfig

/** 语音模型放在应用私有目录 `files/speech/`；下载先写 `.part` 临时文件，校验通过才改名。 */
class AndroidModelFiles(context: Context) : ModelFiles {
    private val dir = File(context.filesDir, "speech").also { it.mkdirs() }

    override fun size(name: String): Long = File(dir, name).takeIf { it.isFile }?.length() ?: -1
    override fun path(name: String): String = File(dir, name).absolutePath
    override fun delete(name: String) { File(dir, name).delete() }
    override fun discardPartial(name: String) { File(dir, "$name.part").delete() }

    override fun partialSize(name: String): Long = File(dir, "$name.part").takeIf { it.isFile }?.length() ?: 0

    override fun readPartial(name: String, onChunk: (ByteArray, Int) -> Unit) {
        File(dir, "$name.part").inputStream().use { input ->
            val buffer = ByteArray(64 * 1024)
            while (true) {
                val n = input.read(buffer)
                if (n < 0) break
                onChunk(buffer, n)
            }
        }
    }

    override fun beginWrite(name: String, append: Boolean): ModelWriter {
        val part = File(dir, "$name.part")
        val out = FileOutputStream(part, append)
        return object : ModelWriter {
            override fun write(bytes: ByteArray, length: Int) = out.write(bytes, 0, length)
            override fun commit() {
                out.close()
                val target = File(dir, name)
                target.delete()
                if (!part.renameTo(target)) throw java.io.IOException("无法保存 $name")
            }
            override fun close() { runCatching { out.close() } }
            override fun abort() {
                runCatching { out.close() }
                part.delete()
            }
        }
    }
}

/**
 * 手机上本地的 SenseVoice 识别（sherpa-onnx），和电脑端同一个模型、同样的设置（自动判断语种、加标点、2 线程、CPU）。
 * 模型没下载、安装包里没带原生库、加载失败都算“用不了”，通话会换下一个引擎。
 */
class SherpaRecognizer(private val files: ModelFiles) : RecognizerEngine {
    override val name = "手机上的语音识别"
    override val local = true
    private var recognizer: OfflineRecognizer? = null
    private val lock = Any()
    private val decoding = kotlinx.coroutines.sync.Mutex()

    private fun load(): OfflineRecognizer = synchronized(lock) {
        recognizer?.let { return it }
        for (spec in SenseVoiceModel.Files) {
            if (files.size(spec.name) != spec.bytes) throw EngineUnavailable("语音识别模型还没有下载")
        }
        val config = OfflineRecognizerConfig(
            featConfig = FeatureConfig(sampleRate = 16000, featureDim = 80),
            modelConfig = OfflineModelConfig(
                senseVoice = OfflineSenseVoiceModelConfig(model = files.path(SenseVoiceModel.ModelName), language = "auto", useInverseTextNormalization = true),
                tokens = files.path(SenseVoiceModel.TokensName),
                numThreads = 2,
                provider = "cpu",
            ),
        )
        try {
            OfflineRecognizer(config = config).also { recognizer = it }
        } catch (e: UnsatisfiedLinkError) {
            throw EngineUnavailable("这个安装包没有带本地语音识别库")
        } catch (e: Throwable) {
            throw EngineUnavailable("语音识别模型加载失败，请重新下载")
        }
    }

    override suspend fun recognize(pcm: ByteArray): String = withContext(Dispatchers.Default) {
        decoding.lock()
        try {
        val engine = load()
        val samples = FloatArray(pcm.size / 2) { i -> (((pcm[2 * i + 1].toInt() shl 8) or (pcm[2 * i].toInt() and 0xFF)).toShort()) / 32768f }
        val stream = engine.createStream()
        try {
            stream.acceptWaveform(samples, 16000)
            engine.decode(stream)
            SenseVoiceModel.cleanText(engine.getResult(stream).text)
        } finally {
            stream.release()
        }
        } finally {
            decoding.unlock()
        }
    }
}

/**
 * Silero VAD（sherpa-onnx）：每 512 个采样算一次“这是人声”的概率，和电脑端 `voice/turn/segmenter.ts` 一样，
 * 概率 ≥ 0.5 算人声，小伴自己在说话时要 ≥ 0.75（它的回声不能被当成用户开口）。
 */
class SileroGate(modelPath: String) : SpeechGate {
    private val vad = Vad(
        config = VadModelConfig(
            sileroVadModelConfig = SileroVadModelConfig(model = modelPath, threshold = 0.5f, minSilenceDuration = 0.1f, minSpeechDuration = 0.05f, windowSize = Window, maxSpeechDuration = 30f),
            sampleRate = 16000, numThreads = 1, provider = "cpu",
        ),
    )
    // Silero V5 needs the last 64 samples of the previous window as context.
    private val window = FloatArray(Overlap + Window)
    private var filled = 0
    private var probability = 0f
    private var level = 0f
    private val fallback = xyz.xiangban.xiaoban.common.community.EnergyGate()
    private var nativeUnavailable = false

    override fun level(frame: ByteArray): Float = if (nativeUnavailable) fallback.level(frame) else level

    override fun voiced(frame: ByteArray, strict: Boolean): Boolean {
        if (nativeUnavailable) return fallback.voiced(frame, strict)
        var peak = 0f
        for (i in 0 until frame.size / 2) {
            val sample = (((frame[2 * i + 1].toInt() shl 8) or (frame[2 * i].toInt() and 0xFF)).toShort()) / 32768f
            peak = maxOf(peak, kotlin.math.abs(sample))
            window[Overlap + filled++] = sample
            if (filled == Window) {
                probability = try { vad.compute(window) } catch (_: LinkageError) {
                    nativeUnavailable = true
                    return fallback.voiced(frame, strict)
                }
                window.copyInto(window, 0, Window, Overlap + Window)
                filled = 0
            }
        }
        level = (peak * 3f).coerceIn(0f, 1f)
        return probability >= if (strict) 0.75f else 0.5f
    }

    override fun release() = vad.release()

    private companion object {
        const val Window = 512
        const val Overlap = 64
    }
}

class AndroidLocalSpeech(context: Context) : LocalSpeech {
    override val files: ModelFiles = AndroidModelFiles(context)
    override val recognizer: RecognizerEngine = SherpaRecognizer(files)
    private val turnLock = kotlinx.coroutines.sync.Mutex()
    private var turnSession: OrtSession? = null

    override suspend fun turnComplete(pcm: ByteArray): Boolean? = withContext(Dispatchers.Default) {
        if (files.size(VoiceModels.SmartTurnName) != VoiceModels.SmartTurn.bytes) return@withContext null
        turnLock.lock()
        try {
            val environment = OrtEnvironment.getEnvironment()
            val session = turnSession ?: OrtSession.SessionOptions().use { options ->
                options.setIntraOpNumThreads(2)
                environment.createSession(files.path(VoiceModels.SmartTurnName), options).also { turnSession = it }
            }
            val features = SmartTurnFeatures.fromPcm(pcm)
            OnnxTensor.createTensor(environment, FloatBuffer.wrap(features), longArrayOf(1, 80, 800)).use { input ->
                session.run(mapOf("input_features" to input)).use { result ->
                    ((result[0] as OnnxTensor).floatBuffer.get() >= 0.5f)
                }
            }
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (_: Exception) {
            null
        } catch (_: LinkageError) {
            null
        } finally {
            turnLock.unlock()
        }
    }

    /** 新的人声检测；模型还没下载或原生库不可用时返回 null，通话就改用声音大小判断。 */
    fun newGate(): SpeechGate? {
        val spec = VoiceModels.Silero
        if (files.size(spec.name) != spec.bytes) return null
        return try {
            SileroGate(files.path(spec.name))
        } catch (e: Throwable) {
            null
        }
    }
}
