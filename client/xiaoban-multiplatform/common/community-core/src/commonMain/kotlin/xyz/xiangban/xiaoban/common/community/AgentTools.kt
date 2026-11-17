package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.JsonArray

/** 平台负责到点弹系统通知（Android 上是闹钟加通知）。 */
interface ReminderScheduler {
    /** 在 [atEpochMs] 提醒用户；通知权限没开等原因设不了就返回 false。 */
    fun schedule(title: String, atEpochMs: Long): Boolean
    fun scheduleResult(title: String, atEpochMs: Long): ReminderScheduleResult =
        if (schedule(title, atEpochMs)) ReminderScheduleResult.Scheduled else ReminderScheduleResult.Failed
    /** Older platform implementations can still provide creation without advertising management. */
    val supportsManagement: Boolean get() = false
    fun createReminder(title: String, atEpochMs: Long): ReminderWriteResult =
        ReminderWriteResult(scheduleResult(title, atEpochMs))
    fun listReminders(): List<LocalReminder> = throw UnsupportedOperationException("当前平台不支持管理提醒")
    fun updateReminder(id: String, title: String?, atEpochMs: Long?): ReminderWriteResult =
        ReminderWriteResult(ReminderScheduleResult.Failed)
    fun cancelReminder(id: String): Boolean = false
}

data class LocalReminder(val id: String, val title: String, val atEpochMs: Long, val status: String)
data class ReminderWriteResult(val status: ReminderScheduleResult, val reminder: LocalReminder? = null)

enum class ReminderScheduleResult(val message: String) {
    Scheduled("已安排提醒，系统可能延后送达。"),
    AwaitingPermission("提醒已保存，允许通知后会自动生效，无需重复设置。"),
    NotificationsDisabled("提醒未设置，请在系统设置中开启小伴通知和提醒渠道。"),
    Failed("提醒未设置，请检查时间或稍后重试。"),
    NotFound("找不到这个提醒，请先用 list_reminders 查询当前提醒。"),
}

private const val MAX_REMINDER_MINUTES = 7 * 24 * 60

/** 智能体的“设提醒”工具：到点由平台弹通知。会改动手机上的东西，所以每次都先问用户。 */
class SetReminderTool(private val scheduler: ReminderScheduler, private val now: () -> Long) : AgentTool {
    override val name = "set_reminder"
    override val description = "给用户设一个提醒，到点会弹出系统通知。minutes 是从现在起多少分钟后（1 到 10080，也就是最多 7 天）。" +
        "用户说的是具体钟点时，先查一下当前时间再换算成分钟数。若返回等待通知权限，提醒已保存，不要重复创建；允许通知后才会生效。"
    override val parameters: JsonObject = buildJsonObject {
        put("type", "object")
        putJsonObject("properties") {
            putJsonObject("title") { put("type", "string"); put("description", "提醒的内容，简短一句话") }
            putJsonObject("minutes") { put("type", "integer"); put("description", "从现在起多少分钟后提醒") }
        }
        put("required", buildJsonArray { add(JsonPrimitive("title")); add(JsonPrimitive("minutes")) })
    }
    override val risk = "write"

    override suspend fun run(arguments: JsonObject): AgentToolResult {
        val title = arguments.reminderString("title").orEmpty()
        val minutes = arguments.reminderMinutes()
        if (title.isEmpty()) return AgentToolResult(false, "提醒的内容不能为空")
        if (minutes == null || minutes < 1 || minutes > MAX_REMINDER_MINUTES) return AgentToolResult(false, "minutes 要在 1 到 $MAX_REMINDER_MINUTES 之间")
        val at = now() + minutes * 60_000L
        currentCoroutineContext().ensureActive()
        val result = scheduler.createReminder(title.take(80), at)
        val message = if (result.status == ReminderScheduleResult.Scheduled)
            "已设置 $minutes 分钟后提醒“${title.take(80)}”。系统可能延后送达。" else result.status.message
        return AgentToolResult(result.accepted, result.output(message))
    }
}

private fun JsonObject.reminderMinutes(): Int? = (this["minutes"] as? JsonPrimitive)?.takeUnless { it.isString }?.intOrNull
private val ReminderWriteResult.accepted get() = status == ReminderScheduleResult.Scheduled || status == ReminderScheduleResult.AwaitingPermission
private fun JsonObject.reminderString(key: String): String? = (this[key] as? JsonPrimitive)?.takeIf { it.isString }?.contentOrNull?.trim()
private fun LocalReminder.json(): JsonObject = buildJsonObject {
    put("id", id); put("title", title); put("at_epoch_ms", atEpochMs); put("status", status)
}
private fun ReminderWriteResult.output(message: String): String = reminder?.let {
    buildJsonObject { put("message", message); put("reminder", it.json()) }.toString()
} ?: message

/** Only advertise operations the platform can actually perform. */
fun reminderTools(scheduler: ReminderScheduler, now: () -> Long): List<AgentTool> = buildList {
    add(SetReminderTool(scheduler, now))
    if (scheduler.supportsManagement) {
        add(ListRemindersTool(scheduler))
        add(UpdateReminderTool(scheduler, now))
        add(CancelReminderTool(scheduler))
    }
}

class ListRemindersTool(private val scheduler: ReminderScheduler) : AgentTool {
    override val name = "list_reminders"
    override val description = "查询本机已保存的提醒（包含等待通知权限、已触发和失败状态），返回稳定 id。修改或取消提醒前先查询，不要猜 id。offset 从 0 开始，limit 为 1–100。"
    override val risk = "read"
    override val parameters = buildJsonObject {
        put("type", "object")
        putJsonObject("properties") {
            putJsonObject("offset") { put("type", "integer"); put("minimum", 0) }
            putJsonObject("limit") { put("type", "integer"); put("minimum", 1); put("maximum", 100) }
        }
        put("additionalProperties", false)
    }
    override suspend fun run(arguments: JsonObject): AgentToolResult {
        val offset = if ("offset" in arguments) (arguments["offset"] as? JsonPrimitive)?.takeUnless { it.isString }?.intOrNull else 0
        val limit = if ("limit" in arguments) (arguments["limit"] as? JsonPrimitive)?.takeUnless { it.isString }?.intOrNull else 20
        if (offset == null || offset < 0 || limit == null || limit !in 1..100) return AgentToolResult(false, "offset 必须非负，limit 必须在 1–100 之间")
        currentCoroutineContext().ensureActive()
        val reminders = scheduler.listReminders().sortedWith(compareBy<LocalReminder> { it.atEpochMs }.thenBy { it.id })
        val page = reminders.drop(offset).take(limit)
        return AgentToolResult(true, buildJsonObject {
            put("reminders", JsonArray(page.map { it.json() })); put("total", reminders.size)
            if (offset < reminders.size && page.size < reminders.size - offset) put("next_offset", offset + page.size)
        }.toString())
    }
}

class UpdateReminderTool(private val scheduler: ReminderScheduler, private val now: () -> Long) : AgentTool {
    override val name = "update_reminder"
    override val description = "按 list_reminders 返回的 id 修改提醒内容或提醒时间。minutes 从现在起计算（1–10080），不传则保留原时间；不传 title 则保留内容。仍使用原提醒 id，不会另建一条。"
    override val parameters = buildJsonObject {
        put("type", "object")
        putJsonObject("properties") {
            putJsonObject("id") { put("type", "string") }
            putJsonObject("title") { put("type", "string") }
            putJsonObject("minutes") { put("type", "integer"); put("minimum", 1); put("maximum", MAX_REMINDER_MINUTES) }
        }
        put("required", JsonArray(listOf(JsonPrimitive("id"))))
        put("additionalProperties", false)
    }
    override suspend fun run(arguments: JsonObject): AgentToolResult {
        val id = arguments.reminderString("id").orEmpty()
        val title = arguments.reminderString("title")
        val minutes = arguments.reminderMinutes()
        if (id.isBlank()) return AgentToolResult(false, "需要提醒 id，请先用 list_reminders 查询")
        if ("title" !in arguments && "minutes" !in arguments) return AgentToolResult(false, "至少提供 title 或 minutes")
        if ("title" in arguments && title.isNullOrBlank()) return AgentToolResult(false, "提醒的内容不能为空")
        if ("minutes" in arguments && (minutes == null || minutes !in 1..MAX_REMINDER_MINUTES)) return AgentToolResult(false, "minutes 要在 1 到 $MAX_REMINDER_MINUTES 之间")
        currentCoroutineContext().ensureActive()
        val result = scheduler.updateReminder(id, title?.take(80), minutes?.let { now() + it * 60_000L })
        val message = if (result.status == ReminderScheduleResult.Scheduled) "提醒已更新。系统可能延后送达。" else result.status.message
        return AgentToolResult(result.accepted, result.output(message))
    }
}

class CancelReminderTool(private val scheduler: ReminderScheduler) : AgentTool {
    override val name = "cancel_reminder"
    override val description = "按 list_reminders 返回的 id 取消并删除一个本机提醒，包含尚在等待通知权限的提醒。不会影响其他提醒。"
    override val parameters = buildJsonObject {
        put("type", "object")
        putJsonObject("properties") { putJsonObject("id") { put("type", "string") } }
        put("required", JsonArray(listOf(JsonPrimitive("id"))))
        put("additionalProperties", false)
    }
    override suspend fun run(arguments: JsonObject): AgentToolResult {
        val id = arguments.reminderString("id").orEmpty()
        if (id.isBlank()) return AgentToolResult(false, "需要提醒 id，请先用 list_reminders 查询")
        currentCoroutineContext().ensureActive()
        return if (scheduler.cancelReminder(id)) AgentToolResult(true, buildJsonObject { put("cancelled", true); put("id", id) }.toString())
        else AgentToolResult(false, ReminderScheduleResult.NotFound.message)
    }
}
