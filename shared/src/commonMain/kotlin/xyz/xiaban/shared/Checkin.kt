package xyz.xiaban.shared

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
enum class Mood {
    @SerialName("idle") Idle,
    @SerialName("smug") Smug,
    @SerialName("happy") Happy,
    @SerialName("welcome") Welcome,
    @SerialName("heartache") Heartache,
    @SerialName("sleepy") Sleepy,
    @SerialName("worried") Worried,
    @SerialName("sparkle") Sparkle,
    @SerialName("shocked") Shocked,
    @SerialName("aggrieved") Aggrieved,
    @SerialName("excited") Excited,
    @SerialName("care") Care,
}

/** [code] mirrors the serial name and is what the database stores. */
@Serializable
enum class TitleCode(val code: String, val label: String, val rare: Boolean, val mood: Mood) {
    @SerialName("exact_on_time") ExactOnTime("exact_on_time", "准点之神", true, Mood.Sparkle),
    @SerialName("weekend_work") WeekendWork("weekend_work", "周末还上班？？", true, Mood.Shocked),
    @SerialName("borrowed_weekend") BorrowedWeekend("borrowed_weekend", "被借走的周末", true, Mood.Aggrieved),
    @SerialName("holiday_countdown") HolidayCountdown("holiday_countdown", "放假倒计时", true, Mood.Excited),
    @SerialName("early_leaver") EarlyLeaver("early_leaver", "早退王者", false, Mood.Smug),
    @SerialName("on_time") OnTime("on_time", "准点战神", false, Mood.Happy),
    @SerialName("normal_human") NormalHuman("normal_human", "正常人类", false, Mood.Welcome),
    @SerialName("overtime_hero") OvertimeHero("overtime_hero", "加班侠", false, Mood.Heartache),
    @SerialName("night_owl") NightOwl("night_owl", "夜行动物", false, Mood.Sleepy),
    @SerialName("immortal") Immortal("immortal", "修仙者", false, Mood.Worried);

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
)

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
