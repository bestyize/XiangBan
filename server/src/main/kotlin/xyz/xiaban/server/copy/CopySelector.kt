package xyz.xiaban.server.copy

import xyz.xiaban.shared.Personality
import xyz.xiaban.shared.TitleCode
import kotlin.random.Random

data class CopyContext(
    val personality: Personality,
    val title: TitleCode,
    /** rain | cold | hot, only when cached weather is available */
    val weather: String? = null,
    /** ISO weekday of the 下班日, 1 = Monday */
    val weekday: Int,
    /** Festival code while on holiday, plus pre_holiday / makeup_day */
    val holidayTags: Set<String> = emptySet(),
    /** Consecutive overtime days including today */
    val overtimeStreak: Int = 0,
    val careMode: Boolean = false,
)

/**
 * Scores lines by how many of their tags match today, then picks at random among the top two score
 * tiers so specific lines win without drowning out the rest. Lines used in the last 30 days are avoided.
 */
class CopySelector(private val library: CopyLibrary, private val random: Random = Random.Default) {

    fun pickReply(ctx: CopyContext, recentlyUsed: Set<String>, disabled: Set<String>): CopyEntry? {
        val mode = if (ctx.careMode) CopyMode.Care else CopyMode.Normal
        val eligible = library.entries.filter { it.personality == ctx.personality && it.mode == mode && it.id !in disabled && matches(it, ctx) }
        if (eligible.isEmpty()) return null

        val scored = eligible.map { it to score(it) }
        val best = scored.maxOf { it.second }
        val top = scored.filter { it.second >= best - 1 }.map { it.first }
        val pool = top.filterNot { it.id in recentlyUsed }.ifEmpty { eligible.filterNot { it.id in recentlyUsed } }.ifEmpty { eligible }
        return pool.random(random)
    }

    fun pickGreeting(personality: Personality, mode: CopyMode, daypart: Daypart, seed: Long, disabled: Set<String>): CopyEntry? {
        val candidates = library.entries.filter {
            it.personality == personality && it.mode == mode && it.id !in disabled && (it.daypart == null || it.daypart == daypart)
        }
        return candidates.takeIf { it.isNotEmpty() }?.let { it[Random(seed).nextInt(it.size)] }
    }

    private fun matches(e: CopyEntry, ctx: CopyContext): Boolean =
        (e.title == null || e.title == ctx.title) &&
            (e.weather == null || e.weather == ctx.weather) &&
            (e.weekdays == null || ctx.weekday in e.weekdays) &&
            (e.holiday == null || e.holiday in ctx.holidayTags) &&
            (e.streakMin == null || ctx.overtimeStreak >= e.streakMin)

    private fun score(e: CopyEntry): Int =
        (if (e.title != null) 4 else 0) +
            (if (e.weather != null) 4 else 0) +
            (if (e.holiday != null) 5 else 0) +
            (if (e.streakMin != null) 5 else 0) +
            (if (e.weekdays != null) 3 else 0)

    companion object {
        fun render(text: String, nickname: String?, overtimeStreak: Int): String =
            text.replace("{name}", nickname ?: "你").replace("{n}", overtimeStreak.toString())
    }
}
