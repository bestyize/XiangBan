package xyz.xiangban.xiaoban.common.community

import io.ktor.http.Url
import kotlinx.serialization.Serializable
import kotlin.io.encoding.Base64
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

const val CUSTOM_MODEL_TIER = "__xiaoban_custom_model__"
private const val CUSTOM_MODEL_PREFIX = "$CUSTOM_MODEL_TIER:"
val DEFAULT_CUSTOM_REASONING_EFFORTS = listOf("low", "medium", "high", "xhigh", "max")
fun isCustomModelTier(tier: String) = tier == CUSTOM_MODEL_TIER || tier.startsWith(CUSTOM_MODEL_PREFIX)
fun customModelTier(providerId: String, model: String): String = "$CUSTOM_MODEL_PREFIX$providerId:${Base64.UrlSafe.encode(model.encodeToByteArray()).trimEnd('=')}"
@OptIn(ExperimentalUuidApi::class)
fun newCustomProviderId(): String = "p-${Uuid.random()}"

/** A device-local provider; credentials are never stored in chat history. */
@Serializable
data class CustomModelConfig(
    val baseUrl: String, val apiKey: String, val model: String,
    val protocol: String = "chat", val contextTokens: Int? = null, val maxOutputTokens: Int? = null,
    val supportsTools: Boolean = true, val supportsVision: Boolean = true,
    val authMode: String = "api-key",
    val id: String = "legacy", val name: String = "", val models: List<String> = emptyList(),
    val legacyModel: String = "",
    val remoteId: String? = null,
    val remoteType: String = "",
    val remoteAvailable: Boolean = true,
    val remoteOwner: String? = null,
    val recommendedModels: List<String> = emptyList(),
    val recommendedDefault: String = "",
    val followRecommended: Boolean = false,
    val consoleUrl: String = "",
    val consoleText: String = "",
    val endpoints: Map<String, String> = emptyMap(),
    val reasoningEfforts: List<String>? = null,
) {
    /** 自定义/第三方服务默认支持五档思考等级；显式存空列表时表示不支持。 */
    fun effectiveReasoningEfforts(): List<String> {
        val raw = if (remoteType in setOf("free", "paid")) reasoningEfforts.orEmpty()
        else (reasoningEfforts ?: DEFAULT_CUSTOM_REASONING_EFFORTS)
        return DEFAULT_CUSTOM_REASONING_EFFORTS.filter { it in raw }
    }
    fun validated(): CustomModelConfig {
        val address = baseUrl.trim().trimEnd('/')
        require(address.length <= 2000) { "API 地址过长" }
        val url = runCatching { Url(address) }.getOrNull()
        require(url != null && url.protocol.name in setOf("http", "https") && url.host.isNotBlank() &&
            url.user == null && url.password == null && url.parameters.isEmpty() && url.fragment.isEmpty()) { "请输入不含账号、查询参数或片段的 HTTP 或 HTTPS API 地址" }
        require(model.isNotBlank() && model.trim().length <= 200) { "请填写模型名" }
        require(id.matches(Regex("[a-zA-Z0-9_-]{1,128}"))) { "模型服务标识无效" }
        require(name.trim().length <= 200) { "服务商名称不能超过 200 个字" }
        val cleanModels = (if (models.isEmpty()) listOf(model) else models).map { it.trim() }.distinct()
        require(cleanModels.size in 1..200 && cleanModels.all { it.isNotEmpty() && it.length <= 200 }) { "请填写 1–200 个有效的模型名称" }
        require(model.trim() in cleanModels) { "默认模型必须在模型列表中" }
        require(apiKey.length <= 4096 && !apiKey.any { it.isWhitespace() }) { "密钥不能包含空格或换行" }
        require(protocol in setOf("chat", "responses", "anthropic")) { "请选择有效的接口协议" }
        require(authMode in setOf("api-key", "bearer")) { "请选择有效的认证方式" }
        require(remoteId == null || remoteId.matches(Regex("[a-zA-Z0-9-]{1,80}")) && remoteType in setOf("third", "free", "paid")) { "远端服务商标识无效" }
        require(contextTokens == null || contextTokens in 16_000..1_000_000) { "上下文窗口需在 16000–1000000 之间" }
        require(maxOutputTokens == null || maxOutputTokens in 1..1_000_000) { "单次输出上限需在 1–1000000 之间" }
        require(contextTokens == null || maxOutputTokens == null || maxOutputTokens < contextTokens) { "单次输出上限必须小于上下文窗口" }
        val cleanEfforts = reasoningEfforts?.let { list -> DEFAULT_CUSTOM_REASONING_EFFORTS.filter { it in list } }
        return copy(baseUrl = address, apiKey = apiKey.trim(), model = model.trim(),
            name = name.trim().ifEmpty { url.host }, models = cleanModels,
            legacyModel = if (id == "legacy" && legacyModel.isEmpty()) model.trim() else legacyModel,
            reasoningEfforts = cleanEfforts)
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

@Serializable
data class CustomModelCatalog(
    val providers: List<CustomModelConfig> = emptyList(), val preferredTier: String? = null,
    val presets: List<RemoteModelProvider> = emptyList(), val catalogueReceived: Boolean = false,
    val refreshAfterSeconds: Long = 21600,
    val officialAiEnabled: Boolean = true,
    val preferredOfficialTier: String? = null,
    val officialReasoningByTier: Map<String, String> = emptyMap(),
)

/** Safe for UI state: deliberately excludes API keys. */
data class CustomProviderView(
    val id: String,
    val name: String,
    val models: List<String>,
    val defaultModel: String,
    val available: Boolean = true,
    val remoteType: String = "",
    val reasoningEfforts: List<String> = emptyList(),
)

interface CustomModelStorage {
    fun load(): CustomModelConfig?
    fun save(config: CustomModelConfig?)
    fun loadCatalog(): CustomModelCatalog = CustomModelCatalog(listOfNotNull(load()))
    fun saveCatalog(catalog: CustomModelCatalog) { require(catalog.providers.size <= 1); save(catalog.providers.firstOrNull()) }
}

class InMemoryCustomModelStorage : CustomModelStorage {
    private var catalog = CustomModelCatalog()
    override fun load() = catalog.providers.firstOrNull()
    override fun save(config: CustomModelConfig?) { catalog = CustomModelCatalog(listOfNotNull(config)) }
    override fun loadCatalog() = catalog
    override fun saveCatalog(catalog: CustomModelCatalog) { this.catalog = catalog }
}

/** Read the original single-provider preference until a catalog is first saved. */
class PreferencesCustomModelStorage(private val get: (String) -> String?, private val put: (String, String) -> Unit) : CustomModelStorage {
    private val oldKey = "custom_model_config"
    private val catalogKey = "custom_model_catalog_v2"
    override fun load(): CustomModelConfig? = get(oldKey)?.takeIf { it.isNotBlank() }?.let {
        try { CommunityJson.decodeFromString(CustomModelConfig.serializer(), it) }
        catch (_: Exception) { throw IllegalStateException("旧模型服务配置无法读取，请检查本机配置") }
    }
    override fun save(config: CustomModelConfig?) {
        put(oldKey, config?.let { CommunityJson.encodeToString(CustomModelConfig.serializer(), it) }.orEmpty())
    }
    override fun loadCatalog(): CustomModelCatalog {
        val stored = get(catalogKey) ?: return CustomModelCatalog(listOfNotNull(load()))
        return try { CommunityJson.decodeFromString(CustomModelCatalog.serializer(), stored) }
        catch (_: Exception) { throw IllegalStateException("模型服务配置无法读取，请检查本机配置") }
    }
    override fun saveCatalog(catalog: CustomModelCatalog) {
        put(catalogKey, CommunityJson.encodeToString(CustomModelCatalog.serializer(), catalog))
    }
}
