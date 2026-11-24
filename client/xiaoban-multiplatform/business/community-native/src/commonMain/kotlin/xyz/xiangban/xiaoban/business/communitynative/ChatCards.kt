package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.LinkInteractionListener
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.ChangedFile
import xyz.xiangban.xiaoban.common.community.ChatRecord
import xyz.xiangban.xiaoban.common.community.MdBlock
import xyz.xiangban.xiaoban.common.community.MdInline
import xyz.xiangban.xiaoban.common.community.TaskProgress
import xyz.xiangban.xiaoban.common.community.TaskState
import xyz.xiangban.xiaoban.common.community.ToolCallInfo
import xyz.xiangban.xiaoban.common.community.ToolStatus
import xyz.xiangban.xiaoban.common.community.activitySummary
import xyz.xiangban.xiaoban.common.community.changedFiles
import xyz.xiangban.xiaoban.common.community.describeCall
import xyz.xiangban.xiaoban.common.community.formatDuration
import xyz.xiangban.xiaoban.common.community.formatTokens
import xyz.xiangban.xiaoban.common.community.parseMarkdown
import xyz.xiangban.xiaoban.common.community.speechText
import xyz.xiangban.xiaoban.common.community.taskProgress
import xyz.xiangban.xiaoban.common.community.toolLabel
import xyz.xiangban.xiaoban.common.link.RemoteApprovalRequest
import xyz.xiangban.xiaoban.common.link.RunCheckpoint
import xyz.xiangban.xiaoban.common.link.TurnCancelled

/*
 * 聊天里的全卡片体系（按 ai-chat-ui.html 定稿实现）：
 * 统一 14–18dp 圆角白底 + 1dp 发丝框，90% 中性灰阶；
 * 成功态统一微信绿实心圆 + 白对勾，失败/高危统一语义红实心圆 + 白叉号，主题色仅落在星标与核心主按钮。
 * 设备聊天和小伴 AI 对话共用；数据在 community-core 的 ChatModels.kt。
 */

private val CardShape = RoundedCornerShape(16.dp)
private val CompactCardShape = RoundedCornerShape(14.dp)
private val DockCardShape = RoundedCornerShape(18.dp)
private const val DismissChoice = "__dismiss__"
private const val ContinueTask = "继续完成刚才的任务"
private const val FixAndContinue = "请修复上面的问题，然后继续完成任务"

internal val ChatSuccess: Color get() = Color(if (ThemeState.dark) 0xFF1EC76FL else 0xFF07C160L)
internal val ChatSuccessSoft: Color get() = if (ThemeState.dark) Color(0x241EC76FL) else Color(0xFFEAF9F0L)
internal val ChatFail: Color get() = Color(if (ThemeState.dark) 0xFFEF5B5FL else 0xFFE5484DL)
internal val ChatFailSoft: Color get() = if (ThemeState.dark) Color(0x21EF5B5FL) else Color(0xFFFFF1F2L)
internal val ChatFailLine: Color get() = if (ThemeState.dark) Color(0x47EF5B5FL) else Color(0xFFF5C6C9L)
internal val ChatWarn: Color get() = Color(if (ThemeState.dark) 0xFFF0A83AL else 0xFFC77700L)
internal val ChatWarnSoft: Color get() = if (ThemeState.dark) Color(0x1FF0A83AL) else Color(0xFFFFF6E6L)
internal val ChatWarnLine: Color get() = if (ThemeState.dark) Color(0x47F0A83AL) else Color(0xFFF3DFB8L)

// ───────────── 基础容器与小部件 ─────────────

@Composable
private fun Spinner(size: Dp = 13.dp, color: Color = Brand) {
    CircularProgressIndicator(Modifier.size(size), color = color, strokeWidth = 1.8.dp)
}

@Composable
internal fun ChatCardSurface(
    modifier: Modifier = Modifier,
    shape: RoundedCornerShape = CardShape,
    tint: Color? = null,
    borderColor: Color = AgentHairline,
    elevated: Boolean = false,
    content: @Composable ColumnScope.() -> Unit,
) {
    val shadowMod = if (elevated) {
        val shadowColor = Color.Black.copy(alpha = if (ThemeState.dark) .36f else .10f)
        Modifier.shadow(if (ThemeState.dark) 6.dp else 4.dp, shape, ambientColor = shadowColor, spotColor = shadowColor)
    } else Modifier
    Column(
        modifier.fillMaxWidth().then(shadowMod).clip(shape)
            .background(tint ?: MaterialTheme.colorScheme.surface)
            .border(1.dp, borderColor, shape),
        content = content,
    )
}

@Composable
private fun CardSurface(modifier: Modifier = Modifier, tint: Color? = null, content: @Composable ColumnScope.() -> Unit) {
    ChatCardSurface(modifier = modifier, shape = CardShape, tint = tint, content = content)
}

private enum class Mark { Done, Active, Approval, Paused, Blocked, Failed, Wait, Pending }

/** 展开步骤/清单前 15dp 状态点：微信绿实心圆+纯白对号，失败为语义红实心圆+纯白叉号。 */
@Composable
private fun StepMark(kind: Mark) {
    val extra = LocalExtraColors.current
    Box(Modifier.size(16.dp), contentAlignment = Alignment.Center) {
        when (kind) {
            Mark.Active -> Spinner(13.dp, color = Brand)
            Mark.Pending -> Box(Modifier.size(14.dp).clip(CircleShape).border(1.5.dp, extra.line, CircleShape))
            else -> {
                val (bg, icon) = when (kind) {
                    Mark.Done -> ChatSuccess to XIcons.CheckBold
                    Mark.Approval -> ChatFail to XIcons.Shield
                    Mark.Paused -> extra.ink3 to XIcons.Pause
                    Mark.Failed -> ChatFail to XIcons.Close
                    Mark.Blocked -> ChatFail to XIcons.Close
                    else -> ChatWarn to XIcons.Question
                }
                Box(Modifier.size(15.dp).clip(CircleShape).background(bg), contentAlignment = Alignment.Center) {
                    XIcon(icon, tint = Color.White, size = 9.5.dp)
                }
            }
        }
    }
}

private enum class BtnKind { Primary, DangerSolid, Neutral, Quiet }

@Composable
private fun SmallButton(
    text: String,
    kind: BtnKind,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    height: Dp = 32.dp,
    shape: RoundedCornerShape = RoundedCornerShape(10.dp),
) {
    val extra = LocalExtraColors.current
    val base = modifier.height(height).clip(shape)
    val styled = when (kind) {
        BtnKind.Primary -> base.background(Brand)
        BtnKind.DangerSolid -> base.background(ChatFail)
        BtnKind.Neutral -> base.background(extra.fill)
        BtnKind.Quiet -> base
    }
    val color = when (kind) {
        BtnKind.Primary, BtnKind.DangerSolid -> Color.White
        BtnKind.Neutral -> MaterialTheme.colorScheme.onSurface
        BtnKind.Quiet -> MaterialTheme.colorScheme.onSurfaceVariant
    }
    Box(styled.clickable(onClick = onClick).padding(horizontal = 12.dp), contentAlignment = Alignment.Center) {
        Text(text, color = color, style = XType.Secondary, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun CodeSnippet(text: String, modifier: Modifier = Modifier, maxLines: Int = 6) {
    val shape = RoundedCornerShape(10.dp)
    Text(
        text,
        modifier.fillMaxWidth().clip(shape)
            .background(MaterialTheme.colorScheme.background)
            .border(1.dp, AgentHairline, shape)
            .padding(horizontal = 10.dp, vertical = 8.dp),
        style = XType.Caption.copy(fontSize = 11.5.sp, lineHeight = 17.sp),
        fontFamily = FontFamily.Monospace,
        color = MaterialTheme.colorScheme.onSurface,
        maxLines = maxLines,
        overflow = TextOverflow.Ellipsis,
    )
}

/** 统一提示/异常条：语义细框 + 极浅语义底 + 左侧状态圆点 + 右侧白底轻胶囊按钮。 */
@Composable
internal fun ChatNoticeBar(
    title: String,
    subtitle: String? = null,
    tone: String = "neutral", // neutral | warn | fail
    actionText: String? = null,
    onAction: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    val shape = CompactCardShape
    val bg = when (tone) {
        "fail" -> ChatFailSoft
        "warn" -> ChatWarnSoft
        else -> MaterialTheme.colorScheme.surface
    }
    val border = when (tone) {
        "fail" -> ChatFailLine
        "warn" -> ChatWarnLine
        else -> AgentHairline
    }
    Row(
        modifier.fillMaxWidth().clip(shape).background(bg).border(1.dp, border, shape)
            .padding(horizontal = 12.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        if (tone == "fail") {
            Box(Modifier.size(16.dp).clip(CircleShape).background(ChatFail), contentAlignment = Alignment.Center) {
                XIcon(XIcons.Close, tint = Color.White, size = 10.dp)
            }
        } else if (tone == "warn") {
            Box(Modifier.size(16.dp).clip(CircleShape).background(ChatWarn), contentAlignment = Alignment.Center) {
                Text("!", style = XType.Caption.copy(fontSize = 10.sp, lineHeight = 10.sp), color = Color.White, fontWeight = FontWeight.ExtraBold)
            }
        }
        Column(Modifier.weight(1f)) {
            Text(title, style = XType.Secondary.copy(fontSize = 12.5.sp, lineHeight = 17.sp), fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurface)
            if (!subtitle.isNullOrBlank()) {
                Text(subtitle, Modifier.padding(top = 2.dp), style = XType.Caption.copy(fontSize = 11.5.sp, lineHeight = 15.sp), color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        if (actionText != null && onAction != null) {
            Box(
                Modifier.height(28.dp).clip(CircleShape)
                    .background(MaterialTheme.colorScheme.surface)
                    .border(1.dp, AgentHairline, CircleShape)
                    .clickable(onClick = onAction)
                    .padding(horizontal = 11.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(actionText, style = XType.Caption.copy(fontSize = 12.sp), fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface, maxLines = 1)
            }
        }
    }
}

@Composable
private fun HintLine(text: String, alert: Boolean, modifier: Modifier = Modifier, button: String? = null, onButton: () -> Unit = {}) {
    ChatNoticeBar(
        title = text,
        tone = if (alert) "fail" else "neutral",
        actionText = button,
        onAction = if (button != null) onButton else null,
        modifier = modifier,
    )
}

// ───────────── 三合一过程摘要卡（思考 + 任务清单 + 工具执行） ─────────────

private enum class TraceTab { Steps, Todo, Think }

@Composable
private fun UnifiedTraceCard(
    reasoning: String,
    progress: TaskProgress?,
    tools: List<ToolCallInfo>,
    live: Boolean,
    waiting: String?,
    onStop: (() -> Unit)?,
    onSend: (String) -> Unit,
    onAdjust: (() -> Unit)?,
    showRecovery: Boolean,
    modifier: Modifier = Modifier,
) {
    val extra = LocalExtraColors.current
    val segments = remember(reasoning) { reasoning.split(Regex("\n\\s*\n")).map { it.trim() }.filter { it.isNotEmpty() } }
    val stepCalls = remember(tools) { tools.filter { it.name != "todo" } }
    val summary = remember(tools) { activitySummary(tools) }
    val hasSteps = stepCalls.isNotEmpty()
    val hasTodo = progress != null
    val hasThink = segments.isNotEmpty()
    if (!hasSteps && !hasTodo && !hasThink) return

    val tabs = remember(hasSteps, hasTodo, hasThink) {
        buildList {
            if (hasSteps) add(TraceTab.Steps)
            if (hasTodo) add(TraceTab.Todo)
            if (hasThink) add(TraceTab.Think)
        }
    }
    val completed = !live && (progress == null || progress.state == TaskState.Completed) && summary.errors == 0
    val needsAttention = progress != null && (progress.state == TaskState.Blocked || progress.state == TaskState.Paused || progress.state == TaskState.Approval || progress.state == TaskState.Waiting)
    var open by rememberSaveable(completed) { mutableStateOf(live || needsAttention) }
    var activeTab by rememberSaveable(hasSteps, hasTodo, hasThink) {
        mutableStateOf(
            when {
                hasSteps -> TraceTab.Steps
                hasTodo -> TraceTab.Todo
                else -> TraceTab.Think
            },
        )
    }
    if (activeTab !in tabs) activeTab = tabs.first()

    val statusText = when {
        waiting == "approval" || progress?.state == TaskState.Approval -> "等待授权"
        waiting == "question" || progress?.state == TaskState.Waiting -> "等你回答"
        live -> {
            val runningTool = stepCalls.lastOrNull { it.status == ToolStatus.Running }
            val currentStep = progress?.let { p -> p.plan.steps.getOrNull(p.current)?.content }
            when {
                currentStep != null -> currentStep
                runningTool != null -> "正在${toolLabel(runningTool.name)}"
                hasThink && !hasSteps && !hasTodo -> "思考中…"
                else -> "执行中…"
            }
        }
        progress?.state == TaskState.Blocked -> "未完成"
        progress?.state == TaskState.Paused -> "已暂停"
        else -> "已完成"
    }

    val metaText = remember(summary, progress, segments.size) {
        buildList {
            if (summary.steps > 0) add("${summary.steps} 步")
            else if (progress != null) add("${progress.completed}/${progress.total}")
            if (segments.isNotEmpty()) add("思考 ${segments.size} 段")
            if (summary.durationMs > 0) add(formatDuration(summary.durationMs))
        }.joinToString(" · ")
    }

    val isAlert = waiting != null || progress?.state == TaskState.Blocked || progress?.state == TaskState.Approval || progress?.state == TaskState.Waiting
    val borderColor = if (progress?.state == TaskState.Blocked) ChatFailLine else AgentHairline

    ChatCardSurface(modifier = modifier, shape = CompactCardShape, borderColor = borderColor) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 40.dp).clickable { open = !open }.padding(start = 12.dp, end = 8.dp, top = 8.dp, bottom = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            // 无底色纯主题色星标（进行中或受阻按语义提示）
            when {
                live && waiting == null -> Spinner(14.dp, color = Brand)
                progress?.state == TaskState.Blocked -> Box(Modifier.size(16.dp).clip(CircleShape).background(ChatFail), contentAlignment = Alignment.Center) {
                    XIcon(XIcons.Close, tint = Color.White, size = 10.dp)
                }
                else -> XIcon(AiHomeIcons.Spark, tint = if (isAlert) ChatFail else extra.brandInk, size = 15.dp)
            }
            Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                Text(
                    statusText,
                    style = XType.Caption.copy(fontSize = 12.5.sp, lineHeight = 16.sp),
                    fontWeight = FontWeight.SemiBold,
                    color = if (isAlert) ChatFail else MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f, fill = false),
                )
                if (metaText.isNotEmpty()) {
                    Text(
                        "· $metaText",
                        style = XType.Caption.copy(fontSize = 12.sp, lineHeight = 16.sp),
                        color = extra.ink3,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                if (summary.errors > 0 && !live) {
                    Text(
                        "· ${summary.errors} 次异常",
                        style = XType.Caption.copy(fontSize = 12.sp),
                        color = ChatFail,
                        maxLines = 1,
                    )
                }
            }
            if (live && progress?.state == TaskState.Running && onStop != null) {
                Box(Modifier.size(26.dp).clip(CircleShape).clickable(onClick = onStop), contentAlignment = Alignment.Center) {
                    XIcon(XIcons.Stop, size = 12.dp, tint = extra.ink3, description = "停止")
                }
            }
            if (live && progress?.state == TaskState.Running && onAdjust != null) {
                Box(Modifier.size(26.dp).clip(CircleShape).clickable(onClick = onAdjust), contentAlignment = Alignment.Center) {
                    XIcon(XIcons.More, size = 14.dp, tint = extra.ink3, description = "调整计划")
                }
            }
            XIcon(XIcons.ChevronDown, tint = extra.ink3, size = 15.dp, modifier = Modifier.rotated(open))
        }

        if (progress != null && progress.state != TaskState.Completed) {
            SegmentBar(progress, Modifier.padding(start = 12.dp, end = 12.dp, bottom = if (open) 8.dp else 10.dp))
        }

        if (open) {
            Box(Modifier.fillMaxWidth().height(1.dp).background(AgentHairline))
            Column(
                Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.background)
                    .padding(horizontal = 12.dp, vertical = 10.dp),
                verticalArrangement = Arrangement.spacedBy(9.dp),
            ) {
                if (tabs.size > 1) {
                    Row(
                        Modifier.clip(RoundedCornerShape(8.dp)).background(extra.fill).padding(2.dp),
                        horizontalArrangement = Arrangement.spacedBy(2.dp),
                    ) {
                        tabs.forEach { tab ->
                            val selected = tab == activeTab
                            val label = when (tab) {
                                TraceTab.Steps -> "步骤 ${summary.steps}"
                                TraceTab.Todo -> "清单 ${progress?.completed ?: 0}/${progress?.total ?: 0}"
                                TraceTab.Think -> "思考 ${segments.size}"
                            }
                            val tabShape = RoundedCornerShape(6.dp)
                            Box(
                                Modifier.clip(tabShape)
                                    .then(if (selected) Modifier.background(MaterialTheme.colorScheme.surface) else Modifier)
                                    .clickable { activeTab = tab }
                                    .padding(horizontal = 9.dp, vertical = 4.dp),
                            ) {
                                Text(
                                    label,
                                    style = XType.Caption.copy(fontSize = 11.5.sp),
                                    fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Medium,
                                    color = if (selected) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant,
                                    maxLines = 1,
                                )
                            }
                        }
                    }
                }

                when (activeTab) {
                    TraceTab.Steps -> {
                        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            stepCalls.forEach { call -> ToolRow(call) }
                        }
                    }
                    TraceTab.Todo -> if (progress != null) {
                        TaskStepsList(progress, onSend = onSend, showRecovery = showRecovery)
                    }
                    TraceTab.Think -> {
                        Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            val shown = if (live) listOf(segments.last()) else segments.take(4)
                            shown.forEach { seg ->
                                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(9.dp)) {
                                    Box(Modifier.padding(top = 3.dp).width(2.dp).heightIn(min = 16.dp).background(extra.line))
                                    Text(
                                        seg,
                                        Modifier.weight(1f),
                                        style = XType.Caption.copy(fontSize = 12.sp, lineHeight = 18.sp),
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        maxLines = 6,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                }
                            }
                            if (!live && segments.size > 4) {
                                Text("… 另有 ${segments.size - 4} 段思考已折叠", style = XType.Caption, color = extra.ink3)
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────── 独立思考与工具调用组件（兼容单独调用） ─────────────

@Composable
fun ReasoningView(reasoning: String, live: Boolean, modifier: Modifier = Modifier) {
    UnifiedTraceCard(
        reasoning = reasoning, progress = null, tools = emptyList(), live = live, waiting = null,
        onStop = null, onSend = {}, onAdjust = null, showRecovery = false, modifier = modifier,
    )
}

@Composable
fun ToolGroup(calls: List<ToolCallInfo>, live: Boolean, modifier: Modifier = Modifier) {
    UnifiedTraceCard(
        reasoning = "", progress = null, tools = calls, live = live, waiting = null,
        onStop = null, onSend = {}, onAdjust = null, showRecovery = false, modifier = modifier,
    )
}

@Composable
private fun ToolRow(call: ToolCallInfo, expandable: Boolean = true) {
    val extra = LocalExtraColors.current
    var open by rememberSaveable(call.id) { mutableStateOf(false) }
    val mark = when (call.status) {
        ToolStatus.Running -> Mark.Active
        ToolStatus.Done -> Mark.Done
        ToolStatus.Error -> Mark.Failed
    }
    val desc = remember(call.name, call.arguments) { describeCall(call.name, call.arguments) }
    Column(Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 26.dp).clip(RoundedCornerShape(8.dp))
                .let { if (expandable) it.clickable { open = !open } else it }
                .padding(vertical = 2.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            StepMark(mark)
            Text(
                toolLabel(call.name),
                style = XType.Caption.copy(fontSize = 12.sp),
                fontWeight = FontWeight.Medium,
                color = if (call.status == ToolStatus.Error) ChatFail else MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
            )
            if (desc.isNotBlank()) {
                Text(desc, Modifier.weight(1f), style = XType.Caption.copy(fontSize = 12.sp), color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            } else {
                Spacer(Modifier.weight(1f))
            }
            if (call.durationMs >= 1000) {
                Text(formatDuration(call.durationMs), style = XType.Caption.copy(fontSize = 11.sp), color = extra.ink3, maxLines = 1)
            }
            if (expandable) XIcon(XIcons.ChevronDown, tint = extra.ink3, size = 13.dp, modifier = Modifier.rotated(open))
        }
        if (open) {
            Column(Modifier.padding(start = 24.dp, top = 4.dp, bottom = 6.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                if (call.arguments.isNotBlank()) {
                    Text("参数", style = XType.Caption, color = extra.ink3)
                    CodeSnippet(call.arguments.take(1200), maxLines = 8)
                }
                if (call.output.isNotBlank()) {
                    Text("结果", style = XType.Caption, color = extra.ink3)
                    CodeSnippet(call.output.take(1200), maxLines = 8)
                }
            }
        }
    }
}

// ───────────── 任务执行卡（独立兼容接口 + 清单复用） ─────────────

@Composable
private fun TaskStepsList(progress: TaskProgress, onSend: (String) -> Unit, showRecovery: Boolean) {
    val extra = LocalExtraColors.current
    val state = progress.state
    val steps = progress.plan.steps
    val current = progress.current
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        steps.forEachIndexed { i, step ->
            val isCurrent = i == current
            val mark = when {
                step.done -> Mark.Done
                !isCurrent -> Mark.Pending
                else -> when (state) {
                    TaskState.Running -> Mark.Active
                    TaskState.Waiting -> Mark.Wait
                    TaskState.Approval -> Mark.Approval
                    TaskState.Paused -> Mark.Paused
                    TaskState.Blocked -> Mark.Failed
                    TaskState.Completed -> Mark.Done
                }
            }
            val tail = when {
                step.done -> step.evidence
                isCurrent -> when (state) {
                    TaskState.Waiting -> "等你回答"
                    TaskState.Approval -> "等待授权"
                    TaskState.Paused -> "已暂停"
                    TaskState.Blocked -> progress.message
                    else -> ""
                }
                else -> ""
            }
            Row(Modifier.fillMaxWidth().heightIn(min = 24.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                StepMark(mark)
                Text(
                    step.content,
                    Modifier.weight(1f),
                    style = XType.Caption.copy(fontSize = 12.5.sp, lineHeight = 17.sp),
                    fontWeight = if (isCurrent) FontWeight.SemiBold else FontWeight.Normal,
                    color = if (!step.done && !isCurrent) extra.ink3 else MaterialTheme.colorScheme.onSurface,
                )
                if (tail.isNotBlank()) {
                    Text(tail, Modifier.widthIn(max = 110.dp), style = XType.Caption.copy(fontSize = 11.sp), color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            }
        }
        when (state) {
            TaskState.Approval -> HintLine("需要你确认下方操作后继续", alert = true, modifier = Modifier.padding(top = 2.dp))
            TaskState.Waiting -> HintLine("小伴有个问题需要你确认", alert = true, modifier = Modifier.padding(top = 2.dp))
            TaskState.Paused -> if (showRecovery) HintLine("任务已暂停", alert = false, modifier = Modifier.padding(top = 2.dp), button = "继续完成", onButton = { onSend(ContinueTask) })
            TaskState.Blocked -> if (showRecovery) {
                ChatNoticeBar(
                    title = progress.message.ifBlank { "执行遇到阻碍，已暂停等你决定" },
                    tone = "fail",
                    actionText = "让小伴修复",
                    onAction = { onSend(FixAndContinue) },
                    modifier = Modifier.padding(top = 2.dp),
                )
            }
            else -> {}
        }
    }
}

@Composable
fun TaskCard(progress: TaskProgress, durationMs: Long, modifier: Modifier = Modifier, onStop: (() -> Unit)? = null, onSend: (String) -> Unit = {}, onAdjust: (() -> Unit)? = null, showRecovery: Boolean = true) {
    UnifiedTraceCard(
        reasoning = "", progress = progress, tools = emptyList(), live = progress.state == TaskState.Running,
        waiting = when (progress.state) {
            TaskState.Approval -> "approval"
            TaskState.Waiting -> "question"
            else -> null
        },
        onStop = onStop, onSend = onSend, onAdjust = onAdjust, showRecovery = showRecovery, modifier = modifier,
    )
}

@Composable
private fun SegmentBar(progress: TaskProgress, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(3.dp)) {
        progress.plan.steps.forEachIndexed { i, step ->
            val color = when {
                step.done -> ChatSuccess
                i != progress.current -> extra.line
                progress.state == TaskState.Running -> Brand.copy(alpha = 0.75f)
                progress.state == TaskState.Paused -> extra.ink3
                else -> ChatFail
            }
            Box(Modifier.weight(1f).height(2.5.dp).clip(RoundedCornerShape(2.dp)).background(color))
        }
    }
}

@Composable
fun TaskMiniBar(progress: TaskProgress, onClick: () -> Unit, onStop: () -> Unit, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val alert = progress.state == TaskState.Waiting || progress.state == TaskState.Approval
    Box(modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
        Row(
            Modifier.widthIn(max = 320.dp).fillMaxWidth().height(36.dp)
                .shadow(4.dp, CircleShape, ambientColor = Color.Black.copy(alpha = .12f), spotColor = Color.Black.copy(alpha = .12f))
                .clip(CircleShape).background(MaterialTheme.colorScheme.surface)
                .border(1.dp, if (alert) ChatFailLine else AgentHairline, CircleShape)
                .clickable(onClick = onClick).padding(start = 14.dp, end = 3.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            when (progress.state) {
                TaskState.Waiting -> XIcon(XIcons.Question, tint = ChatFail, size = 15.dp)
                TaskState.Approval -> XIcon(XIcons.Shield, tint = ChatFail, size = 15.dp)
                else -> Spinner(13.dp)
            }
            val now = progress.plan.steps.getOrNull(progress.current)?.content ?: "任务执行"
            Text(
                when (progress.state) { TaskState.Waiting -> "等你回答"; TaskState.Approval -> "等待授权"; else -> now },
                Modifier.weight(1f), style = XType.Secondary, fontWeight = FontWeight.SemiBold,
                color = if (alert) ChatFail else MaterialTheme.colorScheme.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis,
            )
            Text("${progress.completed}/${progress.total}", style = XType.Caption, color = extra.ink3, maxLines = 1)
            Box(Modifier.size(30.dp).clip(CircleShape).clickable(onClick = onStop), contentAlignment = Alignment.Center) {
                XIcon(XIcons.Stop, size = 13.dp, tint = extra.ink3, description = "停止")
            }
        }
    }
}

@Composable
fun AdjustPlanSheet(pending: List<String>, onDismiss: () -> Unit, onSubmit: (String) -> Unit, panelModifier: Modifier) {
    val extra = LocalExtraColors.current
    var steps by remember { mutableStateOf(pending) }
    var note by remember { mutableStateOf("") }
    val edited = steps.map { it.trim() }.filter { it.isNotEmpty() }
    val changed = edited.isNotEmpty() && edited != pending
    val ready = changed || note.isNotBlank()
    BottomSheet(onDismiss, panelModifier) {
        Text("调整后续计划", style = XType.Section, fontWeight = FontWeight.SemiBold)
        Text("已完成的步骤会保留。提交后先停止当前执行，再由小伴核对现场、调整计划并继续。", Modifier.padding(top = 4.dp), style = XType.Caption, color = extra.ink3)
        Column(Modifier.padding(top = 6.dp).heightIn(max = 220.dp).verticalScroll(rememberScrollState())) {
            steps.forEachIndexed { i, text ->
                Row(Modifier.padding(top = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    Box(
                        Modifier.size(28.dp).clip(CircleShape).clickable(enabled = i > 0) { steps = steps.toMutableList().also { list -> list.add(i - 1, list.removeAt(i)) } },
                        contentAlignment = Alignment.Center,
                    ) { if (i > 0) XIcon(XIcons.ArrowUp, tint = extra.ink3, size = 15.dp, description = "上移") }
                    val shape = RoundedCornerShape(10.dp)
                    BasicTextField(
                        value = text, onValueChange = { v -> steps = steps.mapIndexed { j, old -> if (j == i) v else old } },
                        modifier = Modifier.weight(1f).heightIn(min = 34.dp).clip(shape).background(extra.fill).padding(horizontal = 10.dp, vertical = 7.dp),
                        textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface), cursorBrush = SolidColor(Brand),
                    )
                    Box(Modifier.size(28.dp).clip(CircleShape).clickable { steps = steps.filterIndexed { j, _ -> j != i } }, contentAlignment = Alignment.Center) {
                        XIcon(XIcons.Close, tint = extra.ink3, size = 16.dp, description = "删除这一步")
                    }
                }
            }
        }
        Row(
            Modifier.padding(top = 2.dp).height(30.dp).clip(RoundedCornerShape(8.dp)).clickable { steps = steps + "" }.padding(horizontal = 4.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            XIcon(XIcons.Plus, size = 14.dp)
            Text("添加一步", style = XType.Body, fontWeight = FontWeight.SemiBold)
        }
        val boxShape = RoundedCornerShape(10.dp)
        BasicTextField(
            value = note, onValueChange = { note = it },
            modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).clip(boxShape).background(extra.fill).padding(horizontal = 10.dp, vertical = 8.dp),
            textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface), cursorBrush = SolidColor(Brand), maxLines = 4,
            decorationBox = { inner ->
                Box { if (note.isEmpty()) Text("想怎么改？例如：汇报面向管理层，5 分钟内", style = XType.Secondary, color = extra.ink3); inner() }
            },
        )
        Row(Modifier.padding(top = 8.dp).horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            listOf("更简短", "面向管理层", "跳过最后一步", "多检查一遍").forEach { chip ->
                Text(
                    chip, Modifier.height(34.dp).clip(CircleShape).background(extra.fill).clickable { note = if (note.isBlank()) chip else "${note.trim()}，$chip" }.padding(horizontal = 14.dp, vertical = 8.dp),
                    style = XType.Body, maxLines = 1,
                )
            }
        }
        Row(Modifier.padding(top = 12.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            GhostPillButton("取消", onDismiss, Modifier.weight(1f), height = 44.dp)
            PillButton("调整并继续", {
                val parts = buildList {
                    if (changed) add("后续步骤改为：\n" + edited.mapIndexed { i, step -> "${i + 1}. $step" }.joinToString("\n"))
                    if (note.isNotBlank()) add(note.trim())
                }
                onSubmit(parts.joinToString("\n\n"))
            }, Modifier.weight(1f), enabled = ready, height = 44.dp)
        }
    }
}

// ───────────── 贴底待办卡：小伴提问卡 ─────────────

@Composable
fun AskCard(pending: RemoteApprovalRequest, onAnswer: (String) -> Unit, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    var own by remember(pending.approvalId) { mutableStateOf("") }
    ChatCardSurface(modifier = modifier, shape = DockCardShape, elevated = true) {
        Column(Modifier.padding(horizontal = 14.dp, vertical = 13.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                XIcon(XIcons.Question, tint = MaterialTheme.colorScheme.onSurface, size = 16.dp)
                Text("小伴想问你", Modifier.weight(1f), style = XType.Body.copy(fontSize = 14.sp), fontWeight = FontWeight.SemiBold)
                Text(
                    "跳过",
                    Modifier.clip(RoundedCornerShape(8.dp)).clickable { onAnswer(DismissChoice) }.padding(horizontal = 6.dp, vertical = 3.dp),
                    style = XType.Caption, color = extra.ink3,
                )
            }
            Text(
                pending.question.ifBlank { pending.title },
                style = XType.Secondary.copy(fontSize = 13.sp, lineHeight = 19.sp),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            if (pending.connectors) {
                Text("这一步需要先连接服务，手机上不能完成。可以跳过，或到电脑上处理。", style = XType.Caption, color = extra.ink3)
            } else {
                if (pending.choices.isNotEmpty()) {
                    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        pending.choices.forEachIndexed { i, choice ->
                            Row(
                                Modifier.fillMaxWidth().heightIn(min = 34.dp).clip(RoundedCornerShape(10.dp))
                                    .background(extra.fill).clickable { onAnswer(choice) }
                                    .padding(horizontal = 11.dp, vertical = 7.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                            ) {
                                Box(Modifier.size(18.dp).clip(RoundedCornerShape(6.dp)).background(MaterialTheme.colorScheme.surface), contentAlignment = Alignment.Center) {
                                    Text("${i + 1}", style = XType.Caption.copy(fontSize = 11.sp), fontWeight = FontWeight.SemiBold, color = extra.ink3)
                                }
                                Text(choice, Modifier.weight(1f), style = XType.Secondary.copy(fontSize = 13.sp), fontWeight = FontWeight.Medium)
                            }
                        }
                    }
                }
                if (pending.freeText || pending.choices.isEmpty()) {
                    val shape = RoundedCornerShape(10.dp)
                    val ready = own.isNotBlank()
                    Row(
                        Modifier.fillMaxWidth().heightIn(min = 38.dp).clip(shape)
                            .background(MaterialTheme.colorScheme.background)
                            .border(1.dp, AgentHairline, shape)
                            .padding(start = 10.dp, end = 5.dp, top = 4.dp, bottom = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        XIcon(XIcons.Pen, tint = extra.ink3, size = 14.dp)
                        BasicTextField(
                            value = own, onValueChange = { own = it }, modifier = Modifier.weight(1f),
                            textStyle = XType.Secondary.copy(color = MaterialTheme.colorScheme.onSurface), maxLines = 3, cursorBrush = SolidColor(Brand),
                            decorationBox = { inner ->
                                Box(contentAlignment = Alignment.CenterStart) {
                                    if (own.isEmpty()) Text(if (pending.choices.isEmpty()) "输入你的回答，发送" else "其他想法，直接输入", color = extra.ink3, style = XType.Secondary, maxLines = 1)
                                    inner()
                                }
                            },
                        )
                        Box(
                            Modifier.size(28.dp).clip(CircleShape).background(if (ready) Brand else extra.line).clickable(enabled = ready) { onAnswer(own.trim()) },
                            contentAlignment = Alignment.Center,
                        ) { XIcon(XIcons.ArrowUp, tint = Color.White, size = 15.dp, description = "发送回答") }
                    }
                }
            }
        }
    }
}

// ───────────── 贴底待办卡：权限申请卡 ─────────────

private fun riskVerb(kind: String): String = when (kind) {
    "read" -> "读取文件"
    "write" -> "写入文件"
    "network" -> "联网访问"
    "exec" -> "运行命令"
    else -> "执行操作"
}

@Composable
fun ApprovalCard(pending: RemoteApprovalRequest, onDecision: (String) -> Unit, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val high = pending.isHighRisk
    var full by rememberSaveable(pending.approvalId) { mutableStateOf(false) }
    val tool = if (pending.tool.isBlank()) pending.title else toolLabel(pending.tool)
    ChatCardSurface(
        modifier = modifier,
        shape = DockCardShape,
        borderColor = if (high) ChatFailLine else AgentHairline,
        elevated = true,
    ) {
        Column(Modifier.padding(horizontal = 14.dp, vertical = 13.dp), verticalArrangement = Arrangement.spacedBy(9.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                XIcon(XIcons.Shield, tint = if (high) ChatFail else MaterialTheme.colorScheme.onSurface, size = 16.dp)
                Text(
                    "小伴想要${riskVerb(pending.riskKind)}：$tool",
                    Modifier.weight(1f),
                    style = XType.Body.copy(fontSize = 14.sp, lineHeight = 19.sp),
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    if (high) "高风险" else riskVerb(pending.riskKind),
                    Modifier.clip(RoundedCornerShape(6.dp))
                        .background(if (high) ChatFailSoft else extra.fill)
                        .padding(horizontal = 7.dp, vertical = 2.dp),
                    style = XType.Caption.copy(fontSize = 11.sp),
                    fontWeight = FontWeight.SemiBold,
                    color = if (high) ChatFail else MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                )
            }
            if (high) {
                Text(
                    (pending.danger?.takeIf { it.isNotBlank() } ?: "此操作不可撤销") + "。即使在自动模式下，也需要你逐次确认。",
                    style = XType.Caption.copy(fontSize = 12.sp, lineHeight = 17.sp),
                    color = ChatFail,
                )
            }
            if (pending.externalPath) Note("要访问工作区之外的路径，仅本次有效，不保存长期权限。")
            if (pending.tainted) Note("这段对话读取过外部网页内容，为防夹带诱导，本次操作需逐次确认。")
            val preview = pending.summary.ifBlank { pending.title }
            if (preview.isNotBlank()) {
                Column(verticalArrangement = Arrangement.spacedBy(3.dp)) {
                    CodeSnippet(preview, maxLines = if (full) 40 else 4)
                    if (preview.length > 120 || preview.count { it == '\n' } >= 3) {
                        Text(
                            if (full) "收起" else "查看全部",
                            Modifier.clip(RoundedCornerShape(6.dp)).clickable { full = !full }.padding(vertical = 2.dp),
                            style = XType.Caption, color = extra.ink3,
                        )
                    }
                }
            }
            val allow = pending.options.filter { it.id != "deny" && it.id != DismissChoice }
            val deny = pending.options.firstOrNull { it.id == "deny" || it.id == DismissChoice }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                if (deny != null) {
                    SmallButton(
                        deny.label,
                        if (high) BtnKind.Neutral else BtnKind.Quiet,
                        { onDecision(deny.id) },
                        if (high) Modifier.weight(1f) else Modifier,
                        height = 36.dp,
                    )
                }
                allow.reversed().forEachIndexed { idx, opt ->
                    val isPrimary = idx == allow.lastIndex
                    val kind = when {
                        high && isPrimary -> BtnKind.DangerSolid
                        isPrimary -> BtnKind.Primary
                        else -> BtnKind.Neutral
                    }
                    SmallButton(opt.label, kind, { onDecision(opt.id) }, Modifier.weight(1f), height = 36.dp)
                }
            }
        }
    }
}

@Composable
private fun Note(text: String) {
    val extra = LocalExtraColors.current
    Text(
        text,
        Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(extra.fill).padding(horizontal = 9.dp, vertical = 5.dp),
        style = XType.Caption.copy(lineHeight = 16.sp),
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

// ───────────── 留在对话里的痕迹 ─────────────

@Composable
fun RecordRow(record: ChatRecord, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val answer = record.kind == "answer"
    val ok = record.result != null
    Row(
        modifier.fillMaxWidth().heightIn(min = 30.dp).clip(RoundedCornerShape(10.dp))
            .background(extra.fill).padding(horizontal = 10.dp, vertical = 5.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(7.dp),
    ) {
        Box(
            Modifier.size(15.dp).clip(CircleShape).background(if (ok) ChatSuccess else if (answer) extra.line else ChatFail),
            contentAlignment = Alignment.Center,
        ) {
            XIcon(if (ok) XIcons.CheckBold else XIcons.Close, tint = if (ok || !answer) Color.White else extra.ink3, size = 9.5.dp)
        }
        if (answer) {
            Text(record.title, Modifier.weight(1f, fill = false), style = XType.Caption, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(
                if (ok) "→ ${record.result}" else "已跳过",
                Modifier.weight(1f, fill = ok),
                style = XType.Caption,
                color = if (ok) MaterialTheme.colorScheme.onSurface else extra.ink3,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        } else {
            Text("${record.result ?: "已拒绝"} · ${record.title}", Modifier.weight(1f), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

// ───────────── 一条小伴回复 ─────────────

class TurnExtras(
    val promptTokens: Int = 0,
    val completionTokens: Int = 0,
    val truncated: Boolean = false,
    val onContinue: (() -> Unit)? = null,
    val checkpoint: RunCheckpoint? = null,
    val onUndo: ((String) -> Unit)? = null,
    val onSave: (() -> Unit)? = null,
    val saved: Boolean = false,
    val onSwitchModel: (() -> Unit)? = null,
)

/**
 * 一条小伴回复里的所有内容：三合一过程摘要卡 → 正文 → 文件变更 → 痕迹 → 检查点/异常与操作栏。
 */
@Composable
fun AssistantTurnCards(
    text: String,
    streaming: Boolean,
    error: String?,
    reasoning: String,
    tools: List<ToolCallInfo>,
    records: List<ChatRecord>,
    waiting: String?,
    platform: PlatformActions?,
    idleText: String,
    onStop: (() -> Unit)?,
    onSend: (String) -> Unit,
    onRetry: (() -> Unit)? = null,
    onRegenerate: (() -> Unit)? = null,
    onAdjust: (() -> Unit)? = null,
    extras: TurnExtras = TurnExtras(),
    flat: Boolean = false,
    showTaskRecovery: Boolean = true,
) {
    val stopped = error == TurnCancelled
    val failure = error?.takeIf { !stopped }
    var replyActionsOpen by rememberSaveable { mutableStateOf(false) }
    val progress = remember(tools, streaming, waiting, stopped, failure) {
        taskProgress(tools, running = streaming, waiting = waiting, stopped = stopped, error = failure)
    }
    UnifiedTraceCard(
        reasoning = reasoning,
        progress = progress,
        tools = tools,
        live = streaming,
        waiting = waiting,
        onStop = onStop,
        onSend = onSend,
        onAdjust = if (streaming) onAdjust else null,
        showRecovery = showTaskRecovery,
    )
    if (text.isNotEmpty()) {
        val actions = if (flat && !streaming && failure == null) {
            Modifier.combinedClickable(
                interactionSource = null, indication = null, onLongClickLabel = "回复操作",
                onLongClick = { replyActionsOpen = !replyActionsOpen }, onClick = {},
            )
        } else Modifier
        MarkdownBubble(text, streaming, platform, modifier = actions, flat = flat)
    } else if (streaming && tools.none { it.status == ToolStatus.Running } && reasoning.isBlank() && progress == null) {
        StatusLine(if (waiting != null) "等你确认后继续…" else idleText, alert = waiting != null)
    } else if (streaming && waiting != null) {
        StatusLine("等你确认后继续…", alert = true)
    }
    if (!streaming) {
        val files = remember(tools) { changedFiles(tools) }
        if (files.isNotEmpty()) FileChangesCard(files, delivered = progress?.state == TaskState.Completed)
    }
    records.forEach { RecordRow(it) }
    if (!streaming && extras.truncated && extras.onContinue != null) {
        ChatNoticeBar(title = "回答较长，已保留当前内容", subtitle = "可从断点继续生成", tone = "warn", actionText = "继续", onAction = extras.onContinue)
    }
    if (!streaming) extras.checkpoint?.let { cp -> if (extras.onUndo != null) CheckpointNotice(cp, extras.onUndo) }
    if (!streaming && text.isNotEmpty() && failure == null && (!flat || replyActionsOpen)) {
        ReplyActions(text, platform, onRegenerate, extras)
    }
    if (stopped) ChatNoticeBar(title = "已停止生成", tone = "neutral")
    if (failure != null) {
        ChatNoticeBar(
            title = failure,
            tone = "fail",
            actionText = if (onRetry != null) "重试" else null,
            onAction = onRetry,
        )
        if (extras.onSwitchModel != null) LinkText("换个模型试试 ›", extras.onSwitchModel)
    }
}

// ───────────── 文件变更与回复操作 ─────────────

@Composable
fun FileChangesCard(files: List<ChangedFile>, delivered: Boolean, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    var open by rememberSaveable(files.size) { mutableStateOf(files.size <= 4) }
    ChatCardSurface(modifier = modifier, shape = CardShape) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 40.dp).clickable { open = !open }.padding(horizontal = 13.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            XIcon(XIcons.File, size = 15.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
            Text(
                if (delivered) "交付文件" else "已修改 ${files.size} 个文件",
                style = XType.Secondary.copy(fontSize = 13.sp),
                fontWeight = FontWeight.SemiBold,
            )
            if (delivered) {
                Text("${files.size}", Modifier.clip(RoundedCornerShape(8.dp)).background(extra.fill).padding(horizontal = 7.dp, vertical = 1.dp), style = XType.Caption, color = extra.ink3)
            }
            Spacer(Modifier.weight(1f))
            XIcon(XIcons.ChevronDown, tint = extra.ink3, size = 15.dp, modifier = Modifier.rotated(open))
        }
        if (open) {
            Box(Modifier.fillMaxWidth().height(1.dp).background(AgentHairline))
            Column(
                Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.background).padding(horizontal = 13.dp, vertical = 8.dp),
                verticalArrangement = Arrangement.spacedBy(7.dp),
            ) {
                files.forEach { file ->
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Box(
                            Modifier.size(18.dp).clip(RoundedCornerShape(5.dp))
                                .background(if (file.created) ChatSuccessSoft else ChatWarnSoft),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(
                                if (file.created) "A" else "M",
                                style = XType.Caption.copy(fontSize = 10.sp),
                                fontWeight = FontWeight.Bold,
                                color = if (file.created) ChatSuccess else ChatWarn,
                            )
                        }
                        Text(
                            file.name,
                            Modifier.weight(1f),
                            style = XType.Caption.copy(fontSize = 12.sp),
                            fontFamily = FontFamily.Monospace,
                            color = MaterialTheme.colorScheme.onSurface,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(if (file.created) "新增" else "修改", style = XType.Caption.copy(fontSize = 11.sp), color = extra.ink3, maxLines = 1)
                    }
                }
            }
        }
    }
}

@Composable
private fun ReplyActions(text: String, platform: PlatformActions?, onRegenerate: (() -> Unit)?, extras: TurnExtras) {
    val extra = LocalExtraColors.current
    var copied by remember(text) { mutableStateOf(false) }
    val reader = LocalReplyReader.current
    var mine by remember(text) { mutableStateOf(false) }
    val speaking = mine && (reader == null || reader.speaking)
    if (platform == null && onRegenerate == null && extras.onSave == null) return
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(2.dp)) {
        if (platform != null) {
            Row(
                Modifier.heightIn(min = 30.dp).clip(RoundedCornerShape(8.dp)).clickable { platform.copy(text); copied = true }.padding(horizontal = 7.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                XIcon(if (copied) XIcons.Check else XIcons.Copy, tint = if (copied) ChatSuccess else extra.ink3, size = 15.dp, description = "复制")
                if (copied) Text("已复制", style = XType.Caption, color = ChatSuccess)
            }
        }
        extras.onSave?.let { save ->
            ActionIcon(XIcons.Star, if (extras.saved) Brand else extra.ink3, if (extras.saved) "已存到资料库" else "存到资料库", save)
        }
        if (platform != null) {
            ActionIcon(XIcons.Volume, if (speaking) Brand else extra.ink3, if (speaking) "停止朗读" else "朗读") {
                if (reader != null) {
                    if (speaking) { reader.stop(); mine = false } else { reader.speak(speechText(text)); mine = true }
                } else if (speaking) { platform.stopSpeaking(); mine = false } else mine = platform.speak(speechText(text))
            }
        }
        if (onRegenerate != null) ActionIcon(XIcons.Refresh, extra.ink3, "重新生成", onRegenerate)
        Spacer(Modifier.weight(1f))
        if (extras.promptTokens + extras.completionTokens > 0) {
            Text("输入 ${formatTokens(extras.promptTokens)} · 输出 ${formatTokens(extras.completionTokens)}", style = XType.Caption, color = extra.ink3, maxLines = 1)
        }
    }
}

@Composable
private fun ActionIcon(icon: androidx.compose.ui.graphics.vector.ImageVector, tint: Color, description: String, onClick: () -> Unit) {
    Box(Modifier.size(30.dp).clip(RoundedCornerShape(8.dp)).clickable(onClick = onClick), contentAlignment = Alignment.Center) {
        XIcon(icon, tint = tint, size = 15.dp, description = description)
    }
}

@Composable
private fun CheckpointNotice(cp: RunCheckpoint, onUndo: (String) -> Unit) {
    val extra = LocalExtraColors.current
    var confirm by remember(cp.runId) { mutableStateOf(false) }
    val shape = RoundedCornerShape(12.dp)
    Column(Modifier.fillMaxWidth().clip(shape).background(extra.fill).padding(horizontal = 12.dp, vertical = 8.dp)) {
        when {
            cp.state == "undone" -> Text(cp.message.ifBlank { "已撤销本轮文件改动" }, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
            cp.state == "undoing" -> Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Spinner(12.dp)
                Text("正在撤销…", style = XType.Caption, color = extra.ink3)
            }
            confirm -> {
                Text("确认将这 ${cp.count} 个文件还原到本轮之前的状态？后续修改也会被覆盖。", style = XType.Caption, color = MaterialTheme.colorScheme.onSurface)
                Row(Modifier.padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    SmallButton("确认撤销", BtnKind.DangerSolid, { confirm = false; onUndo(cp.runId) }, height = 30.dp)
                    SmallButton("取消", BtnKind.Neutral, { confirm = false }, height = 30.dp)
                }
            }
            else -> {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    XIcon(XIcons.Refresh, tint = extra.ink3, size = 14.dp)
                    Column(Modifier.weight(1f)) {
                        Text("已保存检查点（修改 ${cp.count} 个文件）", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        if (cp.state == "failed" && cp.message.isNotBlank()) Text(cp.message, style = XType.Caption, color = ChatFail)
                    }
                    Box(
                        Modifier.clip(RoundedCornerShape(7.dp))
                            .background(MaterialTheme.colorScheme.surface)
                            .border(1.dp, AgentHairline, RoundedCornerShape(7.dp))
                            .clickable { confirm = true }
                            .padding(horizontal = 10.dp, vertical = 4.dp),
                    ) {
                        Text("撤销", style = XType.Caption, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
                    }
                }
            }
        }
    }
}

@Composable
private fun StatusLine(text: String, alert: Boolean) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
        if (alert) XIcon(XIcons.Shield, tint = ChatFail, size = 14.dp) else Spinner(13.dp)
        Text(text, style = XType.Caption.copy(fontSize = 12.5.sp), color = if (alert) ChatFail else LocalExtraColors.current.ink3)
    }
}

// ───────────── Markdown 回答（通栏舒展排版） ─────────────

private val EditorialBodyStyle = TextStyle(fontSize = 15.sp, lineHeight = 25.sp, letterSpacing = 0.08.sp)

@Composable
fun MarkdownBubble(text: String, streaming: Boolean, platform: PlatformActions?, modifier: Modifier = Modifier, flat: Boolean = false) {
    val blocks = remember(text, streaming) { parseMarkdown(if (streaming) "$text ▍" else text) }
    val onLink: (String) -> Unit = { url -> if (url.startsWith("https://") || url.startsWith("http://")) platform?.openUrl(url) }
    Column(
        if (flat) modifier.fillMaxWidth() else modifier.clip(AssistantBubbleShape).background(MaterialTheme.colorScheme.surface)
            .border(1.dp, AgentHairline, AssistantBubbleShape).padding(horizontal = 14.dp, vertical = 11.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        blocks.forEach { block -> MarkdownContentBlock(block, platform, onLink) }
    }
}

@Composable
internal fun MarkdownContentBlock(block: MdBlock, platform: PlatformActions?, onLink: (String) -> Unit) {
    val extra = LocalExtraColors.current
    val codeBg = extra.fill
    when (block) {
        is MdBlock.Heading -> Text(
            inlineText(block.inlines, codeBg, onLink),
            Modifier.padding(top = 4.dp),
            fontSize = if (block.level <= 1) 17.sp else if (block.level == 2) 16.sp else 14.5.sp,
            lineHeight = 23.sp,
            fontWeight = FontWeight.SemiBold,
        )
        is MdBlock.Paragraph -> Text(inlineText(block.inlines, codeBg, onLink), style = EditorialBodyStyle)
        is MdBlock.Bullets -> Column(verticalArrangement = Arrangement.spacedBy(5.dp)) {
            block.items.forEach { item -> ListItem("•", item, codeBg, onLink) }
        }
        is MdBlock.Numbered -> Column(verticalArrangement = Arrangement.spacedBy(5.dp)) {
            block.items.forEachIndexed { i, item -> ListItem("${block.start + i}.", item, codeBg, onLink) }
        }
        is MdBlock.Quote -> Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Box(Modifier.padding(top = 4.dp).width(2.5.dp).heightIn(min = 18.dp).background(extra.line))
            Text(inlineText(block.inlines, codeBg, onLink), Modifier.weight(1f), style = XType.Secondary.copy(lineHeight = 22.sp), color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        is MdBlock.Code -> CodeBlock(block.language, block.text, platform)
        MdBlock.Rule -> Hairline()
    }
}

@Composable
private fun ListItem(marker: String, inlines: List<MdInline>, codeBg: Color, onLink: (String) -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(marker, Modifier.width(16.dp), style = EditorialBodyStyle, color = LocalExtraColors.current.ink3)
        Text(inlineText(inlines, codeBg, onLink), Modifier.weight(1f), style = EditorialBodyStyle)
    }
}

@Composable
private fun CodeBlock(language: String, code: String, platform: PlatformActions?) {
    val extra = LocalExtraColors.current
    var copied by remember(code) { mutableStateOf(false) }
    ChatCardSurface(shape = CompactCardShape) {
        Row(
            Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.background)
                .padding(start = 12.dp, end = 6.dp, top = 5.dp, bottom = 5.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(language.ifBlank { "代码" }, Modifier.weight(1f), style = XType.Caption.copy(fontSize = 11.5.sp), color = extra.ink3, maxLines = 1)
            if (platform != null) {
                Row(
                    Modifier.heightIn(min = 26.dp).clip(RoundedCornerShape(6.dp))
                        .clickable { platform.copy(code); copied = true }.padding(horizontal = 7.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    XIcon(if (copied) XIcons.Check else XIcons.Copy, tint = if (copied) ChatSuccess else MaterialTheme.colorScheme.onSurfaceVariant, size = 12.dp)
                    Text(if (copied) "已复制" else "复制", style = XType.Caption.copy(fontSize = 11.5.sp), color = if (copied) ChatSuccess else MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
        Box(Modifier.fillMaxWidth().height(1.dp).background(AgentHairline))
        Text(
            code,
            Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 12.dp, vertical = 10.dp),
            style = XType.Caption.copy(fontSize = 12.sp, lineHeight = 19.sp),
            color = MaterialTheme.colorScheme.onSurface,
            fontFamily = FontFamily.Monospace,
        )
    }
}

private fun inlineText(inlines: List<MdInline>, codeBg: Color, onLink: (String) -> Unit): AnnotatedString = buildAnnotatedString {
    inlines.forEach { part ->
        when (part) {
            is MdInline.Text -> append(part.text)
            is MdInline.Bold -> withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) { append(part.text) }
            is MdInline.Italic -> withStyle(SpanStyle(fontStyle = FontStyle.Italic)) { append(part.text) }
            is MdInline.Code -> withStyle(SpanStyle(fontFamily = FontFamily.Monospace, background = codeBg, fontSize = 13.sp)) { append(" ${part.text} ") }
            is MdInline.Link -> withLink(
                LinkAnnotation.Clickable(
                    tag = part.url,
                    styles = TextLinkStyles(SpanStyle(color = Brand, textDecoration = TextDecoration.Underline)),
                    linkInteractionListener = LinkInteractionListener { onLink(part.url) },
                ),
            ) { append(part.label) }
        }
    }
}

private fun Modifier.rotated(open: Boolean): Modifier = this.rotate(if (open) 180f else 0f)
