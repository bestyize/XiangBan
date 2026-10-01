package xyz.xiaban.shared

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
enum class PercentileScope {
    @SerialName("city") City,
    @SerialName("all") All,
}

/** "你比 [percent]% 的下班了么用户下班早" */
@Serializable
data class PercentileView(val percent: Int, val scope: PercentileScope, val cityName: String?)

@Serializable
data class CheckinStats(
    /** Consecutive check-in days; only missed workdays break it. */
    val streak: Int,
    /** 相伴天数, never reset */
    val totalDays: Int,
    val percentile: PercentileView?,
)

@Serializable
enum class TipKind {
    @SerialName("rain") Rain,
    @SerialName("cold") Cold,
    @SerialName("traffic") Traffic,
    @SerialName("holiday") Holiday,
    @SerialName("makeup") Makeup,
    @SerialName("morning") Morning,
    @SerialName("duty") Duty,
    @SerialName("week") Week,
    @SerialName("month") Month,
    @SerialName("payday") Payday,
}

@Serializable
data class TipView(val kind: TipKind, val text: String)

@Serializable
data class StatsView(
    val totalDays: Int,
    val currentStreak: Int,
    /** Length of the streak that ended, shown as "上次连续 N 天" without any penalty. */
    val lastStreak: Int?,
    val bestStreak: Int,
    val todayOnDutyMinutes: Int?,
    /** HH:mm */
    val weekAvgOffTime: String?,
    val weekCount: Int,
    val monthOvertimeMinutes: Int,
    val monthCount: Int,
    val titleCounts: Map<TitleCode, Int>,
)
