package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.Serializable

@Serializable
data class AgentExecutionBudget(
    val maxTurns: Int? = null,
    val maxToolCalls: Int? = null,
    val maxMinutes: Long? = null,
    val maxTokens: Long? = null,
) {
    fun validate() {
        require(maxTurns == null || maxTurns in 1..1_000_000) { "轮数预算须为 1 到 1000000，留空表示不限制" }
        require(maxToolCalls == null || maxToolCalls in 1..10_000_000) { "工具次数预算须为正数，留空表示不限制" }
        require(maxMinutes == null || maxMinutes in 1..43_200) { "时间预算须为 1 到 43200 分钟，留空表示不限制" }
        require(maxTokens == null || maxTokens in 1..1_000_000_000) { "模型用量预算须为正数，留空表示不限制" }
    }
}

@Serializable
data class CreationDraft(val title: String, val request: String, val approved: Boolean = false)

fun creationTitle(brief: String, kind: String): String =
    Regex("《([^》]+)》").find(brief)?.groupValues?.get(1)?.take(24)
        ?: "$kind · ${brief.trim().lineSequence().first().take(16)}"

/** A conversation owns this snapshot. Gateway metadata and credentials are not saved here. */
@Serializable
data class ChatOptions(
    val tier: String = "",
    val reasoningByTier: Map<String, String> = emptyMap(),
    val contextTokens: Int = 256_000,
    val permissionMode: String = "ask",
    val grants: List<String> = emptyList(),
    val budget: AgentExecutionBudget = AgentExecutionBudget(),
    val creation: CreationDraft? = null,
) {
    fun validate() {
        require(contextTokens in 16_000..1_000_000) { "上下文大小须为 16000 到 1000000" }
        require(permissionMode in setOf("ask", "auto")) { "权限模式无效" }
        budget.validate()
    }
}

private fun legacyCreationRequest(messages: List<ChatTurn>): CreationDraft? {
    val request = messages.firstOrNull { it.role == "user" }?.text ?: return null
    val kind = when {
        request.startsWith("帮我设计并制作一个支持手机触控和电脑键鼠的") -> if ("kind=app" in request) "小程序" else "游戏"
        request.startsWith("请参考《") && "app_read_format" in request -> "改编作品"
        request.startsWith("请先用 app_read_format") && "问我想怎样修改" in request -> "修改作品"
        else -> return null
    }
    val brief = request.substringAfter("我的想法：", request)
    return CreationDraft(creationTitle(brief, kind), request)
}

internal fun legacyCreationTitle(messages: List<ChatTurn>): String? = legacyCreationRequest(messages)?.title

/** Old entries with previous tool effects must continue as existing tasks. */
internal fun legacyCreation(messages: List<ChatTurn>): CreationDraft? {
    val draft = legacyCreationRequest(messages) ?: return null
    val effects = setOf("write_file", "edit_file", "copy_file", "delete_file", "app_create", "app_open", "app_export")
    return draft.takeUnless { messages.any { turn ->
        turn.tools.any { it.name in effects } || turn.records.any { it.kind == "approval" }
    } }
}
