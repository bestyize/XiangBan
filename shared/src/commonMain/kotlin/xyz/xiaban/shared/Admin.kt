package xyz.xiaban.shared

import kotlinx.serialization.Serializable

@Serializable
data class FlagsView(val llmEnabled: Boolean)

@Serializable
data class FlagsUpdate(val llmEnabled: Boolean? = null)

@Serializable
data class ReportRequest(val checkinId: String, val reason: String? = null)

@Serializable
data class ReportView(val id: Long, val copyId: String, val text: String, val reason: String?, val createdAt: Long)

@Serializable
data class OkResponse(val ok: Boolean = true)
