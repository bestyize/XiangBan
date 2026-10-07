package xyz.xiaban.contracts.common

import kotlinx.serialization.Serializable

@Serializable
data class ApiError(val error: String, val message: String)

@Serializable
data class HealthResponse(val status: String, val db: Boolean, val redis: Boolean)

@Serializable
data class OkResponse(val ok: Boolean = true)

/** Switches the web page reads at start-up. */
@Serializable
data class Features(val goHome: Boolean, val appDownloadUrl: String?)
