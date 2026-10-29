# sherpa-onnx 的 Kotlin 接口

这里的 `.kt` 文件来自 [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) v1.13.8 的 `sherpa-onnx/kotlin-api/`，Apache-2.0 许可。
它们通过 JNI 调用 `libsherpa-onnx-jni.so`，包名、类名、字段名必须和同版本的原生库一致。

原生库（`libsherpa-onnx-jni.so`、`libonnxruntime.so`）不进仓库：构建安卓包的工作流会从
`https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.8/sherpa-onnx-v1.13.8-android.tar.bz2`
下载并校验后放进 `androidApp/src/main/jniLibs/`。升级版本时，这里的源文件和工作流里的版本、哈希一起改，并和电脑端 `tools/xiaoban-infer` 用的 sherpa-onnx 版本保持一致。
