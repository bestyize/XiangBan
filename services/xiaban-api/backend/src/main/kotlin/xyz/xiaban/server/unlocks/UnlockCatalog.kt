package xyz.xiaban.server.unlocks

import xyz.xiaban.contracts.afterwork.checkin.TitleCode

data class UnlockDef(val id: String, val name: String, val hint: String)

/** The 30 expressions the web can unlock (PRD 5.8), in the order the collection page shows them. */
object UnlockCatalog {
    const val APP_TOTAL = 75

    val all: List<UnlockDef> = listOf(
        UnlockDef("hello", "初次见面", "第一次打卡"),
        UnlockDef("days_3", "三天之约", "累计打卡 3 天"),
        UnlockDef("days_7", "一周老友", "累计打卡 7 天"),
        UnlockDef("days_14", "半月相伴", "累计打卡 14 天"),
        UnlockDef("days_30", "满月", "累计打卡 30 天"),
        UnlockDef("days_60", "形影不离", "累计打卡 60 天"),
        UnlockDef("days_100", "百日陪伴", "累计打卡 100 天"),
        UnlockDef("title_early_leaver", "得意", "第一次拿到「早退王者」"),
        UnlockDef("title_on_time", "开心", "第一次拿到「准点战神」"),
        UnlockDef("title_normal_human", "欢迎", "第一次拿到「正常人类」"),
        UnlockDef("title_overtime_hero", "心疼", "第一次拿到「加班侠」"),
        UnlockDef("title_night_owl", "犯困", "第一次拿到「夜行动物」"),
        UnlockDef("title_immortal", "担心", "第一次拿到「修仙者」"),
        UnlockDef("title_exact_on_time", "闪闪发光", "拿到稀有称号「准点之神」"),
        UnlockDef("title_weekend_work", "震惊", "拿到稀有称号「周末还上班？？」"),
        UnlockDef("title_borrowed_weekend", "委屈", "拿到稀有称号「被借走的周末」"),
        UnlockDef("title_holiday_countdown", "期待", "拿到稀有称号「放假倒计时」"),
        UnlockDef("streak_3", "三连", "连续打卡 3 天"),
        UnlockDef("streak_5", "满勤", "连续打卡 5 天"),
        UnlockDef("streak_10", "坚持", "连续打卡 10 天"),
        UnlockDef("monday", "周一综合症", "在周一打卡"),
        UnlockDef("friday", "周五万岁", "在周五打卡"),
        UnlockDef("rainy", "带伞", "下雨天打卡"),
        UnlockDef("spring_festival", "红包", "春节假期里打卡"),
        UnlockDef("mid_autumn", "赏月", "中秋假期里打卡"),
        UnlockDef("national_day", "出游", "国庆假期里打卡"),
        UnlockDef("comeback", "你回来啦", "隔了一周以上回来打卡"),
        UnlockDef("redemption", "重新做人", "连续很晚下班之后，准点下班一次"),
        UnlockDef("named", "有名字了", "完成初次见面"),
        UnlockDef("guardian", "安心", "开启平安守护"),
    )

    private val byId = all.associateBy { it.id }

    fun find(id: String): UnlockDef? = byId[id]

    fun forTitle(title: TitleCode): String = "title_${title.code}"
}
