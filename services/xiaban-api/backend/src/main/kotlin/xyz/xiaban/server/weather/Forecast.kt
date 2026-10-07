package xyz.xiaban.server.weather

import kotlinx.serialization.Serializable
import java.time.Instant
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.LocalTime
import java.time.ZoneId

@Serializable
data class Forecast(val fetchedAt: Long, val hourly: List<Hour>, val daily: List<Day>) {
    @Serializable
    data class Hour(val time: Long, val temp: Int, val text: String, val pop: Int? = null, val precip: Double = 0.0) {
        val wet: Boolean get() = precip > 0.0 || (pop ?: 0) >= 50 || WET_WORDS.any { it in text }
    }

    @Serializable
    data class Day(val date: String, val tempMin: Int, val tempMax: Int, val textDay: String, val textNight: String)
}

private val WET_WORDS = listOf("雨", "雪", "雹")

/** What 小伴 can say about the weather; pure functions over a cached forecast. */
object WeatherInsights {
    private val MORNING = LocalTime.of(8, 0)

    /** First wet hour within [hours] from now: the commute home. */
    fun rainSoon(f: Forecast, now: Instant, hours: Long = 4): Forecast.Hour? {
        val until = now.plusSeconds(hours * 3600).toEpochMilli()
        return f.hourly.firstOrNull { it.time >= now.toEpochMilli() - 3_600_000 && it.time <= until && it.wet }
    }

    /** Temperature at the next 08:00, from the hourly forecast. */
    fun nextMorningTemp(f: Forecast, now: Instant, zone: ZoneId): Int? {
        val local = LocalDateTime.ofInstant(now, zone)
        val day = if (local.toLocalTime() < MORNING) local.toLocalDate() else local.toLocalDate().plusDays(1)
        val target = day.atTime(MORNING).atZone(zone).toInstant().toEpochMilli()
        return f.hourly.minByOrNull { kotlin.math.abs(it.time - target) }?.takeIf { kotlin.math.abs(it.time - target) <= 3_600_000 }?.temp
    }

    /** Degrees the next day's high drops below [base]'s high, when it is a real drop (5°C or more). */
    fun tempDrop(f: Forecast, base: LocalDate): Int? {
        val today = f.daily.firstOrNull { it.date == base.toString() } ?: return null
        val tomorrow = f.daily.firstOrNull { it.date == base.plusDays(1).toString() } ?: return null
        return (today.tempMax - tomorrow.tempMax).takeIf { it >= 5 }
    }

    /** Copy-library weather tag. */
    fun tag(f: Forecast, now: Instant, zone: ZoneId, base: LocalDate): String? = when {
        rainSoon(f, now) != null -> "rain"
        tempDrop(f, base) != null || (nextMorningTemp(f, now, zone) ?: 99) <= 5 -> "cold"
        f.daily.firstOrNull { it.date == base.toString() }?.let { it.tempMax >= 33 } == true -> "hot"
        else -> null
    }

    /** One-line description for the model prompt. */
    fun summary(f: Forecast, now: Instant, zone: ZoneId): String = buildList {
        rainSoon(f, now)?.let { add("${LocalDateTime.ofInstant(Instant.ofEpochMilli(it.time), zone).hour} 点有${it.text}") }
        f.hourly.firstOrNull()?.let { add("现在 ${it.temp}°C，${it.text}") }
        nextMorningTemp(f, now, zone)?.let { add("明早 ${it}°C") }
    }.joinToString("，")
}
