package xyz.xiaban.server.support

import xyz.xiaban.server.meta.City
import xyz.xiaban.server.weather.Forecast
import xyz.xiaban.server.weather.WeatherSource
import java.time.Instant
import java.time.LocalDate
import java.time.OffsetDateTime
import java.util.concurrent.atomic.AtomicInteger

class FakeWeather : WeatherSource {
    @Volatile var forecast: Forecast? = null
    val fetches = AtomicInteger()

    fun reset() {
        forecast = null
        fetches.set(0)
    }

    override suspend fun fetch(city: City): Forecast? {
        fetches.incrementAndGet()
        return forecast
    }

    companion object {
        /** Hourly Beijing-time forecast starting at [from]; [wetAt] marks hours with rain. */
        fun forecast(from: String, hours: Int = 24, temp: Int = 18, wetAt: Set<Int> = emptySet(), daily: List<Forecast.Day> = emptyList()): Forecast {
            val start = OffsetDateTime.parse("$from:00+08:00")
            return Forecast(
                fetchedAt = Instant.now().toEpochMilli(),
                hourly = (0 until hours).map { i ->
                    val t = start.plusHours(i.toLong())
                    val wet = t.hour in wetAt
                    Forecast.Hour(t.toInstant().toEpochMilli(), temp, if (wet) "小雨" else "多云", if (wet) 80 else 10, if (wet) 0.6 else 0.0)
                },
                daily = daily,
            )
        }

        fun day(date: String, min: Int, max: Int) = Forecast.Day(LocalDate.parse(date).toString(), min, max, "多云", "晴")
    }
}
