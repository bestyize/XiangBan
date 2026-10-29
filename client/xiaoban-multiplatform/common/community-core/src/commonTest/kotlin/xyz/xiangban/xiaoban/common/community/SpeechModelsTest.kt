package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlinx.coroutines.test.runTest

private class MemoryFiles : ModelFiles {
    val stored = mutableMapOf<String, ByteArray>()
    val partial = mutableMapOf<String, ByteArray>()
    override fun size(name: String) = stored[name]?.size?.toLong() ?: -1
    override fun path(name: String) = "/mem/$name"
    override fun delete(name: String) { stored.remove(name) }
    override fun partialSize(name: String) = partial[name]?.size?.toLong() ?: 0
    override fun readPartial(name: String, onChunk: (ByteArray, Int) -> Unit) { partial[name]?.let { onChunk(it, it.size) } }
    override fun discardPartial(name: String) { partial.remove(name) }
    override fun beginWrite(name: String, append: Boolean): ModelWriter = object : ModelWriter {
        val buffer = (if (append) partial[name] ?: ByteArray(0) else ByteArray(0)).toMutableList()
        override fun write(bytes: ByteArray, length: Int) { for (i in 0 until length) buffer += bytes[i] }
        override fun commit() { stored[name] = buffer.toByteArray(); partial.remove(name) }
        override fun close() { partial[name] = buffer.toByteArray() }
        override fun abort() { partial.remove(name) }
    }
}

class SpeechModelsTest {
    private fun sha(text: String) = Sha256().also { it.update(text.encodeToByteArray()) }.hex()

    @Test
    fun `sha256 matches the standard vectors`() {
        assertEquals("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", sha(""))
        assertEquals("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", sha("abc"))
        val long = "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
        assertEquals("248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1", sha(long))
        // 分几次喂和一次喂结果一样
        val parts = Sha256().also { it.update("a".repeat(1000).encodeToByteArray()); it.update("b".repeat(77).encodeToByteArray()) }.hex()
        assertEquals(sha("a".repeat(1000) + "b".repeat(77)), parts)
    }

    @Test
    fun `sense voice tags are removed`() {
        assertEquals("你好。", SenseVoiceModel.cleanText("<|zh|><|NEUTRAL|><|Speech|><|withitn|>你好。"))
        assertEquals("没有标签", SenseVoiceModel.cleanText("没有标签"))
        assertEquals("a b", SenseVoiceModel.cleanText("a <|x|>b"))
    }

    @Test
    fun `a model is downloaded checked and kept only when it is whole`() = runTest {
        val body = "tokens-content"
        val spec = ModelSpec("tokens.txt", body.length.toLong(), sha(body))
        val files = MemoryFiles()
        val ok = SpeechModels(HttpClient(MockEngine { respond(body, HttpStatusCode.OK) }), files, "https://x/", listOf(spec))
        val seen = mutableListOf<Long>()
        ok.download { done, total -> seen += done; assertEquals(spec.bytes, total) }
        assertTrue(ok.ready)
        assertEquals(body, files.stored["tokens.txt"]!!.decodeToString())
        assertEquals(spec.bytes, seen.last())
        files.delete("tokens.txt")
        val bad = HttpClient(MockEngine { respond("wrong", HttpStatusCode.OK) })
        val models = SpeechModels(bad, files)
        assertFalse(models.ready)
        assertFailsWith<ModelDownloadException> { models.download { _, _ -> } }
        assertTrue(files.stored.isEmpty(), "校验不过不能留下文件")
        val missing = HttpClient(MockEngine { respond("", HttpStatusCode.NotFound) })
        assertFailsWith<ModelDownloadException> { SpeechModels(missing, MemoryFiles()).download { _, _ -> } }
        assertEquals(sha(body), spec.sha256)
    }

    @Test
    fun `files that are already there are not fetched again`() = runTest {
        val files = MemoryFiles()
        for (spec in VoiceModels.Files) files.stored[spec.name] = ByteArray(spec.bytes.toInt())
        var requests = 0
        val models = SpeechModels(HttpClient(MockEngine { requests++; respond("", HttpStatusCode.OK) }), files)
        assertTrue(models.ready)
        var last = 0L
        models.download { done, _ -> last = done }
        assertEquals(0, requests)
        assertEquals(VoiceModels.TotalBytes, last)
    }

    private val body = "0123456789abcdefghijklmnopqrstuvwxyz"
    private val spec get() = ModelSpec("m.bin", body.length.toLong(), sha(body))

    @Test
    fun `an interrupted download is continued from where it stopped`() = runTest {
        val files = MemoryFiles()
        val ranges = mutableListOf<String?>()
        var firstTry = true
        val client = HttpClient(MockEngine { request ->
            val range = request.headers[io.ktor.http.HttpHeaders.Range]
            ranges += range
            if (firstTry) {
                firstTry = false
                respond(body.substring(0, 10), HttpStatusCode.OK)        // 只发来一半就断了
            } else {
                val from = range!!.removePrefix("bytes=").removeSuffix("-").toInt()
                respond(body.substring(from), HttpStatusCode.PartialContent)
            }
        })
        val models = SpeechModels(client, files, "https://x/", listOf(spec))
        val error = assertFailsWith<ModelDownloadException> { models.download { _, _ -> } }
        assertTrue(error.message!!.contains("接着下"), error.message)
        assertEquals(10, files.partial["m.bin"]!!.size, "断了以后临时文件要留着")
        val progress = mutableListOf<Long>()
        models.download { done, _ -> progress += done }
        assertEquals(listOf(null, "bytes=10-"), ranges)
        assertTrue(models.ready)
        assertEquals(body, files.stored["m.bin"]!!.decodeToString())
        assertTrue(10L in progress && progress.last() == 36L, "进度要从已经下的地方接着走：$progress")
        assertTrue(files.partial.isEmpty())
    }

    @Test
    fun `a server that ignores the range sends everything again and the download still ends up right`() = runTest {
        val files = MemoryFiles().also { it.partial["m.bin"] = body.substring(0, 5).encodeToByteArray() }
        val client = HttpClient(MockEngine { respond(body, HttpStatusCode.OK) })
        val models = SpeechModels(client, files, "https://x/", listOf(spec))
        models.download { _, _ -> }
        assertEquals(body, files.stored["m.bin"]!!.decodeToString())
    }

    @Test
    fun `a corrupt leftover or a wrong result is thrown away`() = runTest {
        // 临时文件内容不对：续传完成后校验不过，临时文件被删，下次从头来。
        val files = MemoryFiles().also { it.partial["m.bin"] = "XXXXX".encodeToByteArray() }
        val client = HttpClient(MockEngine { request ->
            val from = request.headers[io.ktor.http.HttpHeaders.Range]!!.removePrefix("bytes=").removeSuffix("-").toInt()
            respond(body.substring(from), HttpStatusCode.PartialContent)
        })
        val models = SpeechModels(client, files, "https://x/", listOf(spec))
        val error = assertFailsWith<ModelDownloadException> { models.download { _, _ -> } }
        assertTrue(error.message!!.contains("校验"), error.message)
        assertTrue(files.partial.isEmpty() && files.stored.isEmpty())
        // 再来一次就是从头下，成功。
        val fresh = HttpClient(MockEngine { respond(body, HttpStatusCode.OK) })
        SpeechModels(fresh, files, "https://x/", listOf(spec)).download { _, _ -> }
        assertTrue(files.stored["m.bin"]!!.decodeToString() == body)
    }

    @Test
    fun `a leftover that is already complete is only checked`() = runTest {
        val files = MemoryFiles().also { it.partial["m.bin"] = body.encodeToByteArray() }
        var requests = 0
        val models = SpeechModels(HttpClient(MockEngine { requests++; respond("", HttpStatusCode.OK) }), files, "https://x/", listOf(spec))
        models.download { _, _ -> }
        assertEquals(0, requests)
        assertTrue(models.ready)
    }
}
