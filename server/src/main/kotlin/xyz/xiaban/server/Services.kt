package xyz.xiaban.server

import xyz.xiaban.server.auth.Tokens
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.SessionService
import xyz.xiaban.server.users.UserRepository

class Services(
    val config: AppConfig,
    val db: Db,
    val redis: Redis,
) : AutoCloseable {
    val tokens = Tokens(config.jwtSecret)
    val cities = Cities.load()
    val users = UserRepository(db)
    val profiles = ProfileService(users, cities)
    val sessions = SessionService(users, profiles, tokens)

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
