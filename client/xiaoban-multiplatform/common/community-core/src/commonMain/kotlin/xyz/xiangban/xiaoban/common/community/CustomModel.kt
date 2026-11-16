package xyz.xiangban.xiaoban.common.community

import io.ktor.http.Url
import kotlinx.serialization.Serializable

const val CUSTOM_MODEL_TIER = "__xiaoban_custom_model__"

/** A single device-local provider supporting the three model wire protocols. */
@Serializable
data class CustomModelConfig(
    val baseUrl: String, val apiKey: String, val model: String,
    val protocol: String = "chat", val contextTokens: Int? = null, val maxOutputTokens: Int? = null,
    val supportsTools: Boolean = true, val supportsVision: Boolean = true,
    val authMode: String = "api-key",
) {
    fun validated(): CustomModelConfig {
        val address = baseUrl.trim().trimEnd('/')
        require(address.length <= 2000) { "API 地址过长" }
        val url = runCatching { Url(address) }.getOrNull()
        require(url != null && url.protocol.name in setOf("http", "https") && url.host.isNotBlank() &&
            url.user == null && url.password == null && url.parameters.isEmpty() && url.fragment.isEmpty()) { "请输入不含账号、查询参数或片段的 HTTP 或 HTTPS API 地址" }
        require(model.isNotBlank() && model.trim().length <= 200) { "请填写模型名" }
        require(apiKey.length <= 4096 && !apiKey.any { it.isWhitespace() }) { "密钥不能包含空格或换行" }
        require(protocol in setOf("chat", "responses", "anthropic")) { "请选择有效的接口协议" }
        require(authMode in setOf("api-key", "bearer")) { "请选择有效的认证方式" }
        require(contextTokens == null || contextTokens in 16_000..1_000_000) { "上下文窗口需在 16000–1000000 之间" }
        require(maxOutputTokens == null || maxOutputTokens in 1..1_000_000) { "单次输出上限需在 1–1000000 之间" }
        require(contextTokens == null || maxOutputTokens == null || maxOutputTokens < contextTokens) { "单次输出上限必须小于上下文窗口" }
        return copy(baseUrl = address, apiKey = apiKey.trim(), model = model.trim())
    }

    fun chatUrl(): String {
        val suffix = when (protocol) { "responses" -> "/responses"; "anthropic" -> "/messages"; else -> "/chat/completions" }
        return when {
            baseUrl.endsWith(suffix) -> baseUrl
            baseUrl.endsWith("/v1") -> "$baseUrl$suffix"
            protocol == "anthropic" || Url(baseUrl).encodedPath.trim('/').isEmpty() -> "$baseUrl/v1$suffix"
            else -> "$baseUrl$suffix"
        }
    }

    fun modelsUrl(): String {
        val suffix = when (protocol) { "responses" -> "/responses"; "anthropic" -> "/messages"; else -> "/chat/completions" }
        return chatUrl().removeSuffix(suffix) + "/models"
    }
}

interface CustomModelStorage {
    fun load(): CustomModelConfig?
    fun save(config: CustomModelConfig?)
}

class InMemoryCustomModelStorage : CustomModelStorage {
    private var value: CustomModelConfig? = null
    override fun load() = value
    override fun save(config: CustomModelConfig?) { value = config }
}
