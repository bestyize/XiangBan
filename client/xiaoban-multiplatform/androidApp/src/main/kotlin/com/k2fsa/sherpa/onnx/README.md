# sherpa-onnx 的 Kotlin 接口

这里的 `.kt` 文件来自 [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) v1.13.8 的 `sherpa-onnx/kotlin-api/`，Apache-2.0 许可。
它们通过 JNI 调用 `libsherpa-onnx-jni.so`，包名、类名、字段名必须和同版本的原生库一致。

原生库不进仓库和 APK：`androidApp/scripts/prepare-voice-runtime.py` 从
`https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.8/sherpa-onnx-v1.13.8-android.tar.bz2`
下载并校验，将 Sherpa 的 ORT 重命名为 `libort_sherpa.so` 以隔离版本；生成随 APK 打包的版本与哈希清单，原生库作为 Release 独立文件发布，并可镜像到桌面同一魔搭仓库的 `android-voice/` 下。首次使用由 `SpeechNativeLoader` 校验后按依赖顺序加载。

升级时，这里的源文件和脚本里的版本、哈希一起改，并和电脑端 `tools/xiaoban-infer` 用的 sherpa-onnx 版本保持一致。
