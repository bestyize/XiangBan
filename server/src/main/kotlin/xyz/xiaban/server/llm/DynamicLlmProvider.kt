package xyz.xiaban.server.llm

import java.time.Duration

/** Proxies LLM calls to the currently active provider configured dynamically in the admin panel. */
class DynamicLlmProvider(
    private val supplier: suspend () -> LlmProvider?,
) : LlmProvider {
    override val name: String get() = "dynamic"

    override suspend fun complete(system: String, user: String, timeout: Duration): String {
        val active = supplier() ?: error("当前未启用或未配置可用的大模型供应商")
        return active.complete(system, user, timeout)
    }

    override suspend fun stream(
        messages: List<Pair<String, String>>,
        timeout: Duration,
        maxTokens: Int,
        temperature: Double,
        onChunk: suspend (String) -> Unit,
    ): String {
        val active = supplier() ?: error("当前未启用或未配置可用的大模型供应商")
        return active.stream(messages, timeout, maxTokens, temperature, onChunk)
    }
}
