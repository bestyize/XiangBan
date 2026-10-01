package xyz.xiaban.server.config

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
