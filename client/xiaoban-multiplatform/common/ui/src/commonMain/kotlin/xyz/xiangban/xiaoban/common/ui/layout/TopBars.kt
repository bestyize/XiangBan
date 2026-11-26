package xyz.xiangban.xiaoban.common.ui.layout

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.ui.controls.Hairline
import xyz.xiangban.xiaoban.common.ui.theme.IconTap
import xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XIcons
import xyz.xiangban.xiaoban.common.ui.theme.XType

/** 底部导航占用的高度（导航 56 + 系统手势条余量）。滚动列表要在底部留出这么多。 */
val NavInset = 72.dp

/** 子页面顶栏：返回、居中标题（可带一行小字）、右侧操作。 */
@Composable
fun ScreenBar(
    title: String,
    onBack: (() -> Unit)?,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    trailing: (@Composable () -> Unit)? = null,
    leading: (@Composable () -> Unit)? = null,
    subtitleContent: (@Composable () -> Unit)? = null,
) {
    Column(modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(56.dp).padding(horizontal = 4.dp), verticalAlignment = Alignment.CenterVertically) {
            if (leading != null) Box(Modifier.size(XDimens.TouchTarget), contentAlignment = Alignment.Center) { leading() }
            else if (onBack != null) IconTap(XIcons.ChevronLeft, "返回", onBack) else Spacer(Modifier.width(XDimens.TouchTarget))
            Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
                Text(title, style = XType.Section, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (subtitleContent != null) subtitleContent()
                else if (subtitle != null) Text(subtitle, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            if (trailing != null) Box(Modifier.widthIn(min = XDimens.TouchTarget).height(XDimens.TouchTarget), contentAlignment = Alignment.Center) { trailing() } else Spacer(Modifier.width(XDimens.TouchTarget))
        }
    }
}

@Composable
fun RefinedScreenBar(
    title: String,
    onBack: () -> Unit,
    divider: Boolean = true,
    titleSize: Float = 17f,
    trailing: @Composable BoxScope.() -> Unit = {},
) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Box(Modifier.fillMaxWidth().height(XDimens.TopBarSubHeight).padding(horizontal = 8.dp)) {
            Text(
                title,
                Modifier.align(Alignment.Center).padding(horizontal = 82.dp),
                style = XType.Section.copy(fontSize = titleSize.sp),
                fontWeight = FontWeight(650),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            IconTap(XIcons.ChevronLeft, "返回", onBack, Modifier.align(Alignment.CenterStart), size = XDimens.IconMedium)
            Box(Modifier.align(Alignment.CenterEnd), content = trailing)
        }
        if (divider) Hairline()
    }
}

@Composable
fun RefinedBarAction(label: String, onClick: () -> Unit, accent: Boolean = false, enabled: Boolean = true) {
    val tint = if (accent) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurfaceVariant
    Box(
        Modifier.height(32.dp).clip(CircleShape).clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(horizontal = 10.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            label,
            style = XType.Secondary.copy(fontSize = 13.5.sp),
            fontWeight = if (accent) FontWeight.SemiBold else FontWeight.Medium,
            color = tint.copy(alpha = if (enabled) 1f else .38f),
            maxLines = 1,
        )
    }
}

/** 一级页面顶栏：标题靠左，右侧操作。 */
@Composable
fun HomeBar(title: String, modifier: Modifier = Modifier, trailing: (@Composable () -> Unit)? = null) {
    Column(modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(XDimens.TopBarHomeHeight).padding(start = 16.dp, end = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(title, Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
            trailing?.invoke()
        }
    }
}
