package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.Serializable

/** 此清单随 APK 打包，不从服务器读取可变的运行库版本或哈希。 */
@Serializable
data class SpeechRuntimeManifest(
    val format: Int,
    val abi: String,
    val version: String,
    val libraries: List<SpeechRuntimeLibrary>,
) {
    fun validate() {
        require(format == 1 && abi == "arm64-v8a") { "不支持的语音组件格式" }
        require(version.matches(Regex("[a-f0-9]{64}"))) { "无效的语音组件版本" }
        require(libraries.map { it.name }.toSet() == Names && libraries.size == Names.size) { "语音组件不完整" }
        libraries.forEach {
            require(it.bytes in 1..100_000_000 && it.sha256.matches(Regex("[a-f0-9]{64}"))) { "无效的语音组件校验信息" }
            require(it.sources.isNotEmpty() && it.sources.all { url -> url.startsWith("https://") }) { "无效的语音组件地址" }
        }
    }

    fun specs(): List<ModelSpec> {
        validate()
        return libraries.map { ModelSpec("voice-runtime/$version/${it.name}", it.bytes, it.sha256, it.sources) }
    }

    companion object {
        val Names = setOf("libort_sherpa.so", "libsherpa-onnx-jni.so", "libonnxruntime.so", "libonnxruntime4j_jni.so")
    }
}

@Serializable
data class SpeechRuntimeLibrary(val name: String, val bytes: Long, val sha256: String, val sources: List<String>)
