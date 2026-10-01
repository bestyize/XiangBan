package xyz.xiaban.shared

import kotlinx.serialization.Serializable

@Serializable
data class FlagsView(val llmEnabled: Boolean, val goHomeEnabled: Boolean)

@Serializable
data class FlagsUpdate(val llmEnabled: Boolean? = null, val goHomeEnabled: Boolean? = null)

/** Switches the web page reads at start-up. */
@Serializable
data class Features(val goHome: Boolean, val appDownloadUrl: String?)

@Serializable
data class ReportRequest(val checkinId: String, val reason: String? = null)

@Serializable
data class ReportView(val id: Long, val copyId: String, val text: String, val reason: String?, val createdAt: Long)

@Serializable
data class OkResponse(val ok: Boolean = true)
