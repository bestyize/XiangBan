package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.sin
import xyz.xiangban.xiaoban.common.community.Ptt
import xyz.xiangban.xiaoban.common.community.PttAction

/*
 * 按住说话（设计稿 59–65 号）：麦克风切到“按住 说话”条；按住录音，松手识别并发送，上滑取消，滑到右边“转文字”放进输入框再改。
 * 按住时页面压暗，上方气泡显示时长和声波；气泡和条都是白或主题色浅底，取消状态用白色。
 */

/** 输入卡片里的“按住 说话”条：按下开始录音，手指移动换成发送、取消或转文字，抬起就处理。 */
@Composable
fun PressBar(voice: VoiceInputController, enabled: Boolean, idleText: String = "按住 说话", onPressStart: () -> Unit = {}, modifier: Modifier = Modifier) {
    val extra = LocalExtraColors.current
    val pressing = voice.pressing
    val (label, icon, bg, fg) = when {
        !enabled -> PressStyle(idleText, XIcons.Mic, extra.fill, extra.ink3)
        !pressing -> PressStyle(idleText, XIcons.Mic, extra.fill, MaterialTheme.colorScheme.onSurface)
        voice.action == PttAction.Cancel -> PressStyle("松开 取消", XIcons.Close, MaterialTheme.colorScheme.surface, MaterialTheme.colorScheme.onSurface)
        voice.action == PttAction.Text -> PressStyle("松开 转文字", XIcons.Pen, MaterialTheme.colorScheme.surface, MaterialTheme.colorScheme.onSurface)
        else -> PressStyle("松开 发送", XIcons.Wave, extra.brandSoft, extra.brandInk)
    }
    Row(
        modifier.fillMaxWidth().height(48.dp).clip(CircleShape).background(bg)
            .pointerInput(enabled, voice) {
                if (!enabled) return@pointerInput
                val upThreshold = 60.dp.toPx()
                awaitEachGesture {
                    val down = awaitFirstDown()
                    down.consume()
                    onPressStart()
                    voice.pressStart()
                    while (true) {
                        val change = awaitPointerEvent().changes.firstOrNull { it.id == down.id } ?: break
                        if (!change.pressed) { voice.pressRelease(); return@awaitEachGesture }
                        voice.pressMove(Ptt.action(change.position.x, change.position.y, size.width.toFloat(), down.position.y, upThreshold))
                        change.consume()
                    }
                    voice.pressCancel()
                }
            },
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically,
    ) {
        XIcon(icon, tint = fg, size = 20.dp)
        Text(label, color = fg, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1)
    }
}

private data class PressStyle(val label: String, val icon: androidx.compose.ui.graphics.vector.ImageVector, val bg: Color, val fg: Color)

/** 按住时盖在整页上的一层：压暗，中间是时长和声波气泡，下面两个圆形“取消”“转文字”，当前选中的放大变白。不拦截触摸。 */
@Composable
fun PressOverlay(voice: VoiceInputController, modifier: Modifier = Modifier) {
    if (!voice.pressing) return
    val action = voice.action
    val left = Ptt.secondsLeft(voice.elapsedMs)
    Box(modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.45f))) {
        Column(Modifier.align(Alignment.Center).padding(bottom = 96.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(Ptt.clock(voice.elapsedMs), color = Color.White, fontSize = 16.sp, fontFamily = FontFamily.Monospace)
            Box(
                Modifier.width(176.dp).height(76.dp).clip(RoundedCornerShape(20.dp)).background(MaterialTheme.colorScheme.surface),
                contentAlignment = Alignment.Center,
            ) {
                when {
                    action == PttAction.Cancel -> Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        XIcon(XIcons.Close, size = 18.dp)
                        Text("取消发送", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                    }
                    left != null -> Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text("还能说 $left 秒", color = Danger, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                        Waveform(if (action == PttAction.Send) Brand else LocalExtraColors.current.ink3)
                    }
                    else -> Waveform(if (action == PttAction.Send) Brand else LocalExtraColors.current.ink3)
                }
            }
            Text(
                when (action) { PttAction.Send -> "松开发送，上滑取消"; PttAction.Cancel -> "松开手指，取消发送"; PttAction.Text -> "松开，转成文字放进输入框" },
                color = Color.White.copy(alpha = 0.8f), fontSize = 14.sp,
            )
        }
        Row(Modifier.align(Alignment.BottomCenter).fillMaxWidth().padding(start = 40.dp, end = 40.dp, bottom = 160.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Bottom) {
            PressTarget(XIcons.Close, "取消", active = action == PttAction.Cancel)
            PressTarget(XIcons.Pen, "转文字", active = action == PttAction.Text)
        }
    }
}

@Composable
private fun PressTarget(icon: androidx.compose.ui.graphics.vector.ImageVector, label: String, active: Boolean) {
    Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Box(
            Modifier.size(if (active) 76.dp else 60.dp).clip(CircleShape).background(if (active) MaterialTheme.colorScheme.surface else Color.White.copy(alpha = 0.25f)),
            contentAlignment = Alignment.Center,
        ) { XIcon(icon, tint = if (active) MaterialTheme.colorScheme.onSurface else Color.White, size = 24.dp) }
        Text(label, color = Color.White, fontSize = 12.sp)
    }
}

/** 声波：一排圆头竖条跟着相位起伏。只是录音中的示意动画，不是真实音量。 */
@Composable
private fun Waveform(color: Color) {
    val transition = rememberInfiniteTransition(label = "ptt")
    val phase by transition.animateFloat(0f, 6.2832f, infiniteRepeatable(animation = tween<Float>(1100, easing = LinearEasing), repeatMode = RepeatMode.Restart), label = "phase")
    Row(Modifier.height(32.dp), horizontalArrangement = Arrangement.spacedBy(3.dp), verticalAlignment = Alignment.CenterVertically) {
        repeat(24) { i ->
            val level = 0.35f + 0.65f * kotlin.math.abs(sin(phase + i * 0.55f) * sin(i * 0.8f + 1f))
            Box(Modifier.size(3.dp, (32f * level).dp).clip(CircleShape).background(color))
        }
    }
}
