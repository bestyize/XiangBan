package xyz.xiaban.contracts.admin.reports

import kotlinx.serialization.Serializable

@Serializable
data class ReportRequest(val checkinId: String, val reason: String? = null)

@Serializable
data class ReportView(val id: Long, val copyId: String, val text: String, val reason: String?, val createdAt: Long)
