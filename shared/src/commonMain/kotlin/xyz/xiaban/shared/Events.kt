package xyz.xiaban.shared

import kotlinx.serialization.Serializable

/** Event names from PRD 10; anything else is dropped. */
object EventNames {
    val ALLOWED = setOf(
        "page_view", "onboarding_step", "checkin", "checkin_undo", "reply_shown",
        "card_generate", "card_save", "card_share", "unlock", "guardian_set", "go_home_click",
    )
}

@Serializable
data class EventIn(val name: String, val ts: Long? = null, val props: Map<String, String> = emptyMap())

@Serializable
data class EventBatch(val events: List<EventIn>)

@Serializable
data class ConsentUpdate(
    /** Version of 用户协议 and 隐私政策 the user accepted */
    val termsVersion: String,
    /** Separate consent to storage on servers outside mainland China during the beta */
    val overseas: Boolean = false,
)
