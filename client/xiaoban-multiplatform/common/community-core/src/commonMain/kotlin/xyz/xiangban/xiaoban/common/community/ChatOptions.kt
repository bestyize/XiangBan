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

/** A conversation owns this snapshot. Gateway metadata and credentials are not saved here. */
@Serializable
data class ChatOptions(
    val tier: String = "",
    val reasoningByTier: Map<String, String> = emptyMap(),
    val contextTokens: Int = 256_000,
    val permissionMode: String = "ask",
    val grants: List<String> = emptyList(),
    val budget: AgentExecutionBudget = AgentExecutionBudget(),
) {
    fun validate() {
        require(contextTokens in 16_000..1_000_000) { "上下文大小须为 16000 到 1000000" }
        require(permissionMode in setOf("ask", "auto")) { "权限模式无效" }
        budget.validate()
    }
}
