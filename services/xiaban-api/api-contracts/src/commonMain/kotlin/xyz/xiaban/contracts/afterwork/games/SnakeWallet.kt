package xyz.xiaban.contracts.afterwork.games

import kotlinx.serialization.Serializable

object SnakeWalletApiPaths {
    const val WALLET = "/api/games/snake-arena/wallet"
    const val ACTIONS = "$WALLET/actions"
}

@Serializable data class SnakePrices(val shield: Int = 20, val revive: Int = 30)
@Serializable data class SnakeWalletView(
    val coins: Int,
    val shields: Int,
    val startShieldSeconds: Int,
    val shareAvailable: Boolean,
    val prices: SnakePrices = SnakePrices(),
)
@Serializable data class SnakeWalletAction(val action: String, val requestId: String, val sessionId: String? = null)
@Serializable data class SnakeWalletResult(val wallet: SnakeWalletView, val coinsEarned: Int = 0, val protectionSeconds: Int = 0)
