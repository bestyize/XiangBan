package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.ui.Alignment
import kotlinx.coroutines.delay
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.CloudSynthesizer
import xyz.xiangban.xiaoban.common.community.EdgeTtsSynthesizer
import xyz.xiangban.xiaoban.common.community.EngineUnavailable
import xyz.xiangban.xiaoban.common.community.ModelDownloadException
import xyz.xiangban.xiaoban.common.community.Ptt
import xyz.xiangban.xiaoban.common.community.PttAction
import xyz.xiangban.xiaoban.common.community.RecognizerEngine
import xyz.xiangban.xiaoban.common.community.SynthesizerEngine
import xyz.xiangban.xiaoban.common.community.VoiceCall

// ───────────── 引擎怎么选 ─────────────

/** 通话和语音输入用的识别引擎，按顺序用：手机本地（模型下好了才有）→ [fallbacks]（电脑上的、云端的）。 */
fun CommunityEnv.recognizers(vararg fallbacks: RecognizerEngine): List<RecognizerEngine> =
    listOfNotNull(speech?.recognizer?.takeIf { xyz.xiangban.xiaoban.common.community.SenseVoiceModel.Files.all { spec -> speech.files.size(spec.name) == spec.bytes } }) + fallbacks

fun CommunityEnv.turnJudge(): (suspend (ByteArray) -> Boolean?)? =
    speech?.takeIf { it.files.size(xyz.xiangban.xiaoban.common.community.VoiceModels.SmartTurnName) == xyz.xiangban.xiaoban.common.community.VoiceModels.SmartTurn.bytes }
        ?.let { local -> { audio -> local.turnComplete(audio) } }

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

// ───────────── 本地语音模型下载 ─────────────

/** 第一次要在手机上直接识别语音时：说明模型多大、从哪来，让用户决定下不下载。 */
@Composable
fun SpeechModelSheet(env: CommunityEnv, panelModifier: Modifier, onFinished: (Boolean) -> Unit) {
    val models = env.speechModels ?: return
    var done by remember { mutableStateOf(0L) }
    var total by remember { mutableStateOf(xyz.xiangban.xiaoban.common.community.VoiceModels.TotalBytes) }
    var running by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var job by remember { mutableStateOf<Job?>(null) }
    BottomSheet({ job?.cancel(); onFinished(false) }, panelModifier) {
        Text("在手机上识别语音", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        Text(
            "下载语音模型（SenseVoice 识别、Silero 人声检测和 Smart Turn 判停，共约 ${(total / 1_000_000)} MB，和电脑上用的是同一套）；已下载的文件会复用。之后识别和判断你说完没有都在手机上完成。建议在 Wi-Fi 下载。",
            Modifier.padding(top = 8.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        if (running || done > 0) {
            val fraction = if (total > 0) (done.toFloat() / total).coerceIn(0f, 1f) else 0f
            Box(Modifier.padding(top = 14.dp).fillMaxWidth().height(6.dp).clip(RoundedCornerShape(3.dp)).background(LocalExtraColors.current.line)) {
                Box(Modifier.fillMaxWidth(fraction).height(6.dp).background(Brand))
            }
            Text("${(fraction * 100).toInt()}%  ${done / 1_000_000} / ${total / 1_000_000} MB", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        }
        error?.let { Text(it, Modifier.padding(top = 8.dp), fontSize = 14.sp, color = Danger) }
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

/**
 * 输入框的“按住说话”（设计规范第 8 节）：按下开始录，松手按手指位置处理——发送（识别后直接发出去）、转文字（识别后放进输入框）或取消；
 * 最长 20 秒，不到 0.5 秒算太短。识别先用手机本地的，不行再用 [engines] 里后面的。
 */
class VoiceInputController(
    private val env: CommunityEnv,
    private val scope: kotlinx.coroutines.CoroutineScope,
    private val engines: () -> List<RecognizerEngine>,
    private val onText: (String) -> Unit,
    private val onSend: (String) -> Unit,
    private val onMessage: (String) -> Unit,
) {
    /** 手指正按着、正在录音。 */
    var pressing by mutableStateOf(false)
        private set
    /** 现在松手会怎样；手指移动时更新。 */
    var action by mutableStateOf(PttAction.Send)
        private set
    var elapsedMs by mutableStateOf(0L)
        private set
    var recognizing by mutableStateOf(false)
        private set
    /** 没有麦克风权限：界面弹出“需要使用麦克风”的面板。 */
    var micDenied by mutableStateOf(false)
    private var starting = false
    private var released = false
    private var ticker: Job? = null
    private var startedAt: kotlin.time.TimeMark? = null

    val busy: Boolean get() = pressing || starting || recognizing

    /** 按下：录音是异步起来的（第一次要先要麦克风权限），起来之前就松手的话，起来后直接丢掉。 */
    fun pressStart() {
        val platform = env.platform
        when {
            platform == null -> onMessage("这个平台暂不支持录音")
            recognizing -> onMessage("正在识别上一段语音")
            pressing || starting -> Unit
            else -> {
                starting = true
                released = false
                scope.launch {
                    platform.stopAudio()
                    val ok = platform.startRecording()
                    starting = false
                    when {
                        !ok -> micDenied = true
                        released -> platform.stopRecording()
                        else -> begin()
                    }
                }
            }
        }
    }

    private fun begin() {
        val mark = kotlin.time.TimeSource.Monotonic.markNow()
        startedAt = mark
        elapsedMs = 0
        action = PttAction.Send
        pressing = true
        ticker = scope.launch {
            while (pressing) {
                kotlinx.coroutines.delay(100)
                elapsedMs = mark.elapsedNow().inWholeMilliseconds
                if (Ptt.expired(elapsedMs)) {
                    onMessage("已到 20 秒，自动发送")
                    end(PttAction.Send)
                }
            }
        }
    }

    fun pressMove(to: PttAction) { if (pressing) action = to }

    /** 手指抬起：按当前位置处理。 */
    fun pressRelease() { if (pressing) end(action) else if (starting) released = true }

    /** 被打断（来电、页面离开、手势被系统取消）：直接丢掉。 */
    fun pressCancel() { if (pressing) end(PttAction.Cancel) else if (starting) released = true }

    private fun end(how: PttAction) {
        if (!pressing) return
        pressing = false
        ticker?.cancel()
        val elapsed = startedAt?.elapsedNow()?.inWholeMilliseconds ?: elapsedMs
        val pcm = env.platform?.stopRecording()
        if (how == PttAction.Cancel || pcm == null) return
        if (Ptt.tooShort(elapsed)) { onMessage("说话时间太短"); return }
        recognizing = true
        scope.launch {
            try {
                val (text, reason) = recognizeWith(engines(), pcm)
                when {
                    text == null -> onMessage(reason ?: "没有识别出来")
                    text.isBlank() -> onMessage("没听清，请再说一次")
                    how == PttAction.Send -> onSend(text)
                    else -> onText(text)
                }
            } finally {
                recognizing = false
            }
        }
    }

    fun release() {
        ticker?.cancel()
        if (pressing) { pressing = false; env.platform?.stopRecording() }
    }
}

@Composable
fun rememberVoiceInput(env: CommunityEnv, engines: () -> List<RecognizerEngine>, onText: (String) -> Unit, onSend: (String) -> Unit, onMessage: (String) -> Unit): VoiceInputController {
    val scope = androidx.compose.runtime.rememberCoroutineScope()
    val controller = remember(env) { VoiceInputController(env, scope, engines, onText, onSend, onMessage) }
    DisposableEffect(controller) { onDispose { controller.release() } }
    return controller
}

/** 一通语音通话的发起与本地模型下载的询问。 */
class VoiceCallHost(private val env: CommunityEnv) {
    var call by mutableStateOf<VoiceCall?>(null)
    /** 通话收起成了顶部胶囊。 */
    var minimized by mutableStateOf(false)
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
            minimized = false
            call = factory().also(::applySettings)
        }
    }

    /** 声音设置里记着的“直接开口打断”和“不朗读”，打电话时带上。 */
    private fun applySettings(call: VoiceCall) {
        SpeechState.settings.bargeIn?.let(call::setBargeIn)
        if (SpeechState.settings.callSilent) call.setSilent(true)
    }

    fun offerFinished(downloaded: Boolean) {
        offering = false
        if (!downloaded) declined = true
        pending?.let { make -> minimized = false; call = make().also(::applySettings) }
        pending = null
    }
}

@Composable
fun rememberVoiceCallHost(env: CommunityEnv): VoiceCallHost = remember(env) { VoiceCallHost(env) }

/**
 * 通话相关的几层界面：询问下载模型的底部弹层、通话全屏界面，收起后是页面顶部的胶囊。
 * 通话的开始、结束都在这里管，所以收起、展开不会打断通话。
 */
@Composable
fun VoiceCallLayer(env: CommunityEnv, host: VoiceCallHost, title: String, onNotice: (String) -> Unit) {
    OverlayTransition(host.offering) { panel -> SpeechModelSheet(env, panel) { host.offerFinished(it) } }
    val current = host.call ?: return
    val state by current.state.collectAsState()
    var seconds by remember(current) { mutableStateOf(0) }
    LaunchedEffect(current) { current.start() }
    LaunchedEffect(current) { while (true) { delay(1000); seconds += 1 } }
    DisposableEffect(current) { onDispose { current.stop() } }
    LaunchedEffect(state.ended) {
        if (state.ended) {
            host.call = null
            host.minimized = false
            state.notice?.let(onNotice)
        }
    }
    if (host.minimized) {
        Box(Modifier.fillMaxSize()) {
            CallCapsule(current, seconds, { host.minimized = false }, Modifier.align(Alignment.TopCenter).statusBarsPadding().padding(top = 64.dp))
        }
    } else {
        VoiceCallScreen(current, title, seconds, onMinimize = { host.minimized = true })
    }
}
