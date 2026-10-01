package xyz.xiaban.server.support

import kotlinx.coroutines.future.await
import kotlinx.coroutines.runBlocking
import org.junit.jupiter.api.Assumptions
import xyz.xiaban.server.Services
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.query
import xyz.xiaban.server.mail.LogMailer
import xyz.xiaban.server.mail.Mail
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
            "ADMIN_TOKEN" to ADMIN_TOKEN,
            "LLM_DAILY_LIMIT" to "1",
            "APP_DOWNLOAD_URL" to "https://example.com/xiangban",
        ),
    )

    val clock = MutableClock()
    val llm = FakeLlm()
    val weather = FakeWeather()
    val mailer = LogMailer()
    const val ADMIN_TOKEN = "test-admin-token"

    private val shared: Services by lazy { Services.create(config, clock, listOf(llm), weather, mailer) }

    /** Waits for the mail worker to deliver a matching message. */
    fun awaitMail(timeoutMs: Long = 5_000, predicate: (Mail) -> Boolean): Mail {
        val end = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < end) {
            synchronized(mailer.sent) { mailer.sent.firstOrNull(predicate) }?.let { return it }
            Thread.sleep(50)
        }
        error("no matching mail within $timeoutMs ms; sent: ${synchronized(mailer.sent) { mailer.sent.map { it.subject } }}")
    }

    fun sentMails(predicate: (Mail) -> Boolean): List<Mail> = synchronized(mailer.sent) { mailer.sent.filter(predicate) }

    /** Returns services backed by an emptied database and Redis, with the clock reset to now. */
    fun services(): Services {
        val reachable = reachable(dbUrl.removePrefix("jdbc:")) && reachable(redisUrl)
        if (!reachable) {
            check(!required) { "PostgreSQL ($dbUrl) or Redis ($redisUrl) is not reachable" }
            Assumptions.assumeTrue(false, "PostgreSQL/Redis not reachable, skipping integration test")
        }
        clock.now = java.time.Instant.now()
        llm.reset()
        weather.reset()
        synchronized(mailer.sent) { mailer.sent.clear() }
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
