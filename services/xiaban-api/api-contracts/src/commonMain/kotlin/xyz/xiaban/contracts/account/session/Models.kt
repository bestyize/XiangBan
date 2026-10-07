package xyz.xiaban.contracts.account.session

import kotlinx.serialization.Serializable
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.common.Features

@Serializable
data class SessionRequest(
    val deviceId: String,
    val timezone: String? = null,
    /** Invite code from a share-card QR link (`inv`), recorded for brand-new accounts only. */
    val ref: String? = null,
    /** Entry point such as `card` (`from`). */
    val from: String? = null,
    /** Allocate a persistent visitor only after an explicit user action. */
    val materialize: Boolean = false,
)

@Serializable
data class SessionResponse(val token: String, val deviceId: String, val profile: ProfileView, val features: Features)
