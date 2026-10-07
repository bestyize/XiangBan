package xyz.xiaban.server.db

import com.zaxxer.hikari.HikariConfig
import com.zaxxer.hikari.HikariDataSource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.flywaydb.core.Flyway
import xyz.xiaban.server.config.DbConfig
import java.sql.Connection

class Db(private val dataSource: HikariDataSource) : AutoCloseable {

    suspend fun <T> read(block: (Connection) -> T): T = withContext(Dispatchers.IO) {
        dataSource.connection.use(block)
    }

    suspend fun <T> tx(block: (Connection) -> T): T = withContext(Dispatchers.IO) {
        dataSource.connection.use { c ->
            c.autoCommit = false
            try {
                block(c).also { c.commit() }
            } catch (e: Throwable) {
                c.rollback()
                throw e
            }
        }
    }

    override fun close() = dataSource.close()

    companion object {
        fun connect(config: DbConfig): Db {
            val hikari = HikariConfig().apply {
                jdbcUrl = config.url
                username = config.user
                password = config.password
                maximumPoolSize = config.poolSize
                poolName = "xiaban-db"
            }
            val ds = HikariDataSource(hikari)
            Flyway.configure().dataSource(ds).locations("classpath:db/migration").load().migrate()
            return Db(ds)
        }
    }
}
