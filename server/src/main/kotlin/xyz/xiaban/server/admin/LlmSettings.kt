package xyz.xiaban.server.admin

import io.ktor.client.HttpClient
import io.ktor.http.HttpStatusCode
import kotlinx.serialization.Serializable
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.config.LlmEndpoint
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.llm.LlmProvider
import xyz.xiaban.server.llm.OpenAiCompatibleProvider
import java.security.MessageDigest
import java.security.SecureRandom
import java.time.Duration
import java.util.Base64
import javax.crypto.Cipher
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec

@Serializable
data class LlmProviderConfig(
    val id: String,
    val name: String,
    val baseUrl: String,
    val apiKey: String = "",
    val model: String,
    val apiKeyConfigured: Boolean = false,
) {
    fun configured() = baseUrl.isNotBlank() && model.isNotBlank() && apiKey.isNotBlank()
    fun redacted() = copy(apiKey = "", apiKeyConfigured = apiKey.isNotBlank())
}

@Serializable
data class LlmSettings(
    val enabled: Boolean = true,
    val activeProviderId: String = "",
    val providers: List<LlmProviderConfig> = emptyList(),
    val webSearchEnabled: Boolean = false,
    val webSearchBaseUrl: String = "",
) {
    fun redacted() = copy(providers = providers.map { it.redacted() })
}

@Serializable
data class SearchTestRequest(val baseUrl: String)

@Serializable
data class SearchTestResponse(val success: Boolean, val count: Int, val sampleTitle: String)

@Serializable
data class LlmTestRequest(
    val id: String? = null,
    val baseUrl: String,
    val apiKey: String = "",
    val model: String,
)

@Serializable
data class LlmTestResponse(val reply: String)

class LlmSettingsStore(
    private val db: Db,
    private val config: AppConfig,
    private val http: HttpClient,
    private val defaultProviders: List<LlmProvider>,
) {
    private val key = SecretKeySpec(
        MessageDigest.getInstance("SHA-256").digest(
            ("xiaban-settings:" + (config.settingsEncryptionKey ?: config.jwtSecret)).toByteArray()
        ),
        "AES"
    )
    private val random = SecureRandom()

    suspend fun current(): LlmSettings {
        val saved = db.read { c ->
            c.queryOne("select encrypted_value from runtime_settings where key = 'llm'") { it.getString(1) }
        }
        if (saved != null) {
            return runCatching {
                AppJson.decodeFromString(LlmSettings.serializer(), decrypt(saved))
            }.getOrElse { LlmSettings() }
        }
        // Fallback to environment variables if present
        val envProviders = config.llm.endpoints.map {
            LlmProviderConfig(
                id = it.name.lowercase(),
                name = it.name,
                baseUrl = it.baseUrl,
                apiKey = it.apiKey,
                model = it.model,
                apiKeyConfigured = it.apiKey.isNotBlank(),
            )
        }
        return LlmSettings(
            enabled = config.llm.enabledByDefault,
            activeProviderId = envProviders.firstOrNull()?.id.orEmpty(),
            providers = envProviders,
        )
    }

    suspend fun save(request: LlmSettings, actor: String? = null): LlmSettings {
        val previous = current()
        val prevMap = previous.providers.associateBy { it.id }

        val sanitizedProviders = request.providers.map { p ->
            val prev = prevMap[p.id]
            val key = p.apiKey.ifBlank { prev?.apiKey.orEmpty() }
            p.copy(
                id = p.id.trim(),
                name = p.name.trim(),
                baseUrl = p.baseUrl.trim().trimEnd('/'),
                model = p.model.trim(),
                apiKey = key,
                apiKeyConfigured = key.isNotBlank(),
            )
        }

        for (p in sanitizedProviders) {
            if (p.name.isBlank()) badRequest("供应商名称不能为空")
            if (p.baseUrl.isBlank() || !p.baseUrl.startsWith("http")) badRequest("接口地址必须是以 http 开头的完整 URL")
            if (p.model.isBlank()) badRequest("模型名称不能为空")
        }

        val next = request.copy(
            activeProviderId = request.activeProviderId.trim(),
            providers = sanitizedProviders,
            webSearchEnabled = request.webSearchEnabled,
            webSearchBaseUrl = request.webSearchBaseUrl.trim().trimEnd('/'),
        )

        val encrypted = encrypt(AppJson.encodeToString(LlmSettings.serializer(), next))
        db.tx { c ->
            c.execute(
                "insert into runtime_settings(key, encrypted_value) values ('llm', ?) on conflict(key) do update set encrypted_value = excluded.encrypted_value, updated_at = now()",
                encrypted
            )
            if (actor != null) {
                c.execute(
                    "insert into admin_audit_log(actor,action,summary) values (?,?,?)",
                    actor,
                    "llm_settings",
                    if (next.enabled) "更新大模型供应商配置（启用，当前: ${next.activeProviderId}）" else "关闭大模型服务"
                )
            }
        }

        return next.redacted()
    }

    suspend fun test(request: LlmTestRequest): String {
        val currentSettings = current()
        val existingApiKey = request.id?.let { id ->
            currentSettings.providers.find { it.id == id }?.apiKey
        }
        val key = request.apiKey.ifBlank { existingApiKey.orEmpty() }
        if (key.isBlank()) {
            badRequest("请填写或先保存 API Key 后再测试")
        }
        val provider = OpenAiCompatibleProvider(
            LlmEndpoint(
                name = "test",
                baseUrl = request.baseUrl.trim().trimEnd('/'),
                apiKey = key,
                model = request.model.trim()
            ),
            http
        )
        return try {
            provider.complete(
                system = "你是小伴，一只猫。",
                user = "测试连通性，请用一句话回复「小伴收到啦」。",
                timeout = Duration.ofSeconds(12)
            )
        } catch (e: Exception) {
            throw ApiException(HttpStatusCode.ServiceUnavailable, "llm_test_failed", "大模型接口测试失败: ${e.message}")
        }
    }

    suspend fun activeProvider(): LlmProvider? {
        val settings = current()
        val target = settings.providers.find { it.id == settings.activeProviderId }
            ?: settings.providers.firstOrNull()
        if (target != null && target.configured()) {
            if (!settings.enabled) return null
            return OpenAiCompatibleProvider(
                LlmEndpoint(target.name, target.baseUrl, target.apiKey, target.model),
                http
            )
        }
        return defaultProviders.firstOrNull()
    }

    private fun encrypt(value: String): String {
        val iv = ByteArray(12).also(random::nextBytes)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key, GCMParameterSpec(128, iv))
        cipher.updateAAD("llm".toByteArray())
        return Base64.getEncoder().encodeToString(iv + cipher.doFinal(value.toByteArray(Charsets.UTF_8)))
    }

    private fun decrypt(value: String): String {
        val bytes = Base64.getDecoder().decode(value)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(128, bytes.copyOfRange(0, 12)))
        cipher.updateAAD("llm".toByteArray())
        return String(cipher.doFinal(bytes.copyOfRange(12, bytes.size)), Charsets.UTF_8)
    }
}
