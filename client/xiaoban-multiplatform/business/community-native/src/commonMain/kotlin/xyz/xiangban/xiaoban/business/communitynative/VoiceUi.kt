package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.CallPhase
import xyz.xiangban.xiaoban.common.community.CloudRecognizer
import xyz.xiangban.xiaoban.common.community.CloudSynthesizer
import xyz.xiangban.xiaoban.common.community.EdgeTtsSynthesizer
import xyz.xiangban.xiaoban.common.community.EngineUnavailable
import xyz.xiangban.xiaoban.common.community.ModelDownloadException
import xyz.xiangban.xiaoban.common.community.RecognizerEngine
import xyz.xiangban.xiaoban.common.community.SynthesizerEngine
import xyz.xiangban.xiaoban.common.community.VoiceCall

// ───────────── 引擎怎么选 ─────────────

/** 通话和语音输入用的识别引擎，按顺序用：手机本地（模型下好了才有）→ [fallbacks]（电脑上的、云端的）。 */
fun CommunityEnv.recognizers(vararg fallbacks: RecognizerEngine): List<RecognizerEngine> =
    listOfNotNull(speech?.recognizer?.takeIf { speechModels?.ready == true }) + fallbacks

/** 合成引擎：手机直连 Edge 朗读（和电脑同一个服务）→ 云端小伴 AI 的语音合成。 */
fun CommunityEnv.synthesizers(): List<SynthesizerEngine> = listOf(EdgeTtsSynthesizer(client), CloudSynthesizer(ai))

/** 依次试每个引擎，第一个出结果的算数；一个都不行返回 null 和原因。 */
suspend fun recognizeWith(engines: List<RecognizerEngine>, pcm: ByteArray): Pair<String?, String?> {
    var reason: String? = null
    for (engine in engines) {
        try {
            return engine.recognize(pcm).trim() to null
        } catch (e: EngineUnavailable) {
            reason = "${engine.name}暂时用不了：${e.message}"
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            reason = e.message ?: "没有识别出来"
        }
    }
    return null to (reason ?: "没有可用的语音识别")
}

// ───────────── 通话界面 ─────────────

/** 语音通话的全屏界面：呼吸圆点、状态、你说的话和小伴的回答、打断和挂断。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun VoiceCallScreen(call: VoiceCall, title: String, subtitle: String, onClosed: (String?) -> Unit) {
    val state by call.state.collectAsState()
    val extra = LocalExtraColors.current
    BackHandler(onBack = { call.stop() })
    LaunchedEffect(Unit) { call.start() }
    LaunchedEffect(state.ended) { if (state.ended) onClosed(state.notice) }
    DisposableEffect(call) { onDispose { call.stop() } }
    Column(
        Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).statusBarsPadding().navigationBarsPadding().padding(horizontal = 24.dp, vertical = 16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(title, fontSize = 17.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(subtitle, Modifier.padding(top = 2.dp), fontSize = 12.sp, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Spacer(Modifier.weight(1f))
        // 呼吸的圆：听的时候随你的声音大小变化，想和说的时候固定大小。
        val grow = if (state.phase == CallPhase.Hearing || state.phase == CallPhase.Listening) state.level else 0f
        val diameter = 120 + (grow * 60).toInt()
        Box(Modifier.size(200.dp), contentAlignment = Alignment.Center) {
            Box(Modifier.size(diameter.dp).clip(CircleShape).background(if (state.phase == CallPhase.Speaking) Brand else if (state.phase == CallPhase.Thinking) extra.line else Brand.copy(alpha = 0.18f)))
            XIcon(
                when (state.phase) { CallPhase.Speaking -> XIcons.Volume; CallPhase.Thinking -> XIcons.SparkleSmall; else -> XIcons.Mic },
                tint = if (state.phase == CallPhase.Speaking) Color.White else Brand, size = 40.dp,
            )
        }
        Text(
            when (state.phase) {
                CallPhase.Starting -> "正在接通…"
                CallPhase.Listening -> "请说话，我在听"
                CallPhase.Hearing -> "听到了…"
                CallPhase.Thinking -> "小伴在想…"
                CallPhase.Speaking -> "小伴在说话"
            },
            Modifier.padding(top = 12.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold,
        )
        if (state.partial.isNotBlank()) Text(state.partial, Modifier.padding(top = 8.dp), fontSize = 14.sp, color = extra.ink3, textAlign = TextAlign.Center)
        if (state.userText.isNotBlank()) Text("你：${state.userText}", Modifier.padding(top = 14.dp).fillMaxWidth(), fontSize = 14.sp, lineHeight = 21.sp, color = extra.ink3, maxLines = 3, overflow = TextOverflow.Ellipsis, textAlign = TextAlign.Center)
        if (state.replyText.isNotBlank()) Text(state.replyText, Modifier.padding(top = 8.dp).fillMaxWidth(), fontSize = 15.sp, lineHeight = 23.sp, maxLines = 6, overflow = TextOverflow.Ellipsis, textAlign = TextAlign.Center)
        state.notice?.let { Text(it, Modifier.padding(top = 12.dp), fontSize = 13.sp, lineHeight = 19.sp, color = Brand, textAlign = TextAlign.Center) }
        Spacer(Modifier.weight(1f))
        Row(horizontalArrangement = Arrangement.spacedBy(40.dp), verticalAlignment = Alignment.CenterVertically) {
            val canInterrupt = state.phase == CallPhase.Speaking || state.phase == CallPhase.Thinking
            CallButton(XIcons.Close, if (canInterrupt) "打断" else "挂断", if (canInterrupt) MaterialTheme.colorScheme.onSurface else Brand, onClick = { if (canInterrupt) call.interrupt() else call.stop() })
            if (canInterrupt) CallButton(XIcons.Close, "挂断", Brand, onClick = { call.stop() })
        }
    }
}

@Composable
private fun CallButton(icon: androidx.compose.ui.graphics.vector.ImageVector, label: String, color: Color, onClick: () -> Unit) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.size(64.dp).clip(CircleShape).background(color).clickable(onClick = onClick), contentAlignment = Alignment.Center) {
            XIcon(icon, tint = Color.White, size = 28.dp, description = label)
        }
        Text(label, Modifier.padding(top = 6.dp), fontSize = 13.sp)
    }
}

// ───────────── 本地语音模型下载 ─────────────

/** 第一次要在手机上直接识别语音时：说明模型多大、从哪来，让用户决定下不下载。 */
@Composable
fun SpeechModelSheet(env: CommunityEnv, panelModifier: Modifier, onFinished: (Boolean) -> Unit) {
    val models = env.speechModels ?: return
    var done by remember { mutableStateOf(0L) }
    var total by remember { mutableStateOf(xyz.xiangban.xiaoban.common.community.SenseVoiceModel.TotalBytes) }
    var running by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var job by remember { mutableStateOf<Job?>(null) }
    BottomSheet({ job?.cancel(); onFinished(false) }, panelModifier) {
        Text("在手机上识别语音", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        Text(
            "下载语音识别模型（SenseVoice，约 ${(total / 1_000_000)} MB，和电脑上用的是同一个），之后说话的识别在手机上完成，不用把录音传出去，也不依赖电脑在线。建议在 Wi-Fi 下载。",
            Modifier.padding(top = 8.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        if (running || done > 0) {
            val fraction = if (total > 0) (done.toFloat() / total).coerceIn(0f, 1f) else 0f
            Box(Modifier.padding(top = 14.dp).fillMaxWidth().height(6.dp).clip(RoundedCornerShape(3.dp)).background(LocalExtraColors.current.line)) {
                Box(Modifier.fillMaxWidth(fraction).height(6.dp).background(Brand))
            }
            Text("${(fraction * 100).toInt()}%  ${done / 1_000_000} / ${total / 1_000_000} MB", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        }
        error?.let { Text(it, Modifier.padding(top = 8.dp), fontSize = 13.sp, color = Brand) }
        Row(Modifier.padding(top = 16.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            GhostPillButton(if (running) "取消" else "先不用", { job?.cancel(); onFinished(false) }, Modifier.weight(1f))
            PillButton(if (running) "下载中…" else if (error != null) "重试" else "下载", {
                error = null
                running = true
                job = env.scope.launch {
                    try {
                        models.download { d, t -> done = d; total = t }
                        running = false
                        onFinished(true)
                    } catch (e: CancellationException) {
                        running = false
                        throw e
                    } catch (e: ModelDownloadException) {
                        running = false
                        error = e.message
                    } catch (e: Exception) {
                        running = false
                        error = "下载失败，请检查网络后重试"
                    }
                }
            }, Modifier.weight(1f), enabled = !running)
        }
    }
}

// ───────────── 页面里用的两个小控制器 ─────────────

/** 输入框的麦克风：点一下开始录，再点一下结束并识别，文字交给 [onText]；最长 20 秒。 */
class VoiceInputController(
    private val env: CommunityEnv,
    private val scope: kotlinx.coroutines.CoroutineScope,
    private val engines: () -> List<RecognizerEngine>,
    private val onText: (String) -> Unit,
    private val onMessage: (String) -> Unit,
) {
    var recording by mutableStateOf(false)
        private set
    var seconds by mutableStateOf(0)
        private set
    var recognizing by mutableStateOf(false)
        private set
    private var timer: Job? = null

    val busy: Boolean get() = recording || recognizing

    fun toggle() {
        val platform = env.platform
        when {
            platform == null -> onMessage("这个平台暂不支持录音")
            recording -> finish(send = true)
            recognizing -> onMessage("正在识别上一段语音")
            else -> scope.launch {
                platform.stopAudio()
                if (!platform.startRecording()) { onMessage("没有麦克风权限，没法录音"); return@launch }
                seconds = 0
                recording = true
                timer = scope.launch {
                    while (recording) {
                        kotlinx.coroutines.delay(1000)
                        seconds += 1
                        if (seconds >= 20) finish(send = true)
                    }
                }
            }
        }
    }

    /** 结束录音；[send] 为假就丢掉。 */
    fun finish(send: Boolean) {
        if (!recording) return
        recording = false
        timer?.cancel()
        val pcm = env.platform?.stopRecording()
        if (!send || pcm == null) return
        recognizing = true
        scope.launch {
            try {
                val (text, reason) = recognizeWith(engines(), pcm)
                when {
                    text == null -> onMessage(reason ?: "没有识别出来")
                    text.isBlank() -> onMessage("没听清，请再说一次")
                    else -> onText(text)
                }
            } finally {
                recognizing = false
            }
        }
    }

    fun release() {
        timer?.cancel()
        if (recording) { recording = false; env.platform?.stopRecording() }
    }
}

@Composable
fun rememberVoiceInput(env: CommunityEnv, engines: () -> List<RecognizerEngine>, onText: (String) -> Unit, onMessage: (String) -> Unit): VoiceInputController {
    val scope = androidx.compose.runtime.rememberCoroutineScope()
    val controller = remember(env) { VoiceInputController(env, scope, engines, onText, onMessage) }
    DisposableEffect(controller) { onDispose { controller.release() } }
    return controller
}

/** 一通语音通话的发起与本地模型下载的询问。 */
class VoiceCallHost(private val env: CommunityEnv) {
    var call by mutableStateOf<VoiceCall?>(null)
    var offering by mutableStateOf(false)
        private set
    private var declined = false
    private var pending: (() -> VoiceCall)? = null

    /** 发起通话。手机支持本地识别但模型还没下载时，先问用户要不要下载（只问一次）。 */
    fun request(factory: () -> VoiceCall) {
        if (call != null || offering) return
        if (env.speech != null && env.speechModels?.ready == false && !declined) {
            pending = factory
            offering = true
        } else {
            call = factory()
        }
    }

    fun offerFinished(downloaded: Boolean) {
        offering = false
        if (!downloaded) declined = true
        pending?.let { call = it() }
        pending = null
    }
}

@Composable
fun rememberVoiceCallHost(env: CommunityEnv): VoiceCallHost = remember(env) { VoiceCallHost(env) }

/** 通话相关的两层界面：询问下载模型的底部弹层，和通话本身的全屏界面。 */
@Composable
fun VoiceCallLayer(env: CommunityEnv, host: VoiceCallHost, title: String, subtitle: String, onNotice: (String) -> Unit) {
    OverlayTransition(host.offering) { panel -> SpeechModelSheet(env, panel) { host.offerFinished(it) } }
    host.call?.let { current ->
        VoiceCallScreen(current, title, subtitle) { notice ->
            host.call = null
            notice?.let(onNotice)
        }
    }
}
