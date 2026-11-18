package xyz.xiangban.xiaoban

import android.app.Activity
import android.os.Bundle
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames

/** Debug-only entry for exercising the real isolated runtime in Android device tests. */
class BuiltinGameSmokeActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val id = intent.getStringExtra("id")
        if (id != null && id in BuiltinMobileGames.ids && !BuiltinMobileGames.isBundledTool(id)) {
            startActivity(RuntimeV2Activity.launchIntent(this).putExtra(RuntimeV2Activity.EXTRA_GAME_ID, id))
        }
        finish()
    }
}
