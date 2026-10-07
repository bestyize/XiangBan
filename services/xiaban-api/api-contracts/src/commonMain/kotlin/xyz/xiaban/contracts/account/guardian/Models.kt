package xyz.xiaban.contracts.account.guardian

import kotlinx.serialization.Serializable

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
