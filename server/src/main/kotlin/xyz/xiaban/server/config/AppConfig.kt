package xyz.xiaban.server.config

data class DbConfig(val url: String, val user: String, val password: String, val poolSize: Int)

data class AppConfig(
    val env: String,
    val port: Int,
    val publicBaseUrl: String,
    val trustProxy: Boolean,
    val db: DbConfig,
    val redisUrl: String,
) {
    val isProd: Boolean get() = env == "prod"

    companion object {
        fun fromEnv(source: Map<String, String> = System.getenv()): AppConfig {
            val env = EnvReader(source)
            return AppConfig(
                env = env.str("XIABAN_ENV", "dev"),
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
