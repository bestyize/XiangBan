package xyz.xiaban.shared

import kotlinx.serialization.Serializable

@Serializable
data class ApiError(val error: String, val message: String)

@Serializable
data class HealthResponse(val status: String, val db: Boolean, val redis: Boolean)
