// 来自 k2-fsa/sherpa-onnx v1.13.8 的 sherpa-onnx/kotlin-api/（Apache-2.0）。类和字段名要和同版本的 libsherpa-onnx-jni.so 对得上，升级时两边一起换。
package com.k2fsa.sherpa.onnx

class OfflineStream(var ptr: Long) {
    init {
        require(ptr != 0L) { "Failed to create native OfflineStream" }
    }

    fun acceptWaveform(samples: FloatArray, sampleRate: Int) =
        acceptWaveform(ptr, samples, sampleRate)

    fun setOption(key: String, value: String) = setOption(ptr, key, value)

    fun getOption(key: String): String = getOption(ptr, key)

    protected fun finalize() {
        if (ptr != 0L) {
            delete(ptr)
            ptr = 0
        }
    }

    fun release() = finalize()

    fun use(block: (OfflineStream) -> Unit) {
        try {
            block(this)
        } finally {
            release()
        }
    }

    private external fun acceptWaveform(ptr: Long, samples: FloatArray, sampleRate: Int)
    private external fun setOption(ptr: Long, key: String, value: String)
    private external fun getOption(ptr: Long, key: String): String
    private external fun delete(ptr: Long)

    companion object {
        init {
            System.loadLibrary("sherpa-onnx-jni")
        }
    }
}
