package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import xyz.xiangban.xiaoban.common.community.CallPhase
import xyz.xiangban.xiaoban.common.community.VoiceCall
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

// 实时对话的深色通话页（设计稿 52–58）：页面的主角是文字，小伴只是中间一个 54dp 的圆形品牌头像，
// 状态靠头像外圈的变化表达；底下三个一样大的按钮：静音、打断、挂断。

private val CallBg = Color(0xFF0F141B)
private val CallFg = Color(0xFFF3F1EC)
private val CallFg2 = CallFg.copy(alpha = 0.68f)
private val CallFg3 = CallFg.copy(alpha = 0.46f)
private val CallFg4 = CallFg.copy(alpha = 0.26f)
private val CallGlass = Color.White.copy(alpha = 0.07f)
private val CallLine = Color.White.copy(alpha = 0.12f)
private val CallHang = Color(0xFFD64545)
private val CallOk = Color(0xFF3DBE86)
private val CallWarn = Color(0xFFF0A050)

private val AvatarSize = 54.dp

/** 语音通话的全屏界面：头像、状态、你说的话和小伴的回答、静音/打断/挂断。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun VoiceCallScreen(call: VoiceCall, title: String, onClosed: (String?) -> Unit) {
    val state by call.state.collectAsState()
    BackHandler(onBack = { call.stop() })
    LaunchedEffect(Unit) { call.start() }
    LaunchedEffect(state.ended) { if (state.ended) onClosed(state.notice) }
    DisposableEffect(call) { onDispose { call.stop() } }
    var seconds by remember { mutableStateOf(0) }
    LaunchedEffect(Unit) { while (true) { delay(1000); seconds += 1 } }

    val canInterrupt = state.phase == CallPhase.Speaking || state.phase == CallPhase.Thinking
    Box(Modifier.fillMaxSize().background(CallBg).drawBehind { drawBackdrop() }) {
        // 两颗淡淡的品牌闪光，压在背景里。
        XIcon(XIcons.SparkleSmall, Modifier.align(Alignment.TopEnd).padding(top = 104.dp, end = 12.dp), tint = Brand.copy(alpha = 0.16f), size = 84.dp)
        XIcon(XIcons.SparkleSmall, Modifier.align(Alignment.BottomStart).padding(bottom = 250.dp, start = 24.dp), tint = Brand.copy(alpha = 0.13f), size = 52.dp)
        Column(Modifier.fillMaxSize().statusBarsPadding().navigationBarsPadding(), horizontalAlignment = Alignment.CenterHorizontally) {
            Text(title, Modifier.padding(top = 18.dp, start = 24.dp, end = 24.dp), color = CallFg, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(clock(seconds), Modifier.padding(top = 5.dp), color = CallFg2, fontSize = 12.sp, fontFamily = FontFamily.Monospace, letterSpacing = 1.sp)
            Spacer(Modifier.weight(0.8f))
            CallAvatar(state.phase, state.level)
            Column(Modifier.fillMaxWidth().height(190.dp).padding(horizontal = 28.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                when (state.phase) {
                    CallPhase.Starting -> StatusLine("正在接通…")
                    CallPhase.Listening -> {
                        StatusLine("请说话，我在听")
                        Text("说完停一下，我就会回答", Modifier.padding(top = 10.dp), color = CallFg3, fontSize = 13.sp)
                    }
                    CallPhase.Hearing -> {
                        StatusLine("听到了…")
                        if (state.partial.isNotBlank()) Text(state.partial, Modifier.padding(top = 22.dp).fillMaxWidth(), color = CallFg, fontSize = 22.sp, lineHeight = 34.sp, textAlign = TextAlign.Center, maxLines = 3, overflow = TextOverflow.Ellipsis)
                    }
                    CallPhase.Thinking -> {
                        StatusLine("小伴在想…")
                        if (state.userText.isNotBlank()) Text("你：${state.userText}", Modifier.padding(top = 18.dp).fillMaxWidth(), color = CallFg3, fontSize = 15.sp, lineHeight = 24.sp, textAlign = TextAlign.Center, maxLines = 2, overflow = TextOverflow.Ellipsis)
                    }
                    CallPhase.Speaking -> {
                        if (state.userText.isNotBlank()) Text("你：${state.userText}", Modifier.fillMaxWidth(), color = CallFg3, fontSize = 12.sp, textAlign = TextAlign.Center, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        if (state.replyText.isNotBlank()) ReplyWindow(state.replyText, state.userText, Modifier.padding(top = 12.dp))
                    }
                }
            }
            state.notice?.let { Notice(it) }
            Spacer(Modifier.weight(1.2f))
            // 小伴说话时直接开口就能打断它；周围回声大、总被误打断时可以关掉，改用“打断”按钮。
            BargePill(state.bargeIn) { call.setBargeIn(!state.bargeIn) }
            Row(Modifier.padding(top = 20.dp, bottom = 18.dp), horizontalArrangement = Arrangement.spacedBy(30.dp), verticalAlignment = Alignment.Bottom) {
                if (state.muted) CallControl(XIcons.MicOff, "取消静音", CallFg, CallBg, null) { call.setMuted(false) }
                else CallControl(XIcons.Mic, "静音", CallGlass, CallFg, CallLine) { call.setMuted(true) }
                if (canInterrupt) CallControl(XIcons.SkipNext, "打断", CallFg, CallBg, null) { call.interrupt() }
                else CallControl(XIcons.Wave, "打断", Color.Transparent, CallFg3, CallFg3, borderWidth = 1.5f) {}
                CallControl(XIcons.Hangup, "挂断", CallHang, Color.White, null) { call.stop() }
            }
        }
    }
}

private fun clock(seconds: Int): String = "${(seconds / 60).toString().padStart(2, '0')}:${(seconds % 60).toString().padStart(2, '0')}"

@Composable
private fun StatusLine(text: String) {
    Text(text, color = CallFg, fontSize = 24.sp, fontWeight = FontWeight.Light, letterSpacing = 3.sp, textAlign = TextAlign.Center, maxLines = 1, overflow = TextOverflow.Ellipsis)
}

/** 小伴的回答：最多露出三行，可以上下滑；中间一行正常显示，上下两行渐渐淡出。 */
@Composable
private fun ReplyWindow(text: String, resetKey: String, modifier: Modifier = Modifier) {
    val scroll = rememberScrollState()
    LaunchedEffect(resetKey) { scroll.scrollTo(0) }
    Box(modifier.fillMaxWidth().height(96.dp)) {
        Text(text, Modifier.fillMaxWidth().verticalScroll(scroll), color = CallFg.copy(alpha = 0.86f), fontSize = 17.sp, lineHeight = 32.sp, textAlign = TextAlign.Center)
        Box(Modifier.align(Alignment.TopCenter).fillMaxWidth().height(34.dp).background(Brush.verticalGradient(listOf(CallBg.copy(alpha = 0.9f), Color.Transparent))))
        Box(Modifier.align(Alignment.BottomCenter).fillMaxWidth().height(34.dp).background(Brush.verticalGradient(listOf(Color.Transparent, CallBg.copy(alpha = 0.9f)))))
    }
}

@Composable
private fun Notice(text: String) {
    Text(
        text,
        Modifier.padding(horizontal = 24.dp).fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(CallGlass).border(0.5.dp, CallLine, RoundedCornerShape(16.dp)).padding(horizontal = 14.dp, vertical = 12.dp),
        color = CallWarn, fontSize = 13.sp, lineHeight = 20.sp,
    )
}

@Composable
private fun BargePill(on: Boolean, onToggle: () -> Unit) {
    Row(
        Modifier.clip(RoundedCornerShape(15.dp)).background(CallGlass).border(0.5.dp, CallLine, RoundedCornerShape(15.dp)).clickable(onClick = onToggle).padding(horizontal = 14.dp, vertical = 7.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(7.dp).clip(CircleShape).background(if (on) CallOk else CallFg4))
        Text(if (on) "直接开口打断：开" else "直接开口打断：关", Modifier.padding(start = 8.dp), color = CallFg2, fontSize = 12.sp)
    }
}

/** 三个一样大（68dp）的圆按钮。[border] 为空就不画边框。 */
@Composable
private fun CallControl(icon: ImageVector, label: String, background: Color, tint: Color, border: Color?, borderWidth: Float = 0.5f, onClick: () -> Unit) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        val ring = if (border != null) Modifier.border(borderWidth.dp, border, CircleShape) else Modifier
        Box(Modifier.size(68.dp).clip(CircleShape).background(background).then(ring).clickable(onClick = onClick), contentAlignment = Alignment.Center) {
            XIcon(icon, tint = tint, size = 28.dp, description = label)
        }
        Text(label, Modifier.padding(top = 8.dp), color = CallFg2, fontSize = 12.sp)
    }
}

// ───────────── 头像和它的外圈 ─────────────

/**
 * 54dp 的圆形品牌头像。外圈表达状态：聆听时淡淡地呼吸；听到你说话时随音量一圈圈扩散；
 * 思考时一圈虚线绕着头像转；小伴说话时头像轻轻跳动、外圈柔和起伏。
 */
@Composable
private fun CallAvatar(phase: CallPhase, level: Float) {
    val transition = rememberInfiniteTransition(label = "call")
    val breathe by transition.animateFloat(1f, 1.045f, infiniteRepeatable(animation = tween<Float>(1700, easing = FastOutSlowInEasing), repeatMode = RepeatMode.Reverse), label = "breathe")
    val beat by transition.animateFloat(1f, 1.09f, infiniteRepeatable(animation = tween<Float>(450, easing = FastOutSlowInEasing), repeatMode = RepeatMode.Reverse), label = "beat")
    val spin by transition.animateFloat(0f, 360f, infiniteRepeatable(animation = tween<Float>(4200, easing = LinearEasing)), label = "spin")
    val ripple by transition.animateFloat(0f, 1f, infiniteRepeatable(animation = tween<Float>(2800, easing = LinearEasing)), label = "ripple")
    val scale = when (phase) {
        CallPhase.Hearing -> 1f + level.coerceIn(0f, 1f) * 0.14f
        CallPhase.Speaking -> beat
        CallPhase.Thinking -> 1f
        else -> breathe
    }
    Box(Modifier.size(160.dp).drawBehind { drawAvatarRings(phase, spin, ripple) }, contentAlignment = Alignment.Center) {
        Box(Modifier.size(AvatarSize).graphicsLayer { scaleX = scale; scaleY = scale }.clip(CircleShape).background(Brand), contentAlignment = Alignment.Center) {
            XIcon(XIcons.Sparkles, tint = Color.White, size = 30.dp)
        }
    }
}

private fun DrawScope.ring(radiusOffset: Float, alpha: Float, width: Float = 1.3f) {
    drawCircle(Brand.copy(alpha = alpha), radius = AvatarSize.toPx() / 2 + radiusOffset.dp.toPx(), style = Stroke(width.dp.toPx()))
}

/** 一圈向外扩散并淡掉的波纹。[progress] 从 0 走到 1。 */
private fun DrawScope.expandingRing(radiusOffset: Float, progress: Float) {
    val base = AvatarSize.toPx() / 2 + radiusOffset.dp.toPx()
    drawCircle(Brand.copy(alpha = 0.45f * (1f - progress)), radius = base * (1f + 0.5f * progress), style = Stroke(1.4.dp.toPx()))
}

private fun DrawScope.drawAvatarRings(phase: CallPhase, spin: Float, ripple: Float) {
    when (phase) {
        CallPhase.Hearing -> {
            ring(8f, 0.40f, 1.6f); ring(18f, 0.22f); ring(28f, 0.10f)
            expandingRing(8f, ripple)
        }
        CallPhase.Speaking -> {
            expandingRing(6f, ripple)
            ring(12f, 0.28f); ring(24f, 0.14f); ring(36f, 0.06f)
        }
        CallPhase.Thinking -> {
            val radius = AvatarSize.toPx() / 2 + 13.dp.toPx()
            rotate(spin) {
                drawCircle(CallFg3, radius = radius, style = Stroke(1.5.dp.toPx(), cap = StrokeCap.Round, pathEffect = PathEffect.dashPathEffect(floatArrayOf(2.dp.toPx(), 6.dp.toPx()), 0f)))
                drawCircle(CallFg, radius = 3.6.dp.toPx(), center = Offset(center.x, center.y - radius))
                val a = 150.0 / 180.0 * PI
                drawCircle(CallFg2, radius = 2.4.dp.toPx(), center = Offset(center.x + radius * cos(a).toFloat(), center.y + radius * sin(a).toFloat()))
            }
        }
        else -> { ring(9f, 0.20f); ring(19f, 0.10f) }
    }
}

// ───────────── 背景 ─────────────

/** 背景：头像后面一团很淡的品牌红光，几圈大同心圆，和零星的淡形状（圆角方块、小圆点、十字）。 */
private fun DrawScope.drawBackdrop() {
    fun at(fx: Float, fy: Float) = Offset(size.width * fx, size.height * fy)
    val c = at(0.5f, 0.31f)
    val glow = size.width * 0.8f
    drawCircle(Brush.radialGradient(listOf(Brand.copy(alpha = 0.20f), Color.Transparent), center = c, radius = glow), radius = glow, center = c)
    for ((r, a) in listOf(120 to 0.10f, 190 to 0.075f, 270 to 0.055f, 360 to 0.04f)) {
        drawCircle(Color.White.copy(alpha = a), radius = r.dp.toPx(), center = c, style = Stroke(1.dp.toPx()))
    }
    fun roundSquare(center: Offset, w: Float, h: Float, degrees: Float, color: Color, corner: Float, stroke: Boolean) {
        rotate(degrees, center) {
            drawRoundRect(
                color, topLeft = Offset(center.x - w.dp.toPx() / 2, center.y - h.dp.toPx() / 2), size = Size(w.dp.toPx(), h.dp.toPx()),
                cornerRadius = CornerRadius(corner.dp.toPx()), style = if (stroke) Stroke(1.2.dp.toPx()) else androidx.compose.ui.graphics.drawscope.Fill,
            )
        }
    }
    roundSquare(at(0.18f, 0.24f), 64f, 64f, 18f, Color.White.copy(alpha = 0.12f), 18f, stroke = true)
    roundSquare(at(0.86f, 0.67f), 92f, 92f, -16f, Brand.copy(alpha = 0.12f), 26f, stroke = false)
    roundSquare(at(0.79f, 0.27f), 78f, 24f, -32f, Color.White.copy(alpha = 0.06f), 12f, stroke = false)
    roundSquare(at(0.06f, 0.55f), 86f, 28f, 24f, Brand.copy(alpha = 0.24f), 14f, stroke = true)
    drawCircle(Brand.copy(alpha = 0.14f), radius = 15.dp.toPx(), center = at(0.9f, 0.42f))
    drawCircle(Color.White.copy(alpha = 0.10f), radius = 8.dp.toPx(), center = at(0.11f, 0.40f))
    drawCircle(Brand.copy(alpha = 0.20f), radius = 6.dp.toPx(), center = at(0.84f, 0.85f))
    drawCircle(Brand.copy(alpha = 0.30f), radius = 7.dp.toPx(), center = at(0.6f, 0.11f), style = Stroke(1.4.dp.toPx()))
    drawCircle(Color.White.copy(alpha = 0.13f), radius = 12.dp.toPx(), center = at(0.16f, 0.85f), style = Stroke(1.2.dp.toPx()))
    for (p in listOf(at(0.25f, 0.15f), at(0.08f, 0.6f), at(0.44f, 0.76f))) {
        val half = 7.dp.toPx()
        drawLine(Brand.copy(alpha = 0.34f), Offset(p.x, p.y - half), Offset(p.x, p.y + half), strokeWidth = 1.6.dp.toPx(), cap = StrokeCap.Round)
        drawLine(Brand.copy(alpha = 0.34f), Offset(p.x - half, p.y), Offset(p.x + half, p.y), strokeWidth = 1.6.dp.toPx(), cap = StrokeCap.Round)
    }
}
