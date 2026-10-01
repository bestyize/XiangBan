package xyz.xiaban.server.weather

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.parameter
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.cancel
import kotlinx.coroutines.future.await
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.serialization.Serializable
import org.slf4j.LoggerFactory
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.meta.City
import xyz.xiaban.server.redis.Redis
import java.time.Clock
import java.time.Duration
import java.time.OffsetDateTime
import java.util.concurrent.ConcurrentHashMap

interface WeatherSource {
    suspend fun fetch(city: City): Forecast?
}

/** 和风天气 v7 API; [host] is the per-project API Host shown in the QWeather console. */
class QWeatherSource(host: String, private val key: String, private val http: HttpClient, private val clock: Clock) : WeatherSource {
    private val base = (if (host.startsWith("http://") || host.startsWith("https://")) host else "https://$host").trimEnd('/')

    override suspend fun fetch(city: City): Forecast? {
        val location = "%.2f,%.2f".format(city.lon ?: return null, city.lat ?: return null)
        val hourly = http.get("$base/v7/weather/24h") { header("X-QW-Api-Key", key); parameter("location", location) }.body<HourlyResponse>()
        val daily = http.get("$base/v7/weather/3d") { header("X-QW-Api-Key", key); parameter("location", location) }.body<DailyResponse>()
        check(hourly.code == "200" && daily.code == "200") { "QWeather answered ${hourly.code}/${daily.code}" }
        return Forecast(
            fetchedAt = clock.millis(),
            hourly = hourly.hourly.map {
                Forecast.Hour(OffsetDateTime.parse(it.fxTime).toInstant().toEpochMilli(), it.temp.toInt(), it.text, it.pop?.toIntOrNull(), it.precip?.toDoubleOrNull() ?: 0.0)
            },
            daily = daily.daily.map { Forecast.Day(it.fxDate, it.tempMin.toInt(), it.tempMax.toInt(), it.textDay, it.textNight) },
        )
    }

    @Serializable private data class HourlyResponse(val code: String, val hourly: List<HourlyItem> = emptyList())
    @Serializable private data class HourlyItem(val fxTime: String, val temp: String, val text: String, val pop: String? = null, val precip: String? = null)
    @Serializable private data class DailyResponse(val code: String, val daily: List<DailyItem> = emptyList())
    @Serializable private data class DailyItem(val fxDate: String, val tempMax: String, val tempMin: String, val textDay: String, val textNight: String)
}

/**
 * Forecasts cached per city for an hour (PRD 5.5). Fetches run in their own scope, so a caller that stops
 * waiting still leaves a warm cache behind, and concurrent misses for one city share one request.
 */
class WeatherService(private val source: WeatherSource?, private val redis: Redis) : AutoCloseable {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val inflight = ConcurrentHashMap<String, Deferred<Forecast?>>()

    suspend fun cached(city: City): Forecast? =
        redis.cmd.get(key(city)).await()?.let { runCatching { AppJson.decodeFromString<Forecast>(it) }.getOrNull() }

    /** Cached forecast, or a fresh one if it arrives within [timeout]; null when weather is off or slow. */
    suspend fun forecast(city: City, timeout: Duration = Duration.ofMillis(1500)): Forecast? {
        if (source == null || city.lat == null) return null
        cached(city)?.let { return it }
        return withTimeoutOrNull(timeout.toMillis()) { load(city).await() }
    }

    /** Warms the cache without waiting, e.g. right after a check-in. */
    suspend fun prefetch(city: City) {
        if (source != null && city.lat != null && cached(city) == null) load(city)
    }

    private fun load(city: City): Deferred<Forecast?> {
        val created = scope.async(start = CoroutineStart.LAZY) {
            runCatching { source!!.fetch(city) }
                .onFailure { log.warn("Weather for {} failed: {}", city.code, it.message) }
                .getOrNull()
                ?.also { runCatching { redis.cmd.setex(key(city), TTL_SECONDS, AppJson.encodeToString(Forecast.serializer(), it)).await() } }
        }
        val existing = inflight.putIfAbsent(city.code, created)
        if (existing != null) {
            created.cancel()
            return existing
        }
        created.invokeOnCompletion { inflight.remove(city.code, created) }
        created.start()
        return created
    }

    override fun close() = scope.cancel()

    private fun key(city: City) = "xb:wx:${city.code}"

    private companion object {
        const val TTL_SECONDS = 3600L
        val log = LoggerFactory.getLogger(WeatherService::class.java)
    }
}
