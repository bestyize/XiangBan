package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/*
 * 按设计稿（“社区与小伴AI 手机原型”）搭的一套基础部件：线条图标、顶栏、悬浮胶囊导航、底部弹层、按钮、输入框、开关、提示条。
 * 所有页面都用这些，样式只在这里改。
 */

/** 悬浮导航占用的底部高度（导航 60 + 底边距 14 + 余量）。滚动列表要在底部留出这么多。 */
val NavInset = 96.dp

// ───────────── 图标 ─────────────

@Composable
fun XIcon(icon: ImageVector, modifier: Modifier = Modifier, tint: Color = MaterialTheme.colorScheme.onSurface, size: Dp = 24.dp, description: String? = null) {
    Icon(icon, description, modifier.size(size), tint)
}

/** 44×44 的点击区域里放一个图标（设计稿里所有图标按钮的尺寸）。 */
@Composable
fun IconTap(icon: ImageVector, description: String, onClick: () -> Unit, modifier: Modifier = Modifier, tint: Color = MaterialTheme.colorScheme.onSurface, size: Dp = 24.dp) {
    Box(modifier.size(44.dp).clip(CircleShape).clickable(onClick = onClick), contentAlignment = Alignment.Center) {
        XIcon(icon, tint = tint, size = size, description = description)
    }
}

// ───────────── 顶栏 ─────────────

/** 子页面顶栏：返回、居中标题（可带一行小字）、右侧操作。 */
@Composable
fun ScreenBar(title: String, onBack: (() -> Unit)?, modifier: Modifier = Modifier, subtitle: String? = null, trailing: (@Composable () -> Unit)? = null, leading: (@Composable () -> Unit)? = null, subtitleContent: (@Composable () -> Unit)? = null) {
    Column(modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(56.dp).padding(horizontal = 4.dp), verticalAlignment = Alignment.CenterVertically) {
            if (leading != null) Box(Modifier.size(44.dp), contentAlignment = Alignment.Center) { leading() }
            else if (onBack != null) IconTap(XIcons.ChevronLeft, "返回", onBack) else Spacer(Modifier.width(44.dp))
            Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
                Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (subtitleContent != null) subtitleContent()
                else if (subtitle != null) Text(subtitle, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            if (trailing != null) Box(Modifier.widthIn(min = 44.dp).height(44.dp), contentAlignment = Alignment.Center) { trailing() } else Spacer(Modifier.width(44.dp))
        }
        Hairline()
    }
}

/** 一级页面顶栏：标题靠左，右侧操作。 */
@Composable
fun HomeBar(title: String, modifier: Modifier = Modifier, trailing: (@Composable () -> Unit)? = null) {
    Column(modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(52.dp).padding(start = 16.dp, end = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(title, Modifier.weight(1f), fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
            trailing?.invoke()
        }
    }
}

// ───────────── 悬浮胶囊导航与发布弹层 ─────────────

private data class NavSlot(val tab: Tab?, val icon: ImageVector, val label: String)

private val NavSlots = listOf(
    NavSlot(Tab.Ai, XIcons.Sparkle, "小伴AI"),
    NavSlot(Tab.Games, XIcons.Gamepad, "游戏"),
    NavSlot(null, XIcons.Plus, "发布"),
    NavSlot(Tab.Community, XIcons.Grid, "社区"),
    NavSlot(Tab.Mine, XIcons.User, "我的"),
)

/** 底部悬浮胶囊导航：当前项展开成红色胶囊并带文字，其余只有图标；中间的“发布”打开发布弹层。 */
@Composable
fun FloatingNav(current: Tab, publishOpen: Boolean, onSelect: (Tab) -> Unit, onPublish: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier.padding(horizontal = 20.dp).padding(bottom = 14.dp).navigationBarsPadding().fillMaxWidth().height(60.dp)
            .shadow(14.dp, RoundedCornerShape(30.dp)).background(MaterialTheme.colorScheme.surface, RoundedCornerShape(30.dp)).padding(horizontal = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        NavSlots.forEach { slot ->
            val target = slot.tab
            val selected = if (target == null) publishOpen else (!publishOpen && target == current)
            val click: () -> Unit = if (target == null) onPublish else ({ onSelect(target) })
            if (selected) {
                Row(
                    Modifier.height(44.dp).clip(RoundedCornerShape(22.dp)).background(Brand).clickable(onClick = click).padding(horizontal = 14.dp),
                    verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    XIcon(slot.icon, tint = Color.White, size = 22.dp)
                    Text(slot.label, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
                }
            } else {
                Box(Modifier.width(48.dp).height(44.dp).clip(RoundedCornerShape(22.dp)).clickable(onClick = click), contentAlignment = Alignment.Center) {
                    XIcon(slot.icon, tint = MaterialTheme.colorScheme.onSurfaceVariant, description = slot.label)
                }
            }
        }
    }
}

/** 遮罩淡入淡出，面板按方向滑动；退出动画结束后才移除内容。 */
@Composable
fun OverlayTransition(visible: Boolean, direction: String = "bottom", content: @Composable (Modifier) -> Unit) {
    AnimatedVisibility(visible = visible, enter = fadeIn(tween(200)), exit = fadeOut(tween(200))) {
        val panel = when (direction) {
            "left" -> Modifier.animateEnterExit(enter = slideInHorizontally(tween(200)) { -it }, exit = slideOutHorizontally(tween(200)) { -it })
            "bottom" -> Modifier.animateEnterExit(enter = slideInVertically(tween(200)) { it }, exit = slideOutVertically(tween(200)) { it })
            else -> Modifier
        }
        content(panel)
    }
}

/** 盖住整页的半透明遮罩，底部是圆角白色面板。点遮罩或按返回键关闭。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun BottomSheet(onDismiss: () -> Unit, panelModifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    BackHandler(onBack = onDismiss)
    Box(
        Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.45f)).clickable(indication = null, interactionSource = null) { onDismiss() },
        contentAlignment = Alignment.BottomCenter,
    ) {
        Column(
            panelModifier.fillMaxWidth().clip(RoundedCornerShape(topStart = 20.dp, topEnd = 20.dp)).background(MaterialTheme.colorScheme.surface)
                .clickable(indication = null, interactionSource = null) {}
                .navigationBarsPadding().padding(start = 16.dp, end = 16.dp, top = 20.dp, bottom = 24.dp),
            content = content,
        )
    }
}

/** 居中的小对话框（遮罩更淡）。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun CenterDialog(onDismiss: () -> Unit, content: @Composable ColumnScope.() -> Unit) {
    BackHandler(onBack = onDismiss)
    Box(
        Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.35f)).clickable(indication = null, interactionSource = null) { onDismiss() }.statusBarsPadding().navigationBarsPadding().imePadding(),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            Modifier.padding(horizontal = 32.dp).widthIn(max = 320.dp).clip(RoundedCornerShape(20.dp)).background(MaterialTheme.colorScheme.surface)
                .clickable(indication = null, interactionSource = null) {}.padding(horizontal = 16.dp, vertical = 16.dp),
            content = content,
        )
    }
}

/** 发布弹层里的一行：左图标方块，右标题与说明。 */
@Composable
fun SheetAction(icon: ImageVector, title: String, desc: String, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        IconBox(icon, 44.dp, 12.dp)
        Column(Modifier.weight(1f)) {
            Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
            Text(desc, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 2.dp))
        }
    }
}

@Composable
fun IconBox(icon: ImageVector, box: Dp = 44.dp, corner: Dp = 12.dp, tint: Color = MaterialTheme.colorScheme.onSurface, iconSize: Dp = 24.dp) {
    Box(Modifier.size(box).clip(RoundedCornerShape(corner)).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
        XIcon(icon, tint = tint, size = iconSize)
    }
}

// ───────────── 按钮 ─────────────

/** 品牌红实心胶囊按钮。 */
@Composable
fun PillButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, height: Dp = 48.dp, icon: ImageVector? = null) {
    Row(
        modifier.height(height).clip(CircleShape).background(if (enabled) Brand else LocalExtraColors.current.line).clickable(enabled = enabled, onClick = onClick).padding(horizontal = 20.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically,
    ) {
        if (icon != null) XIcon(icon, tint = Color.White, size = 20.dp)
        Text(text, color = Color.White, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

/** 黑色实心胶囊按钮（拒绝等）。 */
@Composable
fun DarkPillButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, height: Dp = 48.dp) {
    Box(modifier.height(height).clip(CircleShape).background(MaterialTheme.colorScheme.onSurface).clickable(onClick = onClick).padding(horizontal = 20.dp), contentAlignment = Alignment.Center) {
        Text(text, color = MaterialTheme.colorScheme.surface, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

/** 白底描边胶囊按钮。 */
@Composable
fun GhostPillButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, height: Dp = 48.dp, fontSize: Int = 15, textColor: Color = MaterialTheme.colorScheme.onSurface) {
    Box(
        modifier.height(height).alpha(if (enabled) 1f else 0.5f).clip(CircleShape).background(MaterialTheme.colorScheme.surface).border(1.dp, LocalExtraColors.current.line, CircleShape)
            .clickable(enabled = enabled, onClick = onClick).padding(horizontal = 18.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(text, color = textColor, fontSize = fontSize.sp, maxLines = 1)
    }
}

/** 文字链接样式的小按钮（“全部 ›”这类）。 */
@Composable
fun LinkText(text: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Text(text, modifier.clip(RoundedCornerShape(8.dp)).clickable(onClick = onClick).padding(horizontal = 4.dp, vertical = 6.dp), fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
}

// ───────────── 输入与开关 ─────────────

@Composable
fun TextBox(
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    label: String? = null,
    placeholder: String = "",
    password: Boolean = false,
    keyboardType: KeyboardType = KeyboardType.Text,
    enabled: Boolean = true,
    leadingIcon: ImageVector? = null,
    filled: Boolean = false,
    fontSize: Int = 16,
) {
    Column(modifier) {
        if (label != null) Text(label, fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(bottom = 6.dp))
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
            enabled = enabled,
            textStyle = TextStyle(fontSize = fontSize.sp, color = MaterialTheme.colorScheme.onSurface),
            keyboardOptions = KeyboardOptions(keyboardType = keyboardType),
            singleLine = true,
            visualTransformation = if (password) PasswordVisualTransformation() else VisualTransformation.None,
            cursorBrush = SolidColor(Brand),
            decorationBox = { inner ->
                Row(
                    Modifier.fillMaxWidth().height(48.dp).clip(RoundedCornerShape(12.dp)).background(if (filled) LocalExtraColors.current.fill else MaterialTheme.colorScheme.surface)
                        .border(1.dp, LocalExtraColors.current.line, RoundedCornerShape(12.dp)).padding(horizontal = 14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    if (leadingIcon != null) {
                        XIcon(leadingIcon, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 20.dp)
                        Spacer(Modifier.width(10.dp))
                    }
                    Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                        if (value.isEmpty()) Text(placeholder, color = LocalExtraColors.current.ink3, fontSize = fontSize.sp, maxLines = 1)
                        inner()
                    }
                }
            },
        )
    }
}

/** 多行输入框：和 [TextBox] 同样的描边样式，高度随内容增长，至少 [minHeight]。 */
@Composable
fun AreaBox(value: String, onValueChange: (String) -> Unit, modifier: Modifier = Modifier, placeholder: String = "", minHeight: Dp = 120.dp, enabled: Boolean = true) {
    BasicTextField(
        value = value,
        onValueChange = onValueChange,
        modifier = modifier.fillMaxWidth(),
        enabled = enabled,
        textStyle = TextStyle(fontSize = 15.sp, lineHeight = 24.sp, color = MaterialTheme.colorScheme.onSurface),
        cursorBrush = SolidColor(Brand),
        decorationBox = { inner ->
            Box(
                Modifier.fillMaxWidth().heightIn(min = minHeight).clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.surface)
                    .border(1.dp, LocalExtraColors.current.line, RoundedCornerShape(12.dp)).padding(horizontal = 14.dp, vertical = 12.dp),
            ) {
                if (value.isEmpty()) Text(placeholder, color = LocalExtraColors.current.ink3, fontSize = 15.sp, lineHeight = 24.sp)
                inner()
            }
        },
    )
}

/** 开关：48×28，开启时品牌红。 */
@Composable
fun Toggle(checked: Boolean, onChange: (Boolean) -> Unit, modifier: Modifier = Modifier) {
    Box(
        modifier.size(width = 48.dp, height = 28.dp).clip(CircleShape).background(if (checked) Brand else LocalExtraColors.current.line).clickable { onChange(!checked) }.padding(2.dp),
        contentAlignment = if (checked) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        Box(Modifier.size(24.dp).clip(CircleShape).background(Color.White))
    }
}

/** 勾选框：22×22 圆角方块。 */
@Composable
fun CheckBox(checked: Boolean, onChange: (Boolean) -> Unit, modifier: Modifier = Modifier) {
    val shape = RoundedCornerShape(6.dp)
    Box(
        modifier.size(22.dp).clip(shape).background(if (checked) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.surface)
            .border(2.dp, if (checked) MaterialTheme.colorScheme.onSurface else LocalExtraColors.current.line, shape).clickable { onChange(!checked) },
        contentAlignment = Alignment.Center,
    ) {
        if (checked) XIcon(XIcons.Check, tint = MaterialTheme.colorScheme.surface, size = 16.dp)
    }
}

// ───────────── 状态与提示 ─────────────

/** 在线用实心点，离线用空心点，不只靠颜色区分。 */
@Composable
fun StatusDot(online: Boolean, label: String, fontSize: Int = 12) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        if (online) {
            Box(Modifier.size(8.dp).clip(CircleShape).background(Color(0xFF21A366)))
        } else {
            Box(Modifier.size(8.dp).clip(CircleShape).border(1.5.dp, MaterialTheme.colorScheme.onSurfaceVariant, CircleShape))
        }
        Text(label, fontSize = fontSize.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

/** 页面顶部的提示条。[warn] 为真时用品牌红浅底，否则白底描边。 */
@Composable
fun Banner(title: String, modifier: Modifier = Modifier, desc: String? = null, icon: ImageVector? = null, warn: Boolean = false, actionText: String? = null, onAction: (() -> Unit)? = null) {
    val extra = LocalExtraColors.current
    val shape = RoundedCornerShape(12.dp)
    Row(
        modifier.fillMaxWidth().clip(shape).background(if (warn) extra.brandSoft else MaterialTheme.colorScheme.surface)
            .border(0.5.dp, if (warn) extra.brandSoftLine else extra.hairline, shape).padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        if (icon != null) XIcon(icon, size = 22.dp)
        Column(Modifier.weight(1f)) {
            Text(title, fontSize = if (desc != null) 14.sp else 13.sp, fontWeight = if (desc != null) FontWeight.SemiBold else FontWeight.Normal, lineHeight = 19.sp)
            if (desc != null) Text(desc, fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 2.dp))
        }
        if (actionText != null && onAction != null) GhostPillButton(actionText, onAction, height = 32.dp, fontSize = 13)
    }
}

/** 一组白底列表的小标题（“账号”“关于”）。 */
@Composable
fun GroupLabel(text: String, modifier: Modifier = Modifier) {
    Text(text, modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = 6.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
}

/** 白底通栏的一行：标题、右侧说明、箭头；[first] 为真时不画上边线。 */
@Composable
fun NavRow(title: String, note: String? = null, first: Boolean = false, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        if (!first) Hairline(Modifier.padding(start = 16.dp))
        Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(title, Modifier.weight(1f), fontSize = 15.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (note != null) Text(note, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.widthIn(max = 180.dp))
            XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
        }
    }
}

/** 居中的说明文字块（空状态用）。 */
@Composable
fun EmptyState(icon: ImageVector, title: String, desc: String, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth().padding(horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.size(72.dp).shadow(2.dp, RoundedCornerShape(20.dp)).background(MaterialTheme.colorScheme.surface, RoundedCornerShape(20.dp)), contentAlignment = Alignment.Center) {
            XIcon(icon, size = 32.dp)
        }
        Text(title, Modifier.padding(top = 16.dp), fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
        Text(desc, Modifier.padding(top = 6.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
    }
}

// ───────────── 对话部件（云端小伴与遥控电脑共用） ─────────────

private val UserBubbleShape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp, bottomEnd = 4.dp, bottomStart = 16.dp)
internal val AssistantBubbleShape = RoundedCornerShape(topStart = 4.dp, topEnd = 16.dp, bottomEnd = 16.dp, bottomStart = 16.dp)

@Composable
fun UserBubble(text: String, modifier: Modifier = Modifier, images: Int = 0) {
    Box(modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
        Column(Modifier.widthIn(max = 280.dp).clip(UserBubbleShape).background(Brand).padding(horizontal = 14.dp, vertical = 10.dp)) {
            if (images > 0) {
                Row(Modifier.padding(bottom = if (text.isNotBlank()) 6.dp else 0.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    XIcon(XIcons.Image, tint = Color.White, size = 16.dp)
                    Text("$images 张图片", color = Color.White, fontSize = 13.sp)
                }
            }
            if (text.isNotBlank()) Text(text, color = Color.White, fontSize = 15.sp, lineHeight = 22.sp)
        }
    }
}

/** 小伴的回复：左侧红色圆形头像，右侧白色气泡；气泡里可放文字或自定义内容。 */
@Composable
fun AssistantRow(modifier: Modifier = Modifier, avatar: Boolean = true, content: @Composable ColumnScope.() -> Unit) {
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Top) {
        if (avatar) {
            Box(Modifier.size(28.dp).clip(CircleShape).background(Brand), contentAlignment = Alignment.Center) { XIcon(XIcons.SparkleSmall, tint = Color.White, size = 16.dp) }
        } else {
            Spacer(Modifier.width(28.dp))
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp), content = content)
    }
}

@Composable
fun AssistantBubble(text: String, modifier: Modifier = Modifier, muted: Boolean = false) {
    Text(
        text,
        modifier.clip(AssistantBubbleShape).background(MaterialTheme.colorScheme.surface).padding(horizontal = 12.dp, vertical = 10.dp),
        fontSize = 14.sp, lineHeight = 22.sp, color = if (muted) LocalExtraColors.current.ink3 else MaterialTheme.colorScheme.onSurface,
    )
}

/** 输入卡片：上面一个多行输入框，下面一行放选择器和发送按钮（由调用方通过 [actions] 放）。 */
@Composable
fun ComposerCard(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    actions: @Composable RowScope.() -> Unit,
) {
    Column(modifier.fillMaxWidth().shadow(3.dp, RoundedCornerShape(20.dp)).background(MaterialTheme.colorScheme.surface, RoundedCornerShape(20.dp)).padding(start = 14.dp, end = 14.dp, top = 12.dp, bottom = 10.dp)) {
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.fillMaxWidth(),
            enabled = enabled,
            textStyle = TextStyle(fontSize = 15.sp, color = MaterialTheme.colorScheme.onSurface),
            maxLines = 4,
            cursorBrush = SolidColor(Brand),
            decorationBox = { inner ->
                Box(Modifier.fillMaxWidth().height(32.dp), contentAlignment = Alignment.CenterStart) {
                    if (value.isEmpty()) Text(placeholder, color = LocalExtraColors.current.ink3, fontSize = 15.sp, maxLines = 1)
                    inner()
                }
            },
        )
        Row(Modifier.fillMaxWidth().padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp), content = actions)
    }
}

/**
 * 聊天输入卡片（设计稿 6 号和 35–39 号画板）：输入框，下面一行 + 添加内容、模型、联网、语音输入、（实时对话）、发送或停止。
 * 还没有对应功能的按钮由调用方在回调里提示“即将支持”，不假装能用。
 */
@Composable
fun ChatComposer(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    modelLabel: String,
    onModel: () -> Unit,
    onAttach: () -> Unit,
    onWeb: () -> Unit,
    onVoice: () -> Unit,
    onSend: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    webOn: Boolean = false,
    sending: Boolean = false,
    onStop: (() -> Unit)? = null,
    onLive: (() -> Unit)? = null,
) {
    val extra = LocalExtraColors.current
    ComposerCard(value, onValueChange, placeholder, modifier, enabled = enabled) {
        RoundIconButton(XIcons.Plus, "添加内容", onAttach, MaterialTheme.colorScheme.surface, MaterialTheme.colorScheme.onSurfaceVariant, enabled = enabled, bordered = true)
        Box(Modifier.weight(1f, fill = false)) { ChipPill(modelLabel, trailingIcon = XIcons.ChevronDown, onClick = if (enabled) onModel else null) }
        ChipPill("联网", icon = XIcons.Globe, selected = webOn, onClick = if (enabled) onWeb else null)
        Spacer(Modifier.weight(1f))
        IconTap(XIcons.Mic, "语音输入", onVoice, Modifier.size(36.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 22.dp)
        if (onLive != null) RoundIconButton(XIcons.Wave, "实时对话", onLive, extra.fill, MaterialTheme.colorScheme.onSurface, enabled = enabled)
        if (sending) {
            RoundIconButton(XIcons.Stop, if (onStop != null) "停止" else "正在处理", onStop ?: {}, MaterialTheme.colorScheme.onSurface.copy(alpha = if (onStop != null) 1f else 0.35f), MaterialTheme.colorScheme.surface, enabled = onStop != null)
        } else {
            val canSend = enabled && value.trim().isNotEmpty()
            RoundIconButton(XIcons.ArrowRight, "发送", onSend, if (canSend) Brand else extra.line, Color.White, enabled = canSend)
        }
    }
}

/** “添加内容”底部面板：相机、相册、文件、资料库（设计稿 6 号画板）。 */
@Composable
fun AttachSheet(onDismiss: () -> Unit, onPick: (String) -> Unit, panelModifier: Modifier = Modifier) {
    BottomSheet(onDismiss, panelModifier) {
        Text("添加内容", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        Row(Modifier.padding(top = 16.dp).fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            listOf(XIcons.Camera to "相机", XIcons.Image to "相册", XIcons.File to "文件", XIcons.Book to "资料库").forEach { (icon, label) ->
                Column(Modifier.weight(1f).clickable { onPick(label) }, horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    IconBox(icon, 56.dp, 16.dp)
                    Text(label, fontSize = 13.sp)
                }
            }
        }
        GhostPillButton("取消", onDismiss, Modifier.padding(top = 20.dp).fillMaxWidth())
    }
}

/** 圆形图标按钮（发送、停止、附件）。 */
@Composable
fun RoundIconButton(icon: ImageVector, description: String, onClick: () -> Unit, background: Color, tint: Color, modifier: Modifier = Modifier, enabled: Boolean = true, bordered: Boolean = false) {
    Box(
        modifier.size(36.dp).clip(CircleShape).background(background).let { if (bordered) it.border(1.dp, LocalExtraColors.current.line, CircleShape) else it }.clickable(enabled = enabled, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        XIcon(icon, tint = tint, size = 20.dp, description = description)
    }
}

/** 输入卡片里的小胶囊（模型选择、设备名）。 */
@Composable
fun ChipPill(text: String, modifier: Modifier = Modifier, icon: ImageVector? = null, trailingIcon: ImageVector? = null, onClick: (() -> Unit)? = null, selected: Boolean = false) {
    val extra = LocalExtraColors.current
    Row(
        modifier.height(32.dp).clip(CircleShape).background(if (selected) extra.brandSoft else Color.Transparent).border(1.dp, if (selected) Brand else extra.line, CircleShape).let { if (onClick != null) it.clickable(onClick = onClick) else it }.padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        if (icon != null) XIcon(icon, tint = if (selected) Brand else MaterialTheme.colorScheme.onSurfaceVariant, size = 16.dp)
        Text(text, fontSize = 13.sp, color = if (selected) Brand else MaterialTheme.colorScheme.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.widthIn(max = 160.dp))
        if (trailingIcon != null) XIcon(trailingIcon, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 16.dp)
    }
}

/** 单选行：圆圈、标题和说明（模型选择、思考强度）。 */
@Composable
fun RadioRow(selected: Boolean, title: String, desc: String?, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Row(modifier.fillMaxWidth().alpha(if (enabled) 1f else 0.5f).clickable(onClick = onClick).padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(Modifier.size(20.dp).clip(CircleShape).border(2.dp, if (selected) Brand else LocalExtraColors.current.line, CircleShape), contentAlignment = Alignment.Center) {
            if (selected) Box(Modifier.size(10.dp).clip(CircleShape).background(Brand))
        }
        Column(Modifier.weight(1f)) {
            Text(title, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
            if (desc != null) Text(desc, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 2.dp))
        }
    }
}
