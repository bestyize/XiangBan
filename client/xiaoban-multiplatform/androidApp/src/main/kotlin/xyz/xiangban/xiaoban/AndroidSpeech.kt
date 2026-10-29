package xyz.xiangban.xiaoban

import android.content.Context
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

/** 语音模型放在应用私有目录 `files/speech/`；下载先写 `.part` 临时文件，校验通过才改名。 */
class AndroidModelFiles(context: Context) : ModelFiles {
    private val dir = File(context.filesDir, "speech").also { it.mkdirs() }

    override fun size(name: String): Long = File(dir, name).takeIf { it.isFile }?.length() ?: -1
    override fun path(name: String): String = File(dir, name).absolutePath
    override fun delete(name: String) { File(dir, name).delete() }

    override fun beginWrite(name: String): ModelWriter {
        val part = File(dir, "$name.part")
        val out = FileOutputStream(part)
        return object : ModelWriter {
            override fun write(bytes: ByteArray, length: Int) = out.write(bytes, 0, length)
            override fun commit() {
                out.close()
                val target = File(dir, name)
                target.delete()
                if (!part.renameTo(target)) throw java.io.IOException("无法保存 $name")
            }
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
    private var recognizer: OfflineRecognizer? = null
    private val lock = Any()

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
    }
}

class AndroidLocalSpeech(context: Context) : LocalSpeech {
    override val files: ModelFiles = AndroidModelFiles(context)
    override val recognizer: RecognizerEngine = SherpaRecognizer(files)
}
