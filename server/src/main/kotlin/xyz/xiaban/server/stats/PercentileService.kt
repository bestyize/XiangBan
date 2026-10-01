package xyz.xiaban.server.stats

import kotlinx.coroutines.future.await
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.shared.PercentileScope
import xyz.xiaban.shared.PercentileView
import java.time.LocalDate
import java.util.concurrent.ConcurrentHashMap

/**
 * "你比 xx% 的下班了么用户下班早" (PRD 5.6), from per-minute counts in Redis, never the database.
 *
 * At check-in time today's distribution only holds people who left earlier, which would always read 0%,
 * so the reference is the previous [WINDOW_DAYS] complete 下班日. The sample thresholds apply to the
 * average daily count over that window.
 */
class PercentileService(
    private val redis: Redis,
    private val cities: Cities,
    private val cityMinDaily: Int,
    private val globalMinDaily: Int,
) {
    private val cache = ConcurrentHashMap<String, Pair<Long, IntArray>>()

    suspend fun record(workDay: LocalDate, minute: Int, cityCode: String?, delta: Long = 1) {
        for (key in keys(workDay, cityCode)) {
            redis.cmd.hincrby(key, minute.toString(), delta).await()
            redis.cmd.expire(key, TTL_SECONDS).await()
        }
    }

    suspend fun percentile(workDay: LocalDate, minute: Int, cityCode: String?): PercentileView? {
        val city = cities.find(cityCode)?.takeIf { it.lat != null }
        if (city != null) {
            reference(workDay, cityKey(city.code))?.let { (counts, avgDaily) ->
                if (avgDaily >= cityMinDaily) return PercentileView(percentLater(counts, minute), PercentileScope.City, city.name)
            }
        }
        val (counts, avgDaily) = reference(workDay, ALL) ?: return null
        return if (avgDaily >= globalMinDaily) PercentileView(percentLater(counts, minute), PercentileScope.All, null) else null
    }

    /** Summed histogram of the window and the average daily sample over days that have data. */
    private suspend fun reference(workDay: LocalDate, scope: String): Pair<IntArray, Double>? {
        val total = IntArray(MINUTES)
        var daysWithData = 0
        var sample = 0L
        for (back in 1..WINDOW_DAYS) {
            val day = dayHistogram(workDay.minusDays(back.toLong()), scope)
            val count = day.sum()
            if (count == 0) continue
            daysWithData++
            sample += count
            for (i in 0 until MINUTES) total[i] += day[i]
        }
        return if (daysWithData == 0) null else total to sample.toDouble() / daysWithData
    }

    /**
     * Past days barely change, so a day with data is fetched at most every few minutes. Empty days are not
     * cached: looking them up again is cheap and they may still fill up (late check-ins until 05:00).
     */
    private suspend fun dayHistogram(day: LocalDate, scope: String): IntArray {
        val key = "xb:pct:$day:$scope"
        cache[key]?.takeIf { System.currentTimeMillis() - it.first < CACHE_MS }?.let { return it.second }
        val histogram = IntArray(MINUTES)
        redis.cmd.hgetall(key).await().forEach { (minute, count) ->
            minute.toIntOrNull()?.takeIf { it in 0 until MINUTES }?.let { histogram[it] = count.toIntOrNull()?.coerceAtLeast(0) ?: 0 }
        }
        if (histogram.any { it > 0 }) cache[key] = System.currentTimeMillis() to histogram
        return histogram
    }

    private fun keys(workDay: LocalDate, cityCode: String?) = buildList {
        add("xb:pct:$workDay:$ALL")
        cities.find(cityCode)?.takeIf { it.lat != null }?.let { add("xb:pct:$workDay:${cityKey(it.code)}") }
    }

    private fun cityKey(code: String) = "c:$code"

    companion object {
        const val WINDOW_DAYS = 7
        private const val MINUTES = 1440
        private const val ALL = "all"
        private const val TTL_SECONDS = 10L * 24 * 3600
        private const val CACHE_MS = 5 * 60_000L

        /** Share of the window that left strictly later than [minute], rounded down. */
        fun percentLater(counts: IntArray, minute: Int): Int {
            val total = counts.sum()
            if (total == 0) return 0
            val later = (minute + 1 until counts.size).sumOf { counts[it] }
            return (later * 100L / total).toInt()
        }
    }
}
