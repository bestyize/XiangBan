package xyz.xiaban.contracts.account.profile

import kotlinx.serialization.Serializable
import xyz.xiaban.contracts.companion.personality.Personality
import xyz.xiaban.contracts.companion.personality.PersonalityView

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
    val userId: Long = 0,
    /** Server-confirmed identity; an anonymous session is still a guest even though it has a JWT. */
    val accountType: String = "guest",
    val avatarUrl: String? = null,
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
data class MetaResponse(val cities: List<CityView>, val personalities: List<PersonalityView>)

@Serializable
data class SubscriptionUpdate(val enabled: Boolean)
