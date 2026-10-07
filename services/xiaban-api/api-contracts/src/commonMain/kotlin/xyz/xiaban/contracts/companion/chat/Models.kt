package xyz.xiaban.contracts.companion.chat

import kotlinx.serialization.Serializable

@Serializable
data class ChatMessageView(
    val id: Long? = null,
    val role: String,
    val content: String,
    val createdAt: Long? = null,
)

@Serializable
data class ChatHistoryResponse(
    val messages: List<ChatMessageView>,
)

@Serializable
data class ChatStreamRequest(
    val message: String,
    val guestHistory: List<ChatMessageView> = emptyList(),
)

/** Text to speak aloud; the server picks the voice from the user's companion personality. */
@Serializable
data class ChatSpeakRequest(val text: String)
