package xyz.xiangban.xiaoban.common.ui.overlay

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import xyz.xiangban.xiaoban.common.ui.theme.Danger
import xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.XMotion
import xyz.xiangban.xiaoban.common.ui.theme.XType

/**
 * 底部轻量悬浮提示：默认白底细框轻阴影，错误态固定危险红底白字；
 * 入场 180ms 上浮淡入，停留 2.4s 后 150ms 淡出。
 */
@Composable
fun Toast(
    text: String?,
    onGone: () -> Unit,
    error: Boolean = false,
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null,
) {
    LaunchedEffect(text) {
        if (text != null) {
            delay(XMotion.ToastHoldMs)
            onGone()
        }
    }
    Box(
        Modifier.fillMaxSize().navigationBarsPadding().padding(horizontal = 24.dp, vertical = 96.dp),
        contentAlignment = Alignment.BottomCenter,
    ) {
        AnimatedVisibility(
            visible = text != null,
            enter = fadeIn(tween(XMotion.ToastEnter, easing = XMotion.StandardEasing)) +
                slideInVertically(tween(XMotion.ToastEnter, easing = XMotion.StandardEasing)) { it / 3 },
            exit = fadeOut(tween(XMotion.ToastExit, easing = XMotion.ExitEasing)) +
                slideOutVertically(tween(XMotion.ToastExit, easing = XMotion.ExitEasing)) { it / 4 },
        ) {
            val colors = MaterialTheme.colorScheme
            val extra = LocalExtraColors.current
            val bg = if (error) Danger else colors.surface
            val fg = if (error) Color.White else colors.onSurface
            Row(
                Modifier.shadow(8.dp, CircleShape, ambientColor = Color.Black.copy(alpha = 0.16f), spotColor = Color.Black.copy(alpha = 0.16f))
                    .clip(CircleShape)
                    .background(bg)
                    .then(if (error) Modifier else Modifier.border(1.dp, extra.hairline, CircleShape))
                    .padding(horizontal = 16.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Text(text.orEmpty(), color = fg, style = XType.Secondary, fontWeight = FontWeight.Medium)
                if (!actionLabel.isNullOrBlank() && onAction != null) {
                    Text(
                        actionLabel,
                        Modifier.clickable {
                            onAction()
                            onGone()
                        },
                        color = if (error) Color.White else extra.brandInk,
                        style = XType.Secondary,
                        fontWeight = FontWeight.SemiBold,
                        textDecoration = TextDecoration.Underline,
                    )
                }
            }
        }
    }
}
