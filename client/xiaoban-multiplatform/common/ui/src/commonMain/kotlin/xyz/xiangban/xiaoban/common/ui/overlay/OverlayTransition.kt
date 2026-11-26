package xyz.xiangban.xiaoban.common.ui.overlay

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import xyz.xiangban.xiaoban.common.ui.theme.XMotion

/**
 * 统一浮层过渡容器：遮罩淡入 200ms / 淡出 160ms，面板按方向滑入减速、滑出更快收束。
 * 退出动画完全结束后才从组合树移除内容。
 */
@Composable
fun OverlayTransition(visible: Boolean, direction: String = "bottom", content: @Composable (Modifier) -> Unit) {
    AnimatedVisibility(
        visible = visible,
        enter = fadeIn(XMotion.scrimEnterSpec()),
        exit = fadeOut(XMotion.scrimExitSpec()),
    ) {
        val panel = when (direction) {
            "left" -> Modifier.animateEnterExit(
                enter = slideInHorizontally(XMotion.drawerEnterSpec()) { -it },
                exit = slideOutHorizontally(XMotion.drawerExitSpec()) { -it },
            )
            "right" -> Modifier.animateEnterExit(
                enter = slideInHorizontally(XMotion.drawerEnterSpec()) { it },
                exit = slideOutHorizontally(XMotion.drawerExitSpec()) { it },
            )
            "top" -> Modifier.animateEnterExit(
                enter = slideInVertically(XMotion.sheetEnterSpec()) { -it },
                exit = slideOutVertically(XMotion.sheetExitSpec()) { -it },
            )
            "bottom" -> Modifier.animateEnterExit(
                enter = slideInVertically(XMotion.sheetEnterSpec()) { it },
                exit = slideOutVertically(XMotion.sheetExitSpec()) { it },
            )
            else -> Modifier
        }
        content(panel)
    }
}
