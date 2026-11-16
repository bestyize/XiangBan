package xyz.xiangban.xiaoban.common.community

import io.ktor.http.Url
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlin.io.encoding.Base64

@Serializable
data class RemoteModel(val id: String, val name: String, val description: String = "", val reasoningEfforts: List<String>? = null)

@Serializable
data class RemoteModelProvider(
    val id: String, val provider: String, val description: String = "", val endPoint: String,
    val protocol: String, @SerialName("type") val type: String, val defaultModel: String,
    val models: List<RemoteModel>, val endpoints: Map<String, String> = emptyMap(),
    val consoleUrl: String = "", val consoleText: String = "", val apikey: String = "",
)

@Serializable
data class RemoteOfficialAi(val enabled: Boolean = true)

@Serializable
data class RemoteModelCatalogue(val version: Int, val refreshAfterSeconds: Long = 21600,
    val providers: List<RemoteModelProvider> = emptyList(), val xiaobanAi: RemoteOfficialAi = RemoteOfficialAi())

data class RemotePresetView(val id: String, val name: String, val description: String, val type: String, val added: Boolean)

fun remoteProviderId(id: String): String = "s-" + Base64.UrlSafe.encode(id.encodeToByteArray()).trimEnd('=')

fun remoteProtocol(protocol: String): String = when (protocol) {
    "OpenAI Chat Completions" -> "chat"
    "OpenAI Responses" -> "responses"
    "Anthropic Messages" -> "anthropic"
    else -> throw IllegalArgumentException("供应商使用了不支持的协议")
}

private fun safeRemoteUrl(raw: String): Boolean = runCatching {
    val url = Url(raw)
    url.protocol.name == "https" && url.host.isNotBlank() && url.user == null && url.password == null && url.parameters.isEmpty() && url.fragment.isEmpty()
}.getOrDefault(false)

fun RemoteModelCatalogue.validated(): RemoteModelCatalogue {
    require(version == 1 && refreshAfterSeconds in 300..86400 && providers.size <= 50) { "模型目录版本或刷新周期无效" }
    require(providers.map { it.id }.distinct().size == providers.size) { "模型目录包含重复供应商" }
    providers.forEach { p ->
        require(p.id.matches(Regex("[a-zA-Z0-9-]{1,80}")) && p.id != "custom" && p.provider.isNotBlank() && p.provider.length <= 200) { "模型目录包含无效供应商" }
        require(p.type in setOf("third", "free", "paid") && safeRemoteUrl(p.endPoint)) { "模型目录包含无效地址或类型" }
        remoteProtocol(p.protocol)
        require(p.endpoints.all { (key, url) -> runCatching { remoteProtocol(key); safeRemoteUrl(url) }.getOrDefault(false) }) { "模型目录包含无效协议地址" }
        require(p.consoleUrl.isEmpty() || safeRemoteUrl(p.consoleUrl)) { "模型目录包含无效控制台地址" }
        require(p.apikey.length <= 4096 && p.apikey.none { it.isWhitespace() } && (p.type != "third" || p.apikey.isEmpty()) && (p.type != "paid" || p.apikey.isNotEmpty())) { "模型目录包含无效凭证" }
        require(p.models.size in 1..200 && p.models.all { it.id.isNotBlank() && it.id == it.id.trim() && it.id.length <= 200 } && p.models.map { it.id }.distinct().size == p.models.size && p.models.any { it.id == p.defaultModel }) { "模型目录包含无效模型" }
    }
    return this
}

/** A downlink never exposes its credentials to UI state or the persisted public preset cache. */
fun RemoteModelProvider.publicPreset(): RemoteModelProvider = copy(apikey = "")

private fun RemoteModelProvider.protocolEndpoints(): Map<String, String> =
    endpoints.mapKeys { remoteProtocol(it.key) } + (remoteProtocol(protocol) to (endpoints[protocol] ?: endPoint))

fun RemoteModelProvider.toConfig(owner: String?): CustomModelConfig = CustomModelConfig(
    baseUrl = endPoint, apiKey = if (type == "third") "" else apikey, model = defaultModel,
    protocol = remoteProtocol(protocol), id = remoteProviderId(id), name = provider,
    models = models.map { it.id }, remoteId = id, remoteType = type, remoteOwner = if (apikey.isNotBlank()) owner else null,
    recommendedModels = models.map { it.id }, recommendedDefault = defaultModel, followRecommended = true,
    consoleUrl = consoleUrl, consoleText = consoleText, endpoints = protocolEndpoints(),
).validated()

/** Merge server changes without overwriting a user's third-party key, proxy, or manually added models. */
fun mergeRemoteProvider(local: CustomModelConfig, remote: RemoteModelProvider, owner: String?): CustomModelConfig {
    require(local.remoteId == remote.id) { "供应商不匹配" }
    val managed = remote.type != "third"
    val offered = remote.models.map { it.id }
    val extra = if (managed || local.remoteType != "third") emptyList() else local.models
    val models = (offered + extra).distinct()
    val defaultModel = if (managed || local.followRecommended || local.model !in models) remote.defaultModel else local.model
    val useRemoteEndpoint = managed || local.apiKey.isBlank() || local.remoteType != "third"
    return local.copy(
        name = remote.provider, baseUrl = if (useRemoteEndpoint) remote.endPoint else local.baseUrl,
        protocol = if (useRemoteEndpoint) remoteProtocol(remote.protocol) else local.protocol,
        apiKey = if (managed) remote.apikey else if (local.remoteType == "third") local.apiKey else "",
        model = defaultModel, models = models, remoteType = remote.type, remoteAvailable = true,
        remoteOwner = if (managed && remote.apikey.isNotEmpty()) owner else null,
        recommendedModels = offered, recommendedDefault = remote.defaultModel,
        consoleUrl = remote.consoleUrl, consoleText = remote.consoleText,
        endpoints = remote.protocolEndpoints(),
    ).validated()
}
