package xyz.xiangban.xiaoban.common.ui.controls

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.ui.theme.Brand
import xyz.xiangban.xiaoban.common.ui.theme.Danger
import xyz.xiangban.xiaoban.common.ui.theme.InfoDot
import xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.Success
import xyz.xiangban.xiaoban.common.ui.theme.WarnDot
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XIcon
import xyz.xiangban.xiaoban.common.ui.theme.XType

/**
 * 筛选胶囊（不使用同色系浅底深字搭配）：
 * 未选为中性浅灰底 + 次要深字，选中为主题色实心底 + 白字。
 */
@Composable
fun ChipPill(
    text: String,
    modifier: Modifier = Modifier,
    icon: ImageVector? = null,
    trailingIcon: ImageVector? = null,
    onClick: (() -> Unit)? = null,
    selected: Boolean = false,
) {
    val extra = LocalExtraColors.current
    val bg = if (selected) Brand else extra.fill
    val fg = if (selected) Color.White else MaterialTheme.colorScheme.onSurface
    Row(
        modifier.height(XDimens.ChipHeight).clip(CircleShape).background(bg)
            .let { if (onClick != null) it.clickable(onClick = onClick) else it }
            .padding(horizontal = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(5.dp),
    ) {
        if (icon != null) XIcon(icon, tint = if (selected) Color.White else MaterialTheme.colorScheme.onSurfaceVariant, size = XDimens.IconSmall)
        Text(text, style = XType.Secondary, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Medium, color = fg, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.widthIn(max = 160.dp))
        if (trailingIcon != null) XIcon(trailingIcon, tint = if (selected) Color.White else MaterialTheme.colorScheme.onSurfaceVariant, size = XDimens.IconSmall)
    }
}

enum class StatusTagTone { Neutral, Success, Warn, Danger, Info }

/**
 * 状态 / 语义标签（不使用彩色糖果底）：
 * 统一采用中性灰底（或白底细描边）+ 可选 6dp 语义色小圆点 + 中性文字。
 */
@Composable
fun StatusTag(
    text: String,
    modifier: Modifier = Modifier,
    tone: StatusTagTone = StatusTagTone.Neutral,
    outline: Boolean = false,
    compact: Boolean = false,
) {
    val extra = LocalExtraColors.current
    val dotColor: Color? = when (tone) {
        StatusTagTone.Neutral -> null
        StatusTagTone.Success -> Success
        StatusTagTone.Warn -> WarnDot
        StatusTagTone.Danger -> Danger
        StatusTagTone.Info -> InfoDot
    }
    val base = modifier.clip(CircleShape).let {
        if (outline) it.background(MaterialTheme.colorScheme.surface).border(1.dp, extra.hairline, CircleShape)
        else it.background(extra.fill)
    }
    Row(
        base.padding(horizontal = if (compact) 8.dp else 10.dp, vertical = if (compact) 2.dp else 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        if (dotColor != null) Box(Modifier.size(6.dp).clip(CircleShape).background(dotColor))
        Text(
            text,
            style = XType.Caption,
            fontWeight = FontWeight.Medium,
            color = if (outline) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

/** 在线用实心点，离线用空心点，不只靠颜色区分。 */
@Composable
fun StatusDot(online: Boolean, label: String, fontSize: Int = 12) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        if (online) {
            Box(Modifier.size(8.dp).clip(CircleShape).background(Success))
        } else {
            Box(Modifier.size(8.dp).clip(CircleShape).border(1.5.dp, MaterialTheme.colorScheme.onSurfaceVariant, CircleShape))
        }
        Text(label, fontSize = fontSize.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}
