package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
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
import xyz.xiangban.xiaoban.common.community.ChatRecord
import xyz.xiangban.xiaoban.common.community.MdBlock
import xyz.xiangban.xiaoban.common.community.MdInline
import xyz.xiangban.xiaoban.common.community.TaskProgress
import xyz.xiangban.xiaoban.common.community.TaskState
import xyz.xiangban.xiaoban.common.community.ToolCallInfo
import xyz.xiangban.xiaoban.common.community.ToolStatus
import xyz.xiangban.xiaoban.common.community.ChangedFile
import xyz.xiangban.xiaoban.common.community.formatTokens
import xyz.xiangban.xiaoban.common.community.speechText
import xyz.xiangban.xiaoban.common.community.activitySummary
import xyz.xiangban.xiaoban.common.community.changedFiles
import xyz.xiangban.xiaoban.common.community.describeCall
import xyz.xiangban.xiaoban.common.community.formatDuration
import xyz.xiangban.xiaoban.common.community.parseMarkdown
import xyz.xiangban.xiaoban.common.community.taskProgress
import xyz.xiangban.xiaoban.common.community.toolLabel
import xyz.xiangban.xiaoban.common.link.RemoteApprovalRequest
import xyz.xiangban.xiaoban.common.link.RunCheckpoint
import xyz.xiangban.xiaoban.common.link.TurnCancelled

/*
 * 聊天里的状态卡片，按设计稿 45–51 号画板做：一律紧凑（行高 28–36，按钮高 28），不留大段空白。
 * 设备聊天和小伴AI 对话共用；数据在 community-core 的 ChatModels.kt。
 */

private val CardShape = RoundedCornerShape(16.dp)
private const val DismissChoice = "__dismiss__"
private const val ContinueTask = "继续完成刚才的任务"
private const val FixAndContinue = "请修复上面的问题，然后继续完成任务"

// ───────────── 小部件 ─────────────

@Composable
private fun Spinner(size: Dp = 12.dp) {
    CircularProgressIndicator(Modifier.size(size), color = MaterialTheme.colorScheme.onSurface, strokeWidth = 2.dp)
}

@Composable
private fun CardSurface(modifier: Modifier = Modifier, tint: Color? = null, content: @Composable ColumnScope.() -> Unit) {
    Column(
        modifier.fillMaxWidth().clip(CardShape).background(tint ?: MaterialTheme.colorScheme.surface),
        content = content,
    )
}

private enum class Mark { Done, Active, Approval, Paused, Blocked, Failed, Wait, Pending }

/** 步骤前面 16dp 的小标记：完成是绿底白对号，等授权、受阻、等回答是红，失败是红底白叉，暂停是灰，没开始是空心圈。 */
@Composable
private fun StepMark(kind: Mark) {
    val extra = LocalExtraColors.current
    Box(Modifier.size(16.dp), contentAlignment = Alignment.Center) {
        when (kind) {
            Mark.Active -> Spinner(14.dp)
            Mark.Pending -> Box(Modifier.size(14.dp).clip(CircleShape).border(1.5.dp, extra.line, CircleShape))
            else -> {
                val (bg, icon) = when (kind) {
                    Mark.Done -> Success to XIcons.CheckBold
                    Mark.Approval -> Danger to XIcons.Shield
                    Mark.Paused -> extra.ink3 to XIcons.Pause
                    Mark.Failed -> Danger to XIcons.Close
                    Mark.Blocked -> Danger to XIcons.Warning
                    else -> Danger to XIcons.Question
                }
                Box(Modifier.size(16.dp).clip(CircleShape).background(bg), contentAlignment = Alignment.Center) {
                    XIcon(icon, tint = Color.White, size = 10.dp)
                }
            }
        }
    }
}

/** 卡片里按钮的三级：主操作＝主题色浅底加主题色字，次要＝浅灰底，弱操作（拒绝、取消）＝无底色灰字；高风险卡片里的两个按钮用白底。 */
private enum class BtnKind { Primary, Neutral, Quiet, OnAlertAllow, OnAlertDeny }

/** 卡片里的按钮：默认高 28（提示条里），授权和选择卡片里用 40 并撑满一行等宽排开。 */
@Composable
private fun SmallButton(text: String, kind: BtnKind, onClick: () -> Unit, modifier: Modifier = Modifier, height: Dp = 28.dp) {
    val extra = LocalExtraColors.current
    val base = modifier.height(height).clip(CircleShape)
    val styled = when (kind) {
        BtnKind.Primary -> base.background(extra.brandSoft)
        BtnKind.Neutral -> base.background(extra.fill)
        BtnKind.Quiet -> base
        BtnKind.OnAlertAllow, BtnKind.OnAlertDeny -> base.background(MaterialTheme.colorScheme.surface)
    }
    val color = when (kind) {
        BtnKind.Primary -> extra.brandInk
        BtnKind.Neutral -> MaterialTheme.colorScheme.onSurface
        BtnKind.Quiet -> MaterialTheme.colorScheme.onSurfaceVariant
        BtnKind.OnAlertAllow -> Danger
        BtnKind.OnAlertDeny -> MaterialTheme.colorScheme.onSurface
    }
    Box(styled.clickable(onClick = onClick).padding(horizontal = 12.dp), contentAlignment = Alignment.Center) {
        Text(text, color = color, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

@Composable
private fun CodeSnippet(text: String, modifier: Modifier = Modifier, maxLines: Int = 6) {
    Text(
        text, modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(horizontal = 8.dp, vertical = 5.dp),
        fontSize = 12.sp, lineHeight = 17.sp, fontFamily = FontFamily.Monospace, maxLines = maxLines, overflow = TextOverflow.Ellipsis,
    )
}

/** 提示灰底条：一行说明，右边可以有一个按钮。 */
@Composable
private fun HintLine(text: String, alert: Boolean, modifier: Modifier = Modifier, button: String? = null, onButton: () -> Unit = {}) {
    val extra = LocalExtraColors.current
    val shape = RoundedCornerShape(8.dp)
    Row(
        modifier.fillMaxWidth().clip(shape).background(if (alert) DangerSoft else extra.fill).padding(horizontal = 8.dp, vertical = 5.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text(text, Modifier.weight(1f), fontSize = 12.sp, lineHeight = 17.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (button != null) SmallButton(button, BtnKind.Primary, onButton)
    }
}

// ───────────── 思考 ─────────────

/** 思考过程：回答还没开始时实时展示；结束后收成一行“思考 N 段”，点开看前几段。 */
@Composable
fun ReasoningView(reasoning: String, live: Boolean, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val segments = remember(reasoning) { reasoning.split(Regex("\n\\s*\n")).map { it.trim() }.filter { it.isNotEmpty() } }
    if (segments.isEmpty()) return
    var open by rememberSaveable { mutableStateOf(false) }
    if (live) {
        Column(modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(extra.fill).padding(horizontal = 8.dp, vertical = 4.dp)) {
            Row(Modifier.heightIn(min = 24.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                XIcon(XIcons.SparkleSmall, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 14.dp)
                Text("思考中…", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Spinner(11.dp)
            }
            Text(segments.last(), fontSize = 12.sp, lineHeight = 19.sp, color = extra.ink3, maxLines = 6, overflow = TextOverflow.Ellipsis)
        }
        return
    }
    Column(modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 28.dp).clip(RoundedCornerShape(8.dp)).background(extra.fill).clickable { open = !open }.padding(horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            XIcon(XIcons.SparkleSmall, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 14.dp)
            Text("思考 ${segments.size} 段", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Text("已完成", Modifier.weight(1f), fontSize = 12.sp, color = extra.ink3)
            XIcon(XIcons.ChevronDown, tint = extra.ink3, size = 14.dp, modifier = Modifier.rotated(open))
        }
        if (open) {
            Column(Modifier.padding(start = 4.dp, end = 4.dp, top = 4.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                segments.take(3).forEach { Text(it, Modifier.fillMaxWidth().clip(RoundedCornerShape(6.dp)).background(extra.fill).padding(horizontal = 8.dp, vertical = 3.dp), fontSize = 12.sp, lineHeight = 19.sp, color = extra.ink3, maxLines = 6, overflow = TextOverflow.Ellipsis) }
                if (segments.size > 3) Text("… 另 ${segments.size - 3} 段", Modifier.padding(start = 8.dp), fontSize = 12.sp, color = extra.ink3)
            }
        }
    }
}

// ───────────── 工具调用组 ─────────────

/** 一轮里所有工具调用合成一张折叠卡：摘要“N 步 · 用时 · 异常”，进行中显示最新一步，展开后每步可再展开看内容和结果。 */
@Composable
fun ToolGroup(calls: List<ToolCallInfo>, live: Boolean, modifier: Modifier = Modifier) {
    val rows = remember(calls) { calls.filter { it.name != "todo" } }
    if (rows.isEmpty()) return
    val extra = LocalExtraColors.current
    val summary = remember(calls) { activitySummary(calls) }
    var open by rememberSaveable { mutableStateOf(false) }
    CardSurface(modifier) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 30.dp).clickable { open = !open }.padding(horizontal = 10.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            if (live) Spinner(12.dp)
            Text("本轮执行过程", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            Text(
                "· ${summary.steps} 步" + if (summary.durationMs > 0) " · 用时 ${formatDuration(summary.durationMs)}" else "",
                fontSize = 12.sp, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis,
            )
            if (summary.errors > 0) Text("· ${summary.errors} 次异常", fontSize = 12.sp, color = Danger, maxLines = 1)
            Spacer(Modifier.weight(1f))
            XIcon(XIcons.ChevronDown, tint = extra.ink3, size = 14.dp, modifier = Modifier.rotated(open))
        }
        if (open) {
            Column(Modifier.padding(horizontal = 10.dp)) { rows.forEach { call -> Hairline(); ToolRow(call) } }
        } else if (live) {
            val latest = rows.lastOrNull { it.status == ToolStatus.Running } ?: rows.last()
            Hairline()
            Box(Modifier.padding(horizontal = 10.dp)) { ToolRow(latest, expandable = false) }
        }
    }
}

@Composable
private fun ToolRow(call: ToolCallInfo, expandable: Boolean = true) {
    val extra = LocalExtraColors.current
    var open by rememberSaveable(call.id) { mutableStateOf(false) }
    val mark = when (call.status) { ToolStatus.Running -> Mark.Active; ToolStatus.Done -> Mark.Done; ToolStatus.Error -> Mark.Failed }
    val desc = remember(call.name, call.arguments) { describeCall(call.name, call.arguments) }
    Column(Modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 28.dp).let { if (expandable) it.clickable { open = !open } else it },
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            StepMark(mark)
            Text(toolLabel(call.name), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = if (call.status == ToolStatus.Error) Danger else MaterialTheme.colorScheme.onSurface, maxLines = 1)
            Text(desc, Modifier.weight(1f), fontSize = 12.sp, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (call.durationMs >= 1000) Text(formatDuration(call.durationMs), fontSize = 12.sp, color = extra.ink3, maxLines = 1)
            if (expandable) XIcon(XIcons.ChevronDown, tint = extra.ink3, size = 13.dp, modifier = Modifier.rotated(open))
        }
        if (open) {
            Column(Modifier.padding(start = 22.dp, bottom = 6.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                if (call.arguments.isNotBlank()) {
                    Text("内容", fontSize = 12.sp, color = extra.ink3)
                    CodeSnippet(call.arguments.take(1200), maxLines = 8)
                }
                if (call.output.isNotBlank()) {
                    Text("结果", fontSize = 12.sp, color = extra.ink3)
                    CodeSnippet(call.output.take(1200), maxLines = 8)
                }
            }
        }
    }
}

// ───────────── 任务执行卡 ─────────────

private fun TaskState.chip(): String? = when (this) {
    TaskState.Waiting -> "等你回答"
    TaskState.Approval -> "等待授权"
    TaskState.Blocked -> "未完成"
    TaskState.Paused -> "已暂停"
    else -> null
}

/** 任务计划（来自 `todo` 工具）：分段进度条加步骤列表。进行中带停止；暂停和受阻时给“继续完成”“让小伴修复”。 */
@Composable
fun TaskCard(progress: TaskProgress, durationMs: Long, modifier: Modifier = Modifier, onStop: (() -> Unit)? = null, onSend: (String) -> Unit = {}, onAdjust: (() -> Unit)? = null) {
    val extra = LocalExtraColors.current
    val state = progress.state
    val completed = state == TaskState.Completed
    var open by rememberSaveable(completed) { mutableStateOf(!completed) }
    val alert = state == TaskState.Waiting || state == TaskState.Approval || state == TaskState.Blocked
    val steps = progress.plan.steps
    val current = progress.current
    CardSurface(modifier) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 36.dp).clickable { open = !open }.padding(start = 10.dp, end = 2.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            when (state) {
                TaskState.Running -> Spinner(16.dp)
                TaskState.Completed -> Box(Modifier.size(20.dp).clip(CircleShape).background(Success), contentAlignment = Alignment.Center) { XIcon(XIcons.CheckBold, tint = Color.White, size = 12.dp) }
                TaskState.Paused -> XIcon(XIcons.Pause, tint = extra.ink3, size = 18.dp)
                TaskState.Blocked -> XIcon(XIcons.Warning, tint = Danger, size = 18.dp)
                TaskState.Approval -> XIcon(XIcons.Shield, tint = Danger, size = 18.dp)
                TaskState.Waiting -> XIcon(XIcons.Question, tint = Danger, size = 18.dp)
            }
            Column(Modifier.weight(1f)) {
                if (open || completed || current < 0) {
                    Text(if (completed) "任务完成" else "任务执行", fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
                } else {
                    Text(steps[current].content, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("任务执行", fontSize = 12.sp, color = extra.ink3, maxLines = 1)
                }
            }
            state.chip()?.let { chip ->
                Text(
                    chip, Modifier.clip(RoundedCornerShape(10.dp)).background(if (alert) DangerSoft else extra.fill).padding(horizontal = 8.dp, vertical = 2.dp),
                    fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = if (alert) Danger else extra.ink3, maxLines = 1,
                )
            }
            if (completed && durationMs > 0) Text("用时 ${formatDuration(durationMs)}", fontSize = 12.sp, color = extra.ink3, maxLines = 1)
            Text("${progress.completed}/${progress.total}", fontSize = 12.sp, color = extra.ink3, maxLines = 1)
            if (state == TaskState.Running && onStop != null) {
                Box(Modifier.size(32.dp).clip(CircleShape).clickable(onClick = onStop), contentAlignment = Alignment.Center) { XIcon(XIcons.Stop, size = 14.dp, description = "停止") }
            }
            if (state == TaskState.Running && onAdjust != null) {
                Box(Modifier.size(32.dp).clip(CircleShape).clickable(onClick = onAdjust), contentAlignment = Alignment.Center) { XIcon(XIcons.More, tint = extra.ink3, size = 16.dp, description = "调整计划") }
            }
            XIcon(XIcons.ChevronDown, tint = extra.ink3, size = 16.dp, modifier = Modifier.padding(end = 8.dp).rotated(open))
        }
        if (!completed) SegmentBar(progress, Modifier.padding(start = 10.dp, end = 10.dp, bottom = 6.dp))
        if (open) {
            Hairline()
            Column(Modifier.padding(start = 10.dp, end = 10.dp, bottom = 6.dp)) {
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
                    Row(Modifier.fillMaxWidth().heightIn(min = 28.dp).padding(horizontal = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        StepMark(mark)
                        Text(
                            step.content, Modifier.weight(1f), fontSize = 14.sp, lineHeight = 18.sp,
                            fontWeight = if (isCurrent) FontWeight.SemiBold else FontWeight.Normal,
                            color = if (!step.done && !isCurrent) extra.ink3 else MaterialTheme.colorScheme.onSurface,
                        )
                        if (tail.isNotBlank()) Text(tail, Modifier.widthIn(max = 120.dp), fontSize = 12.sp, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                }
                when (state) {
                    TaskState.Approval -> HintLine("需要你确认下面这个操作", alert = true, modifier = Modifier.padding(top = 6.dp))
                    TaskState.Waiting -> HintLine("小伴有个问题要问你", alert = true, modifier = Modifier.padding(top = 6.dp))
                    TaskState.Paused -> HintLine("已暂停", alert = false, modifier = Modifier.padding(top = 6.dp), button = "继续完成", onButton = { onSend(ContinueTask) })
                    TaskState.Blocked -> {
                        Text(
                            progress.message.ifBlank { "有一步没有成功，已停下等你决定。" } + " 可以让小伴修复后继续。",
                            Modifier.padding(top = 6.dp), fontSize = 12.sp, lineHeight = 17.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Row(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            SmallButton("让小伴修复", BtnKind.Primary, { onSend(FixAndContinue) })
                        }
                    }
                    else -> {}
                }
            }
        }
    }
}

/** 分段进度条：每一步一小段，做完的深色，正在做的按状态上色。 */
@Composable
private fun SegmentBar(progress: TaskProgress, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val ink = MaterialTheme.colorScheme.onSurface
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(3.dp)) {
        progress.plan.steps.forEachIndexed { i, step ->
            val color = when {
                step.done -> ink
                i != progress.current -> extra.line
                progress.state == TaskState.Running -> ink.copy(alpha = 0.45f)
                progress.state == TaskState.Paused -> extra.ink3
                else -> Danger
            }
            Box(Modifier.weight(1f).height(3.dp).clip(RoundedCornerShape(2.dp)).background(color))
        }
    }
}

/** 录音、识别、朗读时输入框上方的一行：红点加状态，右边“取消/停止”。 */
@Composable
fun VoiceBar(status: String, recording: Boolean, hint: String?, onCancel: () -> Unit, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    CardSurface(modifier) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 40.dp).padding(start = 12.dp, end = 4.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            if (recording) Box(Modifier.size(9.dp).clip(CircleShape).background(Danger)) else Spinner(12.dp)
            Column(Modifier.weight(1f)) {
                Text(status, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
                if (hint != null) Text(hint, fontSize = 12.sp, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text(if (recording) "取消" else "停止", Modifier.clip(RoundedCornerShape(8.dp)).clickable(onClick = onCancel).padding(horizontal = 10.dp, vertical = 8.dp), fontSize = 14.sp, color = extra.ink3)
        }
    }
}

/** 任务卡滚出屏幕后吸在输入框上方的一行：当前步骤、进度、停止；点一下回到任务卡。 */
@Composable
fun TaskMiniBar(progress: TaskProgress, onClick: () -> Unit, onStop: () -> Unit, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val alert = progress.state == TaskState.Waiting || progress.state == TaskState.Approval
    Box(modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
        Row(
            Modifier.widthIn(max = 320.dp).fillMaxWidth().height(36.dp).shadow(4.dp, CircleShape).clip(CircleShape).background(MaterialTheme.colorScheme.surface)
                .clickable(onClick = onClick).padding(start = 14.dp, end = 3.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            when (progress.state) {
                TaskState.Waiting -> XIcon(XIcons.Question, tint = Danger, size = 16.dp)
                TaskState.Approval -> XIcon(XIcons.Shield, tint = Danger, size = 16.dp)
                else -> Spinner(14.dp)
            }
            val now = progress.plan.steps.getOrNull(progress.current)?.content ?: "任务执行"
            Text(
                when (progress.state) { TaskState.Waiting -> "等你回答"; TaskState.Approval -> "等待授权"; else -> now },
                Modifier.weight(1f), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = if (alert) Danger else MaterialTheme.colorScheme.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis,
            )
            Text("${progress.completed}/${progress.total}", fontSize = 12.sp, color = extra.ink3, maxLines = 1)
            Box(Modifier.size(30.dp).clip(CircleShape).clickable(onClick = onStop), contentAlignment = Alignment.Center) { XIcon(XIcons.Stop, size = 14.dp, description = "停止") }
        }
    }
}

/** 调整后续计划：已完成的步骤保留，后面的步骤可以改字、上移、删除、添加，再补一句想怎么改。提交后先停当前这一轮，再由小伴核对现场、调整并继续。 */
@Composable
fun AdjustPlanSheet(pending: List<String>, onDismiss: () -> Unit, onSubmit: (String) -> Unit, panelModifier: Modifier) {
    val extra = LocalExtraColors.current
    var steps by remember { mutableStateOf(pending) }
    var note by remember { mutableStateOf("") }
    val edited = steps.map { it.trim() }.filter { it.isNotEmpty() }
    val changed = edited.isNotEmpty() && edited != pending
    val ready = changed || note.isNotBlank()
    BottomSheet(onDismiss, panelModifier) {
        Text("调整后续计划", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
        Text("已完成的步骤会保留。提交后先停止当前执行，再由小伴核对现场、调整计划并继续。", Modifier.padding(top = 4.dp), fontSize = 12.sp, lineHeight = 18.sp, color = extra.ink3)
        Column(Modifier.padding(top = 6.dp).heightIn(max = 220.dp).verticalScroll(rememberScrollState())) {
            steps.forEachIndexed { i, text ->
                Row(Modifier.padding(top = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    Box(
                        Modifier.size(28.dp).clip(CircleShape).clickable(enabled = i > 0) { steps = steps.toMutableList().also { list -> list.add(i - 1, list.removeAt(i)) } },
                        contentAlignment = Alignment.Center,
                    ) { if (i > 0) XIcon(XIcons.ArrowUp, tint = extra.ink3, size = 15.dp, description = "上移") }
                    val shape = RoundedCornerShape(8.dp)
                    BasicTextField(
                        value = text, onValueChange = { v -> steps = steps.mapIndexed { j, old -> if (j == i) v else old } }, modifier = Modifier.weight(1f).heightIn(min = 32.dp).clip(shape).background(extra.fill).padding(horizontal = 10.dp, vertical = 6.dp),
                        textStyle = TextStyle(fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurface), cursorBrush = SolidColor(Brand),
                    )
                    Box(Modifier.size(28.dp).clip(CircleShape).clickable { steps = steps.filterIndexed { j, _ -> j != i } }, contentAlignment = Alignment.Center) { XIcon(XIcons.Close, tint = extra.ink3, size = 16.dp, description = "删除这一步") }
                }
            }
        }
        Row(
            Modifier.padding(top = 2.dp).height(30.dp).clip(RoundedCornerShape(8.dp)).clickable { steps = steps + "" }.padding(horizontal = 4.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            XIcon(XIcons.Plus, size = 14.dp)
            Text("添加一步", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
        }
        val boxShape = RoundedCornerShape(10.dp)
        BasicTextField(
            value = note, onValueChange = { note = it }, modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp).clip(boxShape).background(extra.fill).padding(horizontal = 10.dp, vertical = 8.dp),
            textStyle = TextStyle(fontSize = 14.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurface), cursorBrush = SolidColor(Brand), maxLines = 4,
            decorationBox = { inner ->
                Box { if (note.isEmpty()) Text("想怎么改？例如：汇报面向管理层，5 分钟内", fontSize = 14.sp, lineHeight = 19.sp, color = extra.ink3); inner() }
            },
        )
        Row(Modifier.padding(top = 8.dp).horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            listOf("更简短", "面向管理层", "跳过最后一步", "多检查一遍").forEach { chip ->
                Text(
                    chip, Modifier.height(34.dp).clip(CircleShape).background(extra.fill).clickable { note = if (note.isBlank()) chip else "${note.trim()}，$chip" }.padding(horizontal = 14.dp, vertical = 8.dp),
                    fontSize = 14.sp, maxLines = 1,
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

// ───────────── 选择卡 ─────────────

/** 小伴向你提问：问题、编号选项（整行可点）、自己输入回答、跳过。 */
@Composable
fun AskCard(pending: RemoteApprovalRequest, onAnswer: (String) -> Unit, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    var own by remember(pending.approvalId) { mutableStateOf("") }
    CardSurface(modifier) {
        Column(Modifier.padding(start = 10.dp, end = 10.dp, top = 8.dp, bottom = 10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Box(Modifier.size(18.dp).clip(CircleShape).background(Danger), contentAlignment = Alignment.Center) { XIcon(XIcons.Question, tint = Color.White, size = 11.dp) }
                Text("小伴想问你", Modifier.weight(1f), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = extra.ink3)
                Text("跳过", Modifier.clip(RoundedCornerShape(6.dp)).clickable { onAnswer(DismissChoice) }.padding(horizontal = 6.dp, vertical = 3.dp), fontSize = 12.sp, color = extra.ink3)
            }
            Text(pending.question.ifBlank { pending.title }, Modifier.padding(top = 4.dp), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, lineHeight = 20.sp)
            if (pending.connectors) {
                Text("这一步需要先连接服务，手机上不能完成。可以跳过，或到电脑上处理。", Modifier.padding(top = 4.dp), fontSize = 12.sp, lineHeight = 17.sp, color = extra.ink3)
            } else {
                pending.choices.forEachIndexed { i, choice ->
                    Row(
                        Modifier.padding(top = 6.dp).fillMaxWidth().heightIn(min = 30.dp).clip(RoundedCornerShape(10.dp)).background(extra.fill).clickable { onAnswer(choice) }.padding(horizontal = 10.dp, vertical = 5.dp),
                        verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        Box(Modifier.size(20.dp).clip(RoundedCornerShape(6.dp)).background(MaterialTheme.colorScheme.surface), contentAlignment = Alignment.Center) {
                            Text("${i + 1}", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = extra.ink3)
                        }
                        Text(choice, Modifier.weight(1f), fontSize = 14.sp, lineHeight = 19.sp)
                    }
                }
                if (pending.freeText || pending.choices.isEmpty()) {
                    val shape = RoundedCornerShape(10.dp)
                    val ready = own.isNotBlank()
                    Row(
                        Modifier.padding(top = 6.dp).fillMaxWidth().heightIn(min = 34.dp).clip(shape).background(extra.fill).padding(start = 10.dp, end = 4.dp, top = 3.dp, bottom = 3.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        XIcon(XIcons.Pen, tint = extra.ink3, size = 15.dp)
                        BasicTextField(
                            value = own, onValueChange = { own = it }, modifier = Modifier.weight(1f),
                            textStyle = TextStyle(fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurface), maxLines = 3, cursorBrush = SolidColor(Brand),
                            decorationBox = { inner ->
                                Box(contentAlignment = Alignment.CenterStart) {
                                    if (own.isEmpty()) Text(if (pending.choices.isEmpty()) "输入你的回答，发送" else "其他想法，发送", color = extra.ink3, fontSize = 14.sp, maxLines = 1)
                                    inner()
                                }
                            },
                        )
                        Box(
                            Modifier.size(26.dp).clip(CircleShape).background(if (ready) Brand else extra.line).clickable(enabled = ready) { onAnswer(own.trim()) },
                            contentAlignment = Alignment.Center,
                        ) { XIcon(XIcons.ArrowUp, tint = Color.White, size = 15.dp, description = "发送回答") }
                    }
                }
            }
        }
    }
}

// ───────────── 权限申请卡 ─────────────

private fun riskVerb(kind: String): String = when (kind) {
    "read" -> "读取文件"
    "write" -> "写入文件"
    "network" -> "联网访问"
    "exec" -> "运行命令"
    else -> "执行操作"
}

/** 电脑上的小伴要做一件需要你点头的事：做什么、为什么要特别小心、预览，以及三个紧凑按钮。 */
@Composable
fun ApprovalCard(pending: RemoteApprovalRequest, onDecision: (String) -> Unit, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val high = pending.isHighRisk
    var full by rememberSaveable(pending.approvalId) { mutableStateOf(false) }
    val tool = if (pending.tool.isBlank()) pending.title else toolLabel(pending.tool)
    CardSurface(modifier, tint = if (high) DangerSoft else null) {
        Column(Modifier.padding(start = 10.dp, end = 10.dp, top = 8.dp, bottom = 10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                XIcon(XIcons.Shield, tint = if (high) Danger else MaterialTheme.colorScheme.onSurface, size = 16.dp)
                Text(
                    "小伴想要${riskVerb(pending.riskKind)}：$tool", Modifier.weight(1f), fontSize = 14.sp, lineHeight = 18.sp, maxLines = 2, overflow = TextOverflow.Ellipsis,
                )
            }
            if (high) {
                Text(
                    "高风险操作" + (pending.danger?.takeIf { it.isNotBlank() }?.let { "：$it" } ?: "") + "。即使在“完全信任”下，这类操作也每次都要你确认。",
                    Modifier.padding(top = 6.dp), fontSize = 12.sp, lineHeight = 17.sp, fontWeight = FontWeight.SemiBold, color = Danger,
                )
            }
            if (pending.externalPath) Note("要访问工作区之外的路径。只授权这次，不保存长期权限。")
            if (pending.tainted) Note("这段对话读过网页等外部内容，其中可能夹带诱导，所以这次每一步都要你确认。")
            val preview = pending.summary.ifBlank { pending.title }
            if (preview.isNotBlank()) {
                Column(Modifier.padding(top = 6.dp)) {
                    CodeSnippet(preview, maxLines = if (full) 40 else 4)
                    if (preview.length > 120 || preview.count { it == '\n' } >= 3) {
                        Text(if (full) "收起" else "查看全部", Modifier.clip(RoundedCornerShape(6.dp)).clickable { full = !full }.padding(vertical = 3.dp), fontSize = 12.sp, color = extra.ink3)
                    }
                }
            }
            val allow = pending.options.filter { it.id != "deny" && it.id != DismissChoice }
            val deny = pending.options.firstOrNull { it.id == "deny" || it.id == DismissChoice }
            Row(Modifier.padding(top = 10.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                allow.forEachIndexed { i, opt ->
                    val kind = if (high) BtnKind.OnAlertAllow else if (i == 0) BtnKind.Primary else BtnKind.Neutral
                    SmallButton(opt.label, kind, { onDecision(opt.id) }, Modifier.weight(1f), height = 40.dp)
                }
                if (deny != null) SmallButton(deny.label, if (high) BtnKind.OnAlertDeny else BtnKind.Quiet, { onDecision(deny.id) }, Modifier.weight(1f), height = 40.dp)
            }
        }
    }
}

@Composable
private fun Note(text: String) {
    val extra = LocalExtraColors.current
    Text(text, Modifier.padding(top = 6.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(extra.fill).padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 12.sp, lineHeight = 17.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
}

// ───────────── 留在对话里的痕迹 ─────────────

/** 回答过的问题、处理过的审批：收成一行。 */
@Composable
fun RecordRow(record: ChatRecord, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val answer = record.kind == "answer"
    val ok = record.result != null
    Row(
        modifier.fillMaxWidth().heightIn(min = 28.dp).clip(RoundedCornerShape(8.dp)).background(extra.fill).padding(horizontal = 8.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        // 成功是绿底白对号；拒绝是红底白叉；跳过一个问题只是灰色。
        Box(Modifier.size(16.dp).clip(CircleShape).background(if (ok) Success else if (answer) extra.line else Danger), contentAlignment = Alignment.Center) {
            XIcon(if (ok) XIcons.CheckBold else XIcons.Close, tint = if (ok || !answer) Color.White else extra.ink3, size = 10.dp)
        }
        if (answer) {
            Text(record.title, Modifier.weight(1f, fill = false), fontSize = 12.sp, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(if (ok) "→ ${record.result}" else "已跳过", Modifier.weight(1f, fill = ok), fontSize = 12.sp, color = if (ok) MaterialTheme.colorScheme.onSurface else extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
        } else {
            Text("${record.result ?: "已拒绝"} · ${record.title}", Modifier.weight(1f), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

// ───────────── 一条小伴回复 ─────────────

/** 一条回复的附加信息和动作（设备聊天才有的：用量、被截断、可撤销、存资料库、切换模型）。 */
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
 * 一条小伴回复里的所有内容，按设计稿顺序：思考 → 任务执行卡 → 工具调用组 → 回答 → 留下的痕迹 → 提示。
 * [waiting] 只给正在进行的那一轮：`question` 或 `approval`。
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
) {
    val stopped = error == TurnCancelled
    val failure = error?.takeIf { !stopped }
    if (reasoning.isNotBlank()) ReasoningView(reasoning, live = streaming && text.isEmpty())
    val progress = remember(tools, streaming, waiting, stopped, failure) { taskProgress(tools, running = streaming, waiting = waiting, stopped = stopped, error = failure) }
    if (progress != null) TaskCard(progress, remember(tools) { activitySummary(tools).durationMs }, onStop = onStop, onSend = onSend, onAdjust = if (streaming) onAdjust else null)
    ToolGroup(tools, live = streaming)
    if (text.isNotEmpty()) {
        MarkdownBubble(text, streaming, platform)
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
    if (!streaming && extras.truncated && extras.onContinue != null) HintLine("回答太长，被截断了。", alert = false, button = "继续", onButton = extras.onContinue)
    if (!streaming) extras.checkpoint?.let { cp -> if (extras.onUndo != null) CheckpointNotice(cp, extras.onUndo) }
    if (!streaming && text.isNotEmpty() && failure == null) ReplyActions(text, platform, onRegenerate, extras)
    if (stopped) HintLine("已停止", alert = false)
    if (failure != null) {
        Banner(failure, warn = true, actionText = if (onRetry != null) "重试" else null, onAction = onRetry)
        if (extras.onSwitchModel != null) LinkText("换个模型试试 ›", extras.onSwitchModel)
    }
}

// ───────────── 文件变更与回复操作 ─────────────

/** 本轮写入、修改过的文件。任务完成时叫“交付文件”，否则叫“文件变更”。 */
@Composable
fun FileChangesCard(files: List<ChangedFile>, delivered: Boolean, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    var open by rememberSaveable(files.size) { mutableStateOf(files.size <= 4) }
    CardSurface(modifier) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 30.dp).clickable { open = !open }.padding(horizontal = 10.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            XIcon(XIcons.File, size = 15.dp)
            Text(if (delivered) "交付文件" else "文件变更", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            Text("${files.size}", Modifier.clip(RoundedCornerShape(9.dp)).background(extra.fill).padding(horizontal = 7.dp, vertical = 1.dp), fontSize = 12.sp, color = extra.ink3)
            Spacer(Modifier.weight(1f))
            XIcon(XIcons.ChevronDown, tint = extra.ink3, size = 14.dp, modifier = Modifier.rotated(open))
        }
        if (open) {
            Column(Modifier.padding(horizontal = 10.dp)) {
                files.forEach { file ->
                    Hairline()
                    Row(Modifier.fillMaxWidth().heightIn(min = 30.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(
                            file.extension.ifEmpty { "文件" }, Modifier.widthIn(min = 30.dp).clip(RoundedCornerShape(6.dp)).background(extra.fill).padding(horizontal = 4.dp, vertical = 2.dp),
                            fontSize = 12.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center, maxLines = 1,
                        )
                        Text(file.name, Modifier.weight(1f), fontSize = 14.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(if (file.created) "已写入" else "已修改", fontSize = 12.sp, color = extra.ink3, maxLines = 1)
                    }
                }
            }
        }
    }
}

/** 回复下面一排小图标：复制、存到资料库、朗读、重新生成，右边是本轮用量。 */
@Composable
private fun ReplyActions(text: String, platform: PlatformActions?, onRegenerate: (() -> Unit)?, extras: TurnExtras) {
    val extra = LocalExtraColors.current
    var copied by remember(text) { mutableStateOf(false) }
    var speaking by remember(text) { mutableStateOf(false) }
    if (platform == null && onRegenerate == null && extras.onSave == null) return
    Row(Modifier.fillMaxWidth().padding(start = 2.dp), verticalAlignment = Alignment.CenterVertically) {
        if (platform != null) {
            Row(
                Modifier.heightIn(min = 30.dp).clip(RoundedCornerShape(8.dp)).clickable { platform.copy(text); copied = true }.padding(horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                XIcon(if (copied) XIcons.Check else XIcons.Copy, tint = extra.ink3, size = 15.dp, description = "复制")
                if (copied) Text("已复制", fontSize = 12.sp, color = extra.ink3)
            }
        }
        extras.onSave?.let { save ->
            ActionIcon(XIcons.Star, if (extras.saved) Brand else extra.ink3, if (extras.saved) "已存到资料库" else "存到资料库", save)
        }
        if (platform != null) {
            ActionIcon(XIcons.Volume, if (speaking) Brand else extra.ink3, if (speaking) "停止朗读" else "朗读") {
                if (speaking) { platform.stopSpeaking(); speaking = false } else speaking = platform.speak(speechText(text))
            }
        }
        if (onRegenerate != null) ActionIcon(XIcons.Refresh, extra.ink3, "重新生成", onRegenerate)
        Spacer(Modifier.weight(1f))
        if (extras.promptTokens + extras.completionTokens > 0) {
            Text("输入 ${formatTokens(extras.promptTokens)} · 输出 ${formatTokens(extras.completionTokens)}", fontSize = 12.sp, color = extra.ink3, maxLines = 1)
        }
    }
}

@Composable
private fun ActionIcon(icon: androidx.compose.ui.graphics.vector.ImageVector, tint: Color, description: String, onClick: () -> Unit) {
    Box(Modifier.size(30.dp).clip(RoundedCornerShape(8.dp)).clickable(onClick = onClick), contentAlignment = Alignment.Center) {
        XIcon(icon, tint = tint, size = 15.dp, description = description)
    }
}

/** 这一轮改了文件：撤销这些改动（点了先二次确认），撤销之后留一行结果。 */
@Composable
private fun CheckpointNotice(cp: RunCheckpoint, onUndo: (String) -> Unit) {
    val extra = LocalExtraColors.current
    var confirm by remember(cp.runId) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).background(extra.fill).padding(horizontal = 10.dp, vertical = 6.dp)) {
        when {
            cp.state == "undone" -> Text(cp.message.ifBlank { "已撤销这些改动" }, fontSize = 12.sp, lineHeight = 17.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            cp.state == "undoing" -> Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) { Spinner(12.dp); Text("正在撤销…", fontSize = 12.sp, color = extra.ink3) }
            confirm -> {
                Text("这些文件会回到这一轮回答之前的样子，之后对它们的修改也会被覆盖。", fontSize = 12.sp, lineHeight = 17.sp)
                Row(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    SmallButton("确认撤销", BtnKind.Primary, { confirm = false; onUndo(cp.runId) })
                    SmallButton("取消", BtnKind.Neutral, { confirm = false })
                }
            }
            else -> {
                Text("这一轮改了 ${cp.count} 个文件。", fontSize = 12.sp, lineHeight = 17.sp)
                if (cp.state == "failed" && cp.message.isNotBlank()) Text(cp.message, fontSize = 12.sp, lineHeight = 16.sp, color = Danger)
                Text("撤销这些改动", Modifier.clip(RoundedCornerShape(6.dp)).clickable { confirm = true }.padding(vertical = 4.dp), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = Brand)
            }
        }
    }
}

@Composable
private fun StatusLine(text: String, alert: Boolean) {
    Row(Modifier.padding(start = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        if (alert) XIcon(XIcons.Shield, tint = Danger, size = 14.dp) else Spinner(12.dp)
        Text(text, fontSize = 12.sp, color = if (alert) Danger else LocalExtraColors.current.ink3)
    }
}

// ───────────── Markdown 回答 ─────────────

/** 回答气泡：标题、列表、引用、代码块（带复制）、粗体、行内代码、链接。 */
@Composable
fun MarkdownBubble(text: String, streaming: Boolean, platform: PlatformActions?, modifier: Modifier = Modifier) {
    val blocks = remember(text, streaming) { parseMarkdown(if (streaming) "$text ▍" else text) }
    val extra = LocalExtraColors.current
    val codeBg = extra.fill
    val onLink: (String) -> Unit = { url -> if (url.startsWith("https://") || url.startsWith("http://")) platform?.openUrl(url) }
    Column(
        modifier.clip(AssistantBubbleShape).background(MaterialTheme.colorScheme.surface).padding(horizontal = 12.dp, vertical = 9.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        blocks.forEach { block ->
            when (block) {
                is MdBlock.Heading -> Text(inlineText(block.inlines, codeBg, onLink), fontSize = if (block.level <= 1) 17.sp else if (block.level == 2) 16.sp else 15.sp, lineHeight = 23.sp, fontWeight = FontWeight.Bold)
                is MdBlock.Paragraph -> Text(inlineText(block.inlines, codeBg, onLink), fontSize = 14.sp, lineHeight = 22.sp)
                is MdBlock.Bullets -> block.items.forEach { item -> ListItem("•", item, codeBg, onLink) }
                is MdBlock.Numbered -> block.items.forEachIndexed { i, item -> ListItem("${block.start + i}.", item, codeBg, onLink) }
                is MdBlock.Quote -> Text(inlineText(block.inlines, codeBg, onLink), Modifier.fillMaxWidth().clip(RoundedCornerShape(6.dp)).background(extra.fill).padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 14.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                is MdBlock.Code -> CodeBlock(block.language, block.text, platform)
                MdBlock.Rule -> Hairline()
            }
        }
    }
}

@Composable
private fun ListItem(marker: String, inlines: List<MdInline>, codeBg: Color, onLink: (String) -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(marker, Modifier.width(18.dp), fontSize = 14.sp, lineHeight = 22.sp, color = LocalExtraColors.current.ink3)
        Text(inlineText(inlines, codeBg, onLink), Modifier.weight(1f), fontSize = 14.sp, lineHeight = 22.sp)
    }
}

@Composable
private fun CodeBlock(language: String, code: String, platform: PlatformActions?) {
    var copied by remember(code) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).background(Color(0xFF1E1E1E))) {
        Row(Modifier.fillMaxWidth().padding(start = 10.dp, end = 4.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(language.ifBlank { "代码" }, Modifier.weight(1f), fontSize = 12.sp, color = Color(0xFFBBBBBB), maxLines = 1)
            if (platform != null) {
                Row(
                    Modifier.heightIn(min = 28.dp).clip(RoundedCornerShape(6.dp)).clickable { platform.copy(code); copied = true }.padding(horizontal = 6.dp),
                    verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    XIcon(XIcons.Copy, tint = Color(0xFFBBBBBB), size = 12.dp)
                    Text(if (copied) "已复制" else "复制", fontSize = 12.sp, color = Color(0xFFBBBBBB))
                }
            }
        }
        Text(code, Modifier.padding(start = 10.dp, end = 10.dp, bottom = 8.dp), fontSize = 12.sp, lineHeight = 18.sp, color = Color(0xFFE6E6E6), fontFamily = FontFamily.Monospace)
    }
}

private fun inlineText(inlines: List<MdInline>, codeBg: Color, onLink: (String) -> Unit): AnnotatedString = buildAnnotatedString {
    inlines.forEach { part ->
        when (part) {
            is MdInline.Text -> append(part.text)
            is MdInline.Bold -> withStyle(SpanStyle(fontWeight = FontWeight.Bold)) { append(part.text) }
            is MdInline.Italic -> withStyle(SpanStyle(fontStyle = FontStyle.Italic)) { append(part.text) }
            is MdInline.Code -> withStyle(SpanStyle(fontFamily = FontFamily.Monospace, background = codeBg, fontSize = 14.sp)) { append(part.text) }
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

/** 展开箭头：展开时转 180°。 */
private fun Modifier.rotated(open: Boolean): Modifier = this.rotate(if (open) 180f else 0f)
