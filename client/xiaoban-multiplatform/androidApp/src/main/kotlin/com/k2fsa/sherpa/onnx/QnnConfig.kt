// 来自 k2-fsa/sherpa-onnx v1.13.8 的 sherpa-onnx/kotlin-api/（Apache-2.0）。类和字段名要和同版本的 libsherpa-onnx-jni.so 对得上，升级时两边一起换。
package com.k2fsa.sherpa.onnx

data class QnnConfig(
    var backendLib: String = "",
    var contextBinary: String = "",
    var systemLib: String = "",
)
