package xyz.xiangban.xiaoban.common.community

import java.io.File
import java.security.MessageDigest

/** 私有目录中已下载的原生组件。依赖先加载，每个库在首次加载前校验；未就绪时不碰 JNI 类的静态初始化。 */
class DownloadedSpeechRuntime(
    manifest: SpeechRuntimeManifest,
    private val files: ModelFiles,
    private val loadLibrary: (String) -> Unit = System::load,
) {
    val specs = manifest.specs()
    private val byName = specs.associateBy { it.name.substringAfterLast('/') }
    private val loaded = mutableSetOf<String>()

    @Synchronized
    fun load(name: String) {
        val library = "lib$name.so"
        require(library in byName) { "未知语音运行库：$name" }
        if (library in loaded) return
        when (name) {
            "sherpa-onnx-jni" -> load("ort_sherpa")
            "onnxruntime4j_jni" -> load("onnxruntime")
        }
        val spec = byName.getValue(library)
        val file = File(files.path(spec.name))
        if (!file.isFile || file.length() != spec.bytes) throw EngineUnavailable("本地语音组件还没有下载完成")
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buffer = ByteArray(64 * 1024)
            while (true) {
                val count = input.read(buffer)
                if (count < 0) break
                digest.update(buffer, 0, count)
            }
        }
        val actual = digest.digest().joinToString("") { "%02x".format(it.toInt() and 255) }
        if (actual != spec.sha256) {
            files.delete(spec.name)
            throw EngineUnavailable("本地语音组件校验失败，请重新下载")
        }
        check(file.setReadOnly()) { "无法保护本地语音组件" }
        loadLibrary(file.absolutePath)
        loaded += library
    }
}
