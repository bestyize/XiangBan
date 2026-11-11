package xyz.xiangban.xiaoban

import android.view.ViewGroup
import android.widget.FrameLayout
import androidx.activity.ComponentActivity
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.platform.ViewCompositionStrategy
import xyz.xiangban.xiaoban.business.communitynative.XiaobanConfirmDialog
import xyz.xiangban.xiaoban.business.communitynative.XiaobanTheme

/**
 * 兼容使用原生 Activity / WebView 的页面。
 * 弹窗的 UI 依然取自 Compose DialogKit，不再使用系统 AlertDialog.Builder。
 * 返回可在 Activity 取消、协程取消时安全调用的关闭函数。
 */
internal fun ComponentActivity.showXiaobanConfirmDialog(
    title: String,
    message: String,
    confirmLabel: String,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit = {},
    destructive: Boolean = false,
    cancelLabel: String = "取消",
): () -> Unit {
    val root = findViewById<ViewGroup>(android.R.id.content)
    val composeView = ComposeView(this)
    composeView.setViewCompositionStrategy(ViewCompositionStrategy.DisposeOnDetachedFromWindow)

    fun close() {
        (composeView.parent as? ViewGroup)?.removeView(composeView)
    }

    composeView.setContent {
        XiaobanTheme {
            XiaobanConfirmDialog(
                title = title,
                message = message,
                confirmLabel = confirmLabel,
                destructive = destructive,
                cancelLabel = cancelLabel,
                onConfirm = { close(); onConfirm() },
                onDismiss = { close(); onDismiss() },
            )
        }
    }
    // Compose Dialog 自己拥有窗口；锚点视图只负责提供正确的 Activity 生命周期。
    root.addView(composeView, FrameLayout.LayoutParams(1, 1))
    return ::close
}
