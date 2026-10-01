package xyz.xiaban.server.support

import kotlinx.coroutines.future.await
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assumptions
import xyz.xiaban.server.Services
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.query
import java.net.InetSocketAddress
import java.net.Socket
import java.net.URI

/**
 * Integration tests need a real PostgreSQL and Redis. Locally they are skipped when the services are
 * unreachable; CI sets XIABAN_REQUIRE_INTEGRATION=true so a missing service fails the build instead.
 */
object TestEnv {
    private val env = System.getenv()
    private val dbUrl = env["XIABAN_TEST_DATABASE_URL"] ?: "jdbc:postgresql://localhost:5432/xiaban_test"
    private val redisUrl = env["XIABAN_TEST_REDIS_URL"] ?: "redis://localhost:6379/15"
    private val required = env["XIABAN_REQUIRE_INTEGRATION"] == "true"

    val config: AppConfig = AppConfig.fromEnv(
        mapOf(
            "XIABAN_ENV" to "test",
            "DATABASE_URL" to dbUrl,
            "DATABASE_USER" to (env["XIABAN_TEST_DATABASE_USER"] ?: "xiaban"),
            "DATABASE_PASSWORD" to (env["XIABAN_TEST_DATABASE_PASSWORD"] ?: "xiaban"),
            "DATABASE_POOL_SIZE" to "4",
            "REDIS_URL" to redisUrl,
        ),
    )

    val clock = MutableClock()

    private val shared: Services by lazy { Services.create(config, clock) }

    /** Returns services backed by an emptied database and Redis, with the clock reset to now. */
    fun services(): Services {
        val reachable = reachable(dbUrl.removePrefix("jdbc:")) && reachable(redisUrl)
        if (!reachable) {
            check(!required) { "PostgreSQL ($dbUrl) or Redis ($redisUrl) is not reachable" }
            Assumptions.assumeTrue(false, "PostgreSQL/Redis not reachable, skipping integration test")
        }
        clock.now = java.time.Instant.now()
        reset(shared)
        return shared
    }

    private fun reset(services: Services) = runBlocking {
        services.db.tx { c ->
            val tables = c.query(
                "select tablename from pg_tables where schemaname = 'public' and tablename <> 'flyway_schema_history'",
            ) { it.getString(1) }
            if (tables.isNotEmpty()) c.execute("truncate ${tables.joinToString { "\"$it\"" }} restart identity cascade")
        }
        services.redis.cmd.flushdb().await()
    }

    private fun reachable(url: String): Boolean {
        val uri = URI(url)
        return runCatching {
            Socket().use { it.connect(InetSocketAddress(uri.host, if (uri.port > 0) uri.port else 6379), 500) }
        }.isSuccess
    }
}
