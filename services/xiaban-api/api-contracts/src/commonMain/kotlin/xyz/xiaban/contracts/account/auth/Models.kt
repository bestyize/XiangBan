package xyz.xiaban.contracts.account.auth

import kotlinx.serialization.Serializable
import xyz.xiaban.contracts.account.profile.ProfileView

/**
 * Exchanges an access token issued by the unified account center for this service's own session.
 * The account's unique, unchangeable username is read from the account center, never sent by the client.
 */
@Serializable
data class AccountCenterLoginRequest(
    val accountToken: String,
    val deviceId: String,
    val mergeGuest: Boolean = false,
    val termsVersion: String? = null,
)

@Serializable
data class LogoutRequest(val deviceId: String)

/** When the signed-in account already existed, this device's guest data moves into it only if requested ([merged]). */
@Serializable
data class BindResponse(val token: String, val merged: Boolean, val profile: ProfileView)
