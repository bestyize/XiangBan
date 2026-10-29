package xyz.xiangban.xiaoban.common.community

import kotlin.math.PI
import kotlin.math.sin
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class VoiceAlignmentTest {
    @Test
    fun `smart turn features match desktop reference for the same audio`() {
        val pcm = ByteArray(128_000 * 2)
        for (i in 0 until 48_000) {
            val t = i.toDouble() / 16_000
            val sample = ((0.3 * sin(2 * PI * 220 * t) + 0.1 * sin(2 * PI * 1330 * t)) * 32767).toInt()
            val pos = (80_000 + i) * 2
            pcm[pos] = sample.toByte(); pcm[pos + 1] = (sample shr 8).toByte()
        }
        val result = SmartTurnFeatures.fromPcm(pcm)
        assertEquals(80 * 800, result.size)
        val expected = listOf(Triple(0, 0, -0.2663908), Triple(10, 499, 0.5666980), Triple(10, 500, 1.0859886), Triple(3, 650, 1.3544856), Triple(79, 799, -0.1892592))
        for ((mel, frame, value) in expected) assertTrue(kotlin.math.abs(result[mel * 800 + frame] - value) < 0.03, "mel=$mel frame=$frame got=${result[mel * 800 + frame]}")
    }

    @Test
    fun `voice notes stay hidden and identify interruption progress`() {
        val note = voiceInterruptionNote("之前已经说的内容") + voiceSpokenNote("我想想")
        assertTrue(note.contains("之前已经说的内容"))
        assertEquals("接下来呢", withoutVoiceNotes(note + "接下来呢"))
    }

    @Test
    fun `filler and playback echoes do not count as interruptions`() {
        assertTrue(isVoiceFiller("嗯嗯"))
        assertFalse(isVoiceFiller("嗯，我还有一个问题"))
        assertTrue(isVoiceEcho("下周二下午", "我已经为你安排了下周二下午的提醒"))
        assertFalse(isVoiceEcho("换成星期五", "我已经为你安排了下周二下午的提醒"))
    }

    @Test
    fun `acknowledgements fit the conversation and avoid emotional disclosures`() {
        assertEquals(VoiceAckKind.Hello, voiceAckKind("小伴你好"))
        assertEquals(VoiceAckKind.HowAreYou, voiceAckKind("你好吗"))
        assertEquals(VoiceAckKind.Task, voiceAckKind("帮我查一下天气"))
        assertEquals(VoiceAckKind.Question, voiceAckKind("今天天气怎么样呢"))
        assertEquals(0L, voiceAckDelayMs(VoiceAckKind.Thanks))
        assertEquals(700L, voiceAckDelayMs(VoiceAckKind.Task))
        assertNull(voiceAckKind("我今天很难过"))
        assertNull(voiceAckKind("嗯嗯"))
        assertTrue("我想想" in AllVoiceAckPhrases && "我挺好的呀" in AllVoiceAckPhrases)
    }

    @Test
    fun `an acknowledgement is never the one used last`() {
        val both = VoiceAckPhrases.getValue(VoiceAckKind.Hello)
        assertEquals("嗨，我在呢", chooseVoiceAck(both, last = "你好呀，我在") { 0.0 })
        assertEquals("你好呀，我在", chooseVoiceAck(both, last = "嗨，我在呢") { 0.99 })
        assertEquals("早上好呀", chooseVoiceAck(listOf("早上好呀"), last = "早上好呀"))
        assertNull(chooseVoiceAck(emptyList(), last = null))
    }

    @Test
    fun `stray english words from the recogniser are not speech`() {
        assertTrue(isVoiceStray("The"))
        assertTrue(isVoiceStray("<|en|><|NEUTRAL|>the uh"))
        assertFalse(isVoiceStray("the weather is nice today"))
        assertFalse(isVoiceStray("the 天气"))
        assertTrue(isVoiceNoise("（杂音）"))
        assertTrue(isVoiceNoise("嗯"))
        assertFalse(isVoiceNoise("换成星期五"))
    }

    @Test
    fun `sentences are cut like the desktop does`() {
        // 第一句：停顿满 8 个字就先念出去。
        assertEquals(listOf("好的，我来帮你看一下，"), SpeechSplitter().feed("好的，我来帮你看一下，"))
        // 之后的句子要满 48 个字，遇到停顿才切。
        val later = SpeechSplitter()
        assertEquals(listOf("第一句话。"), later.feed("第一句话。"))
        assertEquals(emptyList(), later.feed("这一句还不长，所以逗号不切开，"))
        // 列表的“1.”念不成一句，留给后面的话。
        val list = SpeechSplitter()
        assertEquals(emptyList(), list.feed("1."))
        assertEquals(listOf("1.打开设置。"), list.feed("打开设置。"))
    }

    @Test
    fun `speech skips emoji families flags modifiers and symbols but keeps keycap numbers`() {
        val text = "你好😊，家人👨‍👩‍👧‍👦，加油👍🏽！ 🇨🇳 1️⃣ 2️⃣。"
        val spoken = speechText(text)
        assertTrue(spoken.contains("你好"))
        assertTrue(spoken.contains("12") || (spoken.contains("1") && spoken.contains("2")))
        assertFalse(spoken.contains("😊"))
        assertFalse(spoken.contains("👨"))
        assertFalse(spoken.contains("🏽"))
        assertFalse(spoken.contains("🇨"))
        assertFalse(spoken.contains("\u200d"))
        assertEquals("", speechText("😊❤️👍🏽"))
    }

    @Test
    fun `manual speech omits code fences and link destinations`() {
        val spoken = speechText("你好，[文档](https://example.org)。\n```js\nalert(1)\n```\n完成。")
        assertTrue(spoken.contains("文档"))
        assertTrue(spoken.contains("完成"))
        assertFalse(spoken.contains("example.org"))
        assertFalse(spoken.contains("alert"))
    }

    @Test
    fun `live transcript snapshots are marked stale when speech continues`() {
        val gate = object : SpeechGate {
            override fun voiced(frame: ByteArray, strict: Boolean) = frame[0] == 1.toByte()
            override fun level(frame: ByteArray) = 0f
        }
        val detector = EndpointDetector(gate = gate)
        var preview: VadResult? = null
        repeat(60) {
            val result = detector.feed(byteArrayOf(1, 0))
            if (result.preview != null) preview = result
        }
        val recorded = assertNotNull(preview)
        assertNotNull(recorded.preview)
        var end: VadResult? = null
        repeat(60) { if (end?.utterance == null) end = detector.feed(byteArrayOf(0, 0)) }
        assertNotNull(end?.utterance)
        assertTrue(end!!.speechVersion > recorded.speechVersion)
    }

    @Test
    fun `speech drops quotation marks and stray symbols like the desktop`() {
        assertEquals("他说你好，她说再见，还有好和行、可以。", withoutSpokenSymbols("他说“你好”，她说\"再见\"，还有‘好’和「行」、『可以』。"))
        assertEquals("开始 然后 don't stop now", withoutSpokenSymbols("'开始' 然后 don't stop 'now'"))
        assertEquals("2 3 等于 6", withoutSpokenSymbols("2 * 3 等于 6"))
        assertEquals("文件 report final v2 已保存", withoutSpokenSymbols("文件 report_final_v2 已保存"))
        assertEquals("A B C D", withoutSpokenSymbols("A | B ~ C ^ D"))
        assertEquals("重点 和 强调", speechText("**重点** 和 *强调*"))
    }
}
