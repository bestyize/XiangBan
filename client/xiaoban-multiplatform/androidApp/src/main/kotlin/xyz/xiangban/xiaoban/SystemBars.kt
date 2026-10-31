package xyz.xiangban.xiaoban

import android.app.Activity
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat

/**
 * 整个应用不显示底部的系统手势条（导航栏）：从屏幕底边上滑会临时露出来，再滑一次才回桌面，过一会儿自己收起。
 * 状态栏照常显示。系统在弹窗、切回应用后可能把导航栏放出来，所以每次拿到焦点都再藏一次。
 */
fun Activity.hideNavigationBar() {
    val controller = WindowCompat.getInsetsController(window, window.decorView)
    controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
    controller.hide(WindowInsetsCompat.Type.navigationBars())
}
