package xyz.xiaban.contracts.account.consent

import kotlinx.serialization.Serializable

@Serializable
data class ConsentUpdate(
    /** Version of 用户协议 and 隐私政策 the user accepted */
    val termsVersion: String,
    /** Separate consent to storage on servers outside mainland China during the beta */
    val overseas: Boolean = false,
)
