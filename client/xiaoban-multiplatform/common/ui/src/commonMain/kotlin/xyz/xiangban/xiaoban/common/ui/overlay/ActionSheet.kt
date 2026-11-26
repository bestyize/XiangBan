package xyz.xiangban.xiaoban.common.ui.overlay

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.ui.controls.Hairline
import xyz.xiangban.xiaoban.common.ui.theme.Danger
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XIcon
import xyz.xiangban.xiaoban.common.ui.theme.XType

data class ActionSheetItem(
    val label: String,
    val icon: ImageVector? = null,
    val danger: Boolean = false,
    val onClick: () -> Unit,
)

/**
 * 底部操作菜单（与 BottomSheet 完全一致）：
 * 20dp 顶圆角纯白壳、顶部 36×4 拖拽条、轻标题、左对齐 52dp 操作行 + 左右 16dp 内缩 0.5px 细分隔线，
 * 底部以同样 0.5px 细线分隔居中无底色「取消」行。
 */
@Composable
fun ActionSheet(
    items: List<ActionSheetItem>,
    onDismiss: () -> Unit,
    panelModifier: Modifier = Modifier,
    title: String? = "更多操作",
    contextText: String? = null,
    cancelLabel: String = "取消",
) {
    val colors = MaterialTheme.colorScheme
    BottomSheet(
        onDismiss = onDismiss,
        panelModifier = panelModifier,
        flush = true,
        cornerRadius = XDimens.RadiusSheet,
        topPadding = 0.dp,
        showHandle = true,
        cancelLabel = cancelLabel,
    ) {
        if (!title.isNullOrBlank() || !contextText.isNullOrBlank()) {
            Column(Modifier.fillMaxWidth().padding(start = XDimens.Space16, end = XDimens.Space16, top = 6.dp, bottom = 10.dp)) {
                if (!title.isNullOrBlank()) Text(title, style = XType.Section, fontWeight = FontWeight.SemiBold, color = colors.onSurface)
                if (!contextText.isNullOrBlank()) {
                    Text(contextText, Modifier.padding(top = 2.dp), style = XType.Caption, color = colors.onSurfaceVariant)
                }
            }
        }
        items.forEachIndexed { idx, item ->
            if (idx > 0) Hairline(Modifier.padding(horizontal = XDimens.Space16))
            val fg = if (item.danger) Danger else colors.onSurface
            Row(
                Modifier.fillMaxWidth().heightIn(min = XDimens.ActionSheetRowHeight)
                    .clickable(role = Role.Button) {
                        onDismiss()
                        item.onClick()
                    }
                    .padding(horizontal = XDimens.Space16, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                if (item.icon != null) XIcon(item.icon, tint = fg, size = XDimens.IconMedium)
                Text(item.label, style = XType.Body, fontWeight = if (item.danger) FontWeight.SemiBold else FontWeight.Medium, color = fg)
            }
        }
    }
}
