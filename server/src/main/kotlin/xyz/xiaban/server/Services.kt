package xyz.xiaban.server

import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.redis.Redis

class Services(
    val config: AppConfig,
    val db: Db,
    val redis: Redis,
) : AutoCloseable {

    override fun close() {
        redis.close()
        db.close()
    }

    companion object {
        fun create(config: AppConfig): Services {
            val db = Db.connect(config.db)
            val redis = Redis.connect(config.redisUrl)
            return Services(config, db, redis)
        }
    }
}
