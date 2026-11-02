package xyz.xiangban.xiaoban

import android.content.Context
import kotlinx.serialization.json.Json
import xyz.xiangban.xiaoban.common.community.DownloadedSpeechRuntime
import xyz.xiangban.xiaoban.common.community.EngineUnavailable
import xyz.xiangban.xiaoban.common.community.ModelFiles
import xyz.xiangban.xiaoban.common.community.ModelSpec
import xyz.xiangban.xiaoban.common.community.SpeechRuntimeManifest

/** Sherpa 及 ORT 的唯一加载入口；启动只读 APK 内清单，不联网、不初始化语音引擎。 */
object SpeechNativeLoader {
    private var runtime: DownloadedSpeechRuntime? = null

    @Synchronized
    fun configure(context: Context, files: ModelFiles): List<ModelSpec> {
        if (runtime == null) {
            val manifest = context.assets.open("voice-runtime.json").bufferedReader().use {
                Json.decodeFromString<SpeechRuntimeManifest>(it.readText())
            }
            runtime = DownloadedSpeechRuntime(manifest, files)
        }
        return runtime!!.specs
    }

    /** ORT Android 原本硬编码 System.loadLibrary；构建时仅将其加载调用重定向到这里。 */
    @JvmStatic
    @Synchronized
    fun load(name: String) {
        val installed = runtime ?: throw EngineUnavailable("本地语音组件未初始化")
        try {
            installed.load(name)
        } catch (e: LinkageError) {
            throw EngineUnavailable("本地语音组件加载失败，请重试", e)
        }
    }
}
