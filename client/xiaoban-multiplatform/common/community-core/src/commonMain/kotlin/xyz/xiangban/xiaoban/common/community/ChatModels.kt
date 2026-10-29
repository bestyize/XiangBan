package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive

// 聊天页里的卡片用到的数据：工具调用、任务计划、问答记录。云端对话（手机内核）和设备聊天（遥控电脑）共用。
// 规则和桌面版保持一致：桌面 `src/ui/chat/task-progress.ts`、`view.ts`、`format.ts`。

@Serializable
enum class ToolStatus { Running, Done, Error }

@Serializable
/** 一次工具调用。`arguments` 是模型给的原始 JSON 文本，`output` 是工具返回的文字。 */
data class ToolCallInfo(
    val id: String,
    val name: String,
    val arguments: String = "",
    val output: String = "",
    val status: ToolStatus = ToolStatus.Running,
    val startedAtMs: Long = 0,
    val endedAtMs: Long = 0,
) {
    val durationMs: Long get() = if (startedAtMs > 0 && endedAtMs >= startedAtMs) endedAtMs - startedAtMs else 0
}

@Serializable
/** 对话里留下的一行痕迹：回答过的问题、处理过的审批。 */
data class ChatRecord(
    /** `answer`（回答过的问题）或 `approval`（处理过的审批）。 */
    val kind: String,
    val title: String,
    /** 回答的内容；`null` 表示跳过或拒绝。 */
    val result: String?,
)

private val ToolLabels = mapOf(
    "shell" to "运行命令", "bash" to "运行命令", "read_file" to "读取文件", "write_file" to "写入文件", "edit_file" to "修改文件",
    "list_dir" to "查看文件夹", "grep" to "搜索文件", "web_fetch" to "打开网页", "web_search" to "搜索网页", "todo" to "任务清单",
    "ask_user" to "向你提问", "time_now" to "查看时间", "memory_read" to "读取笔记", "memory_write" to "保存笔记", "load_skill" to "读取技能",
    "set_reminder" to "设置提醒", "care_set" to "设置提醒", "care_list" to "查看提醒", "care_delete" to "删除提醒",
    "wallpaper_list" to "查看壁纸", "wallpaper_apply" to "更换壁纸", "wallpaper_create" to "创作壁纸",
    "effect_list" to "查看特效", "effect_set" to "切换特效", "effect_create" to "创作特效",
    "widget_list" to "查看桌面组件", "widget_add" to "添加桌面组件", "widget_create" to "创作桌面组件",
    "game_list" to "查看游戏列表", "game_open" to "打开游戏", "game_create" to "创作游戏", "game_close" to "退出游戏",
    "process_start" to "启动后台命令", "process_read" to "查看后台输出", "process_stop" to "停止后台命令",
    "connector_search" to "查找连接器", "connector_request" to "连接服务", "connector_call" to "调用连接器工具",
    "research" to "调研子任务", "community_share" to "分享到论坛", "community_open" to "打开官网或论坛",
)

/** 工具的中文名；不认识的原样显示。 */
fun toolLabel(name: String): String = ToolLabels[name] ?: name.ifBlank { "工具" }

/** 卡片上一行的简短说明：命令、路径、网址这类最能说明“在做什么”的参数。 */
fun describeCall(name: String, argumentsJson: String): String {
    val obj = runCatching { CommunityJson.parseToJsonElement(argumentsJson) as? JsonObject }.getOrNull() ?: return ""
    fun pick(vararg keys: String): String? = keys.firstNotNullOfOrNull { key ->
        (obj[key] as? JsonPrimitive)?.contentOrNull?.trim()?.takeIf { it.isNotEmpty() && it != "null" }
    }
    val text = when (name) {
        "shell", "bash" -> pick("command")
        "set_reminder" -> listOfNotNull(pick("title"), pick("minutes")?.let { "$it 分钟后" }).joinToString(" · ").ifBlank { null }
        else -> pick("command", "path", "url", "query", "pattern", "kind", "title", "name", "id", "text", "question", "expression", "gameId")
    }
    val line = text.orEmpty().lineSequence().firstOrNull().orEmpty().trim()
    return if (line.length > 90) line.take(90) + "…" else line
}

// ───────────── 任务计划（todo 工具） ─────────────

data class TaskStep(val content: String, val status: String, val evidence: String = "") {
    val done: Boolean get() = status == "completed"
}

data class TaskPlan(val steps: List<TaskStep>, val handoff: String = "") {
    val completed: Int get() = steps.count { it.done }
    /** 正在做的那一步；没有就是第一个没做的；全做完了是 -1。 */
    val current: Int get() = steps.indexOfFirst { it.status == "in_progress" }.takeIf { it >= 0 } ?: steps.indexOfFirst { !it.done }
}

private const val TodoOk = "Todo list updated:"

/** 只认已经成功的 `todo` 调用（和桌面一样）：半截的、被拒绝的更新不算计划。 */
fun taskPlan(calls: List<ToolCallInfo>): TaskPlan? {
    for (call in calls.asReversed()) {
        if (call.name != "todo" || call.status != ToolStatus.Done || !call.output.startsWith(TodoOk)) continue
        val args = runCatching { CommunityJson.parseToJsonElement(call.arguments) as? JsonObject }.getOrNull() ?: continue
        val todos = args["todos"] as? JsonArray ?: continue
        if (todos.size > 50) continue
        val steps = mutableListOf<TaskStep>()
        var valid = true
        for (item in todos) {
            val o = item as? JsonObject
            val content = (o?.get("content") as? JsonPrimitive)?.contentOrNull?.trim()
            val status = (o?.get("status") as? JsonPrimitive)?.contentOrNull
            if (content.isNullOrEmpty() || content.length > 300 || status !in setOf("pending", "in_progress", "completed")) { valid = false; break }
            steps += TaskStep(content, status!!, (o["evidence"] as? JsonPrimitive)?.contentOrNull.orEmpty())
        }
        if (!valid || steps.count { it.status == "in_progress" } > 1) continue
        val handoff = (args["handoff"] as? JsonPrimitive)?.contentOrNull.orEmpty()
        return if (steps.isEmpty()) null else TaskPlan(steps, handoff)
    }
    return null
}

enum class TaskState { Running, Waiting, Approval, Blocked, Paused, Completed }

data class TaskProgress(val plan: TaskPlan, val state: TaskState, val message: String = "") {
    val completed: Int get() = plan.completed
    val total: Int get() = plan.steps.size
    val current: Int get() = plan.current
}

private fun operationKey(call: ToolCallInfo): String = "${call.name}:${runCatching { CommunityJson.parseToJsonElement(call.arguments).toString() }.getOrDefault(call.arguments)}"

/** 失败了、后来又没有用同样的调用成功的那些调用（`todo` 不算）。 */
fun unresolvedCalls(calls: List<ToolCallInfo>): List<ToolCallInfo> {
    val open = LinkedHashMap<String, ToolCallInfo>()
    for (call in calls) {
        if (call.name == "todo") continue
        when (call.status) {
            ToolStatus.Error -> open[operationKey(call)] = call
            ToolStatus.Done -> open.remove(operationKey(call))
            ToolStatus.Running -> {}
        }
    }
    return open.values.toList()
}

/**
 * 这一轮的任务进度；没有计划就是 `null`。
 * [running] 这一轮是否还在跑，[waiting] 在等用户：`question`（回答问题）或 `approval`（授权），[stopped] 是不是被用户停掉的。
 */
fun taskProgress(calls: List<ToolCallInfo>, running: Boolean, waiting: String? = null, stopped: Boolean = false, error: String? = null): TaskProgress? {
    val plan = taskPlan(calls) ?: return null
    if (plan.completed == plan.steps.size) return TaskProgress(plan, TaskState.Completed)
    val unresolved = unresolvedCalls(calls)
    return when {
        running && waiting == "approval" -> TaskProgress(plan, TaskState.Approval)
        running && waiting == "question" -> TaskProgress(plan, TaskState.Waiting)
        running -> TaskProgress(plan, TaskState.Running)
        error != null -> TaskProgress(plan, TaskState.Blocked, error)
        unresolved.isNotEmpty() -> TaskProgress(plan, TaskState.Blocked, unresolved.last().output.lineSequence().firstOrNull().orEmpty().take(120))
        stopped -> TaskProgress(plan, TaskState.Paused)
        else -> TaskProgress(plan, TaskState.Paused)
    }
}

/** “本轮执行过程”摘要用：几步、几次异常、总用时（拿不到时间就是 0）。 */
data class ActivitySummary(val steps: Int, val errors: Int, val durationMs: Long)

fun activitySummary(calls: List<ToolCallInfo>): ActivitySummary {
    val real = calls.filter { it.name != "todo" }
    val started = real.map { it.startedAtMs }.filter { it > 0 }.minOrNull()
    val ended = real.map { it.endedAtMs }.filter { it > 0 }.maxOrNull()
    return ActivitySummary(real.size, real.count { it.status == ToolStatus.Error }, if (started != null && ended != null && ended >= started) ended - started else 0)
}

/** 12 秒、1 分 05 秒。 */
fun formatDuration(ms: Long): String {
    val seconds = (ms / 1000).coerceAtLeast(0)
    return if (seconds < 60) "${seconds} 秒" else "${seconds / 60} 分 ${(seconds % 60).toString().padStart(2, '0')} 秒"
}
