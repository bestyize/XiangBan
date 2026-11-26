package xyz.xiangban.xiaoban.common.ui.overlay

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.ui.controls.Hairline
import xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.OverlayScrim
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XType

/**
 * 统一底部弹层：仅顶角 20dp 圆角、安全区与软键盘避让、可选顶部拖拽指示条与底部细线分隔操作栏：
 * - 仅传 [cancelLabel]：无底色居中「取消」行；
 * - 同时传 [cancelLabel] 与 [confirmLabel]：1:1 等宽中性双操作栏（左灰字取消 | 右墨色加粗确认，无大片主题色块）。
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun BottomSheet(
    onDismiss: () -> Unit,
    panelModifier: Modifier = Modifier,
    flush: Boolean = false,
    cornerRadius: Dp = XDimens.RadiusSheet,
    onBack: () -> Unit = onDismiss,
    topPadding: Dp = if (flush) 16.dp else 20.dp,
    scrim: Color = OverlayScrim,
    showHandle: Boolean = false,
    cancelLabel: String? = null,
    confirmLabel: String? = null,
    onConfirm: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    BackHandler(onBack = onBack)
    val hasFooter = !cancelLabel.isNullOrBlank() || (!confirmLabel.isNullOrBlank() && onConfirm != null)
    Box(
        Modifier.fillMaxSize().background(scrim).clickable(indication = null, interactionSource = null) { onDismiss() }.imePadding(),
        contentAlignment = Alignment.BottomCenter,
    ) {
        Column(
            panelModifier.fillMaxWidth().clip(RoundedCornerShape(topStart = cornerRadius, topEnd = cornerRadius))
                .background(MaterialTheme.colorScheme.surface)
                .clickable(indication = null, interactionSource = null) {}
                .navigationBarsPadding(),
        ) {
            if (showHandle) {
                Box(Modifier.fillMaxWidth().padding(top = 10.dp, bottom = 4.dp), contentAlignment = Alignment.Center) {
                    Box(Modifier.size(width = 36.dp, height = 4.dp).clip(CircleShape).background(LocalExtraColors.current.line))
                }
            }
            Column(
                Modifier.fillMaxWidth().let {
                    if (flush) it.padding(top = if (showHandle) 0.dp else topPadding)
                    else it.padding(start = XDimens.Space16, end = XDimens.Space16, top = if (showHandle) 4.dp else topPadding, bottom = if (hasFooter) 12.dp else XDimens.Space24)
                },
                content = content,
            )
            if (hasFooter) {
                Hairline(Modifier.padding(horizontal = XDimens.Space16))
                Row(Modifier.fillMaxWidth().heightIn(min = 54.dp), verticalAlignment = Alignment.CenterVertically) {
                    if (!cancelLabel.isNullOrBlank()) {
                        Box(
                            Modifier.weight(1f).heightIn(min = 54.dp)
                                .clickable(role = Role.Button, onClick = onDismiss)
                                .padding(horizontal = XDimens.Space16, vertical = 12.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(cancelLabel, style = XType.Body, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                    if (!cancelLabel.isNullOrBlank() && !confirmLabel.isNullOrBlank() && onConfirm != null) {
                        Box(Modifier.width(XDimens.Hairline).height(22.dp).background(LocalExtraColors.current.hairline))
                    }
                    if (!confirmLabel.isNullOrBlank() && onConfirm != null) {
                        Box(
                            Modifier.weight(1f).heightIn(min = 54.dp)
                                .clickable(role = Role.Button, onClick = onConfirm)
                                .padding(horizontal = XDimens.Space16, vertical = 12.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(confirmLabel, style = XType.Body, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
                        }
                    }
                }
            }
        }
    }
}
