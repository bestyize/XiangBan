package xyz.xiaban.server.config

import xyz.xiaban.server.mail.SmtpConfig

data class DbConfig(val url: String, val user: String, val password: String, val poolSize: Int)

/** An OpenAI-compatible chat endpoint, e.g. 豆包 (Volcengine Ark) or 通义千问 (DashScope compatible mode). */
data class LlmEndpoint(val name: String, val baseUrl: String, val apiKey: String, val model: String)

data class LlmConfig(
    /** Initial state of the global switch; the admin API overrides it at runtime. */
    val enabledByDefault: Boolean,
    val endpoints: List<LlmEndpoint>,
    val dailyLimitPerUser: Int,
    val timeoutMs: Long,
)

data class AppConfig(
    val env: String,
    val port: Int,
    val publicBaseUrl: String,
    val trustProxy: Boolean,
    val db: DbConfig,
    val redisUrl: String,
    val jwtSecret: String,
    val llm: LlmConfig,
    /** Enables /api/admin when set. */
    val adminToken: String?,
    /** 和风天气 API Host and key; weather tips are skipped when either is missing. */
    val qweatherHost: String?,
    val qweatherKey: String?,
    /** Average daily sample needed before showing a city-level or site-wide percentile (PRD 5.6). */
    val percentileCityMin: Int,
    val percentileGlobalMin: Int,
    /** Outgoing mail; without it mail is only logged, which is not acceptable in production. */
    val smtp: SmtpConfig?,
    /** 带小伴回家 default (PRD 8.4, phase three) and where the button sends people */
    val goHomeEnabled: Boolean,
    val appDownloadUrl: String?,
    val adminUsername: String = "admin",
    val adminBootstrapPassword: String? = null,
    val settingsEncryptionKey: String? = null,
    /** Unified account center: token issuer (`iss`), base URL for JWKS and /api/me, accepted `aud` values. */
    val accountIssuer: String = "https://account.xiangban.xyz",
    val accountBaseUrl: String = accountIssuer,
    val accountAudiences: Set<String> = setOf("xiaban-web", "xiaban-api"),
) {
    val isProd: Boolean get() = env == "prod"

    companion object {
        fun fromEnv(source: Map<String, String> = System.getenv()): AppConfig {
            val env = EnvReader(source)
            val name = env.str("XIABAN_ENV", "dev")
            return AppConfig(
                env = name,
                port = env.int("PORT", 8080),
                publicBaseUrl = env.str("PUBLIC_BASE_URL", "http://localhost:5173").trimEnd('/'),
                trustProxy = env.bool("TRUST_PROXY", false),
                db = DbConfig(
                    url = env.str("DATABASE_URL", "jdbc:postgresql://localhost:5432/xiaban"),
                    user = env.str("DATABASE_USER", "xiaban"),
                    password = env.str("DATABASE_PASSWORD", "xiaban"),
                    poolSize = env.int("DATABASE_POOL_SIZE", 10),
                ),
                redisUrl = env.str("REDIS_URL", "redis://localhost:6379/0"),
                jwtSecret = env.optional("JWT_SECRET") ?: run {
                    check(name != "prod") { "JWT_SECRET is required when XIABAN_ENV=prod" }
                    "dev-only-secret-${java.util.UUID.randomUUID()}"
                },
                llm = LlmConfig(
                    enabledByDefault = env.bool("LLM_ENABLED", false),
                    endpoints = listOfNotNull(env.llmEndpoint("PRIMARY"), env.llmEndpoint("BACKUP")),
                    dailyLimitPerUser = env.int("LLM_DAILY_LIMIT", 3),
                    timeoutMs = env.int("LLM_TIMEOUT_MS", 3000).toLong(),
                ),
                adminToken = env.optional("ADMIN_TOKEN"),
                qweatherHost = env.optional("QWEATHER_API_HOST"),
                qweatherKey = env.optional("QWEATHER_API_KEY"),
                percentileCityMin = env.int("PERCENTILE_CITY_MIN", 100),
                percentileGlobalMin = env.int("PERCENTILE_GLOBAL_MIN", 50),
                smtp = env.optional("SMTP_HOST")?.let { host ->
                    SmtpConfig(
                        host = host,
                        port = env.int("SMTP_PORT", 465),
                        user = env.str("SMTP_USER", ""),
                        password = env.str("SMTP_PASSWORD", ""),
                        from = env.optional("SMTP_FROM") ?: error("SMTP_FROM is required with SMTP_HOST"),
                        fromName = env.str("SMTP_FROM_NAME", "下班啦"),
                        ssl = env.bool("SMTP_SSL", true),
                    )
                },
                goHomeEnabled = env.bool("GO_HOME_ENABLED", false),
                appDownloadUrl = env.optional("APP_DOWNLOAD_URL"),
                adminUsername = env.str("ADMIN_USERNAME", "admin"),
                adminBootstrapPassword = env.optional("ADMIN_BOOTSTRAP_PASSWORD") ?: "123456",
                settingsEncryptionKey = env.optional("SETTINGS_ENCRYPTION_KEY"),
                accountIssuer = env.str("ACCOUNT_ISSUER", "https://account.xiangban.xyz").trimEnd('/'),
                accountBaseUrl = (env.optional("ACCOUNT_BASE_URL") ?: env.str("ACCOUNT_ISSUER", "https://account.xiangban.xyz")).trimEnd('/'),
                accountAudiences = env.str("ACCOUNT_AUDIENCES", "xiaban-web,xiaban-api").split(',').map { it.trim() }.filter { it.isNotEmpty() }.toSet(),
            )
        }

        private fun EnvReader.llmEndpoint(slot: String): LlmEndpoint? {
            val baseUrl = optional("LLM_${slot}_BASE_URL") ?: return null
            return LlmEndpoint(
                name = str("LLM_${slot}_NAME", slot.lowercase()),
                baseUrl = baseUrl,
                apiKey = optional("LLM_${slot}_API_KEY") ?: error("LLM_${slot}_API_KEY is required with LLM_${slot}_BASE_URL"),
                model = optional("LLM_${slot}_MODEL") ?: error("LLM_${slot}_MODEL is required with LLM_${slot}_BASE_URL"),
            )
        }
    }
}

class EnvReader(private val source: Map<String, String>) {
    fun optional(key: String): String? = source[key]?.trim()?.takeIf { it.isNotEmpty() }
    fun str(key: String, default: String): String = optional(key) ?: default
    fun int(key: String, default: Int): Int =
        optional(key)?.let { it.toIntOrNull() ?: error("$key must be an integer, got '$it'") } ?: default
    fun bool(key: String, default: Boolean): Boolean = optional(key)?.let { it.equals("true", true) || it == "1" } ?: default
}
