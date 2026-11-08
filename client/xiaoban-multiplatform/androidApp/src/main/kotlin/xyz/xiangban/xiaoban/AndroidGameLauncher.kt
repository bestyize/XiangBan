package xyz.xiangban.xiaoban

import android.app.Activity

/** 内置作品与手机里自己做的作品都从这里进入协议 2 宿主。 */
internal object AndroidGameLauncher {
    fun open(activity: Activity, id: String): Boolean {
        if (!MOBILE_GAME_ID.matches(id) || activity.isFinishing || activity.isDestroyed || !RuntimeV2Activity.supported(activity)) return false
        val builtin = bundledGameRoot(id) != null
        val context = activity.applicationContext
        val opened: (() -> Unit)? = if (id == RUNTIME_V2_FIXTURE) null else ({ AndroidGameHistory.opened(context, id) })
        return runCatching {
            activity.startActivity(RuntimeV2Activity.launchIntent(activity, opened).apply {
                putExtra(RuntimeV2Activity.EXTRA_GAME_ID, id)
                if (!builtin) {
                    putExtra(RuntimeV2Activity.EXTRA_WORKSPACE, true)
                    putExtra(RuntimeV2Activity.EXTRA_NETWORK_DISABLED, false)
                }
            })
            true
        }.getOrDefault(false)
    }
}

internal val MOBILE_GAME_ID = Regex("[a-z0-9][a-z0-9-]{1,39}")
