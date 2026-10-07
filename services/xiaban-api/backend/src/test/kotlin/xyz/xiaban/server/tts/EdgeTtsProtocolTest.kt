package xyz.xiaban.server.tts

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

class EdgeTtsProtocolTest {

    @Test
    fun `sec-ms-gec rounds the clock to a five minute window and is stable within it`() {
        val base = 1_800_000_000L
        // Both timestamps fall in the same 300-second window.
        assertEquals(EdgeTtsProtocol.secMsGec(base), EdgeTtsProtocol.secMsGec(base + 299))
        // The next window differs.
        assertTrue(EdgeTtsProtocol.secMsGec(base) != EdgeTtsProtocol.secMsGec(base + 300))
        // 64 hex characters, uppercase.
        assertEquals(64, EdgeTtsProtocol.secMsGec(base).length)
        assertTrue(EdgeTtsProtocol.secMsGec(base).all { it in "0123456789ABCDEF" })
    }

    @Test
    fun `voice names are normalized to the form the service expects`() {
        assertEquals(
            "Microsoft Server Speech Text to Speech Voice (zh-CN, XiaoxiaoNeural)",
            EdgeTtsProtocol.normalizeVoice("zh-CN-XiaoxiaoNeural"),
        )
        // Regional voices keep the extra segment.
        assertEquals(
            "Microsoft Server Speech Text to Speech Voice (zh-CN-liaoning, XiaobeiNeural)",
            EdgeTtsProtocol.normalizeVoice("zh-CN-liaoning-XiaobeiNeural"),
        )
        // An already-normalized name passes through untouched.
        val already = "Microsoft Server Speech Text to Speech Voice (zh-CN, XiaoxiaoNeural)"
        assertEquals(already, EdgeTtsProtocol.normalizeVoice(already))
    }

    @Test
    fun `ssml escapes the text so markup in a reply cannot break the request`() {
        val ssml = EdgeTtsProtocol.ssml(EdgeTtsVoiceOptions(), "a < b & c > d")
        assertTrue(ssml.contains("a &lt; b &amp; c &gt; d"), ssml)
    }

    @Test
    fun `prosody values are validated at construction`() {
        assertFailsWith<IllegalArgumentException> { EdgeTtsVoiceOptions(rate = "10%") }
        assertFailsWith<IllegalArgumentException> { EdgeTtsVoiceOptions(pitch = "+10") }
        assertFailsWith<IllegalArgumentException> { EdgeTtsVoiceOptions(volume = "loud") }
        EdgeTtsVoiceOptions(rate = "-10%", pitch = "+20Hz", volume = "+0%")
    }

    @Test
    fun `long text is chunked on whitespace without splitting characters`() {
        val text = "这是一句话。 ".repeat(600)
        val chunks = Utf8TextChunker.split(text, maxBytes = 512)
        assertTrue(chunks.size > 1, "expected several chunks, got ${chunks.size}")
        for (chunk in chunks) {
            assertTrue(chunk.toByteArray(Charsets.UTF_8).size <= 512, "chunk exceeded the byte budget")
            assertTrue(chunk.isNotBlank())
        }
        // Nothing is dropped: the concatenated chunks carry the same visible text.
        assertEquals(text.replace(" ", ""), chunks.joinToString("").replace(" ", ""))
    }

    @Test
    fun `blank text produces no chunks`() {
        assertTrue(Utf8TextChunker.split("   \n\t ").isEmpty())
    }

    @Test
    fun `binary frames are split into headers and payload`() {
        val payload = byteArrayOf(1, 2, 3, 4)
        val header = "Path:audio\r\nX-RequestId:abc".toByteArray()
        val frame = ByteArray(2 + header.size + 2 + payload.size)
        frame[0] = ((header.size + 2) shr 8).toByte()
        frame[1] = ((header.size + 2) and 0xff).toByte()
        header.copyInto(frame, 2)
        frame[2 + header.size] = '\r'.code.toByte()
        frame[2 + header.size + 1] = '\n'.code.toByte()
        payload.copyInto(frame, 2 + header.size + 2)

        val parsed = parseBinaryAudioFrame(frame)
        assertEquals("audio", parsed.headers["Path"])
        assertTrue(payload.contentEquals(parsed.data))
    }

    @Test
    fun `the two byte zero frame ends a stream without audio`() {
        val parsed = parseBinaryAudioFrame(byteArrayOf(0, 0))
        assertTrue(parsed.data.isEmpty())
    }

    @Test
    fun `text responses are split into path and payload`() {
        val response = parseTextResponse("Path:turn.end\r\nX-RequestId:abc\r\n\r\n{}")
        assertEquals("turn.end", response.path)
        assertEquals("{}", response.payload)
    }
}
