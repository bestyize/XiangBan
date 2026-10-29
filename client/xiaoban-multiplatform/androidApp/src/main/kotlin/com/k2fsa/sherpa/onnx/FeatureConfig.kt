// 来自 k2-fsa/sherpa-onnx v1.13.8 的 sherpa-onnx/kotlin-api/（Apache-2.0）。类和字段名要和同版本的 libsherpa-onnx-jni.so 对得上，升级时两边一起换。
package com.k2fsa.sherpa.onnx

data class FeatureConfig(
    var sampleRate: Int = 16000,
    var featureDim: Int = 80,
    var dither: Float = 0.0f
)

fun getFeatureConfig(sampleRate: Int, featureDim: Int): FeatureConfig {
    return FeatureConfig(sampleRate = sampleRate, featureDim = featureDim)
}
