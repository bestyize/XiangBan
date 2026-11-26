package xyz.xiangban.xiaoban.common.ui.overlay

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import xyz.xiangban.xiaoban.common.ui.theme.Brand
import xyz.xiangban.xiaoban.common.ui.theme.Danger
import xyz.xiangban.xiaoban.common.ui.theme.DangerSoft
import xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.OverlayScrim
import xyz.xiangban.xiaoban.common.ui.theme.ThemeState
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XIcon
import xyz.xiangban.xiaoban.common.ui.theme.XIcons
import xyz.xiangban.xiaoban.common.ui.theme.XMotion
import xyz.xiangban.xiaoban.common.ui.theme.XType

private val CardShape = RoundedCornerShape(XDimens.RadiusDialog)

fun Modifier.dialogCardSurface(surface: Color): Modifier =
    dropShadow(CardShape, Shadow(radius = 60.dp, color = Color.Black.copy(alpha = .24f), offset = DpOffset(0.dp, 20.dp)))
        .clip(CardShape).background(surface)

@Composable
fun BoxScope.DialogCardClose(onDismiss: () -> Unit, enabled: Boolean, fill: Color = Color.Transparent, ink: Color) {
    Box(
        Modifier.align(Alignment.TopEnd).padding(4.dp).size(XDimens.TouchTarget).clip(CircleShape)
            .clickable(enabled = enabled, role = Role.Button, onClick = onDismiss),
        contentAlignment = Alignment.Center,
    ) {
        Box(Modifier.size(32.dp).background(fill, CircleShape), contentAlignment = Alignment.Center) {
            XIcon(XIcons.Close, size = 17.dp, tint = ink, description = "关闭")
        }
    }
}

/**
 * 统一 24dp 圆角居中卡片弹窗（收口确认、危险操作、表单/输入等场景）。
 * 主按钮 48dp 全圆胶囊（常规主题色 / 危险操作固定危险红），次按钮 44dp 无底色灰字，入场 220ms 标准减速。
 */
@Composable
fun CardDialog(
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
) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val dismiss by rememberUpdatedState(onDismiss)
    val enterProgress = remember { Animatable(0f) }
    LaunchedEffect(Unit) {
        enterProgress.animateTo(1f, tween(XMotion.DialogEnter, easing = XMotion.StandardEasing))
    }
    val offsetPx = with(LocalDensity.current) { 8.dp.toPx() }
    val p = enterProgress.value

    Dialog(
        onDismissRequest = { if (dismissEnabled) dismiss() },
        properties = DialogProperties(dismissOnBackPress = dismissEnabled, dismissOnClickOutside = dismissEnabled, usePlatformDefaultWidth = false),
    ) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Box(Modifier.matchParentSize().pointerInput(dismissEnabled) {
                detectTapGestures { if (dismissEnabled) dismiss() }
            })
            Box(
                Modifier.safeDrawingPadding().imePadding().padding(horizontal = XDimens.Space24, vertical = 14.dp)
                    .widthIn(max = 342.dp).fillMaxWidth()
                    .graphicsLayer {
                        alpha = p
                        val s = 0.95f + 0.05f * p
                        scaleX = s
                        scaleY = s
                        translationY = offsetPx * (1f - p)
                    }
                    .dialogCardSurface(colors.surface)
                    .pointerInput(Unit) { detectTapGestures {} },
            ) {
                Column(Modifier.fillMaxWidth()) {
                    Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
                        cover?.invoke()
                        Column(Modifier.fillMaxWidth().padding(start = 18.dp, end = 18.dp, top = if (cover == null) 21.dp else 15.dp)) {
                            Row(
                                Modifier.fillMaxWidth().padding(end = if (cover == null) 28.dp else 0.dp),
                                verticalAlignment = Alignment.Top,
                                horizontalArrangement = Arrangement.spacedBy(10.dp),
                            ) {
                                if (cover == null) Box(Modifier.size(24.dp), contentAlignment = Alignment.Center) {
                                    XIcon(icon, size = 22.dp, tint = if (danger) Danger else colors.onSurface)
                                }
                                Column {
                                    Text(title, color = colors.onSurface, style = XType.Title)
                                    if (!description.isNullOrBlank()) {
                                        Spacer(Modifier.height(6.dp))
                                        Text(description, color = colors.onSurfaceVariant, style = XType.Secondary)
                                    }
                                }
                            }
                            if (content != null) {
                                Spacer(Modifier.height(15.dp))
                                content()
                            }
                        }
                    }
                    Column(
                        Modifier.fillMaxWidth().padding(start = 18.dp, end = 18.dp, top = 16.dp, bottom = 11.dp),
                        verticalArrangement = Arrangement.spacedBy(3.dp),
                    ) {
                        if (confirmLabel != null) {
                            val activeBg = if (danger) Danger else Brand
                            Row(
                                Modifier.fillMaxWidth().heightIn(min = XDimens.ButtonPrimaryHeight).clip(CircleShape)
                                    .background(if (confirmEnabled) activeBg else extra.fill)
                                    .clickable(enabled = confirmEnabled, role = Role.Button, onClick = onConfirm).padding(8.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
                            ) {
                                val ink = if (confirmEnabled) Color.White else colors.onSurfaceVariant
                                Text(confirmLabel, color = ink, style = XType.Body)
                                actionIcon?.let { XIcon(it, tint = ink, size = XDimens.IconSmall) }
                            }
                        }
                        Box(
                            Modifier.fillMaxWidth().heightIn(min = XDimens.TouchTarget).clip(CircleShape)
                                .clickable(enabled = dismissEnabled, role = Role.Button, onClick = onDismiss).padding(6.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(dismissLabel, color = colors.onSurfaceVariant.copy(alpha = if (dismissEnabled) 1f else .55f), style = XType.Body)
                        }
                    }
                }
                DialogCardClose(
                    onDismiss, dismissEnabled,
                    if (cover == null) Color.Transparent else Color.Black.copy(alpha = .4f),
                    if (cover == null) colors.onSurfaceVariant else Color.White,
                )
            }
        }
    }
}

/** 轻量居中卡片容器（与 [CardDialog] 统一 24dp 圆角与统一遮罩，供自定义内容弹窗使用）。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun CenterDialog(onDismiss: () -> Unit, content: @Composable ColumnScope.() -> Unit) {
    BackHandler(onBack = onDismiss)
    val enterProgress = remember { Animatable(0f) }
    LaunchedEffect(Unit) {
        enterProgress.animateTo(1f, tween(XMotion.DialogEnter, easing = XMotion.StandardEasing))
    }
    val offsetPx = with(LocalDensity.current) { 8.dp.toPx() }
    val p = enterProgress.value

    Box(
        Modifier.fillMaxSize().background(OverlayScrim.copy(alpha = OverlayScrim.alpha * p))
            .clickable(indication = null, interactionSource = null) { onDismiss() }
            .statusBarsPadding().navigationBarsPadding().imePadding(),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            Modifier.padding(horizontal = XDimens.Space24).widthIn(max = 342.dp).fillMaxWidth()
                .graphicsLayer {
                    alpha = p
                    val s = 0.95f + 0.05f * p
                    scaleX = s
                    scaleY = s
                    translationY = offsetPx * (1f - p)
                }
                .dialogCardSurface(MaterialTheme.colorScheme.surface)
                .clickable(indication = null, interactionSource = null) {}
                .padding(horizontal = 18.dp, vertical = 18.dp),
            content = content,
        )
    }
}

@Composable
fun DialogNotice(text: String, error: Boolean = false) {
    val dark = ThemeState.dark
    val fill = if (error) DangerSoft else if (dark) Color(0xFF392F21) else Color(0xFFFCF2DF)
    val ink = if (error) Danger else if (dark) Color(0xFFF0C380) else Color(0xFF825119)
    Row(
        Modifier.fillMaxWidth().background(fill, RoundedCornerShape(XDimens.RadiusSmall)).padding(horizontal = 12.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        XIcon(XIcons.Info, Modifier.padding(top = 1.dp), tint = ink, size = 15.dp)
        Text(text, style = XType.Secondary, color = ink)
    }
}

@Composable
fun DialogChoice(
    label: String,
    selected: Boolean,
    onSelect: () -> Unit,
    modifier: Modifier = Modifier,
    caption: String? = null,
    icon: ImageVector? = null,
    enabled: Boolean = true,
) {
    val extra = LocalExtraColors.current
    val chip = icon == null
    val shape = RoundedCornerShape(10.dp)
    val ink = MaterialTheme.colorScheme.onSurfaceVariant
    Row(
        modifier.heightIn(min = if (chip) XDimens.TouchTarget else 49.dp).alpha(if (enabled) 1f else .55f).clip(shape)
            .then(
                if (chip) {
                    if (selected) Modifier.background(MaterialTheme.colorScheme.surface).border(1.5.dp, Brand, shape)
                    else Modifier.background(extra.fill)
                } else Modifier,
            )
            .selectable(selected, enabled = enabled, role = Role.RadioButton, onClick = onSelect)
            .padding(horizontal = if (chip) 9.dp else 2.dp, vertical = 7.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        if (chip) ChoiceIndicator(selected, compact = true)
        if (icon != null) Box(Modifier.size(32.dp).background(extra.fill, RoundedCornerShape(9.dp)), contentAlignment = Alignment.Center) {
            XIcon(icon, tint = ink, size = 17.dp)
        }
        Column(Modifier.weight(1f)) {
            Text(label, style = if (chip) XType.Caption else XType.Body, color = MaterialTheme.colorScheme.onSurface)
            if (caption != null) Text(caption, style = XType.Caption, color = ink)
        }
        if (!chip) ChoiceIndicator(selected, compact = false)
    }
}

@Composable
private fun ChoiceIndicator(selected: Boolean, compact: Boolean) {
    Box(
        Modifier.size(if (compact) 14.dp else 18.dp)
            .border(if (selected) 4.dp else 1.dp, if (selected) Brand else LocalExtraColors.current.ink3, CircleShape),
    )
}
