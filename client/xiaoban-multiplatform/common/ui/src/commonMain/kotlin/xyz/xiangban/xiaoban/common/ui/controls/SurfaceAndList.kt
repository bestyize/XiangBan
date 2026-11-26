package xyz.xiangban.xiaoban.common.ui.controls

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.ui.theme.Brand
import xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XIcon
import xyz.xiangban.xiaoban.common.ui.theme.XIcons
import xyz.xiangban.xiaoban.common.ui.theme.XType

/** 页面底很浅时，白色卡片靠一圈细描边和底色分开，不加阴影。 */
@Composable
fun Modifier.outlinedCard(corner: Dp = XDimens.RadiusCard): Modifier {
    val shape = RoundedCornerShape(corner)
    return clip(shape).background(MaterialTheme.colorScheme.surface).border(1.dp, LocalExtraColors.current.hairline, shape)
}

@Composable
fun Hairline(modifier: Modifier = Modifier) {
    HorizontalDivider(modifier, thickness = XDimens.Hairline, color = LocalExtraColors.current.hairline)
}

@Composable
fun WhiteCard(
    modifier: Modifier = Modifier,
    padding: Dp = XDimens.Space16,
    horizontalPadding: Dp = padding,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(modifier.fillMaxWidth().outlinedCard(XDimens.RadiusCard).padding(horizontal = horizontalPadding, vertical = padding), content = content)
}

/** 一组白底列表的小标题（“账号”“关于”）。 */
@Composable
fun GroupLabel(text: String, modifier: Modifier = Modifier) {
    Text(text, modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = 6.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
}

/** 白底通栏的一行：标题、右侧说明、箭头；[first] 为真时不画上边线。 */
@Composable
fun NavRow(title: String, note: String? = null, first: Boolean = false, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        if (!first) Hairline(Modifier.padding(start = 16.dp))
        Row(
            Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(title, Modifier.weight(1f), style = XType.Body, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (note != null) Text(note, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.widthIn(max = 180.dp))
            XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = XDimens.IconSmall)
        }
    }
}

/** 页面顶部的提示条。[warn] 为真时用主题色浅底，否则白底细描边。 */
@Composable
fun Banner(
    title: String,
    modifier: Modifier = Modifier,
    desc: String? = null,
    icon: ImageVector? = null,
    warn: Boolean = false,
    actionText: String? = null,
    onAction: (() -> Unit)? = null,
) {
    val extra = LocalExtraColors.current
    val shape = RoundedCornerShape(XDimens.RadiusCard)
    val base = if (warn) modifier.fillMaxWidth().clip(shape).background(extra.brandSoft)
    else modifier.fillMaxWidth().outlinedCard(XDimens.RadiusCard)
    Row(
        base.padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        if (icon != null) XIcon(icon, size = 22.dp)
        Column(Modifier.weight(1f)) {
            Text(title, style = XType.Body, fontWeight = if (desc != null) FontWeight.SemiBold else FontWeight.Normal)
            if (desc != null) Text(desc, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 2.dp))
        }
        if (actionText != null && onAction != null) GhostPillButton(actionText, onAction, height = 32.dp, fontSize = 14)
    }
}

/** 居中的说明文字块（空状态用）。 */
@Composable
fun EmptyState(icon: ImageVector, title: String, desc: String, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth().padding(horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.size(72.dp).outlinedCard(XDimens.RadiusSheet), contentAlignment = Alignment.Center) {
            XIcon(icon, size = 32.dp)
        }
        Text(title, Modifier.padding(top = 16.dp), style = XType.Title, fontWeight = FontWeight.SemiBold)
        Text(desc, Modifier.padding(top = 6.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
    }
}

@Composable
fun Spinner(modifier: Modifier = Modifier.fillMaxSize()) {
    Box(modifier, contentAlignment = Alignment.Center) { CircularProgressIndicator(strokeWidth = 2.dp, color = Brand) }
}

@Composable
fun CenterMessage(text: String, action: String? = null, onAction: (() -> Unit)? = null, modifier: Modifier = Modifier.fillMaxSize()) {
    Column(modifier.padding(16.dp), verticalArrangement = Arrangement.Center, horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.size(44.dp).clip(RoundedCornerShape(XDimens.RadiusSmall)).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
            XIcon(XIcons.File, size = 22.dp, tint = LocalExtraColors.current.ink3)
        }
        Text(text, Modifier.padding(top = 12.dp), color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center, style = XType.Secondary)
        if (action != null && onAction != null) OutlinedButton(onClick = onAction, Modifier.padding(top = 12.dp)) { Text(action) }
    }
}
