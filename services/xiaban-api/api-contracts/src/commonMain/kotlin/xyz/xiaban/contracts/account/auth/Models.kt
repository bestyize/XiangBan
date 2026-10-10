package xyz.xiaban.contracts.account.auth

import kotlinx.serialization.Serializable
import xyz.xiaban.contracts.account.profile.ProfileView

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

/** Exchanges an access token issued by the unified account center for this service's own session. */
@Serializable
data class AccountCenterLoginRequest(
    val accountToken: String,
    val deviceId: String,
    val mergeGuest: Boolean = false,
    val username: String? = null,
    val termsVersion: String? = null,
)

@Serializable
data class LogoutRequest(val deviceId: String)

/** When the email already belongs to another account, this device's data moves into it ([merged]). */
@Serializable
data class BindResponse(val token: String, val merged: Boolean, val profile: ProfileView)
