package xyz.xiaban.shared

import kotlinx.serialization.Serializable

/** Event names from PRD 10 plus the ones the admin dashboard needs; anything else is dropped. */
object EventNames {
    val ALLOWED = setOf(
        "page_view", "onboarding_step", "checkin", "checkin_undo", "reply_shown",
        "card_generate", "card_save", "card_share", "unlock", "guardian_set", "go_home_click",
        /** props: game */
        "game_start",
        /** props: game, score */
        "game_finish",
        /** props: quiz, mode = new | resume | review | retake */
        "quiz_start",
        /** props: quiz */
        "quiz_complete",
    )

    /** Guests may only report what they can actually do without an account. */
    val GUEST_ALLOWED = setOf(
        "page_view", "onboarding_step", "checkin", "reply_shown", "card_generate", "card_save", "card_share",
        "game_start", "game_finish", "quiz_start", "quiz_complete",
    )
}

@Serializable
data class EventIn(val name: String, val ts: Long? = null, val props: Map<String, String> = emptyMap())

/** [deviceId] identifies a guest; it is hashed before storage and ignored for signed-in users. */
@Serializable
data class EventBatch(val events: List<EventIn>, val deviceId: String? = null)

@Serializable
data class ConsentUpdate(
    /** Version of 用户协议 and 隐私政策 the user accepted */
    val termsVersion: String,
    /** Separate consent to storage on servers outside mainland China during the beta */
    val overseas: Boolean = false,
)
