package xyz.xiaban.server

import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.compression.ContentEncoding
import io.ktor.serialization.kotlinx.json.json
import xyz.xiaban.server.auth.Tokens
import xyz.xiaban.server.calendar.HolidayCalendar
import xyz.xiaban.server.checkin.CheckinRepository
import xyz.xiaban.server.checkin.CheckinService
import xyz.xiaban.server.checkin.CheckinViews
import xyz.xiaban.server.checkin.Polisher
import xyz.xiaban.server.checkin.ReplyPlanner
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.copy.CopyLibrary
import xyz.xiaban.server.copy.CopyModeration
import xyz.xiaban.server.copy.CopySelector
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.flags.FeatureFlags
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.llm.LlmProvider
import xyz.xiaban.server.llm.LlmQuota
import xyz.xiaban.server.llm.OpenAiCompatibleProvider
import xyz.xiaban.server.llm.PolishQueue
import xyz.xiaban.server.llm.PolishService
import xyz.xiaban.server.memory.MemoryService
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.stats.PercentileService
import xyz.xiaban.server.stats.StatsService
import xyz.xiaban.server.stats.TipsService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import xyz.xiaban.server.account.AccountMerge
import xyz.xiaban.server.account.EmailBinding
import xyz.xiaban.server.guardian.GuardianRepository
import xyz.xiaban.server.guardian.GuardianScheduler
import xyz.xiaban.server.guardian.GuardianService
import xyz.xiaban.server.mail.LogMailer
import xyz.xiaban.server.mail.MailQueue
import xyz.xiaban.server.mail.Mailer
import xyz.xiaban.server.mail.SmtpMailer
import xyz.xiaban.server.recall.RecallJob
import xyz.xiaban.server.recall.UnsubscribeLinks
import xyz.xiaban.server.traffic.TrafficRules
import xyz.xiaban.server.unlocks.UnlockService
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.SessionService
import xyz.xiaban.server.users.UserRepository
import xyz.xiaban.server.weather.QWeatherSource
import xyz.xiaban.server.weather.WeatherService
import xyz.xiaban.server.weather.WeatherSource
import java.time.Clock
import java.time.Duration
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation as ClientContentNegotiation

class Services(
    val config: AppConfig,
    val db: Db,
    val redis: Redis,
    val clock: Clock = Clock.systemUTC(),
    llmProviders: List<LlmProvider>? = null,
    weatherSource: WeatherSource? = null,
    mailer: Mailer? = null,
) : AutoCloseable {
    val http = HttpClient(CIO) {
        expectSuccess = false
        install(ClientContentNegotiation) { json(AppJson) }
        install(HttpTimeout) {
            connectTimeoutMillis = 2_000
            requestTimeoutMillis = 5_000
        }
        install(ContentEncoding) { gzip() }
    }

    val users = UserRepository(db)
    val tokens = Tokens(config.jwtSecret, redis = redis, accountState = users::authState)
    val cities = Cities.load()
    val calendar = HolidayCalendar.load()
    val flags = FeatureFlags(redis, config.llm.enabledByDefault, config.goHomeEnabled, config.appDownloadUrl)

    val profiles = ProfileService(users, cities)
    val avatars = xyz.xiaban.server.users.AvatarService(db)
    val quizzes = xyz.xiaban.server.quizzes.QuizRecords(db)
    val games = xyz.xiaban.server.games.GameRecords(db)
    val afterwork = xyz.xiaban.server.afterwork.AfterworkRecords(db)
    val sessions = SessionService(users, profiles, tokens, flags)

    val weather = WeatherService(
        source = weatherSource ?: config.qweatherHost?.let { host -> config.qweatherKey?.let { QWeatherSource(host, it, http, clock) } },
        redis = redis,
    )
    val traffic = TrafficRules.load(calendar)

    val copyLibrary = CopyLibrary.load()
    val moderation = CopyModeration(db, redis)
    val memories = MemoryService(db)
    val checkinRepository = CheckinRepository(db)
    val stats = StatsService(checkinRepository, calendar)
    val unlocks = UnlockService(db)
    val percentiles = PercentileService(redis, cities, config.percentileCityMin, config.percentileGlobalMin)
    val polisher = Polisher(
        checkins = checkinRepository,
        profiles = profiles,
        cities = cities,
        memories = memories,
        service = PolishService(llmProviders ?: config.llm.endpoints.map { OpenAiCompatibleProvider(it, http) }, Duration.ofMillis(config.llm.timeoutMs)),
        flags = flags,
        quota = LlmQuota(redis, config.llm.dailyLimitPerUser),
        queue = PolishQueue(redis, clock),
        weather = weather,
        clock = clock,
    )
    val checkins = CheckinService(
        profiles = profiles,
        checkins = checkinRepository,
        calendar = calendar,
        cities = cities,
        planner = ReplyPlanner(CopySelector(copyLibrary), moderation, calendar),
        memories = memories,
        polisher = polisher,
        weather = weather,
        stats = stats,
        tips = TipsService(weather, traffic, calendar, cities),
        percentiles = percentiles,
        views = CheckinViews(checkinRepository, percentiles, calendar, flags),
        unlocks = unlocks,
        clock = clock,
        afterChange = { userId ->
            guardian.reschedule(userId)
            users.resetRecall(userId)
        },
    )

    val emailSettings = xyz.xiaban.server.admin.EmailSettingsStore(db, config)
    val administrators = xyz.xiaban.server.admin.AdminAccounts(db, redis, config)
    val adminUsers = xyz.xiaban.server.admin.AdminUsers(db, clock, cities)
    private val runtimeMailer = xyz.xiaban.server.mail.RuntimeMailer(emailSettings)
    private val suppliedMailer = mailer
    val mailer: Mailer = mailer ?: runtimeMailer
    val emailAvailable: suspend () -> Boolean = { suppliedMailer != null || runtimeMailer.available() }
    val mail = MailQueue(redis, db, this.mailer, clock)
    val guardianRepository = GuardianRepository(db)
    val guardian: GuardianService by lazy { GuardianService(guardianRepository, users, checkinRepository, calendar, redis, clock) }
    val guardianScheduler = GuardianScheduler(guardian, guardianRepository, users, mail, redis, config.publicBaseUrl, clock)
    val unsubscribeLinks = UnsubscribeLinks(config.jwtSecret, config.publicBaseUrl)
    val recall = RecallJob(db, users, CopySelector(copyLibrary), mail, unsubscribeLinks, config.publicBaseUrl, clock)
    val emailBinding = EmailBinding(redis, users, profiles, tokens, AccountMerge(db), emailAvailable,
        { this.mailer.send(it) }, { guardian.reschedule(it) })
    val events = xyz.xiaban.server.events.EventService(db, clock)
    val metrics = xyz.xiaban.server.events.MetricsService(db, clock)
    val account = xyz.xiaban.server.account.AccountService(db, profiles, guardian)

    /** Background work tied to the application lifecycle. */
    fun startWorkers(scope: CoroutineScope) {
        polisher.start(scope)
        mail.start(scope)
        guardianScheduler.start(scope)
        recall.start(scope)
        // Guest events have no account to be deleted with; drop them after the retention period, once a day.
        scope.launch {
            while (isActive) {
                runCatching { events.purgeGuestEvents() }
                delay(24 * 3600 * 1000L)
            }
        }
    }

    override fun close() {
        weather.close()
        http.close()
        redis.close()
        db.close()
    }

    companion object {
        fun create(
            config: AppConfig,
            clock: Clock = Clock.systemUTC(),
            llmProviders: List<LlmProvider>? = null,
            weatherSource: WeatherSource? = null,
            mailer: Mailer? = null,
        ): Services {
            val db = Db.connect(config.db)
            val redis = Redis.connect(config.redisUrl)
            return Services(config, db, redis, clock, llmProviders, weatherSource, mailer)
        }
    }
}
