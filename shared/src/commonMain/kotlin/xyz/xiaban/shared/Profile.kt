package xyz.xiaban.shared

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
enum class Personality(val label: String) {
    @SerialName("toxic") Toxic("毒舌心软"),
    @SerialName("gentle") Gentle("温柔治愈"),
    @SerialName("chatty") Chatty("元气话痨"),
    @SerialName("tsundere") Tsundere("高冷傲娇"),
}

@Serializable
data class SessionRequest(
    val deviceId: String,
    val timezone: String? = null,
    /** Invite code from a share-card QR link (`inv`), recorded for brand-new accounts only. */
    val ref: String? = null,
    /** Entry point such as `card` (`from`). */
    val from: String? = null,
)

@Serializable
data class SessionResponse(val token: String, val deviceId: String, val profile: ProfileView)

@Serializable
data class ProfileView(
    val companionName: String,
    val personality: Personality,
    val nickname: String?,
    val cityCode: String?,
    val cityName: String?,
    val timezone: String,
    /** "HH:mm" */
    val workStart: String,
    /** "HH:mm" */
    val workEnd: String,
    /** Day of month the salary arrives, for the payday countdown */
    val payday: Int?,
    val showCityOnCard: Boolean,
    /** Goes into the share-card QR link instead of the user id. */
    val inviteCode: String,
    /** Bound email, masked; null while the account is anonymous */
    val email: String?,
    val recallOptIn: Boolean,
    /** Version of the terms the user accepted; ask again when it differs from the current one */
    val termsVersion: String?,
    val onboarded: Boolean,
    val createdAt: Long,
)

/** Omitted fields stay unchanged; an empty [nickname] or a [payday] of 0 clears it. */
@Serializable
data class ProfileUpdate(
    val companionName: String? = null,
    val personality: Personality? = null,
    val nickname: String? = null,
    val cityCode: String? = null,
    val workStart: String? = null,
    val workEnd: String? = null,
    val payday: Int? = null,
    val showCityOnCard: Boolean? = null,
    val onboarded: Boolean? = null,
)

@Serializable
data class CityView(val code: String, val name: String, val province: String)

@Serializable
data class PersonalityView(val code: Personality, val name: String, val desc: String, val sample: String)

@Serializable
data class MetaResponse(val cities: List<CityView>, val personalities: List<PersonalityView>)
