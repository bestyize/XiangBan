package xyz.xiaban.contracts.afterwork.unlocks

import kotlinx.serialization.Serializable

@Serializable
data class UnlockView(val id: String, val name: String, val hint: String, val unlockedAt: Long?)

@Serializable
data class UnlocksResponse(
    /** The full set, completed in the 相伴 App. */
    val appTotal: Int,
    /** How many the web can unlock. */
    val webTotal: Int,
    val unlockedCount: Int,
    val items: List<UnlockView>,
)
