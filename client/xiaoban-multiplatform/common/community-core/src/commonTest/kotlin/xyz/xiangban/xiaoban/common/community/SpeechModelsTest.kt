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
    override fun size(name: String) = stored[name]?.size?.toLong() ?: -1
    override fun path(name: String) = "/mem/$name"
    override fun delete(name: String) { stored.remove(name) }
    override fun beginWrite(name: String): ModelWriter = object : ModelWriter {
        val buffer = mutableListOf<Byte>()
        override fun write(bytes: ByteArray, length: Int) { for (i in 0 until length) buffer += bytes[i] }
        override fun commit() { stored[name] = buffer.toByteArray() }
        override fun abort() {}
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
        for (spec in SenseVoiceModel.Files) files.stored[spec.name] = ByteArray(spec.bytes.toInt())
        var requests = 0
        val models = SpeechModels(HttpClient(MockEngine { requests++; respond("", HttpStatusCode.OK) }), files)
        assertTrue(models.ready)
        var last = 0L
        models.download { done, _ -> last = done }
        assertEquals(0, requests)
        assertEquals(SenseVoiceModel.TotalBytes, last)
    }
}
