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

/** 平台负责到点弹系统通知（Android 上是闹钟加通知）。 */
interface ReminderScheduler {
    /** 在 [atEpochMs] 提醒用户；通知权限没开等原因设不了就返回 false。 */
    fun schedule(title: String, atEpochMs: Long): Boolean
    fun scheduleResult(title: String, atEpochMs: Long): ReminderScheduleResult =
        if (schedule(title, atEpochMs)) ReminderScheduleResult.Scheduled else ReminderScheduleResult.Failed
}

enum class ReminderScheduleResult(val message: String) {
    Scheduled("已安排提醒，系统可能延后送达。"),
    AwaitingPermission("提醒已保存，允许通知后会自动生效，无需重复设置。"),
    NotificationsDisabled("提醒未设置，请在系统设置中开启小伴通知和提醒渠道。"),
    Failed("提醒未设置，请检查时间或稍后重试。"),
}

private const val MAX_REMINDER_MINUTES = 7 * 24 * 60

/** 智能体的“设提醒”工具：到点由平台弹通知。会改动手机上的东西，所以每次都先问用户。 */
class SetReminderTool(private val scheduler: ReminderScheduler, private val now: () -> Long) : AgentTool {
    override val name = "set_reminder"
    override val description = "给用户设一个提醒，到点会弹出系统通知。minutes 是从现在起多少分钟后（1 到 10080，也就是最多 7 天）。" +
        "用户说的是具体钟点时，先查一下当前时间再换算成分钟数。"
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
        val title = arguments["title"]?.jsonPrimitive?.contentOrNull?.trim().orEmpty()
        val minutes = arguments["minutes"]?.jsonPrimitive?.contentOrNull?.toDoubleOrNull()?.toLong()
        if (title.isEmpty()) return AgentToolResult(false, "提醒的内容不能为空")
        if (minutes == null || minutes < 1 || minutes > MAX_REMINDER_MINUTES) return AgentToolResult(false, "minutes 要在 1 到 $MAX_REMINDER_MINUTES 之间")
        val at = now() + minutes * 60_000
        currentCoroutineContext().ensureActive()
        val result = scheduler.scheduleResult(title.take(80), at)
        return if (result == ReminderScheduleResult.Scheduled) {
            AgentToolResult(true, "已设置 $minutes 分钟后提醒“${title.take(80)}”。系统可能延后送达。")
        } else AgentToolResult(false, result.message)
    }
}
