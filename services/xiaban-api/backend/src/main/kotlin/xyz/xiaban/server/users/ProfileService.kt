package xyz.xiaban.server.users

import io.ktor.http.HttpStatusCode
import java.time.LocalTime
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException
import xyz.xiaban.contracts.account.profile.MetaResponse
import xyz.xiaban.contracts.account.profile.ProfileUpdate
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.companion.personality.Personality
import xyz.xiaban.contracts.companion.personality.PersonalityView
import xyz.xiaban.server.companion.label
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.mail.maskEmail
import xyz.xiaban.server.meta.Cities

val HHMM: DateTimeFormatter = DateTimeFormatter.ofPattern("HH:mm")

class ProfileService(private val users: UserRepository, private val cities: Cities) {

    private val meta = MetaResponse(
        cities = cities.all.map { it.view },
        personalities = listOf(
            PersonalityView(Personality.Toxic, Personality.Toxic.label, "损你一句，再关心你", "再晚一点我都要帮你交社保了。快回来吃饭。"),
            PersonalityView(Personality.Gentle, Personality.Gentle.label, "安慰你，陪着你", "今天辛苦啦，路上慢慢走，我在家等你。"),
            PersonalityView(Personality.Chatty, Personality.Chatty.label, "热闹，碎碎念", "终于下班！今天吃什么？我建议吃两顿！"),
            PersonalityView(Personality.Tsundere, Personality.Tsundere.label, "嘴硬心软", "哼，才不是在等你。……路上注意安全。"),
        ),
    )

    fun meta(): MetaResponse = meta

    fun visitor(timezone: String) = ProfileView(companionName = "小伴", personality = Personality.Toxic, nickname = null,
        cityCode = null, cityName = null, timezone = timezone, workStart = "09:00", workEnd = "18:00", payday = null,
        showCityOnCard = false, inviteCode = "", email = null, recallOptIn = false, termsVersion = null,
        onboarded = false, createdAt = 0, userId = 0, accountType = "guest", avatarUrl = null)

    /** A token whose account was merged away or deleted answers 401, so the client starts over from its device. */
    suspend fun user(userId: Long): User = users.find(userId)
        ?: throw ApiException(HttpStatusCode.Unauthorized, "unauthorized", "登录状态过期了，刷新一下页面")

    suspend fun view(userId: Long): ProfileView = toView(user(userId))

    suspend fun update(userId: Long, req: ProfileUpdate): ProfileView {
        val changes = ProfileChanges(
            companionName = req.companionName?.let(TextRules::companionName),
            personality = req.personality,
            nicknameChanged = req.nickname != null,
            nickname = req.nickname?.let(TextRules::nickname),
            cityCode = req.cityCode?.also { if (cities.find(it) == null) badRequest("没有这个城市", "invalid_city") },
            workStart = req.workStart?.let(::parseTime),
            workEnd = req.workEnd?.let(::parseTime),
            paydayChanged = req.payday != null,
            payday = req.payday?.takeIf { it != 0 }?.also { if (it !in 1..31) badRequest("发薪日是每月 1–31 号", "invalid_payday") },
            showCityOnCard = req.showCityOnCard,
            markOnboarded = req.onboarded == true,
        )
        users.update(userId, changes)
        return view(userId)
    }

    fun toView(user: User) = ProfileView(
        companionName = user.companionName,
        personality = user.personality,
        nickname = user.nickname,
        cityCode = user.cityCode,
        cityName = cities.find(user.cityCode)?.name,
        timezone = user.timezone.id,
        workStart = user.workStart.format(HHMM),
        workEnd = user.workEnd.format(HHMM),
        payday = user.payday,
        showCityOnCard = user.showCityOnCard,
        inviteCode = user.inviteCode,
        email = user.email?.let(::maskEmail),
        recallOptIn = user.recallOptIn,
        termsVersion = user.termsVersion,
        onboarded = user.onboardedAt != null,
        createdAt = user.createdAt.toEpochMilli(),
        userId = user.id,
        accountType = if (user.email != null && user.emailVerifiedAt != null) "registered" else "guest",
        avatarUrl = user.avatarKey?.let { "/api/avatars/$it" },
    )

    private fun parseTime(value: String): LocalTime = try {
        LocalTime.parse(value, HHMM)
    } catch (_: DateTimeParseException) {
        badRequest("时间格式应该是 HH:mm", "invalid_time")
    }
}
