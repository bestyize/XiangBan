package xyz.xiaban.contracts.afterwork.checkin

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import xyz.xiaban.contracts.afterwork.stats.CheckinStats
import xyz.xiaban.contracts.afterwork.stats.TipView
import xyz.xiaban.contracts.afterwork.unlocks.UnlockView
import xyz.xiaban.contracts.companion.personality.Mood

/** [code] mirrors the serial name and is what the database stores. */
@Serializable
enum class TitleCode(val code: String) {
    @SerialName("exact_on_time") ExactOnTime("exact_on_time"),
    @SerialName("weekend_work") WeekendWork("weekend_work"),
    @SerialName("borrowed_weekend") BorrowedWeekend("borrowed_weekend"),
    @SerialName("holiday_countdown") HolidayCountdown("holiday_countdown"),
    @SerialName("early_leaver") EarlyLeaver("early_leaver"),
    @SerialName("on_time") OnTime("on_time"),
    @SerialName("normal_human") NormalHuman("normal_human"),
    @SerialName("overtime_hero") OvertimeHero("overtime_hero"),
    @SerialName("night_owl") NightOwl("night_owl"),
    @SerialName("immortal") Immortal("immortal");

    companion object {
        fun of(code: String): TitleCode = entries.first { it.code == code }
    }
}

@Serializable
enum class ReplySource {
    @SerialName("library") Library,
    @SerialName("llm") Llm,
}

@Serializable
data class TitleView(val code: TitleCode, val name: String, val rare: Boolean, val mood: Mood)

@Serializable
data class ReplyView(val text: String, val source: ReplySource, val pending: Boolean)

@Serializable
data class CheckinView(
    val id: String,
    /** yyyy-MM-dd of the 下班日 */
    val workDay: String,
    val checkedAt: Long,
    /** HH:mm in the user's time zone */
    val localTime: String,
    val title: TitleView,
    val mood: Mood,
    val reply: ReplyView,
    /** Three late nights in a row: softer lines and a caring face. */
    val careMode: Boolean,
    /** Epoch millis until which the check-in can be undone; null once that is no longer possible. */
    val undoUntil: Long?,
    val stats: CheckinStats,
    /** Set when the 下班日 falls in a statutory holiday, for the holiday card style. */
    val holiday: HolidayTag? = null,
    /** Expressions this check-in unlocked; only filled in the check-in response itself. */
    val newUnlocks: List<UnlockView> = emptyList(),
    /** Day 7 of a streak with 带小伴回家 switched on (PRD 8.4) */
    val goHome: Boolean = false,
)

@Serializable
data class HolidayTag(val code: String, val name: String)

@Serializable
enum class GreetingKind {
    @SerialName("idle") Idle,
    @SerialName("return") Return,
}

@Serializable
data class GreetingView(val text: String, val mood: Mood, val kind: GreetingKind)

@Serializable
data class TodayResponse(
    val workDay: String,
    val checkin: CheckinView?,
    /** Shown before checking in. */
    val greeting: GreetingView? = null,
    /** Practical info for tonight, once checked in. */
    val tips: List<TipView> = emptyList(),
)
