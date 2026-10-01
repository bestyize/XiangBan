package xyz.xiaban.server.users

import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.notFound
import xyz.xiaban.server.meta.Cities
import xyz.xiaban.shared.MetaResponse
import xyz.xiaban.shared.Personality
import xyz.xiaban.shared.PersonalityView
import xyz.xiaban.shared.ProfileUpdate
import xyz.xiaban.shared.ProfileView
import java.time.LocalTime
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException

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

    suspend fun user(userId: Long): User = users.find(userId) ?: notFound("找不到这个账号", "user_not_found")

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
        onboarded = user.onboardedAt != null,
        createdAt = user.createdAt.toEpochMilli(),
    )

    private fun parseTime(value: String): LocalTime = try {
        LocalTime.parse(value, HHMM)
    } catch (_: DateTimeParseException) {
        badRequest("时间格式应该是 HH:mm", "invalid_time")
    }
}
