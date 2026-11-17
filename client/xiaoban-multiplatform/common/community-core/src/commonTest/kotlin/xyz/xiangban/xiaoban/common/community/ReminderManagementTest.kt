package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.*
import kotlin.test.*

class ReminderManagementTest {
    private class Scheduler : ReminderScheduler {
        override val supportsManagement = true
        val saved = linkedMapOf<String, LocalReminder>()
        var next = 0
        var pendingPermission = false
        override fun schedule(title: String, atEpochMs: Long): Boolean = error("use managed creation")
        override fun createReminder(title: String, atEpochMs: Long): ReminderWriteResult {
            val item = LocalReminder("r${++next}", title, atEpochMs, if (pendingPermission) "waiting_for_permission" else "scheduled")
            saved[item.id] = item
            return ReminderWriteResult(if (pendingPermission) ReminderScheduleResult.AwaitingPermission else ReminderScheduleResult.Scheduled, item)
        }
        override fun listReminders() = saved.values.toList()
        override fun updateReminder(id: String, title: String?, atEpochMs: Long?): ReminderWriteResult {
            val before = saved[id] ?: return ReminderWriteResult(ReminderScheduleResult.NotFound)
            val after = before.copy(title = title ?: before.title, atEpochMs = atEpochMs ?: before.atEpochMs)
            saved[id] = after
            return ReminderWriteResult(ReminderScheduleResult.Scheduled, after)
        }
        override fun cancelReminder(id: String) = saved.remove(id) != null
    }
    private fun args(text: String) = Json.parseToJsonElement(text).jsonObject

    @Test fun `query modify and cancel use the same stable reminder id`() = runTest {
        val scheduler = Scheduler()
        val tools = reminderTools(scheduler) { 1_000L }.associateBy { it.name }
        val first = tools.getValue("set_reminder").run(args("""{"title":"喝水","minutes":10}"""))
        assertTrue(first.ok)
        val id = args(first.output).getValue("reminder").jsonObject.getValue("id").jsonPrimitive.content
        tools.getValue("set_reminder").run(args("""{"title":"开会","minutes":20}"""))
        val updated = tools.getValue("update_reminder").run(args("""{"id":"$id","minutes":30}"""))
        assertTrue(updated.ok, updated.output)
        assertEquals(2, scheduler.saved.size)
        assertEquals(LocalReminder(id, "喝水", 1_801_000L, "scheduled"), scheduler.saved[id])
        val listed = tools.getValue("list_reminders").run(args("""{"limit":1}"""))
        assertEquals(1, args(listed.output).getValue("reminders").jsonArray.size)
        assertEquals(1, args(listed.output).getValue("next_offset").jsonPrimitive.int)
        assertTrue(tools.getValue("cancel_reminder").run(args("""{"id":"$id"}""")).ok)
        assertEquals(listOf("开会"), scheduler.saved.values.map { it.title })
        assertFalse(tools.getValue("update_reminder").run(args("""{"id":"$id","minutes":5}""")).ok)
    }

    @Test fun `pending notification permission keeps a manageable receipt without claiming delivery`() = runTest {
        val scheduler = Scheduler().apply { pendingPermission = true }
        val result = SetReminderTool(scheduler) { 0L }.run(args("""{"title":"喝水","minutes":5}"""))
        assertTrue(result.ok, "saved pending permission is accepted, not a reason to create a duplicate")
        val receipt = args(result.output).getValue("reminder").jsonObject
        assertEquals("waiting_for_permission", receipt.getValue("status").jsonPrimitive.content)
        val id = receipt.getValue("id").jsonPrimitive.content
        assertTrue(CancelReminderTool(scheduler).run(args("""{"id":"$id"}""")).ok)
        assertTrue(scheduler.saved.isEmpty())
    }

    @Test fun `invalid changes do not mutate reminders and unsupported hosts advertise creation only`() = runTest {
        val scheduler = Scheduler()
        val tool = SetReminderTool(scheduler) { 0L }
        assertFalse(tool.run(args("""{"title":"喝水","minutes":1.5}""")).ok)
        assertFalse(tool.run(args("""{"title":"喝水","minutes":"5"}""")).ok)
        assertTrue(scheduler.saved.isEmpty())
        val legacy = object : ReminderScheduler { override fun schedule(title: String, atEpochMs: Long) = true }
        assertEquals(listOf("set_reminder"), reminderTools(legacy) { 0L }.map { it.name })
        assertEquals("read", ListRemindersTool(scheduler).risk)
        assertEquals("write", UpdateReminderTool(scheduler) { 0L }.risk)
        assertEquals("write", CancelReminderTool(scheduler).risk)
    }
}
