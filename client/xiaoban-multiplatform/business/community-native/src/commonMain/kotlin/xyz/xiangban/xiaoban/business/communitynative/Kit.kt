package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.ui.controls.AreaBox as UiAreaBox
import xyz.xiangban.xiaoban.common.ui.controls.Banner as UiBanner
import xyz.xiangban.xiaoban.common.ui.controls.CheckBox as UiCheckBox
import xyz.xiangban.xiaoban.common.ui.controls.ChipPill as UiChipPill
import xyz.xiangban.xiaoban.common.ui.controls.EmptyState as UiEmptyState
import xyz.xiangban.xiaoban.common.ui.controls.GhostPillButton as UiGhostPillButton
import xyz.xiangban.xiaoban.common.ui.controls.GroupLabel as UiGroupLabel
import xyz.xiangban.xiaoban.common.ui.controls.LinkText as UiLinkText
import xyz.xiangban.xiaoban.common.ui.controls.NavRow as UiNavRow
import xyz.xiangban.xiaoban.common.ui.controls.PillButton as UiPillButton
import xyz.xiangban.xiaoban.common.ui.controls.RadioRow as UiRadioRow
import xyz.xiangban.xiaoban.common.ui.controls.RoundIconButton as UiRoundIconButton
import xyz.xiangban.xiaoban.common.ui.controls.StatusDot as UiStatusDot
import xyz.xiangban.xiaoban.common.ui.controls.TextBox as UiTextBox
import xyz.xiangban.xiaoban.common.ui.controls.Toggle as UiToggle
import xyz.xiangban.xiaoban.common.ui.controls.outlinedCard as uiOutlinedCard
import xyz.xiangban.xiaoban.common.ui.layout.HomeBar as UiHomeBar
import xyz.xiangban.xiaoban.common.ui.layout.NavInset as UiNavInset
import xyz.xiangban.xiaoban.common.ui.layout.RefinedBarAction as UiRefinedBarAction
import xyz.xiangban.xiaoban.common.ui.layout.RefinedScreenBar as UiRefinedScreenBar
import xyz.xiangban.xiaoban.common.ui.layout.ScreenBar as UiScreenBar
import xyz.xiangban.xiaoban.common.ui.overlay.BottomSheet as UiBottomSheet
import xyz.xiangban.xiaoban.common.ui.overlay.CenterDialog as UiCenterDialog
import xyz.xiangban.xiaoban.common.ui.overlay.OverlayTransition as UiOverlayTransition
import xyz.xiangban.xiaoban.common.ui.theme.IconBox as UiIconBox
import xyz.xiangban.xiaoban.common.ui.theme.IconTap as UiIconTap
import xyz.xiangban.xiaoban.common.ui.theme.OverlayScrim
import xyz.xiangban.xiaoban.common.ui.theme.XDimens
import xyz.xiangban.xiaoban.common.ui.theme.XIcon as UiXIcon

/*
 * 通用基础 UI 原语已抽离至 `:common:ui`（theme / controls / layout / overlay）；
 * 本文件仅保留包内薄转发以及社区/小伴 AI 专属业务部件（主底栏、聊天气泡、输入卡片、添加内容面板）。
 */

val NavInset: Dp = UiNavInset

// ───────────── 委托到 `:common:ui` 的通用原语转发 ─────────────

@Composable
fun XIcon(icon: ImageVector, modifier: Modifier = Modifier, tint: Color = MaterialTheme.colorScheme.onSurface, size: Dp = 24.dp, description: String? = null) =
    UiXIcon(icon, modifier, tint, size, description)

@Composable
fun IconTap(icon: ImageVector, description: String, onClick: () -> Unit, modifier: Modifier = Modifier, tint: Color = MaterialTheme.colorScheme.onSurface, size: Dp = 24.dp) =
    UiIconTap(icon, description, onClick, modifier, tint, size)

@Composable
fun IconBox(icon: ImageVector, box: Dp = 44.dp, corner: Dp = 16.dp, tint: Color = MaterialTheme.colorScheme.onSurface, iconSize: Dp = 24.dp) =
    UiIconBox(icon, box, corner, tint, iconSize)

@Composable
fun ScreenBar(title: String, onBack: (() -> Unit)?, modifier: Modifier = Modifier, subtitle: String? = null, trailing: (@Composable () -> Unit)? = null, leading: (@Composable () -> Unit)? = null, subtitleContent: (@Composable () -> Unit)? = null) =
    UiScreenBar(title, onBack, modifier, subtitle, trailing, leading, subtitleContent)

@Composable
internal fun RefinedScreenBar(title: String, onBack: () -> Unit, divider: Boolean = true, titleSize: Float = 17f, trailing: @Composable BoxScope.() -> Unit = {}) =
    UiRefinedScreenBar(title, onBack, divider, titleSize, trailing)

@Composable
internal fun RefinedBarAction(label: String, onClick: () -> Unit, accent: Boolean = false, enabled: Boolean = true) =
    UiRefinedBarAction(label, onClick, accent, enabled)

@Composable
fun HomeBar(title: String, modifier: Modifier = Modifier, trailing: (@Composable () -> Unit)? = null) =
    UiHomeBar(title, modifier, trailing)

@Composable
fun OverlayTransition(visible: Boolean, direction: String = "bottom", content: @Composable (Modifier) -> Unit) =
    UiOverlayTransition(visible, direction, content)

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
) = UiBottomSheet(onDismiss, panelModifier, flush, cornerRadius, onBack, topPadding, scrim, showHandle, cancelLabel, confirmLabel, onConfirm, content)

@Composable
fun CenterDialog(onDismiss: () -> Unit, content: @Composable ColumnScope.() -> Unit) =
    UiCenterDialog(onDismiss, content)

@Composable
fun Modifier.outlinedCard(corner: Dp): Modifier = uiOutlinedCard(corner)

@Composable
fun PillButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, height: Dp = 48.dp, icon: ImageVector? = null) =
    UiPillButton(text, onClick, modifier, enabled, height, icon)

@Composable
fun GhostPillButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, height: Dp = 48.dp, fontSize: Int = 15, textColor: Color = MaterialTheme.colorScheme.onSurface, onPage: Boolean = false) =
    UiGhostPillButton(text, onClick, modifier, enabled, height, fontSize, textColor, onPage)

@Composable
fun LinkText(text: String, onClick: () -> Unit, modifier: Modifier = Modifier) =
    UiLinkText(text, onClick, modifier)

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
) = UiTextBox(value, onValueChange, modifier, label, placeholder, password, keyboardType, enabled, leadingIcon, filled, fontSize)

@Composable
fun AreaBox(value: String, onValueChange: (String) -> Unit, modifier: Modifier = Modifier, placeholder: String = "", minHeight: Dp = 120.dp, enabled: Boolean = true, filled: Boolean = false) =
    UiAreaBox(value, onValueChange, modifier, placeholder, minHeight, enabled, filled)

@Composable
fun Toggle(checked: Boolean, onChange: (Boolean) -> Unit, modifier: Modifier = Modifier) =
    UiToggle(checked, onChange, modifier)

@Composable
fun CheckBox(checked: Boolean, onChange: (Boolean) -> Unit, modifier: Modifier = Modifier) =
    UiCheckBox(checked, onChange, modifier)

@Composable
fun StatusDot(online: Boolean, label: String, fontSize: Int = 12) =
    UiStatusDot(online, label, fontSize)

@Composable
fun Banner(title: String, modifier: Modifier = Modifier, desc: String? = null, icon: ImageVector? = null, warn: Boolean = false, actionText: String? = null, onAction: (() -> Unit)? = null) =
    UiBanner(title, modifier, desc, icon, warn, actionText, onAction)

@Composable
fun GroupLabel(text: String, modifier: Modifier = Modifier) =
    UiGroupLabel(text, modifier)

@Composable
fun NavRow(title: String, note: String? = null, first: Boolean = false, onClick: () -> Unit) =
    UiNavRow(title, note, first, onClick)

@Composable
fun EmptyState(icon: ImageVector, title: String, desc: String, modifier: Modifier = Modifier) =
    UiEmptyState(icon, title, desc, modifier)

@Composable
fun RoundIconButton(icon: ImageVector, description: String, onClick: () -> Unit, background: Color, tint: Color, modifier: Modifier = Modifier, enabled: Boolean = true) =
    UiRoundIconButton(icon, description, onClick, background, tint, modifier, enabled)

@Composable
fun ChipPill(text: String, modifier: Modifier = Modifier, icon: ImageVector? = null, trailingIcon: ImageVector? = null, onClick: (() -> Unit)? = null, selected: Boolean = false) =
    UiChipPill(text, modifier, icon, trailingIcon, onClick, selected)

@Composable
fun RadioRow(selected: Boolean, title: String, desc: String?, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) =
    UiRadioRow(selected, title, desc, onClick, modifier, enabled)

// ───────────── 业务部件（保留在 business:community-native） ─────────────

internal val MainBottomNavHeight = XDimens.BottomNavHeight

/** 应用一级页面共用的五格底栏。 */
@Composable
internal fun MainBottomNav(current: Tab, onSelect: (Tab) -> Unit, onPublish: () -> Unit, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    val ink3 = LocalExtraColors.current.ink3
    Column(modifier.fillMaxWidth().background(colors.surface).navigationBarsPadding()) {
        Box(Modifier.fillMaxWidth().height(1.dp).background(AgentHairline))
        Row(Modifier.fillMaxWidth().height(MainBottomNavHeight - 1.dp).selectableGroup(), verticalAlignment = Alignment.CenterVertically) {
            val tabs = listOf(Tab.Ai to AiHomeIcons.Spark, Tab.Games to AiHomeIcons.Grid, Tab.Community to AiHomeIcons.Compass, Tab.Mine to AiHomeIcons.User)
            tabs.forEachIndexed { index, (tab, icon) ->
                if (index == 2) BottomPublishButton(onPublish, Modifier.weight(1f))
                val active = tab == current
                Column(
                    Modifier.weight(1f).fillMaxHeight()
                        .selectable(active, interactionSource = null, indication = null, role = Role.Tab) { onSelect(tab) },
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
                ) {
                    val tint = if (active) colors.onBackground else ink3
                    XIcon(icon, size = 21.dp, tint = tint)
                    Text(
                        if (tab == Tab.Ai) "小伴 AI" else tab.label,
                        style = XType.Nav, color = tint, fontWeight = if (active) FontWeight(650) else FontWeight.Normal,
                    )
                }
            }
        }
    }
}

@Composable
private fun BottomPublishButton(onClick: () -> Unit, modifier: Modifier = Modifier) {
    Box(modifier.fillMaxHeight().clickable(role = Role.Button, onClick = onClick), contentAlignment = Alignment.Center) {
        Box(Modifier.size(44.dp, 32.dp).background(Brand, RoundedCornerShape(10.dp)), contentAlignment = Alignment.Center) {
            XIcon(AiHomeIcons.Plus, description = "发布", tint = Color.White, size = 20.dp)
        }
    }
}

/** 发布弹层里的一行：左图标方块，右标题与说明。 */
@Composable
fun SheetAction(icon: ImageVector, title: String, desc: String, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        IconBox(icon, 44.dp, 16.dp)
        Column(Modifier.weight(1f)) {
            Text(title, style = XType.Section, fontWeight = FontWeight.SemiBold)
            Text(desc, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 2.dp))
        }
    }
}

// ───────────── 对话部件（云端小伴与遥控电脑共用） ─────────────

private val UserBubbleShape = RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp, bottomEnd = 6.dp, bottomStart = 18.dp)
internal val AssistantBubbleShape = RoundedCornerShape(topStart = 4.dp, topEnd = 16.dp, bottomEnd = 16.dp, bottomStart = 16.dp)

@Composable
fun UserBubble(text: String, modifier: Modifier = Modifier, images: Int = 0) {
    val ink = MaterialTheme.colorScheme.onSurface
    val extra = LocalExtraColors.current
    Box(modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
        Column(Modifier.widthIn(max = 316.dp).clip(UserBubbleShape).background(extra.fill).padding(horizontal = 14.dp, vertical = 11.dp)) {
            if (images > 0) {
                Row(Modifier.padding(bottom = if (text.isNotBlank()) 6.dp else 0.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    XIcon(XIcons.Image, tint = ink, size = 16.dp)
                    Text("$images 张图片", color = ink, style = XType.Body)
                }
            }
            if (text.isNotBlank()) Text(text, color = ink, style = XType.Body)
        }
    }
}

@Composable
fun AssistantRow(modifier: Modifier = Modifier, avatar: Boolean = false, content: @Composable ColumnScope.() -> Unit) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(11.dp), content = content)
}

@Composable
fun AssistantBubble(text: String, modifier: Modifier = Modifier, muted: Boolean = false) {
    Text(
        text,
        modifier.clip(AssistantBubbleShape).background(MaterialTheme.colorScheme.surface).padding(horizontal = 12.dp, vertical = 10.dp),
        style = XType.Secondary, color = if (muted) LocalExtraColors.current.ink3 else MaterialTheme.colorScheme.onSurface,
    )
}

@Composable
fun ComposerCard(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    voiceBar: (@Composable () -> Unit)? = null,
    actions: @Composable RowScope.() -> Unit,
) {
    Column(modifier.fillMaxWidth().shadow(3.dp, RoundedCornerShape(20.dp)).background(MaterialTheme.colorScheme.surface, RoundedCornerShape(20.dp)).padding(start = 14.dp, end = 14.dp, top = 12.dp, bottom = 10.dp)) {
        if (voiceBar != null) {
            voiceBar()
        } else {
            BasicTextField(
                value = value,
                onValueChange = onValueChange,
                modifier = Modifier.fillMaxWidth(),
                enabled = enabled,
                textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface),
                maxLines = 4,
                cursorBrush = SolidColor(Brand),
                decorationBox = { inner ->
                    Box(Modifier.fillMaxWidth().height(32.dp), contentAlignment = Alignment.CenterStart) {
                        if (value.isEmpty()) Text(placeholder, color = LocalExtraColors.current.ink3, style = XType.Body, maxLines = 1)
                        inner()
                    }
                },
            )
        }
        Row(Modifier.fillMaxWidth().padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp), content = actions)
    }
}

@Composable
fun ChatComposer(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    onModel: () -> Unit,
    onAttach: () -> Unit,
    voice: VoiceInputController,
    voiceMode: Boolean,
    onVoiceMode: (Boolean) -> Unit,
    onSend: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    onVoicePress: () -> Unit = {},
    modelIcon: ImageVector = XIcons.Brain,
    modelPinned: Boolean = false,
    onPermission: (() -> Unit)? = null,
    fullTrust: Boolean = false,
    sending: Boolean = false,
    hasAttachments: Boolean = false,
    onStop: (() -> Unit)? = null,
    onLive: (() -> Unit)? = null,
) {
    val extra = LocalExtraColors.current
    val muted = MaterialTheme.colorScheme.onSurfaceVariant
    ComposerCard(
        value, onValueChange, placeholder, modifier, enabled = enabled,
        voiceBar = if (voiceMode) ({ PressBar(voice, enabled, if (sending) "小伴正在回答，按住可以打断" else "按住 说话", onVoicePress) }) else null,
    ) {
        if (voiceMode) ToolbarIcon(XIcons.Keyboard, "切回键盘输入", { onVoiceMode(false) }, true)
        ToolbarIcon(XIcons.Plus, "添加内容", onAttach, enabled)
        Box {
            ToolbarIcon(modelIcon, "模型", onModel, enabled)
            if (modelPinned) Box(Modifier.align(Alignment.TopEnd).padding(top = 6.dp, end = 6.dp).size(8.dp).clip(CircleShape).background(Danger))
        }
        if (onPermission != null) ToolbarIcon(XIcons.Lock, if (fullTrust) "权限：自动模式" else "权限：默认权限", onPermission, enabled, tint = if (fullTrust) Danger else muted)
        Spacer(Modifier.weight(1f))
        if (!voiceMode) ToolbarIcon(XIcons.Mic, "语音输入", { onVoiceMode(true) }, true)
        if (onLive != null) ToolbarIcon(XIcons.Wave, "实时对话", onLive, enabled)
        if (!voiceMode) {
            if (sending) {
                RoundIconButton(XIcons.Stop, if (onStop != null) "停止" else "正在处理", onStop ?: {}, Brand.copy(alpha = if (onStop != null) 1f else 0.35f), Color.White, enabled = onStop != null)
            } else {
                val canSend = enabled && (value.trim().isNotEmpty() || hasAttachments)
                RoundIconButton(XIcons.ArrowRight, "发送", onSend, if (canSend) Brand else extra.line, Color.White, enabled = canSend)
            }
        }
    }
}

@Composable
private fun ToolbarIcon(icon: ImageVector, description: String, onClick: () -> Unit, enabled: Boolean, tint: Color = MaterialTheme.colorScheme.onSurfaceVariant) {
    Box(Modifier.size(36.dp).alpha(if (enabled) 1f else 0.4f).clip(CircleShape).clickable(enabled = enabled, onClick = onClick), contentAlignment = Alignment.Center) {
        XIcon(icon, tint = tint, size = 20.dp, description = description)
    }
}

/** “添加内容”底部面板：相机、相册、文件、资料库。 */
@Composable
fun AttachSheet(onDismiss: () -> Unit, onPick: (String) -> Unit, panelModifier: Modifier = Modifier, showImageGeneration: Boolean = false) {
    BottomSheet(onDismiss, panelModifier, showHandle = true, cancelLabel = "取消") {
        Text("添加内容", style = XType.Title, fontWeight = FontWeight.SemiBold)
        Row(Modifier.padding(top = 16.dp).fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            listOf(XIcons.Camera to "相机", XIcons.Image to "相册", XIcons.File to "文件", XIcons.Book to "资料库").forEach { (icon, label) ->
                Column(Modifier.weight(1f).clickable { onPick(label) }, horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    IconBox(icon, 56.dp, 16.dp)
                    Text(label, style = XType.Body)
                }
            }
        }
        if (showImageGeneration) {
            GhostPillButton("生成图片", { onPick("生成图片") }, Modifier.padding(top = 16.dp).fillMaxWidth())
        }
    }
}
