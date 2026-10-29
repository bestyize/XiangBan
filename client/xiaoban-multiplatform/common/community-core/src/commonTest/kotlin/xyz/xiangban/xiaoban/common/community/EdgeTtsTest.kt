package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class EdgeTtsTest {
    @Test
    fun `the access token matches the one the computer computes`() {
        // 向量取自电脑端 tts/drm.rs 的测试。
        assertEquals("2CD23A2CA0B2595A-F3C846C370E05F9CC83ACC5F995C5F5D87A4D0F14CE8936A0F902E64099E38A2", EdgeDrm.secMsGec(1_700_000_000))
        assertEquals(EdgeDrm.secMsGec(1_700_000_000), EdgeDrm.secMsGec(1_700_000_001))
        assertNotEquals(EdgeDrm.secMsGec(1_700_000_000), EdgeDrm.secMsGec(1_700_000_300))
    }

    @Test
    fun `timestamps, voices and ssml follow the read aloud format`() {
        assertEquals("Tue Nov 14 2023 22:13:20 GMT+0000 (Coordinated Universal Time)", EdgeProtocol.timestamp(1_700_000_000))
        assertEquals("Thu Jan 01 1970 00:00:00 GMT+0000 (Coordinated Universal Time)", EdgeProtocol.timestamp(0))
        assertEquals("Thu Feb 29 2024 12:00:01 GMT+0000 (Coordinated Universal Time)", EdgeProtocol.timestamp(1_709_208_001))
        assertEquals("Microsoft Server Speech Text to Speech Voice (zh-CN, XiaoxiaoNeural)", EdgeProtocol.voiceName("zh-CN-XiaoxiaoNeural"))
        val ssml = EdgeProtocol.ssml("ID", "zh-CN-XiaoxiaoNeural", 0, 0, 100, "a<&", 0)
        assertTrue(ssml.contains("a&lt;&amp;") && ssml.contains("rate='+0%'") && ssml.contains("volume='+0%'") && ssml.contains("Path:ssml"), ssml)
        assertTrue(EdgeProtocol.wsUrl("CID", 1_700_000_000).contains("Sec-MS-GEC=2CD23A2CA0B2595A-"))
    }

    @Test
    fun `text is cut into pieces that fit and never in the middle of a character`() {
        val parts = EdgeProtocol.chunks("小伴 & <你好> 😀 ".repeat(500))
        assertTrue(parts.size > 1 && parts.all { it.encodeToByteArray().size <= 4096 })
        assertEquals("你好。", EdgeProtocol.chunks("  你好。 ").single())
        assertFailsWith<EngineUnavailable> { EdgeProtocol.chunks("\u0000\u0001") }
        assertFailsWith<EngineUnavailable> { EdgeProtocol.chunks("字".repeat(10_001)) }
    }

    @Test
    fun `response frames are validated`() {
        val head = "Path:audio\r\nContent-Type:audio/mpeg\r\n".encodeToByteArray()
        val frame = byteArrayOf((head.size shr 8).toByte(), head.size.toByte()) + head + "MP3".encodeToByteArray()
        assertEquals("MP3", EdgeProtocol.audioFrame(frame).decodeToString())
        assertEquals(0, EdgeProtocol.audioFrame(byteArrayOf(0, 0)).size)
        assertFailsWith<EngineUnavailable> { EdgeProtocol.audioFrame(byteArrayOf(0, 1, 2)) }
        assertEquals("turn.end", EdgeProtocol.textPath("X-RequestId:1\r\nPath:turn.end\r\n\r\n{}"))
    }
}
