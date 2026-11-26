package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.ui.overlay.CardDialog as UiCardDialog
import xyz.xiangban.xiaoban.common.ui.overlay.DialogCardClose as UiDialogCardClose
import xyz.xiangban.xiaoban.common.ui.overlay.DialogChoice as UiDialogChoice
import xyz.xiangban.xiaoban.common.ui.overlay.DialogNotice as UiDialogNotice
import xyz.xiangban.xiaoban.common.ui.overlay.dialogCardSurface as uiDialogCardSurface

internal fun Modifier.dialogCardSurface(surface: Color): Modifier = uiDialogCardSurface(surface)

@Composable
internal fun BoxScope.DialogCardClose(onDismiss: () -> Unit, enabled: Boolean, fill: Color, ink: Color) =
    UiDialogCardClose(onDismiss, enabled, fill, ink)

/** 社区业务居中卡片弹窗：底层委托给 `:common:ui` 的 [UiCardDialog]。 */
@Composable
internal fun CommunityCardDialog(
    title: String,
    confirmLabel: String?,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
    icon: ImageVector = XIcons.Info,
    iconColor: Color = Color(0xFF293036),
    description: String? = null,
    dismissLabel: String = "取消",
    confirmEnabled: Boolean = true,
    dismissEnabled: Boolean = true,
    danger: Boolean = false,
    actionIcon: ImageVector? = null,
    cover: (@Composable () -> Unit)? = null,
    content: (@Composable () -> Unit)? = null,
) = UiCardDialog(
    title = title,
    confirmLabel = confirmLabel,
    onConfirm = onConfirm,
    onDismiss = onDismiss,
    icon = icon,
    iconColor = iconColor,
    description = description,
    dismissLabel = dismissLabel,
    confirmEnabled = confirmEnabled,
    dismissEnabled = dismissEnabled,
    danger = danger,
    actionIcon = actionIcon,
    cover = cover,
    content = content,
)

@Composable
internal fun DialogNotice(text: String, error: Boolean = false) = UiDialogNotice(text, error)

@Composable
internal fun DialogChoice(
    label: String,
    selected: Boolean,
    onSelect: () -> Unit,
    modifier: Modifier = Modifier,
    caption: String? = null,
    icon: ImageVector? = null,
    enabled: Boolean = true,
) = UiDialogChoice(label, selected, onSelect, modifier, caption, icon, enabled)

// 以下为业务定制插画头图，保留在 business:community-native。

@Composable
internal fun AttachmentDialogCover(filename: String) {
    val extension = filename.substringAfterLast('.', "").takeIf { it.length in 1..6 }?.uppercase().orEmpty()
    Box(Modifier.fillMaxWidth().height(136.dp).background(Brush.linearGradient(listOf(Color(0xFF343E70), Color(0xFF7265B7), Color(0xFFC992C0)))),
        contentAlignment = Alignment.Center) {
        Box(Modifier.size(93.dp, 70.dp).rotate(-7f)) {
            Box(Modifier.align(Alignment.TopStart).size(44.dp, 14.dp).background(Color(0xFFB1A5E5), RoundedCornerShape(7.dp)))
            Box(Modifier.align(Alignment.Center).size(87.dp, 55.dp).rotate(6f).background(Color(0xFFF6EDFF), RoundedCornerShape(5.dp)))
            Box(Modifier.align(Alignment.BottomCenter).size(93.dp, 50.dp).background(Color(0xFFA99BD3), RoundedCornerShape(8.dp))
                .border(1.dp, Color(0xFFD7CEED), RoundedCornerShape(8.dp)).padding(7.dp), contentAlignment = Alignment.BottomStart) {
                if (extension.isNotEmpty()) Text(extension, style = XType.Caption, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis)
                else XIcon(XIcons.File, size = 16.dp, tint = Color.White)
            }
        }
    }
}

@Composable
internal fun AppUpdateDialogCover() {
    Box(Modifier.fillMaxWidth().height(136.dp).background(Brush.linearGradient(listOf(Color(0xFF732838), Color(0xFFC95872), Color(0xFFF8B8B3)))),
        contentAlignment = Alignment.Center) {
        val shape = RoundedCornerShape(21.dp)
        Box(Modifier.size(72.dp).rotate(-9f).background(Color.White.copy(alpha = .2f), shape)
            .border(1.dp, Color.White.copy(alpha = .55f), shape), contentAlignment = Alignment.Center) {
            XIcon(XIcons.Sparkles, size = 44.dp, tint = Color.White)
        }
    }
}
