package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties

private val CardShape = RoundedCornerShape(24.dp)

internal fun Modifier.dialogCardSurface(surface: Color): Modifier =
    dropShadow(CardShape, Shadow(radius = 60.dp, color = Color.Black.copy(alpha = .25f), offset = DpOffset(0.dp, 20.dp)))
        .clip(CardShape).background(surface)

@Composable
internal fun BoxScope.DialogCardClose(onDismiss: () -> Unit, enabled: Boolean, fill: Color, ink: Color) {
    Box(Modifier.align(Alignment.TopEnd).padding(4.dp).size(44.dp).clip(CircleShape)
        .clickable(enabled = enabled, role = Role.Button, onClick = onDismiss), contentAlignment = Alignment.Center) {
        Box(Modifier.size(32.dp).background(fill, CircleShape), contentAlignment = Alignment.Center) {
            XIcon(XIcons.Close, size = 15.dp, tint = ink, description = "关闭")
        }
    }
}

/** Modal window for both inline and lazy-list callers. Only the body scrolls above the actions. */
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
    actionIcon: ImageVector? = null,
    cover: (@Composable () -> Unit)? = null,
    content: (@Composable () -> Unit)? = null,
) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val dismiss by rememberUpdatedState(onDismiss)
    Dialog(onDismissRequest = { if (dismissEnabled) dismiss() },
        properties = DialogProperties(dismissOnBackPress = dismissEnabled, dismissOnClickOutside = dismissEnabled, usePlatformDefaultWidth = false)) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Box(Modifier.matchParentSize().pointerInput(dismissEnabled) {
                detectTapGestures { if (dismissEnabled) dismiss() }
            })
            Box(Modifier.safeDrawingPadding().imePadding().padding(horizontal = 24.dp, vertical = 14.dp)
                .widthIn(max = 342.dp).fillMaxWidth().dialogCardSurface(colors.surface)
                .pointerInput(Unit) { detectTapGestures {} }) {
                Column(Modifier.fillMaxWidth()) {
                    Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
                        cover?.invoke()
                        Column(Modifier.fillMaxWidth().padding(start = 18.dp, end = 18.dp, top = if (cover == null) 21.dp else 15.dp)) {
                            Row(Modifier.fillMaxWidth().padding(end = if (cover == null) 24.dp else 0.dp),
                                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(13.dp)) {
                                if (cover == null) Box(Modifier.size(44.dp).background(iconColor, RoundedCornerShape(13.dp)), contentAlignment = Alignment.Center) {
                                    XIcon(icon, size = 24.dp, tint = Color.White)
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
                    Column(Modifier.fillMaxWidth().padding(start = 18.dp, end = 18.dp, top = 16.dp, bottom = 11.dp),
                        verticalArrangement = Arrangement.spacedBy(3.dp)) {
                        if (confirmLabel != null) {
                            Row(Modifier.fillMaxWidth().heightIn(min = 48.dp).clip(CircleShape)
                                .background(if (confirmEnabled) Brand else extra.fill)
                                .clickable(enabled = confirmEnabled, role = Role.Button, onClick = onConfirm).padding(8.dp),
                                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally)) {
                                val ink = if (confirmEnabled) Color.White else colors.onSurfaceVariant
                                Text(confirmLabel, color = ink, style = XType.Body)
                                actionIcon?.let { XIcon(it, tint = ink, size = 16.dp) }
                            }
                        }
                        Box(Modifier.fillMaxWidth().heightIn(min = 44.dp).clip(CircleShape)
                            .clickable(enabled = dismissEnabled, role = Role.Button, onClick = onDismiss).padding(6.dp),
                            contentAlignment = Alignment.Center) {
                            Text(dismissLabel, color = colors.onSurfaceVariant.copy(alpha = if (dismissEnabled) 1f else .55f), style = XType.Body)
                        }
                    }
                }
                DialogCardClose(onDismiss, dismissEnabled, if (cover == null) extra.fill else Color.Black.copy(alpha = .4f),
                    if (cover == null) colors.onSurfaceVariant else Color.White)
            }
        }
    }
}

@Composable
internal fun DialogNotice(text: String, error: Boolean = false) {
    val dark = ThemeState.dark
    val fill = if (error) DangerSoft else if (dark) Color(0xFF392F21) else Color(0xFFFCF2DF)
    val ink = if (error) Danger else if (dark) Color(0xFFF0C380) else Color(0xFF825119)
    Row(Modifier.fillMaxWidth().background(fill, RoundedCornerShape(12.dp)).padding(horizontal = 12.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        XIcon(XIcons.Info, Modifier.padding(top = 1.dp), tint = ink, size = 15.dp)
        Text(text, style = XType.Secondary, color = ink)
    }
}

@Composable
internal fun DialogChoice(
    label: String, selected: Boolean, onSelect: () -> Unit, modifier: Modifier = Modifier,
    caption: String? = null, icon: ImageVector? = null, enabled: Boolean = true,
) {
    val extra = LocalExtraColors.current
    val chip = icon == null
    val shape = RoundedCornerShape(10.dp)
    val ink = MaterialTheme.colorScheme.onSurfaceVariant
    Row(modifier.heightIn(min = if (chip) 44.dp else 49.dp).alpha(if (enabled) 1f else .55f).clip(shape)
        .then(if (chip) Modifier.background(if (selected) extra.brandSoft else extra.fill) else Modifier)
        .selectable(selected, enabled = enabled, role = Role.RadioButton, onClick = onSelect)
        .padding(horizontal = if (chip) 9.dp else 2.dp, vertical = 7.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
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
    Box(Modifier.size(if (compact) 14.dp else 18.dp)
        .border(if (selected) 4.dp else 1.dp, if (selected) Brand else LocalExtraColors.current.ink3, CircleShape))
}

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
