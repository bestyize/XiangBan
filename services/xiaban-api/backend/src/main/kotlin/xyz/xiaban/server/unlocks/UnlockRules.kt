package xyz.xiaban.server.unlocks

import java.time.DayOfWeek
import java.time.LocalDate
import xyz.xiaban.contracts.afterwork.checkin.TitleCode

/** What a fresh check-in looks like, as far as the collection cares. Nothing here depends on sharing. */
data class UnlockContext(
    val workDay: LocalDate,
    val title: TitleCode,
    val totalDays: Int,
    val streak: Int,
    val weatherTag: String?,
    val holidayCode: String?,
    /** Calendar days since the previous check-in, null for the first one */
    val daysSincePrevious: Long?,
    val previousWasCare: Boolean,
)

object UnlockRules {
    private val DAY_MILESTONES = listOf(3, 7, 14, 30, 60, 100)
    private val STREAK_MILESTONES = listOf(3, 5, 10)
    private val ON_TIME = setOf(TitleCode.EarlyLeaver, TitleCode.OnTime, TitleCode.ExactOnTime)
    private val HOLIDAYS = setOf("spring_festival", "mid_autumn", "national_day")

    fun earned(ctx: UnlockContext): Set<String> = buildSet {
        if (ctx.totalDays >= 1) add("hello")
        DAY_MILESTONES.filter { ctx.totalDays >= it }.forEach { add("days_$it") }
        STREAK_MILESTONES.filter { ctx.streak >= it }.forEach { add("streak_$it") }
        add(UnlockCatalog.forTitle(ctx.title))
        when (ctx.workDay.dayOfWeek) {
            DayOfWeek.MONDAY -> add("monday")
            DayOfWeek.FRIDAY -> add("friday")
            else -> Unit
        }
        if (ctx.weatherTag == "rain") add("rainy")
        ctx.holidayCode?.takeIf { it in HOLIDAYS }?.let(::add)
        if ((ctx.daysSincePrevious ?: 0) >= 7) add("comeback")
        if (ctx.previousWasCare && ctx.title in ON_TIME) add("redemption")
    }
}
