package xyz.xiaban.shared

import kotlinx.serialization.Serializable

@Serializable
data class EmailCodeRequest(val email: String)

@Serializable
data class BindEmailRequest(val email: String, val code: String, val username: String? = null)

@Serializable
data class EmailLoginRequest(
    val email: String,
    val code: String,
    val deviceId: String,
    val mergeGuest: Boolean = false,
    val username: String? = null,
    /** Accepted on the login page; optional for older clients. */
    val termsVersion: String? = null,
)

@Serializable
data class LogoutRequest(val deviceId: String)

/** When the email already belongs to another account, this device's data moves into it ([merged]). */
@Serializable
data class BindResponse(val token: String, val merged: Boolean, val profile: ProfileView)

@Serializable
data class SubscriptionUpdate(val enabled: Boolean)

@Serializable
data class GuardianView(
    val contactName: String,
    val contactEmailMasked: String,
    /** How the contact knows the user, used in the email to them */
    val selfName: String,
    val thresholdDays: Int,
    val paused: Boolean,
    /** When the user gets a heads-up, and when the contact would be told; null when nothing is scheduled */
    val nextRemindAt: Long?,
    val nextNotifyAt: Long?,
)

@Serializable
data class GuardianUpdate(
    val contactName: String,
    val contactEmail: String,
    val selfName: String,
    val thresholdDays: Int = 2,
    /** Agrees to the terms of the safety check */
    val consent: Boolean,
    /** Confirms the contact has been told in person */
    val informedContact: Boolean,
)

@Serializable
data class GuardianPause(val paused: Boolean)
