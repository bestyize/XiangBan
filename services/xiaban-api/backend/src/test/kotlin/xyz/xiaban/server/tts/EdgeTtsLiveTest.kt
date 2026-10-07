package xyz.xiaban.server.tts

import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertTrue

/**
 * Live check against the real Edge read-aloud endpoint. The service is unofficial and
 * external, so this stays inert in normal runs and is enabled with EDGE_TTS_LIVE_TEST=1.
 */
class EdgeTtsLiveTest {
    @Test
    fun `synthesizes chinese speech to valid mp3`() = runBlocking {
        if (System.getenv("EDGE_TTS_LIVE_TEST") != "1") return@runBlocking

        val http = edgeTtsHttpClient()
        try {
            val audio = EdgeTtsClient(http).synthesize("你好，我是小伴。今天辛苦了。")
            assertTrue(audio.size > 1_000, "expected real audio, got ${audio.size} bytes")
            // MP3 frames start with 0xFFEx/0xFFFx, or the file carries an ID3 tag.
            val looksLikeMp3 = (audio[0] == 0xFF.toByte() && (audio[1].toInt() and 0xE0) == 0xE0) ||
                audio.decodeToString(0, 3) == "ID3"
            assertTrue(looksLikeMp3, "not an MP3: ${audio.copyOfRange(0, 3).joinToString(" ") { "%02x".format(it) }}")
        } finally {
            http.close()
        }
    }

    @Test
    fun `lists chinese voices`() = runBlocking {
        if (System.getenv("EDGE_TTS_LIVE_TEST") != "1") return@runBlocking

        val http = edgeTtsHttpClient()
        try {
            val zh = EdgeTtsClient(http).voices().filter { it.locale == "zh-CN" }
            assertTrue(zh.any { it.shortName == "zh-CN-XiaoxiaoNeural" }, "expected Xiaoxiao among $zh")
        } finally {
            http.close()
        }
    }
}
