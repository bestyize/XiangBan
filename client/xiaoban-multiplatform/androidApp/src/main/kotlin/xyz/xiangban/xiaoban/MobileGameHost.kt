package xyz.xiangban.xiaoban

import android.app.Activity
import android.content.Intent

/** Only opens our private game activity; the game never receives Android objects or file URLs. */
class AndroidMobileGameHost(private val activity: Activity) {
    fun openGame(id: String): Boolean = launch(id, export = false)

    fun exportGame(id: String): Boolean = launch(id, export = true)

    private fun launch(id: String, export: Boolean): Boolean {
        if (!MOBILE_GAME_ID.matches(id) || activity.isFinishing || activity.isDestroyed) return false
        return runCatching {
            activity.startActivity(Intent(activity, MobileGameActivity::class.java).apply {
                putExtra(MobileGameActivity.EXTRA_GAME_ID, id)
                putExtra(MobileGameActivity.EXTRA_EXPORT, export)
            })
            true
        }.getOrDefault(false)
    }
}

internal val MOBILE_GAME_ID = Regex("[a-z0-9][a-z0-9-]{1,39}")
