package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject

class SetReminderToolTest {
    private class Fake(val accept: Boolean = true) : ReminderScheduler {
        val calls = mutableListOf<Pair<String, Long>>()
        override fun schedule(title: String, atEpochMs: Long): Boolean { calls += title to atEpochMs; return accept }
    }

    private fun args(text: String) = Json.parseToJsonElement(text).jsonObject

    @Test
    fun `sets a reminder minutes from now`() = runTest {
        val fake = Fake()
        val result = SetReminderTool(fake) { 1_000_000L }.run(args("""{"title":"喝水","minutes":10}"""))
        assertTrue(result.ok, result.output)
        assertEquals(listOf("喝水" to 1_000_000L + 600_000L), fake.calls)
        assertTrue(result.output.contains("10 分钟后"))
    }

    @Test
    fun `refuses empty titles and out-of-range minutes without scheduling`() = runTest {
        val fake = Fake()
        val tool = SetReminderTool(fake) { 0L }
        assertFalse(tool.run(args("""{"title":"  ","minutes":5}""")).ok)
        assertFalse(tool.run(args("""{"title":"x","minutes":0}""")).ok)
        assertFalse(tool.run(args("""{"title":"x","minutes":999999}""")).ok)
        assertFalse(tool.run(args("""{"title":"x"}""")).ok)
        assertEquals(emptyList(), fake.calls)
    }

    @Test
    fun `reports it when the platform cannot schedule`() = runTest {
        val result = SetReminderTool(Fake(accept = false)) { 0L }.run(args("""{"title":"x","minutes":5}"""))
        assertFalse(result.ok)
        assertTrue(result.output.contains("通知"))
    }

    @Test
    fun `summaries read well and errors lose the status prefix`() {
        assertEquals("喝水 · 10 分钟后", summarizeToolArguments("set_reminder", """{"title":"喝水","minutes":10}"""))
        assertEquals("额度不足", friendlyAgentError("HTTP 402: 额度不足"))
        assertEquals("智能体执行出错", friendlyAgentError(null))
    }
}
