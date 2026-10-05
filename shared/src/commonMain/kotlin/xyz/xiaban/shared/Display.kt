package xyz.xiaban.shared

import kotlinx.serialization.Serializable

@Serializable
data class DisplayItem(val id: String, val visible: Boolean, val name: String = "")

/** Public navigation/catalog presentation only; does not delete records or disable deep links. */
@Serializable
data class DisplaySettings(
    val chatTabVisible: Boolean,
    val games: List<DisplayItem> = emptyList(),
    val quizzes: List<DisplayItem> = emptyList(),
)
