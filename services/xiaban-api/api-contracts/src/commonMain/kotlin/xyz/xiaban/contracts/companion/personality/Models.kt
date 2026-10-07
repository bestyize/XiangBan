package xyz.xiaban.contracts.companion.personality

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
enum class Mood {
    @SerialName("idle") Idle,
    @SerialName("smug") Smug,
    @SerialName("happy") Happy,
    @SerialName("welcome") Welcome,
    @SerialName("heartache") Heartache,
    @SerialName("sleepy") Sleepy,
    @SerialName("worried") Worried,
    @SerialName("sparkle") Sparkle,
    @SerialName("shocked") Shocked,
    @SerialName("aggrieved") Aggrieved,
    @SerialName("excited") Excited,
    @SerialName("care") Care,
}

@Serializable
enum class Personality {
    @SerialName("toxic") Toxic,
    @SerialName("gentle") Gentle,
    @SerialName("chatty") Chatty,
    @SerialName("tsundere") Tsundere,
}

@Serializable
data class PersonalityView(val code: Personality, val name: String, val desc: String, val sample: String)
