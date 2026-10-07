package xyz.xiaban.server.checkin

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import xyz.xiaban.server.flags.FeatureFlags
import xyz.xiaban.server.llm.LlmQuota
import xyz.xiaban.server.llm.PolishInput
import xyz.xiaban.server.llm.PolishQueue
import xyz.xiaban.server.llm.PolishService
import xyz.xiaban.server.memory.MemoryService
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.server.users.HHMM
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.weather.WeatherInsights
import xyz.xiaban.server.weather.WeatherService
import java.time.Clock
import java.time.Duration
import java.time.LocalDateTime

/** Optional model rewrite of the library line: show the original first, swap in the rewrite if it lands in time. */
class Polisher(
    private val checkins: CheckinRepository,
    private val profiles: ProfileService,
    private val cities: Cities,
    private val memories: MemoryService,
    private val service: PolishService,
    private val flags: FeatureFlags,
    private val quota: LlmQuota,
    private val queue: PolishQueue,
    private val weather: WeatherService,
    private val clock: Clock,
) {
    /** Returns true when a rewrite has been queued for this fresh check-in. */
    suspend fun request(row: CheckinRow): Boolean {
        if (!service.available || !flags.llmEnabled() || !quota.tryAcquire(row.userId, row.workDay)) return false
        queue.enqueue(row.id)
        return true
    }

    suspend fun isPending(checkinId: Long): Boolean = queue.isPending(checkinId)

    suspend fun await(checkinId: Long, maxWait: Duration) = queue.awaitDone(checkinId, maxWait)

    fun start(scope: CoroutineScope): Job = queue.startWorker(scope, ::handle)

    private suspend fun handle(checkinId: Long, enqueuedAt: Long) {
        if (clock.millis() - enqueuedAt > STALE_MS) return
        val row = checkins.find(checkinId)?.takeIf { it.revokedAt == null } ?: return
        val user = profiles.user(row.userId)
        val city = cities.find(user.cityCode)?.takeIf { it.lat != null }
        val forecast = city?.let { weather.cached(it) }
        val rewritten = service.polish(
            PolishInput(
                original = row.libraryReply,
                companionName = user.companionName,
                personality = user.personality,
                nickname = user.nickname,
                titleLabel = row.title.label,
                localTime = LocalDateTime.ofInstant(row.checkedAt, user.timezone).format(HHMM),
                cityName = city?.name,
                weather = forecast?.let { WeatherInsights.summary(it, clock.instant(), user.timezone) },
                memories = memories.recent(user.id),
            ),
        )
        if (rewritten != null) checkins.setPolished(row.id, rewritten)
    }

    private companion object {
        /** The page stops waiting after 3 s; a rewrite landing much later would only contradict a saved card. */
        const val STALE_MS = 5_000L
    }
}
