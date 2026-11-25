package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.CallPhase
import xyz.xiangban.xiaoban.common.community.CallPrompt
import xyz.xiangban.xiaoban.common.community.CallPromptOption
import xyz.xiangban.xiaoban.common.community.CallState
import xyz.xiangban.xiaoban.common.community.VoiceCall
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.roundToInt
import kotlin.math.sin

// 实时语音通话深色沉浸页（设计稿 prototypes/voice-call-redesign.html）：
// 以「双星相伴 · 日落珊瑚」88dp 立体光球为核心，光晕严格绑定球心、随状态有机呼吸；
// 思考态双星反向缓转 + 流光弧呼吸，聆听态 4.2s 潮汐慢呼吸，回答态球体上收至 64dp 释放渐隐歌词字幕区。

private val CallBgTop = Color(0xFF0C0F15)
private val CallBgMid = Color(0xFF0B0E14)
private val CallBgBottom = Color(0xFF080A0E)
private val CallCapsuleBg = Color(0xF0181C24)
private val CallFg = Color(0xFFF4F2ED)
private val CallFg2 = CallFg.copy(alpha = 0.70f)
private val CallFg3 = CallFg.copy(alpha = 0.44f)
private val CallGlass = Color.White.copy(alpha = 0.08f)
private val CallGlassSubtle = Color.White.copy(alpha = 0.055f)
private val CallLine = Color.White.copy(alpha = 0.13f)
private val CallLineSubtle = Color.White.copy(alpha = 0.09f)
private val CallHang = Color(0xFFDC4248)
private val CallWarn = Color(0xFFF0A854)
private val CallLiveGreen = Color(0xFF42D68A)

private data class CallOrbPalette(
    val start: Color,
    val end: Color,
    val glow: Color,
    val aura: Color,
)

private fun currentOrbPalette(): CallOrbPalette = when (ThemeState.accent.id) {
    "green" -> CallOrbPalette(Color(0xFF3CC586), Color(0xFF219A66), Color(0xFF34B97C), Color(0xFFB4FFDE))
    "blue" -> CallOrbPalette(Color(0xFF4F91FF), Color(0xFF2D68DB), Color(0xFF4486F5), Color(0xFFBCDEFF))
    "purple" -> CallOrbPalette(Color(0xFF9A7CF5), Color(0xFF6C4CD6), Color(0xFF8E6CEC), Color(0xFFDECEFF))
    "amber" -> CallOrbPalette(Color(0xFFF0A63A), Color(0xFFC96D18), Color(0xFFE8922C), Color(0xFFFFEAB4))
    "graphite" -> CallOrbPalette(Color(0xFF464A54), Color(0xFF22242A), Color(0xFF848A9A), Color(0xFFFFFFFF))
    else -> CallOrbPalette(Color(0xFFFF6584), Color(0xFFF9744B), Color(0xFFFF6B72), Color(0xFFFFE0A3))
}

/** 语音通话的全屏界面：双星光球、状态呼吸、你说的话与小伴的歌词式回答、静音/朗读/挂断。左上角和返回键收起成胶囊。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun VoiceCallScreen(call: VoiceCall, title: String, seconds: Int, onMinimize: () -> Unit) {
    val state by call.state.collectAsState()
    val palette = currentOrbPalette()
    BackHandler(onBack = onMinimize)

    val showingReply = state.prompt == null && (
        state.phase == CallPhase.Speaking ||
            (state.phase == CallPhase.Listening && state.silent && state.replyText.isNotBlank())
        )
    val compactOrb = state.prompt != null || showingReply

    val transition = rememberInfiniteTransition(label = "voiceCall")
    val thinkPhase by transition.animateFloat(
        0f, 1f,
        infiniteRepeatable(tween(3600, easing = LinearEasing), RepeatMode.Restart),
        label = "thinkPhase",
    )
    val listenPhase by transition.animateFloat(
        0f, 1f,
        infiniteRepeatable(tween(4200, easing = LinearEasing), RepeatMode.Restart),
        label = "listenPhase",
    )
    val sweepSpin by transition.animateFloat(
        0f, 360f,
        infiniteRepeatable(tween(2800, easing = LinearEasing), RepeatMode.Restart),
        label = "sweepSpin",
    )
    val fastRipple by transition.animateFloat(
        0f, 1f,
        infiniteRepeatable(tween(1650, easing = LinearEasing), RepeatMode.Restart),
        label = "fastRipple",
    )
    val eqWave by transition.animateFloat(
        0f, 1f,
        infiniteRepeatable(tween(900, easing = FastOutSlowInEasing), RepeatMode.Reverse),
        label = "eqWave",
    )
    val livePulse by transition.animateFloat(
        0.45f, 1f,
        infiniteRepeatable(tween(1200, easing = FastOutSlowInEasing), RepeatMode.Reverse),
        label = "livePulse",
    )

    Box(
        Modifier
            .fillMaxSize()
            .background(
                Brush.verticalGradient(
                    0f to CallBgTop,
                    0.64f to CallBgMid,
                    1f to CallBgBottom,
                ),
            ),
    ) {
        Column(
            Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .navigationBarsPadding(),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            CallTopBar(
                title = title,
                seconds = seconds,
                muted = state.muted,
                dotAlpha = if (state.muted) 1f else livePulse,
                onMinimize = onMinimize,
            )

            // 主舞台：普通态上下留白居中；回答态 / 确认态球体轻盈上收，为歌词字幕或确认卡让出黄金中部。
            Column(
                Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .padding(horizontal = 26.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = if (compactOrb) Arrangement.Top else Arrangement.Center,
            ) {
                if (compactOrb) Spacer(Modifier.height(10.dp))

                CallOrb(
                    phase = state.phase,
                    level = state.level,
                    compact = compactOrb,
                    showEq = state.prompt == null && state.phase == CallPhase.Speaking && !state.silent,
                    palette = palette,
                    thinkPhase = thinkPhase,
                    listenPhase = listenPhase,
                    sweepSpin = sweepSpin,
                    fastRipple = fastRipple,
                    eqWave = eqWave,
                )

                val textTopGap = if (compactOrb) 8.dp else 12.dp
                Column(
                    Modifier
                        .fillMaxWidth()
                        .padding(top = textTopGap)
                        .let { if (compactOrb) it.weight(1f, fill = false) else it.heightIn(min = 136.dp) },
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    val prompt = state.prompt
                    if (prompt != null) {
                        StatusLine(if (prompt.question) "等你选择" else "等你确认", fontSize = 19.sp)
                        CallPromptCard(prompt, call::answerPrompt)
                    } else when (state.phase) {
                        CallPhase.Starting -> {
                            StatusLine("正在接通…")
                            Text("请稍候，马上开始对话", Modifier.padding(top = 10.dp), color = CallFg2, style = XType.Secondary)
                        }
                        CallPhase.Listening -> if (state.silent && state.replyText.isNotBlank()) {
                            ReplyBlock(state.userText, state.replyText, null)
                        } else {
                            StatusLine(if (state.muted) "已静音" else "请说话，我在听")
                            Text(
                                if (state.muted) "点下方取消静音后就能继续说话" else "说完停一下，我就会回答",
                                Modifier.padding(top = 10.dp),
                                color = CallFg2,
                                style = XType.Secondary,
                                textAlign = TextAlign.Center,
                            )
                        }
                        CallPhase.Hearing -> {
                            Text("听到了…", color = CallFg2, fontSize = 15.sp, fontWeight = FontWeight.Medium)
                            if (state.partial.isNotBlank()) {
                                Text(
                                    state.partial,
                                    Modifier.padding(top = 16.dp).fillMaxWidth(),
                                    color = CallFg,
                                    fontSize = 20.sp,
                                    fontWeight = FontWeight.Medium,
                                    lineHeight = 31.sp,
                                    textAlign = TextAlign.Center,
                                    maxLines = 4,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                        }
                        CallPhase.Thinking -> {
                            val breathAlpha = 0.76f + 0.24f * smoothBreath(thinkPhase)
                            StatusLine("小伴在想…", alpha = breathAlpha)
                            if (state.userText.isNotBlank()) {
                                UserQuotePill(state.userText, Modifier.padding(top = 16.dp))
                            }
                        }
                        CallPhase.Speaking -> ReplyBlock(state.userText, state.replyText, state.reading)
                    }
                }
            }

            state.notice?.let {
                Notice(it, Modifier.padding(bottom = 10.dp))
            }

            CallFooter(
                state = state,
                palette = palette,
                onInterrupt = { call.interrupt() },
                onToggleMute = { call.setMuted(!state.muted) },
                onToggleSilent = {
                    val next = !state.silent
                    call.setSilent(next)
                    SpeechState.update { it.copy(callSilent = next) }
                },
                onHangup = { call.stop() },
            )
        }
    }
}

internal fun clock(seconds: Int): String = "${(seconds / 60).toString().padStart(2, '0')}:${(seconds % 60).toString().padStart(2, '0')}"

@Composable
private fun CallTopBar(title: String, seconds: Int, muted: Boolean, dotAlpha: Float, onMinimize: () -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .height(56.dp)
            .padding(horizontal = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier
                .size(40.dp)
                .clip(CircleShape)
                .background(CallGlass)
                .border(0.5.dp, CallLineSubtle, CircleShape)
                .clickable(onClick = onMinimize),
            contentAlignment = Alignment.Center,
        ) {
            XIcon(XIcons.ChevronDown, tint = CallFg.copy(alpha = 0.84f), size = 20.dp, description = "收起")
        }
        Column(
            Modifier.weight(1f).padding(horizontal = 12.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(3.dp),
        ) {
            Text(
                title,
                color = CallFg,
                style = XType.Section,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                Box(
                    Modifier
                        .size(6.dp)
                        .graphicsLayer { alpha = dotAlpha }
                        .clip(CircleShape)
                        .background(if (muted) CallWarn else CallLiveGreen),
                )
                Text(
                    clock(seconds),
                    color = CallFg2,
                    fontSize = 11.5.sp,
                    lineHeight = 14.sp,
                    fontFamily = FontFamily.Monospace,
                    letterSpacing = 0.8.sp,
                )
            }
        }
        Spacer(Modifier.size(40.dp))
    }
}

@Composable
private fun StatusLine(text: String, fontSize: androidx.compose.ui.unit.TextUnit = 22.sp, alpha: Float = 1f) {
    Text(
        text,
        color = CallFg.copy(alpha = alpha),
        fontSize = fontSize,
        fontWeight = FontWeight.Medium,
        letterSpacing = 0.6.sp,
        textAlign = TextAlign.Center,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun UserQuotePill(userText: String, modifier: Modifier = Modifier) {
    Text(
        buildAnnotatedString {
            withStyle(SpanStyle(color = CallFg.copy(alpha = 0.48f))) { append("你  ") }
            withStyle(SpanStyle(color = CallFg.copy(alpha = 0.78f))) { append(userText) }
        },
        modifier
            .clip(RoundedCornerShape(16.dp))
            .background(CallGlassSubtle)
            .border(0.5.dp, CallLineSubtle, RoundedCornerShape(16.dp))
            .padding(horizontal = 15.dp, vertical = 8.dp),
        fontSize = 14.5.sp,
        lineHeight = 22.sp,
        textAlign = TextAlign.Center,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
    )
}

/** 你说的那句（轻量单行胶囊）和小伴的渐隐歌词式回答。 */
@Composable
private fun ReplyBlock(userText: String, replyText: String, reading: IntRange?) {
    if (userText.isNotBlank()) {
        Text(
            "你：$userText",
            Modifier
                .clip(CircleShape)
                .background(Color.White.copy(alpha = 0.045f))
                .padding(horizontal = 12.dp, vertical = 4.dp),
            color = CallFg.copy(alpha = 0.54f),
            style = XType.Caption,
            textAlign = TextAlign.Center,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
    if (replyText.isNotBlank()) {
        ReplyWindow(replyText, reading, userText, Modifier.padding(top = 12.dp))
    }
}

/**
 * 小伴的回答，像音乐播放器的歌词：224dp 窗口配合上下渐隐遮罩，可上下滑；
 * 念到哪句就把那句点亮并滚到窗口中间，其余文字柔和降暗；仅看文字模式全量高亮显示。
 */
@Composable
private fun ReplyWindow(text: String, reading: IntRange?, resetKey: String, modifier: Modifier = Modifier) {
    val scroll = rememberScrollState()
    var layout by remember { mutableStateOf<TextLayoutResult?>(null) }
    LaunchedEffect(resetKey) { scroll.scrollTo(0) }
    val lit = reading?.let { it.first.coerceIn(0, text.length) until (it.last + 1).coerceIn(0, text.length) }?.takeUnless { it.isEmpty() }
    val shown = remember(text, lit) {
        if (lit == null) AnnotatedString(text)
        else AnnotatedString(
            text,
            spanStyles = listOf(
                AnnotatedString.Range(
                    SpanStyle(color = CallFg, fontWeight = FontWeight.Medium),
                    lit.first,
                    lit.last + 1,
                ),
            ),
        )
    }
    val density = LocalDensity.current
    val padPx = with(density) { ReplyPadHeight.toPx() }
    val viewPx = with(density) { ReplyViewportHeight.toPx() }
    val target = layout?.takeIf { lit != null && it.layoutInput.text.length == text.length }?.let { l ->
        val top = padPx + l.getLineTop(l.getLineForOffset(lit!!.first))
        val bottom = padPx + l.getLineBottom(l.getLineForOffset(lit.last))
        (if (bottom - top > viewPx) top else (top + bottom - viewPx) / 2).roundToInt().coerceAtLeast(0)
    }
    LaunchedEffect(target) { if (target != null) scroll.animateScrollTo(target, tween(420, easing = FastOutSlowInEasing)) }

    Box(
        modifier
            .fillMaxWidth()
            .height(ReplyViewportHeight)
            .graphicsLayer(compositingStrategy = CompositingStrategy.Offscreen)
            .drawWithContent {
                drawContent()
                drawRect(
                    brush = Brush.verticalGradient(
                        0f to Color.Transparent,
                        0.18f to Color.Black,
                        0.80f to Color.Black,
                        1f to Color.Transparent,
                    ),
                    blendMode = BlendMode.DstIn,
                )
            },
    ) {
        Text(
            shown,
            Modifier
                .fillMaxWidth()
                .verticalScroll(scroll)
                .padding(vertical = ReplyPadHeight),
            color = if (lit == null) CallFg.copy(alpha = 0.88f) else CallFg.copy(alpha = 0.36f),
            fontSize = 17.5.sp,
            lineHeight = 28.sp,
            textAlign = TextAlign.Center,
            onTextLayout = { layout = it },
        )
    }
}

private val ReplyViewportHeight = 224.dp
private val ReplyPadHeight = 52.dp

@Composable
private fun Notice(text: String, modifier: Modifier = Modifier) {
    Text(
        text,
        modifier
            .padding(horizontal = 24.dp)
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .background(CallGlass)
            .border(0.5.dp, CallLine, RoundedCornerShape(16.dp))
            .padding(horizontal = 14.dp, vertical = 11.dp),
        color = CallWarn,
        style = XType.Secondary,
        textAlign = TextAlign.Center,
    )
}

@Composable
private fun CallFooter(
    state: CallState,
    palette: CallOrbPalette,
    onInterrupt: () -> Unit,
    onToggleMute: () -> Unit,
    onToggleSilent: () -> Unit,
    onHangup: () -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .padding(start = 24.dp, end = 24.dp, top = 6.dp, bottom = 22.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(18.dp),
    ) {
        if (state.prompt == null) {
            val canInterrupt = state.phase == CallPhase.Thinking || state.phase == CallPhase.Speaking
            val (dotColor, hintText) = when {
                canInterrupt -> palette.start to "直接开口或轻点此处打断"
                state.phase == CallPhase.Hearing -> palette.start to "正在拾音 · 说完停顿自动发送"
                state.phase == CallPhase.Listening && state.muted -> CallWarn to "麦克风已静音 · 点击下方取消静音"
                state.phase == CallPhase.Listening && state.silent && state.replyText.isNotBlank() ->
                    CallLiveGreen to "仅看文字 · 可随时继续开口提问"
                state.phase == CallPhase.Listening -> CallLiveGreen to "麦克风收音正常 · 自动判停"
                else -> CallFg3 to "正在准备语音通道…"
            }
            Row(
                Modifier
                    .height(30.dp)
                    .clip(CircleShape)
                    .background(CallGlassSubtle)
                    .border(0.5.dp, CallLineSubtle, CircleShape)
                    .let { if (canInterrupt) it.clickable(onClick = onInterrupt) else it }
                    .padding(horizontal = 13.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(7.dp),
            ) {
                Box(Modifier.size(6.dp).clip(CircleShape).background(dotColor))
                Text(hintText, color = CallFg.copy(alpha = 0.66f), style = XType.Caption)
            }
        }

        // 三个 62dp 圆形主控键：静音（麦克风）、朗读中 / 仅看文字（出声开关）、挂断。
        Row(horizontalArrangement = Arrangement.spacedBy(30.dp), verticalAlignment = Alignment.Top) {
            if (state.muted) {
                CallControl(XIcons.MicOff, "已静音", CallFg, CallBgTop, null, onToggleMute)
            } else {
                CallControl(XIcons.Mic, "静音", CallGlass, CallFg, CallLine, onToggleMute)
            }
            if (state.silent) {
                CallControl(VolumeOffIcon, "仅看文字", CallFg, CallBgTop, null, onToggleSilent)
            } else {
                CallControl(XIcons.Volume, "朗读中", CallGlass, CallFg, CallLine, onToggleSilent)
            }
            CallControl(XIcons.Hangup, "挂断", CallHang, Color.White, Color.White.copy(alpha = 0.14f), onHangup)
        }
    }
}

/** 关了朗读的喇叭：喇叭加一个叉（图标集里没有这一个，和 [XIcons.Volume] 同样的画法）。 */
private val VolumeOffIcon: ImageVector by lazy {
    ImageVector.Builder(name = "VolumeOff", defaultWidth = 24.dp, defaultHeight = 24.dp, viewportWidth = 24f, viewportHeight = 24f)
        .addPath(
            pathData = addPathNodes("M4 10v4h4l5 4V6L8 10zM16.5 9.5l5 5M21.5 9.5l-5 5"),
            stroke = SolidColor(Color.Black), strokeLineWidth = 1.8f, strokeLineCap = StrokeCap.Round, strokeLineJoin = StrokeJoin.Round,
        )
        .build()
}

/** 62dp 圆形控制按钮，主次上收敛在 88dp 核心光球之下。 */
@Composable
private fun CallControl(
    icon: ImageVector,
    label: String,
    background: Color,
    tint: Color,
    border: Color?,
    onClick: () -> Unit,
) {
    Column(Modifier.width(68.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        val ring = if (border != null) Modifier.border(0.5.dp, border, CircleShape) else Modifier
        Box(
            Modifier
                .size(62.dp)
                .clip(CircleShape)
                .background(background)
                .then(ring)
                .clickable(onClick = onClick),
            contentAlignment = Alignment.Center,
        ) {
            XIcon(icon, tint = tint, size = 25.dp, description = label)
        }
        Text(label, Modifier.padding(top = 9.dp), color = CallFg2, style = XType.Caption, maxLines = 1)
    }
}

// ───────────── 双星相伴光球与有机呼吸动效 ─────────────

/**
 * 88dp（回答态轻盈上收至 64dp）的「双星相伴 · 日落珊瑚」立体光球。
 * 氛围光晕与状态光环全部绘制在球体自身容器中心，永远严格同心：
 * - 思考态：3.6s 胸腔式深呼吸脉动 + 大小双星无缝 90° 反向缓转 + 渐隐流光弧环；
 * - 聆听态：4.2s 潮汐慢呼吸 + 两道错拍扩散涟漪 + 大星轻摇与陪伴小星闪烁微旋；
 * - 拾音态：随音量实时脉动与快速声波涟漪；
 * - 回答态：球体上收至 64dp 并显示动态五柱声纹。
 */
@Composable
private fun CallOrb(
    phase: CallPhase,
    level: Float,
    compact: Boolean,
    showEq: Boolean,
    palette: CallOrbPalette,
    thinkPhase: Float,
    listenPhase: Float,
    sweepSpin: Float,
    fastRipple: Float,
    eqWave: Float,
) {
    val anchorSize: Dp by animateDpAsState(if (compact) 132.dp else 214.dp, tween(320, easing = FastOutSlowInEasing), label = "orbAnchor")
    val coreSize: Dp by animateDpAsState(if (compact) 64.dp else 88.dp, tween(320, easing = FastOutSlowInEasing), label = "orbCore")

    val orbScale = when (phase) {
        CallPhase.Thinking -> 0.94f + 0.145f * thinkBreathCurve(thinkPhase)
        CallPhase.Listening, CallPhase.Starting -> 0.95f + 0.125f * smoothBreath(listenPhase)
        CallPhase.Hearing -> 0.99f + level.coerceIn(0f, 1f) * 0.14f + 0.05f * smoothBreath(fastRipple)
        CallPhase.Speaking -> 0.99f + 0.065f * eqWave
    }

    Box(
        Modifier
            .size(anchorSize)
            .drawBehind {
                drawOrbAtmosphere(
                    phase = phase,
                    compact = compact,
                    palette = palette,
                    thinkPhase = thinkPhase,
                    listenPhase = listenPhase,
                    sweepSpin = sweepSpin,
                    fastRipple = fastRipple,
                    eqWave = eqWave,
                )
            },
        contentAlignment = Alignment.Center,
    ) {
        Box(
            Modifier
                .size(coreSize)
                .graphicsLayer { scaleX = orbScale; scaleY = orbScale }
                .clip(CircleShape)
                .background(
                    Brush.linearGradient(
                        colors = listOf(palette.start, palette.end),
                        start = Offset(0f, 0f),
                        end = Offset(Float.POSITIVE_INFINITY, Float.POSITIVE_INFINITY),
                    ),
                )
                .drawBehind {
                    // 官方 AppIcon 暖桃金内辉 + 顶部釉面高光
                    drawCircle(
                        Brush.radialGradient(
                            colors = listOf(palette.aura.copy(alpha = 0.30f), Color.Transparent),
                            center = Offset(size.width * 0.50f, size.height * 0.44f),
                            radius = size.minDimension * 0.56f,
                        ),
                    )
                    drawCircle(
                        Brush.verticalGradient(
                            0f to Color.White.copy(alpha = 0.22f),
                            0.46f to Color.Transparent,
                        ),
                    )
                }
                .border(1.dp, Color.White.copy(alpha = 0.22f), CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            if (showEq) {
                SpeakingEqBars(eqWave)
            } else {
                TwinStarsMark(
                    phase = phase,
                    size = if (compact) 42.dp else 56.dp,
                    thinkPhase = thinkPhase,
                    listenPhase = listenPhase,
                    fastRipple = fastRipple,
                )
            }
        }
    }
}

private fun DrawScope.drawOrbAtmosphere(
    phase: CallPhase,
    compact: Boolean,
    palette: CallOrbPalette,
    thinkPhase: Float,
    listenPhase: Float,
    sweepSpin: Float,
    fastRipple: Float,
    eqWave: Float,
) {
    val breath = when (phase) {
        CallPhase.Thinking -> thinkBreathCurve(thinkPhase)
        CallPhase.Listening, CallPhase.Starting -> smoothBreath(listenPhase)
        CallPhase.Hearing -> smoothBreath(fastRipple)
        CallPhase.Speaking -> eqWave
    }

    // 1. 双层严格同心氛围光（外层漫射 + 内层暖辉）
    val outerBase = if (compact) 180.dp.toPx() else 230.dp.toPx()
    val outerRadius = outerBase * (0.91f + 0.25f * breath)
    val outerAlpha = 0.21f + 0.12f * breath
    drawCircle(
        Brush.radialGradient(
            0f to palette.glow.copy(alpha = outerAlpha),
            0.30f to palette.glow.copy(alpha = outerAlpha * 0.44f),
            0.56f to palette.glow.copy(alpha = outerAlpha * 0.12f),
            0.76f to Color.Transparent,
            center = center,
            radius = outerRadius,
        ),
        radius = outerRadius,
        center = center,
    )
    val innerRadius = (if (compact) 68.dp.toPx() else 94.dp.toPx()) * (0.88f + 0.34f * breath)
    drawCircle(
        Brush.radialGradient(
            0f to palette.aura.copy(alpha = 0.20f + 0.14f * breath),
            0.46f to palette.glow.copy(alpha = 0.14f + 0.10f * breath),
            0.80f to Color.Transparent,
            center = center,
            radius = innerRadius,
        ),
        radius = innerRadius,
        center = center,
    )

    // 2. 分阶段呼吸环与波纹
    when (phase) {
        CallPhase.Thinking -> {
            val b1 = thinkBreathCurve(thinkPhase)
            val b2 = thinkBreathCurve(wrapPhase(thinkPhase - 0.125f))
            val b3 = thinkBreathCurve(wrapPhase(thinkPhase - 0.25f))
            drawCircle(palette.glow.copy(alpha = 0.16f + 0.28f * b1), radius = 59.dp.toPx() * (0.93f + 0.20f * b1), style = Stroke(1.2.dp.toPx()))
            drawCircle(Color.White.copy(alpha = 0.04f + 0.07f * b2), radius = 78.dp.toPx() * (0.94f + 0.17f * b2), style = Stroke(1.dp.toPx()))
            if (!compact) {
                drawCircle(palette.glow.copy(alpha = 0.03f + 0.10f * b3), radius = 98.dp.toPx() * (0.95f + 0.13f * b3), style = Stroke(1.dp.toPx()))
            }

            // 渐隐呼吸流光弧环
            val sweepRadius = 61.dp.toPx() * (0.94f + 0.15f * b1)
            drawCircle(Color.White.copy(alpha = 0.09f), radius = sweepRadius, style = Stroke(1.dp.toPx()))
            rotate(sweepSpin) {
                drawCircle(
                    brush = Brush.sweepGradient(
                        0.00f to Color.Transparent,
                        0.52f to Color.Transparent,
                        0.76f to palette.glow.copy(alpha = 0.45f),
                        0.96f to palette.aura.copy(alpha = 0.92f),
                        1.00f to Color.Transparent,
                        center = center,
                    ),
                    radius = sweepRadius,
                    style = Stroke(2.3.dp.toPx(), cap = StrokeCap.Round),
                )
            }
        }
        CallPhase.Listening, CallPhase.Starting -> {
            val b1 = smoothBreath(listenPhase)
            val b2 = smoothBreath(wrapPhase(listenPhase - 0.14f))
            val b3 = smoothBreath(wrapPhase(listenPhase - 0.28f))
            drawCircle(palette.glow.copy(alpha = 0.14f + 0.24f * b1), radius = (if (compact) 44.dp else 59.dp).toPx() * (0.94f + 0.21f * b1), style = Stroke(1.2.dp.toPx()))
            drawCircle(Color.White.copy(alpha = 0.03f + 0.06f * b2), radius = (if (compact) 58.dp else 78.dp).toPx() * (0.95f + 0.18f * b2), style = Stroke(1.dp.toPx()))
            if (!compact) {
                drawCircle(palette.glow.copy(alpha = 0.02f + 0.08f * b3), radius = 98.dp.toPx() * (0.95f + 0.15f * b3), style = Stroke(1.dp.toPx()))
                // 两道错拍慢扩散潮汐涟漪
                drawTidalRipple(48.dp.toPx(), listenPhase, palette.glow, 0.34f)
                drawTidalRipple(48.dp.toPx(), wrapPhase(listenPhase + 0.5f), palette.aura, 0.22f)
            }
        }
        CallPhase.Hearing -> {
            drawCircle(palette.glow.copy(alpha = 0.28f), radius = 58.dp.toPx(), style = Stroke(1.3.dp.toPx()))
            drawTidalRipple(48.dp.toPx(), fastRipple, palette.glow, 0.56f)
            drawTidalRipple(48.dp.toPx(), wrapPhase(fastRipple + 0.5f), palette.glow, 0.44f)
        }
        CallPhase.Speaking -> {
            drawCircle(palette.glow.copy(alpha = 0.16f + 0.14f * eqWave), radius = 44.dp.toPx() * (0.96f + 0.09f * eqWave), style = Stroke(1.1.dp.toPx()))
            drawTidalRipple(36.dp.toPx(), fastRipple, palette.glow, 0.42f)
        }
    }
}

private fun DrawScope.drawTidalRipple(baseRadius: Float, progress: Float, color: Color, peakAlpha: Float) {
    val eased = 1f - (1f - progress) * (1f - progress)
    val radius = baseRadius * (0.96f + 1.12f * eased)
    val alpha = peakAlpha * (1f - progress) * (1f - 0.35f * progress)
    if (alpha > 0.005f) {
        drawCircle(color.copy(alpha = alpha), radius = radius, style = Stroke(1.4.dp.toPx()))
    }
}

/** 官方「双星相伴」纯白双星标，两颗星各自围绕几何中心独立脉动与旋转。 */
@Composable
private fun TwinStarsMark(
    phase: CallPhase,
    size: Dp,
    thinkPhase: Float,
    listenPhase: Float,
    fastRipple: Float,
) {
    val mainPath = remember { buildMainStarPath() }
    val subPath = remember { buildSubStarPath() }

    val mainScale: Float
    val mainAngle: Float
    val mainAlpha: Float
    val subScale: Float
    val subAngle: Float
    val subAlpha: Float
    val subShiftX: Float
    val subShiftY: Float

    when (phase) {
        CallPhase.Thinking -> {
            val bMain = thinkBreathCurve(thinkPhase)
            val bSub = thinkBreathCurve(wrapPhase(thinkPhase - 0.116f))
            mainScale = 0.90f + 0.21f * bMain
            mainAngle = 90f * quarterTurnCurve(thinkPhase)
            mainAlpha = 0.90f + 0.10f * bMain
            val subP = wrapPhase(thinkPhase - 0.116f)
            subScale = 0.76f + 0.46f * bSub
            subAngle = -90f * quarterTurnCurve(subP)
            subAlpha = 0.68f + 0.32f * bSub
            subShiftX = 2.6f * bSub
            subShiftY = 1.0f - 4.0f * bSub
        }
        CallPhase.Listening, CallPhase.Starting -> {
            val bMain = smoothBreath(listenPhase)
            val subP = wrapPhase(listenPhase - 0.13f)
            val bSub = smoothBreath(subP)
            mainScale = 0.91f + 0.19f * bMain
            mainAngle = -9f + 18f * bMain
            mainAlpha = 0.90f + 0.10f * bMain
            subScale = 0.78f + 0.40f * bSub
            subAngle = 90f * quarterTurnCurve(subP)
            subAlpha = 0.68f + 0.32f * bSub
            subShiftX = 2.0f * bSub
            subShiftY = 1.5f - 4.0f * bSub
        }
        CallPhase.Hearing -> {
            val b1 = smoothBreath(fastRipple)
            val b2 = smoothBreath(wrapPhase(fastRipple - 0.18f))
            mainScale = 0.96f + 0.12f * b1
            mainAngle = 0f
            mainAlpha = 1f
            subScale = 0.92f + 0.20f * b2
            subAngle = 0f
            subAlpha = 0.85f + 0.15f * b2
            subShiftX = 0f
            subShiftY = -1.2f * b2
        }
        CallPhase.Speaking -> {
            mainScale = 1f
            mainAngle = 0f
            mainAlpha = 1f
            subScale = 1f
            subAngle = 0f
            subAlpha = 1f
            subShiftX = 0f
            subShiftY = 0f
        }
    }

    Canvas(Modifier.size(size)) {
        val unit = this.size.minDimension / 120f
        withTransform({
            translate(55.5f * unit, 62.5f * unit)
            rotate(mainAngle, pivot = Offset.Zero)
            scale(unit * 0.96f * mainScale, unit * 0.96f * mainScale, pivot = Offset.Zero)
        }) {
            drawPath(mainPath, Color.White.copy(alpha = mainAlpha))
        }
        withTransform({
            translate((82.5f + subShiftX) * unit, (35.5f + subShiftY) * unit)
            rotate(subAngle, pivot = Offset.Zero)
            scale(unit * 0.47f * subScale, unit * 0.47f * subScale, pivot = Offset.Zero)
        }) {
            drawPath(subPath, Color.White.copy(alpha = subAlpha))
        }
    }
}

@Composable
private fun SpeakingEqBars(wave: Float) {
    val heights = listOf(
        8f + 12f * wave,
        18f - 10f * wave,
        11f + 9f * (1f - wave),
        20f - 11f * wave,
        9f + 11f * wave,
    )
    Row(Modifier.height(22.dp), horizontalArrangement = Arrangement.spacedBy(3.5.dp), verticalAlignment = Alignment.CenterVertically) {
        heights.forEach { h ->
            Box(Modifier.size(3.dp, h.dp).clip(CircleShape).background(Color.White))
        }
    }
}

internal fun buildMainStarPath(): Path = Path().apply {
    moveTo(-3.2f, -30.2f)
    quadraticTo(0f, -34f, 3.2f, -30.2f)
    cubicTo(7.6f, -15.6f, 15.6f, -7.6f, 30.2f, -3.2f)
    quadraticTo(34f, 0f, 30.2f, 3.2f)
    cubicTo(15.6f, 7.6f, 7.6f, 15.6f, 3.2f, 30.2f)
    quadraticTo(0f, 34f, -3.2f, 30.2f)
    cubicTo(-7.6f, 15.6f, -15.6f, 7.6f, -30.2f, 3.2f)
    quadraticTo(-34f, 0f, -30.2f, -3.2f)
    cubicTo(-15.6f, -7.6f, -7.6f, -15.6f, -3.2f, -30.2f)
    close()
}

internal fun buildSubStarPath(): Path = Path().apply {
    moveTo(-3.8f, -29.8f)
    quadraticTo(0f, -34f, 3.8f, -29.8f)
    cubicTo(8.5f, -15f, 15f, -8.5f, 29.8f, -3.8f)
    quadraticTo(34f, 0f, 29.8f, 3.8f)
    cubicTo(15f, 8.5f, 8.5f, 15f, 3.8f, 29.8f)
    quadraticTo(0f, 34f, -3.8f, 29.8f)
    cubicTo(-8.5f, 15f, -15f, 8.5f, -29.8f, 3.8f)
    quadraticTo(-34f, 0f, -29.8f, -3.8f)
    cubicTo(-15f, -8.5f, -8.5f, -15f, -3.8f, -29.8f)
    close()
}

private fun wrapPhase(t: Float): Float {
    val r = t % 1f
    return if (r < 0f) r + 1f else r
}

/** 0→1→0 的自然潮汐余弦呼吸曲线。 */
private fun smoothBreath(t: Float): Float = (1f - cos(2.0 * PI * t).toFloat()) * 0.5f

/** 思考态胸腔呼吸曲线：吸气舒张(0..0.46) → 短暂蓄力停顿(0.46..0.56) → 轻柔呼出(0.56..1.0)。 */
private fun thinkBreathCurve(t: Float): Float = when {
    t < 0.46f -> {
        val u = t / 0.46f
        (1f - cos(PI * u).toFloat()) * 0.5f
    }
    t < 0.56f -> 1f - 0.08f * ((t - 0.46f) / 0.10f)
    else -> {
        val u = (t - 0.56f) / 0.44f
        0.92f * (1f + cos(PI * u).toFloat()) * 0.5f
    }
}

/** 四芒星 0→1 的缓动转角进度：吸气到 0.80 间平滑转完 90°，末尾 0.20 稳住，与四芒星 90° 对称性无缝衔接。 */
private fun quarterTurnCurve(t: Float): Float {
    val u = (t / 0.82f).coerceIn(0f, 1f)
    return u * u * (3f - 2f * u)
}

// ───────────── 收起成胶囊 ─────────────

private fun phaseLabel(phase: CallPhase): String = when (phase) {
    CallPhase.Starting -> "正在接通"
    CallPhase.Listening -> "聆听中"
    CallPhase.Hearing -> "听你说话"
    CallPhase.Thinking -> "小伴在想"
    CallPhase.Speaking -> "小伴在说话"
}

/** 收起后留在页面顶部的深色胶囊：声音起伏、状态、通话时长，右边红色挂断；点一下回到通话页。 */
@Composable
fun CallCapsule(call: VoiceCall, seconds: Int, onExpand: () -> Unit, modifier: Modifier = Modifier) {
    val state by call.state.collectAsState()
    val palette = currentOrbPalette()
    val transition = rememberInfiniteTransition(label = "capsule")
    val wobble by transition.animateFloat(
        0f, 1f,
        infiniteRepeatable(animation = tween<Float>(700, easing = LinearEasing), repeatMode = RepeatMode.Reverse),
        label = "wobble",
    )
    val active = state.phase == CallPhase.Hearing || state.phase == CallPhase.Speaking || state.phase == CallPhase.Thinking
    Row(
        modifier
            .padding(horizontal = 16.dp)
            .fillMaxWidth()
            .height(46.dp)
            .clip(CircleShape)
            .background(CallCapsuleBg)
            .border(0.5.dp, CallLine, CircleShape)
            .clickable(onClick = onExpand)
            .padding(start = 16.dp, end = 5.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Row(Modifier.height(16.dp), horizontalArrangement = Arrangement.spacedBy(2.5.dp), verticalAlignment = Alignment.CenterVertically) {
            listOf(0.45f, 0.92f, 0.62f, 0.80f).forEachIndexed { i, base ->
                val h = if (active) base * (0.52f + 0.48f * (if (i % 2 == 0) wobble else 1f - wobble)) else 0.36f
                Box(Modifier.size(2.5.dp, (16f * h).dp).clip(CircleShape).background(palette.start))
            }
        }
        Text(
            "通话中 · ${phaseLabel(state.phase)}",
            Modifier.weight(1f),
            color = CallFg,
            style = XType.Body,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(clock(seconds), color = CallFg2, style = XType.Caption, fontFamily = FontFamily.Monospace)
        Box(
            Modifier
                .size(36.dp)
                .clip(CircleShape)
                .background(CallHang)
                .clickable { call.stop() },
            contentAlignment = Alignment.Center,
        ) {
            XIcon(XIcons.Hangup, tint = Color.White, size = 18.dp, description = "挂断")
        }
    }
}

// ───────────── 通话里的确认和选择 ─────────────

/** 小伴要你确认一个操作、或让你选一个答案：直接在通话页里点，不用挂断。 */
@Composable
private fun CallPromptCard(prompt: CallPrompt, onChoose: (String) -> Unit) {
    Column(
        Modifier
            .padding(top = 14.dp)
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .background(CallGlass)
            .border(0.5.dp, CallLine, RoundedCornerShape(20.dp))
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        val badgeText = prompt.danger?.let { "高风险操作 · $it" } ?: if (prompt.question) "等待你的选择" else "需确认操作"
        Text(
            badgeText,
            Modifier
                .clip(CircleShape)
                .background(CallWarn.copy(alpha = 0.16f))
                .padding(horizontal = 9.dp, vertical = 3.dp),
            color = CallWarn,
            fontSize = 11.sp,
            fontWeight = FontWeight.SemiBold,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            prompt.title,
            color = CallFg,
            style = XType.Body,
            fontWeight = FontWeight.SemiBold,
            maxLines = 3,
            overflow = TextOverflow.Ellipsis,
        )
        if (prompt.detail.isNotBlank()) {
            Text(
                prompt.detail,
                Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(10.dp))
                    .background(Color.Black.copy(alpha = 0.25f))
                    .padding(horizontal = 10.dp, vertical = 8.dp),
                color = CallFg.copy(alpha = 0.62f),
                style = XType.Caption,
                fontFamily = FontFamily.Monospace,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
            )
        }
        if (prompt.question) {
            Column(
                Modifier.heightIn(max = 200.dp).verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                prompt.options.forEach { option -> PromptButton(option, Modifier.fillMaxWidth(), onChoose) }
            }
        } else {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                prompt.options.forEach { option -> PromptButton(option, Modifier.weight(1f), onChoose) }
            }
        }
    }
}

@Composable
private fun PromptButton(option: CallPromptOption, modifier: Modifier, onChoose: (String) -> Unit) {
    val isAllow = option.role == CallPromptOption.Role.Allow
    val (bg, fg) = when (option.role) {
        CallPromptOption.Role.Allow -> CallFg to CallBgTop
        CallPromptOption.Role.Neutral -> Color.White.copy(alpha = 0.12f) to CallFg
        CallPromptOption.Role.Deny -> Color.White.copy(alpha = 0.08f) to CallFg2
    }
    Box(
        modifier
            .height(42.dp)
            .clip(CircleShape)
            .background(bg)
            .let { if (!isAllow) it.border(0.5.dp, CallLineSubtle, CircleShape) else it }
            .clickable { onChoose(option.id) }
            .padding(horizontal = 12.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(option.label, color = fg, style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}
