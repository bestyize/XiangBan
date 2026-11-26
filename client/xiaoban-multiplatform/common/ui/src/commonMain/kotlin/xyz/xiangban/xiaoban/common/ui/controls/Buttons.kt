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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.ui.theme.Brand
import xyz.xiangban.xiaoban.common.ui.theme.Danger
import xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XIcon
import xyz.xiangban.xiaoban.common.ui.theme.XType

/** 主操作胶囊按钮：主题色实心底 + 白字（不使用同色系浅底）。 */
@Composable
fun PillButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    height: Dp = XDimens.ButtonPrimaryHeight,
    icon: ImageVector? = null,
) {
    Row(
        modifier.height(height).clip(CircleShape).background(if (enabled) Brand else LocalExtraColors.current.line)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick).padding(horizontal = 20.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (icon != null) XIcon(icon, tint = Color.White, size = XDimens.IconMedium)
        Text(text, color = Color.White, style = if (height >= 44.dp) XType.Section else XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

/** 描边次主按钮：纯白底 + 细描边 + 深字（不使用同色系深浅搭配）。 */
@Composable
fun OutlinePillButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    height: Dp = XDimens.ButtonHeight,
    danger: Boolean = false,
) {
    val extra = LocalExtraColors.current
    val fg = if (danger) Danger else MaterialTheme.colorScheme.onSurface
    val border = if (danger) extra.hairline else extra.line
    Box(
        modifier.height(height).alpha(if (enabled) 1f else 0.45f).clip(CircleShape)
            .background(MaterialTheme.colorScheme.surface).border(1.dp, border, CircleShape)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick).padding(horizontal = 18.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(text, color = fg, style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

/** 危险主按钮：固定危险红实心底 + 白字（不随主题色变化）。 */
@Composable
fun DangerPillButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    height: Dp = XDimens.ButtonPrimaryHeight,
) {
    Box(
        modifier.height(height).clip(CircleShape).background(if (enabled) Danger else LocalExtraColors.current.line)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick).padding(horizontal = 20.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(text, color = Color.White, style = if (height >= 44.dp) XType.Section else XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

/** 次要胶囊按钮：中性浅灰底深色字，不描边。直接放在灰色页面上时（[onPage]）用白底。 */
@Composable
fun GhostPillButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    height: Dp = XDimens.ButtonPrimaryHeight,
    fontSize: Int = 15,
    textColor: Color = MaterialTheme.colorScheme.onSurface,
    onPage: Boolean = false,
) {
    Box(
        modifier.height(height).alpha(if (enabled) 1f else 0.5f).clip(CircleShape)
            .background(if (onPage) MaterialTheme.colorScheme.surface else LocalExtraColors.current.fill)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick).padding(horizontal = 18.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(text, color = textColor, fontSize = fontSize.sp, maxLines = 1)
    }
}

/** 弱操作按钮（取消、拒绝）：无底色灰字。 */
@Composable
fun TextPillButton(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    height: Dp = XDimens.TouchTarget,
    textColor: Color = MaterialTheme.colorScheme.onSurfaceVariant,
) {
    Box(
        modifier.height(height).alpha(if (enabled) 1f else 0.5f).clip(CircleShape)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick).padding(horizontal = 16.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(text, color = textColor, style = XType.Body, maxLines = 1)
    }
}

/** 文字链接样式的小按钮（“全部 ›”这类）。 */
@Composable
fun LinkText(text: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Text(text, modifier.clip(RoundedCornerShape(XDimens.RadiusInner)).clickable(onClick = onClick).padding(horizontal = 4.dp, vertical = 6.dp), style = XType.Body, fontWeight = FontWeight.SemiBold)
}

/** 圆形图标按钮（发送、停止）。 */
@Composable
fun RoundIconButton(icon: ImageVector, description: String, onClick: () -> Unit, background: Color, tint: Color, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Box(
        modifier.size(36.dp).clip(CircleShape).background(background).clickable(enabled = enabled, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        XIcon(icon, tint = tint, size = XDimens.IconMedium, description = description)
    }
}
