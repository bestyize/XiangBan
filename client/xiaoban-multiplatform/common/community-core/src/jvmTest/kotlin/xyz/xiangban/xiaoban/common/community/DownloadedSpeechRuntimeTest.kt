package xyz.xiangban.xiaoban.common.community

import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse

class DownloadedSpeechRuntimeTest {
    private val content = "test native library".encodeToByteArray()
    private fun manifest() = SpeechRuntimeManifest(1, "arm64-v8a", "a".repeat(64), SpeechRuntimeManifest.Names.map {
        SpeechRuntimeLibrary(it, content.size.toLong(), Sha256().also { it.update(content) }.hex(), listOf("https://example.com/$it"))
    })

    private class Files(val root: File) : ModelFiles {
        override fun path(name: String) = root.resolve(name).absolutePath
        override fun size(name: String) = File(path(name)).takeIf { it.isFile }?.length() ?: -1
        override fun delete(name: String) { File(path(name)).delete() }
        override fun partialSize(name: String) = 0L
        override fun readPartial(name: String, onChunk: (ByteArray, Int) -> Unit) = Unit
        override fun discardPartial(name: String) = Unit
        override fun beginWrite(name: String, append: Boolean): ModelWriter = error("not used")
    }

    private fun withFiles(test: (Files) -> Unit) {
        val root = kotlin.io.path.createTempDirectory("speech-runtime").toFile()
        try { test(Files(root)) } finally { root.deleteRecursively() }
    }

    private fun install(files: Files, manifest: SpeechRuntimeManifest) {
        manifest.specs().forEach { spec ->
            File(files.path(spec.name)).also { it.parentFile.mkdirs(); it.writeBytes(content) }
        }
    }

    @Test
    fun `missing libraries can be downloaded and loaded in the same process`() = withFiles { files ->
        val spec = manifest()
        val loaded = mutableListOf<String>()
        val runtime = DownloadedSpeechRuntime(spec, files) { loaded += File(it).name }
        assertFailsWith<EngineUnavailable> { runtime.load("sherpa-onnx-jni") }
        install(files, spec)
        runtime.load("sherpa-onnx-jni")
        runtime.load("sherpa-onnx-jni")
        runtime.load("onnxruntime4j_jni")
        assertEquals(listOf("libort_sherpa.so", "libsherpa-onnx-jni.so", "libonnxruntime.so", "libonnxruntime4j_jni.so"), loaded)
    }

    @Test
    fun `same size corruption is removed before any code executes`() = withFiles { files ->
        val spec = manifest()
        install(files, spec)
        val target = File(files.path(spec.specs().first { it.name.endsWith("/libonnxruntime.so") }.name))
        target.writeBytes(ByteArray(content.size))
        val loaded = mutableListOf<String>()
        val runtime = DownloadedSpeechRuntime(spec, files) { loaded += it }
        assertFailsWith<EngineUnavailable> { runtime.load("onnxruntime") }
        assertEquals(emptyList(), loaded)
        assertFalse(target.exists())
        target.writeBytes(content)
        runtime.load("onnxruntime")
        assertEquals(listOf(target.absolutePath), loaded)
    }

    @Test
    fun `a failed native load can be retried`() = withFiles { files ->
        val spec = manifest()
        install(files, spec)
        var attempts = 0
        val runtime = DownloadedSpeechRuntime(spec, files) {
            if (attempts++ == 0) throw UnsatisfiedLinkError("temporary")
        }
        assertFailsWith<UnsatisfiedLinkError> { runtime.load("onnxruntime") }
        runtime.load("onnxruntime")
        assertEquals(2, attempts)
        assertFailsWith<IllegalArgumentException> { runtime.load("../../other") }
        assertFailsWith<IllegalArgumentException> { spec.copy(version = "../other").validate() }
        assertFailsWith<IllegalArgumentException> { spec.copy(abi = "x86").validate() }
    }
}
