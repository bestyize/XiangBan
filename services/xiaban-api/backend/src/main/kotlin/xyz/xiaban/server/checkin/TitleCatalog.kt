package xyz.xiaban.server.checkin

import xyz.xiaban.contracts.afterwork.checkin.TitleCode
import xyz.xiaban.contracts.companion.personality.Mood

data class TitleDefinition(val label: String, val rare: Boolean, val mood: Mood)

/** Server-owned presentation and rarity policy; API enums only carry wire codes. */
object TitleCatalog {
    private val definitions = mapOf(
        TitleCode.ExactOnTime to TitleDefinition("准点之神", true, Mood.Sparkle),
        TitleCode.WeekendWork to TitleDefinition("周末还上班？？", true, Mood.Shocked),
        TitleCode.BorrowedWeekend to TitleDefinition("被借走的周末", true, Mood.Aggrieved),
        TitleCode.HolidayCountdown to TitleDefinition("放假倒计时", true, Mood.Excited),
        TitleCode.EarlyLeaver to TitleDefinition("早退王者", false, Mood.Smug),
        TitleCode.OnTime to TitleDefinition("准点战神", false, Mood.Happy),
        TitleCode.NormalHuman to TitleDefinition("正常人类", false, Mood.Welcome),
        TitleCode.OvertimeHero to TitleDefinition("加班侠", false, Mood.Heartache),
        TitleCode.NightOwl to TitleDefinition("夜行动物", false, Mood.Sleepy),
        TitleCode.Immortal to TitleDefinition("修仙者", false, Mood.Worried),
    )
    fun definition(code: TitleCode): TitleDefinition = requireNotNull(definitions[code])
}

val TitleCode.label: String get() = TitleCatalog.definition(this).label
val TitleCode.rare: Boolean get() = TitleCatalog.definition(this).rare
val TitleCode.mood: Mood get() = TitleCatalog.definition(this).mood
