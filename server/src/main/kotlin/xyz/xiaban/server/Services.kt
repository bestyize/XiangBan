package xyz.xiaban.server

import xyz.xiaban.server.auth.Tokens
import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.checkin.CheckinRepository
import xyz.xiaban.server.checkin.CheckinService
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.SessionService
import xyz.xiaban.server.users.UserRepository
import java.time.Clock

class Services(
    val config: AppConfig,
    val db: Db,
    val redis: Redis,
    val clock: Clock = Clock.systemUTC(),
) : AutoCloseable {
    val tokens = Tokens(config.jwtSecret)
    val cities = Cities.load()
    val calendar = HolidayCalendar.load()

    val users = UserRepository(db)
    val profiles = ProfileService(users, cities)
    val sessions = SessionService(users, profiles, tokens)

    val checkinRepository = CheckinRepository(db)
    val checkins = CheckinService(profiles, checkinRepository, calendar, clock)

    override fun close() {
        redis.close()
        db.close()
    }

    companion object {
        fun create(config: AppConfig, clock: Clock = Clock.systemUTC()): Services {
            val db = Db.connect(config.db)
            val redis = Redis.connect(config.redisUrl)
            return Services(config, db, redis, clock)
        }
    }
}
