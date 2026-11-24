package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.time.Clock
import xyz.xiangban.xiaoban.common.community.ChatSummary
import xyz.xiangban.xiaoban.common.community.ChatTaskStatus
import xyz.xiangban.xiaoban.common.community.CloudChatState
import xyz.xiangban.xiaoban.common.community.MobileGame

private class LocalClock(val month: Int, val day: Int, val weekday: Int, val hour: Int)

private fun localClock(epochMs: Long, offsetMinutes: Int): LocalClock {
    val minutes = epochMs.floorDiv(60_000L) + offsetMinutes
    val days = minutes.floorDiv(1440L)
    val hour = (minutes.mod(1440L) / 60).toInt()
    // Civil-from-days (H. Hinnant); 1970-01-01 is a Thursday.
    val z = days + 719_468
    val era = z.floorDiv(146_097L)
    val doe = z - era * 146_097
    val yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365
    val doy = doe - (365 * yoe + yoe / 4 - yoe / 100)
    val mp = (5 * doy + 2) / 153
    val day = (doy - (153 * mp + 2) / 5 + 1).toInt()
    val month = (if (mp < 10) mp + 3 else mp - 9).toInt()
    return LocalClock(month, day, (days + 3).mod(7L).toInt(), hour)
}

private val Weekdays = listOf("星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日")

private fun greeting(hour: Int): Pair<String, String> = when (hour) {
    in 5..8 -> "早上好" to "新的一天，从一件小事开始"
    in 9..11 -> "上午好" to "专心做事，别的我帮你记着"
    in 12..13 -> "中午好" to "吃过饭了吗？歇一会儿也好"
    in 14..17 -> "下午好" to "有点累的话，换件事做做"
    in 18..22 -> "晚上好" to "今天辛苦了，想聊点什么？"
    else -> "夜深了" to "早点休息，想说的话我都在"
}

// ───────────── 小伴AI 首页 ─────────────

/** 小伴AI 首页：问候、输入卡片（点一下进入对话页）、创作推荐、试试这样问我、继续对话。 */

internal val AgentHairline: Color get() = Color(if (ThemeState.dark) 0xFF2A2A2D else 0xFFEDEDED)
internal val AgentLine: Color get() = Color(if (ThemeState.dark) 0xFF363639 else 0xFFE4E4E4)
private val AgentDangerSoft: Color get() = Color(if (ThemeState.dark) 0xFF361E22 else 0xFFFFF3F4)

internal val AgentAccentLine: Color
    get() = if (ThemeState.dark) Color.White.copy(alpha = .08f) else Color(when (ThemeState.accent.id) {
        "green" -> 0xFFC4E3D0
        "blue" -> 0xFFC5D8F8
        "purple" -> 0xFFD4C8F7
        "amber" -> 0xFFECD4AF
        "graphite" -> 0xFFD3D3D8
        else -> 0xFFF5C7CD
    })

@Composable
internal fun AgentIconButton(icon: ImageVector, label: String, onClick: () -> Unit, modifier: Modifier = Modifier, size: Dp = 40.dp, glyph: Dp = 20.dp, tint: Color = MaterialTheme.colorScheme.onBackground) {
    Box(modifier.size(size).clip(RoundedCornerShape(12.dp))
        .clickable(indication = null, interactionSource = null, role = Role.Button, onClick = onClick), contentAlignment = Alignment.Center) {
        XIcon(icon, description = label, size = glyph, tint = tint)
    }
}

@Composable
internal fun AgentTopBar(model: String, exhausted: Boolean, waiting: Boolean, onTasks: () -> Unit, onConfig: () -> Unit, back: (() -> Unit)?) {
    val ink3 = LocalExtraColors.current.ink3
    Box(Modifier.fillMaxWidth().statusBarsPadding().height(48.dp).padding(horizontal = 8.dp), contentAlignment = Alignment.Center) {
        Box(Modifier.align(Alignment.CenterStart)) {
            AgentIconButton(if (back == null) AiHomeIcons.Tasks else AiHomeIcons.Back,
                if (back == null) "并行任务" else "返回", back ?: onTasks)
            if (waiting && back == null) {
                Box(Modifier.align(Alignment.TopEnd).padding(top = 6.dp, end = 6.dp).size(11.dp)
                    .background(MaterialTheme.colorScheme.background, CircleShape).padding(2.dp).background(Danger, CircleShape))
            }
        }
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text("小伴 AI", style = XType.Title.copy(fontSize = 16.sp, lineHeight = 20.sp, letterSpacing = (-.15).sp), fontWeight = FontWeight.SemiBold)
            Row(Modifier.padding(top = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                if (exhausted) Box(Modifier.size(5.dp).background(ChatWarn, CircleShape))
                Text(
                    if (exhausted) "点数已用完" else model,
                    style = XType.Caption.copy(fontSize = 11.5.sp, lineHeight = 14.sp),
                    color = if (exhausted) ChatWarn else ink3,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.widthIn(max = 200.dp),
                )
            }
        }
        AgentIconButton(AiHomeIcons.More, "个人 Agent 配置", onConfig, Modifier.align(Alignment.CenterEnd))
    }
}

@Composable
internal fun AgentMascot(compact: Boolean = false) {
    val extra = LocalExtraColors.current
    val colors = MaterialTheme.colorScheme
    val accent = Brand
    val shape = RoundedCornerShape(if (compact) 14.dp else 18.dp)
    Box(Modifier.size(if (compact) 44.dp else 56.dp).background(extra.brandSoft, shape)
        .border(1.dp, AgentAccentLine, shape), contentAlignment = Alignment.Center) {
        Canvas(Modifier.size(if (compact) 28.dp else 36.dp)) {
            scale(size.width / 48f, pivot = Offset.Zero) {
                drawLine(accent, Offset(24f, 9f), Offset(24f, 15f), 2.6f, StrokeCap.Round)
                drawCircle(accent, 3f, Offset(24f, 7.5f))
                drawRoundRect(colors.surface, Offset(9f, 14.5f), Size(30f, 23f), CornerRadius(11.5f))
                drawRoundRect(accent, Offset(9f, 14.5f), Size(30f, 23f), CornerRadius(11.5f), style = Stroke(2.4f))
                drawRoundRect(colors.onBackground, Offset(14.5f, 20f), Size(19f, 12f), CornerRadius(6f))
                drawCircle(colors.surface, 2.1f, Offset(20.5f, 26f))
                drawCircle(colors.surface, 2.1f, Offset(27.5f, 26f))
                drawCircle(accent.copy(alpha = .45f), 2f, Offset(7.2f, 26f))
                drawCircle(accent.copy(alpha = .45f), 2f, Offset(40.8f, 26f))
            }
        }
    }
}

@OptIn(kotlin.time.ExperimentalTime::class)
@Composable
internal fun AgentWelcome(offsetMinutes: Int, compact: Boolean, onPrompt: (String) -> Unit) {
    val clock = remember(offsetMinutes) { localClock(Clock.System.now().toEpochMilliseconds(), offsetMinutes) }
    val (hello, subtitle) = greeting(clock.hour)
    val colors = MaterialTheme.colorScheme
    val ink3 = LocalExtraColors.current.ink3
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 10.dp),
        verticalArrangement = Arrangement.Center) {
        Column(Modifier.fillMaxWidth().padding(bottom = if (compact) 10.dp else 18.dp), horizontalAlignment = Alignment.CenterHorizontally) {
            AgentMascot(compact)
            Spacer(Modifier.height(if (compact) 8.dp else 12.dp))
            if (!compact) {
                Text("${clock.month}月${clock.day}日 · ${Weekdays[clock.weekday]}", style = XType.Caption, color = ink3, fontWeight = FontWeight.Medium)
                Spacer(Modifier.height(3.dp))
            }
            Text(hello, style = XType.Headline.copy(fontSize = if (compact) 21.sp else 25.sp, lineHeight = if (compact) 28.sp else 32.sp,
                letterSpacing = (-.6).sp), fontWeight = FontWeight.Bold)
            Text(subtitle, Modifier.padding(top = 4.dp), style = XType.Secondary, color = colors.onSurfaceVariant)
        }
        if (!compact) {
            val shape = RoundedCornerShape(16.dp)
            val icons = listOf(AiHomeIcons.Target, AiHomeIcons.Pen, AiHomeIcons.Book)
            val tints = listOf(Color(0xFFC87332), Color(0xFF2E8B5E), Color(0xFF346EA8))
            Column(Modifier.fillMaxWidth().clip(shape).background(colors.surface).border(1.dp, AgentHairline, shape)) {
                Text("直接问小伴", Modifier.padding(start = 14.dp, top = 9.dp, bottom = 5.dp), style = XType.Caption, color = ink3, fontWeight = FontWeight.SemiBold)
                listOf(StarterPrompts[0], StarterPrompts[2], StarterPrompts[1]).forEachIndexed { index, prompt ->
                    if (index != 0) Box(Modifier.padding(start = 40.dp, end = 14.dp).fillMaxWidth().height(1.dp).background(AgentHairline))
                    Row(Modifier.fillMaxWidth().clickable { onPrompt(prompt) }.padding(horizontal = 14.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        XIcon(icons[index], size = 16.dp, tint = tints[index])
                        Text(prompt, Modifier.weight(1f), style = XType.Body.copy(fontSize = 14.sp, lineHeight = 20.sp),
                            maxLines = 1, overflow = TextOverflow.Ellipsis)
                        XIcon(AiHomeIcons.ArrowUpRight, size = 14.dp, tint = ink3.copy(alpha = .6f))
                    }
                }
            }
        }
    }
}

@Composable
internal fun AgentCapabilities(imagesEnabled: Boolean, computerOnline: Boolean, onImage: () -> Unit, onGame: () -> Unit, onDecor: () -> Unit, onComputer: () -> Unit) {
    Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(start = 14.dp, end = 14.dp, top = 2.dp, bottom = 6.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        if (imagesEnabled) AgentCapability(AiHomeIcons.Image, "创作图片", onImage)
        AgentCapability(AiHomeIcons.Game, "创作游戏", onGame)
        AgentCapability(AiHomeIcons.Decor, "桌面装扮", onDecor)
        AgentCapability(AiHomeIcons.Pc, "遥控电脑", onComputer, computerOnline)
    }
}

@Composable
private fun AgentCapability(icon: ImageVector, text: String, onClick: () -> Unit, online: Boolean = false) {
    val colors = MaterialTheme.colorScheme
    Row(Modifier.height(29.dp).clip(CircleShape).background(colors.surface).border(1.dp, AgentHairline, CircleShape)
        .clickable(onClick = onClick).padding(horizontal = 10.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
        XIcon(icon, tint = colors.onSurfaceVariant, size = 13.dp)
        Text(text, style = XType.Caption, fontWeight = FontWeight.Medium, color = colors.onSurfaceVariant, maxLines = 1)
        if (online) Box(Modifier.size(6.dp).background(Success, CircleShape))
    }
}

@Composable
internal fun AgentQuotaBanner(onClick: () -> Unit) {
    ChatNoticeBar(
        title = "官方点数已用完",
        subtitle = "可兑换额度或切换自定义模型",
        tone = "warn",
        actionText = "去配置",
        onAction = onClick,
        modifier = Modifier.padding(start = 12.dp, end = 12.dp, bottom = 8.dp),
    )
}

@Composable
internal fun AgentComposer(
    value: String, onValue: (String) -> Unit, placeholder: String, compact: Boolean, hasAttachments: Boolean,
    voice: VoiceInputController, voiceMode: Boolean, onVoiceMode: (Boolean) -> Unit, onAttach: () -> Unit,
    inputFocus: FocusRequester,
    onLive: () -> Unit, onSend: () -> Unit, sending: Boolean, onStop: () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val elevated = if (ThemeState.dark) 6.dp else 3.dp
    val shadow = Color.Black.copy(alpha = if (ThemeState.dark) .38f else .10f)
    val border = AgentHairline
    val canSend = value.isNotBlank() || hasAttachments
    val orb by animateColorAsState(
        when {
            sending -> colors.onBackground
            canSend -> Brand
            else -> extra.brandSoft
        },
        label = "发送按钮颜色",
    )
    Column(Modifier.fillMaxWidth().background(colors.background)) {
        Box(Modifier.fillMaxWidth().height(0.5.dp).background(AgentHairline))
        Row(
            Modifier.fillMaxWidth().padding(horizontal = if (compact) 10.dp else 12.dp)
                .padding(top = if (compact) 7.dp else 9.dp, bottom = if (compact) 8.dp else 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Box(
                Modifier.size(42.dp).shadow(elevated, CircleShape, ambientColor = shadow, spotColor = shadow)
                    .clip(CircleShape).background(colors.surface).border(1.dp, border, CircleShape)
                    .clickable(indication = null, interactionSource = null, role = Role.Button, onClick = onAttach),
                contentAlignment = Alignment.Center,
            ) {
                XIcon(AiHomeIcons.Plus, description = "添加附件", size = 19.dp, tint = colors.onSurface)
            }
            val pill = RoundedCornerShape(22.dp)
            Row(
                Modifier.weight(1f).heightIn(min = 44.dp).shadow(elevated, pill, ambientColor = shadow, spotColor = shadow)
                    .background(colors.surface, pill).border(1.dp, border, pill)
                    .padding(start = 15.dp, end = 5.dp, top = 4.dp, bottom = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(2.dp),
            ) {
                Box(Modifier.weight(1f)) {
                    if (voiceMode) {
                        PressBar(voice, true, "按住 说话", onPressStart = { if (sending) onStop() })
                    } else {
                        BasicTextField(
                            value, onValue, modifier = Modifier.fillMaxWidth().focusRequester(inputFocus), singleLine = true,
                            textStyle = XType.Body.copy(color = colors.onBackground), cursorBrush = SolidColor(Brand),
                            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                            keyboardActions = KeyboardActions(onSend = { if (canSend && !sending) onSend() }),
                            decorationBox = { field ->
                                Box {
                                    if (value.isEmpty()) Text(placeholder, style = XType.Body, color = extra.ink3, maxLines = 1)
                                    field()
                                }
                            },
                        )
                    }
                }
                AgentIconButton(
                    if (voiceMode) XIcons.Keyboard else AiHomeIcons.Mic,
                    if (voiceMode) "切回键盘输入" else "语音输入",
                    { onVoiceMode(!voiceMode) },
                    size = 34.dp, glyph = 18.dp, tint = colors.onSurfaceVariant,
                )
                Box(
                    Modifier.size(34.dp).clip(CircleShape).background(orb)
                        .clickable(role = Role.Button, onClick = if (sending) onStop else if (canSend) onSend else onLive),
                    contentAlignment = Alignment.Center,
                ) {
                    when {
                        sending -> XIcon(XIcons.Stop, description = "停止", size = 16.dp, tint = colors.background)
                        canSend -> XIcon(AiHomeIcons.ArrowUp, description = "发送", size = 17.dp, tint = Color.White)
                        else -> XIcon(AiHomeIcons.Wave, description = "实时语音通话", size = 17.dp, tint = extra.brandInk)
                    }
                }
            }
        }
    }
}

internal fun CloudChatState.parallelTasks(): List<ChatSummary> = conversations.filter { item ->
    item.id != conversationId && (item.id in runningSessions || item.id in imageSessions ||
        item.taskStatus == ChatTaskStatus.WaitingForPlan || item.taskStatus == ChatTaskStatus.Interrupted)
}

@Composable
internal fun AgentTaskStrip(cloud: CloudChatState, compact: Boolean, reviewPlan: (String) -> Unit, onClick: () -> Unit) {
    val tasks = cloud.parallelTasks()
    if (tasks.isEmpty()) return
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val waiting = tasks.firstOrNull { it.id in cloud.waitingSessions || it.taskStatus == ChatTaskStatus.WaitingForPlan || it.taskStatus == ChatTaskStatus.Interrupted }
    val plan = tasks.singleOrNull()?.takeIf { it.taskStatus == ChatTaskStatus.WaitingForPlan }
    val shape = RoundedCornerShape(if (compact) 11.dp else 14.dp)
    val tint = if (waiting == null) ChatSuccess else ChatFail
    Row(Modifier.padding(start = 14.dp, end = 14.dp, top = 2.dp, bottom = 6.dp).fillMaxWidth()
        .clip(shape).background(if (waiting == null) colors.surface else ChatFailSoft)
        .border(1.dp, if (waiting == null) AgentHairline else ChatFailLine, shape)
        .clickable { if (plan != null) reviewPlan(plan.id) else onClick() }.padding(horizontal = 12.dp, vertical = if (compact) 6.dp else 9.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(9.dp)) {
        Box(Modifier.size(13.dp).background(tint.copy(alpha = .18f), CircleShape).padding(3.dp).background(tint, CircleShape))
        Column(Modifier.weight(1f)) {
            val title = when (waiting?.taskStatus) {
                ChatTaskStatus.WaitingForPlan -> "${waiting.title} · 方案已就绪"
                ChatTaskStatus.Interrupted -> "${waiting.title} · 制作已暂停"
                else -> if (waiting != null) "${waiting.title} · 等待确认" else "${tasks.size} 个任务后台制作中"
            }
            Text(title, style = XType.Secondary.copy(fontSize = if (compact) 12.5.sp else 13.sp, lineHeight = 17.sp),
                color = if (waiting == null) colors.onBackground else Danger, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (!compact && (waiting == null || tasks.size > 1)) {
                val detail = if (waiting == null) tasks.joinToString(" · ") { it.title }
                    else "另 ${tasks.size - 1} 个任务进行中 · 不打断当前聊天"
                Text(detail, style = XType.Caption.copy(fontSize = 11.sp, lineHeight = 15.sp), color = colors.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        Text(if (plan != null) "查看方案" else if (waiting?.id in cloud.waitingSessions) "去确认" else "查看",
            Modifier.background(if (waiting == null) extra.fill else colors.surface, CircleShape)
                .padding(horizontal = if (compact) 8.dp else 10.dp, vertical = if (compact) 2.dp else 4.dp),
            style = XType.Caption, fontWeight = FontWeight.SemiBold, color = if (waiting == null) colors.onSurfaceVariant else Danger)
    }
}

@OptIn(androidx.compose.ui.ExperimentalComposeUiApi::class)
@Composable
internal fun AgentSheet(title: String, dismiss: () -> Unit, panel: Modifier, content: @Composable ColumnScope.() -> Unit) {
    androidx.compose.ui.backhandler.BackHandler(onBack = dismiss)
    BoxWithConstraints(Modifier.fillMaxSize().background(Color.Black.copy(alpha = .32f))
        .clickable(indication = null, interactionSource = null, onClick = dismiss).imePadding(), contentAlignment = Alignment.BottomCenter) {
        val shape = RoundedCornerShape(topStart = 22.dp, topEnd = 22.dp)
        Column(panel.fillMaxWidth().heightIn(max = maxHeight * .78f)
            .clip(shape).background(MaterialTheme.colorScheme.surface)
            .clickable(indication = null, interactionSource = null) {}.navigationBarsPadding()
            .padding(start = 18.dp, end = 18.dp, top = 10.dp, bottom = 26.dp)) {
            Box(Modifier.align(Alignment.CenterHorizontally).size(34.dp, 4.dp).background(AgentLine, CircleShape))
            Row(Modifier.fillMaxWidth().padding(top = 14.dp, bottom = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(title, Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight(650))
                AgentIconButton(AiHomeIcons.Close, "关闭", dismiss, size = 32.dp)
            }
            content()
        }
    }
}

@Composable
internal fun AgentChoiceRow(
    selected: Boolean, title: String, description: String?, onClick: () -> Unit,
    modifier: Modifier = Modifier, enabled: Boolean = true,
) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    Row(modifier.padding(bottom = 4.dp).fillMaxWidth().clip(RoundedCornerShape(12.dp))
        .background(if (selected) extra.brandSoft else Color.Transparent).alpha(if (enabled) 1f else .5f)
        .selectable(selected, enabled = enabled, role = Role.RadioButton, onClick = onClick)
        .padding(horizontal = 10.dp, vertical = 11.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Column(Modifier.weight(1f)) {
            Text(title, style = XType.Body.copy(fontSize = 14.sp), fontWeight = FontWeight.SemiBold,
                color = if (selected) extra.brandInk else colors.onBackground, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (!description.isNullOrBlank()) Text(description, Modifier.padding(top = 2.dp), style = XType.Caption, color = colors.onSurfaceVariant)
        }
        if (selected) XIcon(AiHomeIcons.Check, tint = Brand, size = 20.dp)
    }
}

@Composable
internal fun AgentTasks(cloud: CloudChatState, close: () -> Unit, panel: Modifier, open: (String) -> Unit) {
    val active = cloud.parallelTasks()
    val completed = cloud.conversations.filter { it.id != cloud.conversationId && it.taskStatus == ChatTaskStatus.Completed }.take(8)
    AgentSheet("并行任务", close, panel) {
        Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
            if (active.isEmpty() && completed.isEmpty()) {
                Text("暂无并行任务", Modifier.padding(vertical = 10.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            if (active.isNotEmpty()) {
                TaskGroupTitle("进行中")
                active.forEach { item ->
                    val plan = item.id !in cloud.runningSessions && item.taskStatus == ChatTaskStatus.WaitingForPlan
                    val waiting = plan || item.id in cloud.waitingSessions
                    val detail = when {
                        plan -> "创作方案已就绪 · 等待确认"
                        waiting -> "等待你确认或回答"
                        item.taskStatus == ChatTaskStatus.Interrupted -> "制作已暂停 · 可查看已保存进度"
                        item.id in cloud.imageSessions -> "生图进行中"
                        item.taskStatus == ChatTaskStatus.Planning -> "正在拟定创作方案"
                        else -> "制作中"
                    }
                    AgentTaskRow(item.title, detail, if (waiting) Danger else Success,
                        onClick = { open(item.id) }, action = if (plan) "查看方案" else null)
                }
            }
            if (completed.isNotEmpty()) {
                TaskGroupTitle("最近完成")
                completed.forEach { item -> AgentTaskRow(item.title, relativeTime(item.updatedAtMs), null, onClick = { open(item.id) }) }
            }
        }
    }
}

@Composable
private fun TaskGroupTitle(text: String) {
    Text(text, Modifier.padding(start = 2.dp, top = 12.dp, bottom = 6.dp),
        style = XType.Caption, color = LocalExtraColors.current.ink3, fontWeight = FontWeight.Medium)
}

@Composable
private fun AgentTaskRow(title: String, detail: String, status: Color?, onClick: () -> Unit, action: String? = null) {
    val extra = LocalExtraColors.current
    Row(Modifier.padding(bottom = 4.dp).fillMaxWidth().clip(RoundedCornerShape(12.dp)).clickable(onClick = onClick)
        .padding(horizontal = 10.dp, vertical = 11.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        if (status != null) Box(Modifier.padding(horizontal = 3.dp).size(7.dp).background(status, CircleShape))
        else XIcon(AiHomeIcons.Check, size = 17.dp, tint = extra.ink3)
        Column(Modifier.weight(1f)) {
            Text(title, style = XType.Body.copy(fontSize = 14.sp), fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(detail, Modifier.padding(top = 2.dp), style = XType.Caption, color = status ?: MaterialTheme.colorScheme.onSurfaceVariant)
        }
        if (action != null) Text(action, Modifier.clip(CircleShape).background(extra.brandSoft)
            .padding(horizontal = 10.dp, vertical = 5.dp), color = extra.brandInk, style = XType.Caption, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
internal fun AgentTopicDivider(epochMs: Long, offsetMinutes: Int) {
    Row(Modifier.fillMaxWidth().padding(vertical = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Box(Modifier.weight(1f).height(1.dp).background(AgentHairline))
        Row(Modifier.background(LocalExtraColors.current.fill, CircleShape).padding(horizontal = 9.dp, vertical = 2.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            XIcon(AiHomeIcons.SparkleLine, size = 11.dp, tint = Brand)
            val time = timeOfDay(epochMs, offsetMinutes)
            Text("新话题 · $time（上下文已重置）", style = XType.Caption.copy(fontSize = 11.sp),
                fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Box(Modifier.weight(1f).height(1.dp).background(AgentHairline))
    }
}

private fun timeOfDay(epochMs: Long, offsetMinutes: Int): String {
    val minutes = (epochMs.floorDiv(60_000L) + offsetMinutes).mod(1440L)
    return "${minutes / 60}:${(minutes % 60).toString().padStart(2, '0')}"
}

@OptIn(kotlin.time.ExperimentalTime::class)
private fun dateAndTime(epochMs: Long, offsetMinutes: Int): String {
    val millisPerDay = 86_400_000L
    val offset = offsetMinutes * 60_000L
    val today = (Clock.System.now().toEpochMilliseconds() + offset).floorDiv(millisPerDay)
    val day = (epochMs + offset).floorDiv(millisPerDay)
    val date = localClock(epochMs, offsetMinutes)
    val label = if (today == day) "今天" else "${date.month}月${date.day}日"
    return "$label ${timeOfDay(epochMs, offsetMinutes)}"
}

@Composable
internal fun AgentTimestamp(epochMs: Long, offsetMinutes: Int) {
    Text(dateAndTime(epochMs, offsetMinutes), Modifier.fillMaxWidth().padding(vertical = 2.dp),
        style = XType.Caption.copy(fontSize = 11.sp), textAlign = TextAlign.Center, color = LocalExtraColors.current.ink3)
}

@Composable
internal fun AgentReminderResult(title: String, epochMs: Long, offsetMinutes: Int, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    ChatCardSurface(shape = RoundedCornerShape(16.dp)) {
        Row(
            Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(11.dp),
        ) {
            Box(
                Modifier.size(36.dp).clip(RoundedCornerShape(10.dp)).background(extra.fill),
                contentAlignment = Alignment.Center,
            ) {
                XIcon(XIcons.Bell, tint = colors.onSurface, size = 18.dp)
            }
            Column(Modifier.weight(1f)) {
                Text(title, style = XType.Body.copy(fontSize = 14.sp), fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text("${dateAndTime(epochMs, offsetMinutes)} · 已加入系统提醒", Modifier.padding(top = 2.dp), style = XType.Caption.copy(fontSize = 12.sp), color = colors.onSurfaceVariant)
            }
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                Box(Modifier.size(14.dp).clip(CircleShape).background(ChatSuccess), contentAlignment = Alignment.Center) {
                    XIcon(XIcons.CheckBold, tint = Color.White, size = 9.dp)
                }
                Text("已设", style = XType.Caption.copy(fontSize = 11.5.sp), fontWeight = FontWeight.SemiBold, color = ChatSuccess)
            }
        }
    }
}

@Composable
internal fun AgentUserMessage(text: String) {
    val extra = LocalExtraColors.current
    val shape = RoundedCornerShape(topStart = 18.dp, topEnd = 18.dp, bottomEnd = 6.dp, bottomStart = 18.dp)
    var expanded by rememberSaveable(text) { mutableStateOf(false) }
    var overflowed by remember(text) { mutableStateOf(false) }
    Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
        Column(
            Modifier.widthIn(max = 316.dp).clip(shape).background(extra.fill)
                .padding(horizontal = 14.dp, vertical = 11.dp),
        ) {
            Text(
                text,
                style = XType.Body.copy(fontSize = 14.5.sp, lineHeight = 22.sp),
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = if (expanded) Int.MAX_VALUE else 4,
                overflow = TextOverflow.Ellipsis,
                onTextLayout = { result -> if (!expanded && result.hasVisualOverflow) overflowed = true },
            )
            if (overflowed || expanded) {
                Row(
                    Modifier.padding(top = 6.dp).clip(RoundedCornerShape(6.dp))
                        .clickable { expanded = !expanded }.padding(vertical = 1.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(3.dp),
                ) {
                    Text(
                        if (expanded) "收起" else "展开全文",
                        style = XType.Caption.copy(fontSize = 12.sp),
                        fontWeight = FontWeight.Medium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    XIcon(
                        if (expanded) XIcons.ArrowUp else XIcons.ChevronDown,
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        size = 12.dp,
                    )
                }
            }
        }
    }
}

@Composable
internal fun AgentReply(content: @Composable ColumnScope.() -> Unit) {
    Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(11.dp), content = content)
}

/** 作品交付独立卡片：图标 + 作品名与形态 + 主色「打开」按钮 + 底部路径提示。 */
@Composable
internal fun AgentDeliverableCard(work: MobileGame, onOpen: () -> Unit, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val kindLabel = if (work.kind == "app") "工具" else "小游戏"
    val orientationLabel = when (work.orientation) {
        "landscape" -> "横屏"
        "auto" -> "自适应"
        else -> "竖屏"
    }
    val iconBg = if (ThemeState.dark) Color(0xFF2F231A) else Color(0xFFFFEFE3)
    val iconTint = if (ThemeState.dark) Color(0xFFF6AD55) else Color(0xFFC25E00)
    ChatCardSurface(modifier = modifier, shape = RoundedCornerShape(16.dp), elevated = true) {
        Row(
            Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 13.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Box(
                Modifier.size(44.dp).clip(RoundedCornerShape(12.dp)).background(iconBg)
                    .border(1.dp, iconTint.copy(alpha = 0.18f), RoundedCornerShape(12.dp)),
                contentAlignment = Alignment.Center,
            ) {
                XIcon(if (work.kind == "app") AiHomeIcons.Book else AiHomeIcons.Game, tint = iconTint, size = 22.dp)
            }
            Column(Modifier.weight(1f)) {
                Text(work.name, style = XType.Body.copy(fontSize = 15.sp), fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(
                    "$kindLabel · $orientationLabel · 已安装到本机",
                    Modifier.padding(top = 3.dp),
                    style = XType.Caption.copy(fontSize = 12.sp),
                    color = colors.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Box(
                Modifier.height(34.dp).clip(CircleShape).background(Brand).clickable(onClick = onOpen)
                    .padding(horizontal = 16.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text("打开", style = XType.Secondary.copy(fontSize = 13.sp), fontWeight = FontWeight.SemiBold, color = Color.White, maxLines = 1)
            }
        }
        Box(Modifier.fillMaxWidth().height(1.dp).background(AgentHairline))
        Row(
            Modifier.fillMaxWidth().background(colors.background).padding(horizontal = 14.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text("下次可在「应用 → 最近用过」打开", style = XType.Caption.copy(fontSize = 11.5.sp), color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text("协议 2", style = XType.Caption.copy(fontSize = 11.5.sp), color = extra.ink3, maxLines = 1)
        }
    }
}

/** 制作方案确认悬浮卡片。 */
@Composable
internal fun AgentPlanConfirmCard(
    sending: Boolean,
    ready: Boolean,
    canConfirm: Boolean,
    onEdit: () -> Unit,
    onPrimary: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val extra = LocalExtraColors.current
    ChatCardSurface(modifier = modifier, shape = RoundedCornerShape(18.dp), elevated = true) {
        Column(Modifier.padding(horizontal = 14.dp, vertical = 13.dp), verticalArrangement = Arrangement.spacedBy(9.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                XIcon(AiHomeIcons.Spark, tint = extra.brandInk, size = 16.dp)
                Text(
                    when {
                        sending -> "正在拟定制作方案…"
                        ready -> "制作方案已就绪"
                        else -> "先生成制作方案"
                    },
                    Modifier.weight(1f),
                    style = XType.Body.copy(fontSize = 14.sp),
                    fontWeight = FontWeight.SemiBold,
                )
                Text(
                    if (sending) "生成中" else if (ready) "待确认" else "未开始",
                    Modifier.clip(RoundedCornerShape(6.dp)).background(extra.fill).padding(horizontal = 7.dp, vertical = 2.dp),
                    style = XType.Caption.copy(fontSize = 11.sp),
                    fontWeight = FontWeight.SemiBold,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Text(
                when {
                    sending -> "方案完成后会等你确认，确认后才开始写入并安装到手机。"
                    ready -> "可以继续补充修改需求，确认无误后点击开始制作。"
                    else -> "生成方案确认无误后，再开始制作作品。"
                },
                style = XType.Caption.copy(fontSize = 12.5.sp, lineHeight = 18.sp),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (!sending) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    if (ready) {
                        Box(
                            Modifier.weight(1f).height(36.dp).clip(RoundedCornerShape(10.dp))
                                .background(extra.fill).clickable(onClick = onEdit),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text("修改需求", style = XType.Secondary.copy(fontSize = 13.sp), fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
                        }
                    }
                    Box(
                        Modifier.weight(1f).height(36.dp).clip(RoundedCornerShape(10.dp))
                            .background(if (canConfirm) Brand else extra.line)
                            .clickable(enabled = canConfirm, onClick = onPrimary),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            if (ready) "开始制作" else "重新生成方案",
                            style = XType.Secondary.copy(fontSize = 13.sp),
                            fontWeight = FontWeight.SemiBold,
                            color = Color.White,
                        )
                    }
                }
            }
        }
    }
}

/** 制作中断续作悬浮卡片。 */
@Composable
internal fun AgentCreationResumeCard(
    stage: String?,
    summary: String,
    nextStep: String,
    canResume: Boolean,
    onResume: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ChatCardSurface(modifier = modifier, shape = RoundedCornerShape(18.dp), borderColor = ChatWarnLine, elevated = true) {
        Column(Modifier.padding(horizontal = 14.dp, vertical = 13.dp), verticalArrangement = Arrangement.spacedBy(9.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Box(Modifier.size(16.dp).clip(CircleShape).background(ChatWarn), contentAlignment = Alignment.Center) {
                    Text("!", style = XType.Caption.copy(fontSize = 10.sp, lineHeight = 10.sp), color = Color.White, fontWeight = FontWeight.ExtraBold)
                }
                Text(
                    if (!stage.isNullOrBlank()) "制作在「$stage」阶段中断" else "制作已中断",
                    Modifier.weight(1f),
                    style = XType.Body.copy(fontSize = 14.sp),
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    "可续作",
                    Modifier.clip(RoundedCornerShape(6.dp)).background(ChatWarnSoft).padding(horizontal = 7.dp, vertical = 2.dp),
                    style = XType.Caption.copy(fontSize = 11.sp),
                    fontWeight = FontWeight.SemiBold,
                    color = ChatWarn,
                )
            }
            Text(
                listOf(summary, "待完成：$nextStep").filter { it.isNotBlank() }.joinToString(" · "),
                style = XType.Caption.copy(fontSize = 12.5.sp, lineHeight = 18.sp),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
            )
            if (canResume) {
                Box(
                    Modifier.fillMaxWidth().height(36.dp).clip(RoundedCornerShape(10.dp))
                        .background(Brand).clickable(onClick = onResume),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("继续制作", style = XType.Secondary.copy(fontSize = 13.sp), fontWeight = FontWeight.SemiBold, color = Color.White)
                }
            }
        }
    }
}

// ───────────── 对话列表侧栏 ─────────────

/** 左侧对话列表：新对话、按工作空间分组的对话（正在看的那个用主题色浅底，长按或“…”打开操作）、模型与点数、设置。 */
@Composable
internal fun AiDrawer(
    cloud: CloudChatState, onClose: () -> Unit, onNew: () -> Unit, onOpen: (String) -> Unit, onDelete: (ChatSummary) -> Unit,
    onModels: () -> Unit, onSettings: () -> Unit, panelModifier: Modifier,
    onCreateWorkspace: () -> Unit, onEditWorkspace: (String) -> Unit,
) {
    var collapsed by remember { mutableStateOf(setOf<String?>()) }
    val extra = LocalExtraColors.current
    Column(
        panelModifier.fillMaxSize()
            .clip(RoundedCornerShape(topEnd = 24.dp, bottomEnd = 24.dp))
            .background(MaterialTheme.colorScheme.surface).statusBarsPadding().navigationBarsPadding(),
    ) {
        Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 8.dp, top = 8.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("对话", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
            IconTap(XIcons.Close, "收起侧栏", onClose, size = 20.dp)
        }
        Row(
            Modifier.padding(horizontal = 12.dp).fillMaxWidth().height(46.dp).clip(RoundedCornerShape(14.dp))
                .background(extra.brandSoft).clickable(onClick = onNew).padding(horizontal = 14.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            XIcon(XIcons.NewChat, tint = extra.brandInk, size = 20.dp)
            Text("开启新对话", style = XType.Body, fontWeight = FontWeight.SemiBold, color = extra.brandInk)
        }
        Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 8.dp, top = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("工作空间", Modifier.weight(1f), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
            IconTap(XIcons.Plus, "新建工作空间", onCreateWorkspace, size = 18.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        LazyColumn(Modifier.weight(1f).fillMaxWidth(), contentPadding = PaddingValues(start = 8.dp, end = 8.dp, bottom = 8.dp)) {
            (cloud.workspaces.map { it as String? } + listOf<String?>(null)).forEach { group ->
                val chats = cloud.conversations.filter { it.workspace == group && it.mainTopic == null }
                item(key = "group:${group ?: ""}") {
                    Row(
                        Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp))
                            .clickable { collapsed = if (group in collapsed) collapsed - group else collapsed + group }
                            .padding(start = 12.dp, end = if (group == null) 12.dp else 0.dp).height(40.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        XIcon(if (group in collapsed) XIcons.ChevronRight else XIcons.ChevronDown, size = 14.dp, tint = extra.ink3)
                        Text(group ?: "未分组", Modifier.weight(1f), style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(chats.size.toString(), style = XType.Caption, color = extra.ink3)
                        if (group != null) IconTap(XIcons.More, "管理工作空间 $group", { onEditWorkspace(group) }, size = 16.dp, tint = extra.ink3)
                    }
                }
                if (group !in collapsed) {
                    if (chats.isEmpty()) item(key = "empty:${group ?: ""}") {
                        Text(
                            if (group == null) "新的对话会出现在这里" else "在对话菜单里移入聊天",
                            Modifier.padding(start = 34.dp, top = 4.dp, bottom = 12.dp), style = XType.Caption, color = extra.ink3,
                        )
                    }
                    items(chats, key = { it.id }) { item ->
                        val current = item.id == cloud.conversationId
                        Row(
                            Modifier.padding(vertical = 1.dp).fillMaxWidth().clip(RoundedCornerShape(12.dp))
                                .background(if (current) extra.brandSoft else Color.Transparent)
                                .combinedClickable(onLongClick = { onDelete(item) }) { onOpen(item.id) }
                                .padding(start = 34.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Column(Modifier.weight(1f).padding(vertical = 10.dp)) {
                                Text(
                                    item.title, style = XType.Body, maxLines = 1, overflow = TextOverflow.Ellipsis,
                                    color = if (current) extra.brandInk else MaterialTheme.colorScheme.onSurface,
                                    fontWeight = if (current) FontWeight.SemiBold else FontWeight.Normal,
                                )
                                (when (item.id) {
                                    in cloud.waitingSessions -> "等待你确认或回答"
                                    in cloud.runningSessions -> "任务进行中"
                                    in cloud.imageSessions -> "生图进行中"
                                    else -> relativeTime(item.updatedAtMs)
                                }).takeIf { it.isNotEmpty() }?.let {
                                    Text(it, Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
                                }
                            }
                            IconTap(XIcons.More, "对话操作", { onDelete(item) }, size = 16.dp, tint = extra.ink3)
                        }
                    }
                }
                item(key = "gap:${group ?: ""}") { Spacer(Modifier.height(6.dp)) }
            }
        }
        Hairline()
        Row(Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 6.dp)) {
            DrawerLink(XIcons.Star, "模型配置", onModels, Modifier.weight(1f))
            DrawerLink(XIcons.Sliders, "设置", onSettings, Modifier.weight(1f))
        }
    }
}

@Composable
private fun DrawerLink(icon: ImageVector, text: String, onClick: () -> Unit, modifier: Modifier) {
    Row(
        modifier.height(44.dp).clip(RoundedCornerShape(12.dp)).clickable(onClick = onClick).padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        XIcon(icon, size = 20.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(text, style = XType.Secondary)
    }
}
