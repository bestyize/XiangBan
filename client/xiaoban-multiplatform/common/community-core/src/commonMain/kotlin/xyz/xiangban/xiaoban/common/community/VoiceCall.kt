package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.flow.updateAndGet
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withTimeoutOrNull
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.TimeSource

/** 语音对话要用到的平台能力（录音、听写、播放、朗读）。手机上由 Android 实现，不支持的默认什么也不做。 */
interface VoicePlatform {
    /** 开始流式录音：每 20ms 一帧（16kHz、单声道、16 位小端 PCM）调用 [onFrame]（在后台线程）。第一次会向用户要麦克风权限。返回停止录音的函数；不能录返回 null。 */
    suspend fun streamRecording(onFrame: (ByteArray) -> Unit): (() -> Unit)? = null

    /** 用手机自带的语音识别听一句话：[onPartial] 给中间结果，[onResult] 给（文字, 出错原因）；没人说话时文字是空串。返回取消函数；不支持返回 null。 */
    fun listenOnce(onPartial: (String) -> Unit, onResult: (String?, String?) -> Unit): (() -> Unit)? = null

    /** 用手机自带的语音合成念一句话，念完回调 [onDone]（是否成功）。返回是否开始。 */
    fun speakText(text: String, onDone: (Boolean) -> Unit): Boolean = false

    /** 播放一段音频，播完或出错回调 [onDone]；被 [stopAudio] 打断时不回调。返回是否开始播放。 */
    fun playAudio(bytes: ByteArray, mime: String, onDone: () -> Unit): Boolean = false
    fun stopAudio() {}
    fun stopSpeaking() {}

    /** 这台手机现在能不能把小伴自己的声音从麦克风里去掉（系统回声消除可用，或戴着耳机）。不能的话“直接开口打断”默认关闭，免得它被自己的声音打断。 */
    fun bargeInSafe(): Boolean = true

    /** “有没有人在说话”的判断；每次调用给一个新的（带自己的状态）。没有更准的就返回 null，用声音大小判断。 */
    fun speechGate(): SpeechGate? = null
}

/** 通话里小伴要你确认的操作或让你选的问题；点一个选项就回答，不用挂断。 */
class CallPrompt(val title: String, val detail: String, val danger: String?, val question: Boolean, val options: List<CallPromptOption>)

/** [role]：允许（主操作）、普通选项、拒绝或跳过（弱操作）。 */
class CallPromptOption(val id: String, val label: String, val role: Role = Role.Neutral) {
    enum class Role { Allow, Neutral, Deny }
}

enum class CallPhase { Starting, Listening, Hearing, Thinking, Speaking }

data class CallState(
    val phase: CallPhase = CallPhase.Starting,
    /** 声音大小 0 到 1，画呼吸动画用。 */
    val level: Float = 0f,
    /** 手机自带识别给的中间结果。 */
    val partial: String = "",
    /** 这一轮识别出的你说的话。 */
    val userText: String = "",
    /** 这一轮小伴的回答（边生成边更新）。 */
    val replyText: String = "",
    /** 正在念的那句在 [replyText] 里的位置；通话页像歌词一样把它滚到中间、点亮。没在念回答时为 null。 */
    val reading: IntRange? = null,
    /** 要让你知道的事：出错原因、需要你去确认操作、好久没人说话。 */
    val notice: String? = null,
    val ended: Boolean = false,
    /** 小伴说话时，你直接开口是否能打断它（有回声的环境可以关掉）。 */
    val bargeIn: Boolean = true,
    /** 静音：麦克风的声音不再交给识别，小伴听不到你，也不会被你的声音打断。 */
    val muted: Boolean = false,
    /** 关了朗读（通话页右上角的喇叭）：小伴照常听、照常回答，回答只显示文字，不念出来。 */
    val silent: Boolean = false,
    /** 小伴正等你确认的操作或选择；有它时通话页显示卡片，回答之后小伴接着做。 */
    val prompt: CallPrompt? = null,
)

/** 这次通话没法继续，原因要告诉用户。 */
class CallError(message: String) : Exception(message)

/**
 * 一通“语音对话”：听你说完一句，交给小伴想，再把回答念出来，然后接着听。可以随时 [interrupt] 打断它念，[stop] 挂断。
 * 子类决定一句话怎么变成回答、怎么念出来。
 */
abstract class VoiceCall(protected val scope: CoroutineScope, protected val platform: VoicePlatform) {
    // 默认就能开口打断；回声消除不可用的手机也一样，靠本地识别比对回声（confirmBarge）挡住小伴自己的声音。
    private val _state = MutableStateFlow(CallState(bargeIn = true))
    val state: StateFlow<CallState> = _state.asStateFlow()
    private var job: Job? = null
    private var playback: CompletableDeferred<Boolean>? = null
    protected var cachedRecognition: String? = null
    /** 停顿时就开始的识别（和判停同时跑，和电脑端一样）；说话人没再开口，这就是这句话的结果。 */
    protected var speculativeRecognition: Deferred<String?>? = null
    /** 小伴上一次出声结束的时刻。 */
    @Volatile private var playedAt: TimeSource.Monotonic.ValueTimeMark? = null
    /** 刚听到的这句是在小伴说话时、或它刚说完半秒内开口的：里面可能混着它自己的声音。 */
    protected var heardOverPlayback = false
    private val mic = Channel<ByteArray>(Channel.UNLIMITED)
    private var stopMic: (() -> Unit)? = null
    @Volatile private var pendingBarge: ByteArray? = null
    @Volatile private var bargeTriggered = false

    @Volatile protected var interrupted = false
    @Volatile private var muted = false

    fun start() {
        if (job != null) return
        job = scope.launch {
            try {
                run()
            } catch (e: CancellationException) {
                throw e
            } catch (e: CallError) {
                update { it.copy(notice = e.message) }
            } catch (e: Exception) {
                update { it.copy(notice = e.message ?: "语音通话出错了") }
            } finally {
                stopMic?.invoke()
                stopMic = null
                mic.close()
                platform.stopAudio()
                platform.stopSpeaking()
                update { it.copy(ended = true, level = 0f) }
            }
        }
    }

    /** 回答通话里的确认或选择（[CallPrompt] 的某个选项）。 */
    open fun answerPrompt(id: String) {}

    /** 挂断。 */
    fun stop() {
        job?.cancel()
    }

    /** 打断小伴：不念了，马上回到听你说。 */
    open fun interrupt() {
        interrupted = true
        platform.stopAudio()
        platform.stopSpeaking()
        playback?.complete(true)
    }

    /** 静音/取消静音。静音时正在听的半句话作废，回到“请说话”。 */
    fun setMuted(on: Boolean) {
        muted = on
        if (on) drainMic()
        update { it.copy(muted = on, level = if (on) 0f else it.level, phase = if (on && it.phase == CallPhase.Hearing) CallPhase.Listening else it.phase) }
    }

    /** 打开/关闭“小伴说话时直接开口打断”。 */
    fun setBargeIn(on: Boolean) = update { it.copy(bargeIn = on) }

    /** 关掉/打开朗读。关掉时正在念的那句马上停，回答接着生成、照常显示。 */
    fun setSilent(on: Boolean) {
        update { it.copy(silent = on, reading = if (on) null else it.reading) }
        if (on) {
            platform.stopAudio()
            platform.stopSpeaking()
            playback?.complete(true)
        }
    }

    protected abstract suspend fun run()

    protected open suspend fun confirmBarge(audio: ByteArray): Boolean = true

    protected fun update(change: (CallState) -> CallState) {
        _state.update(change)
    }

    protected fun phase(phase: CallPhase) = update { it.copy(phase = phase, level = if (phase == CallPhase.Listening || phase == CallPhase.Hearing) it.level else 0f) }

    /** 麦克风在整通电话里一直开着（这样小伴说话时也能听见你开口）；第一次需要时才打开。 */
    private suspend fun openMic() {
        if (stopMic != null) return
        stopMic = platform.streamRecording { if (!muted) mic.trySend(it) } ?: throw CallError("没有麦克风权限，没法通话")
    }

    private fun drainMic() {
        while (mic.tryReceive().isSuccess) { /* 想和说的时候录下的声音已经过期 */ }
    }

    private fun newDetector(strict: Boolean, useJudge: Boolean = false) = EndpointDetector(platform.speechGate() ?: EnergyGate(), strict = strict, useJudge = useJudge)

    /**
     * 听到一句话：返回它的录音；等了很久没人说话返回 null；不能录音时抛 [CallError]。
     * 给了 [judge] 就用它判断“说完了吗”（停顿很短就能收尾，没说完的会多等一会儿），否则固定停顿一会儿就算说完。
     * 小伴说话时用户开口打断的那句话已经被录下来了，直接交出来。
     */
    protected suspend fun listenUtterance(
        judge: (suspend (ByteArray) -> Boolean)? = null,
        preview: (suspend (ByteArray) -> String)? = null,
        speculate: Boolean = false,
    ): ByteArray? {
        speculativeRecognition = null
        pendingBarge?.let { pendingBarge = null; interrupted = false; heardOverPlayback = true; return it }
        interrupted = false
        heardOverPlayback = false
        openMic()
        drainMic()
        update { it.copy(phase = CallPhase.Listening, partial = "") }
        val detector = newDetector(strict = false, useJudge = judge != null)
        val verdicts = Channel<Pair<Int, Boolean>>(Channel.UNLIMITED)
        val previews = Channel<Pair<Int, String>>(Channel.UNLIMITED)
        var previewJob: Job? = null
        val judges = mutableListOf<Job>()
        var latestPreview: Pair<Int, String>? = null
        var speculation: Pair<Int, Deferred<String?>>? = null
        var handedOver = false
        var heard = false
        cachedRecognition = null
        try {
            var counter = 0
            for (frame in mic) {
                while (true) { val v = verdicts.tryReceive().getOrNull() ?: break; detector.verdict(v.first, v.second) }
                while (true) {
                    val p = previews.tryReceive().getOrNull() ?: break
                    latestPreview = p
                    if (p.second.isNotBlank() && !isVoiceNoise(p.second)) update { it.copy(partial = p.second) }
                }
                val result = detector.feed(frame)
                if (result.speaking && !heard) {
                    heard = true
                    heardOverPlayback = playedAt?.let { it.elapsedNow() < EchoTail } == true
                }
                // 声音大小不用每 20ms 都刷新界面。
                if (++counter % 5 == 0) update { it.copy(level = result.level, phase = if (result.speaking) CallPhase.Hearing else CallPhase.Listening) }
                if (result.ask != null && judge != null) {
                    val ticket = result.askTicket
                    val audio = result.ask
                    judges += scope.launch { verdicts.trySend(ticket to (try { judge(audio) } catch (e: CancellationException) { throw e } catch (e: Exception) { false })) }
                    // 判“说完了吗”的同时就把这段识别了：判定说完时文字已经好了，不用再等一遍识别。
                    if (speculate && preview != null && speculation?.first != result.speechVersion) {
                        speculation?.second?.cancel()
                        speculation = result.speechVersion to scope.async {
                            try { preview(audio) } catch (e: CancellationException) { throw e } catch (_: Exception) { null }
                        }
                    }
                }
                if (result.preview != null && preview != null && previewJob?.isActive != true) {
                    val audio = result.preview
                    val version = result.speechVersion
                    previewJob = scope.launch {
                        try { previews.trySend(version to preview(audio)) } catch (e: CancellationException) { throw e } catch (_: Exception) { }
                    }
                }
                if (result.utterance != null) {
                    cachedRecognition = latestPreview?.takeIf { it.first == result.speechVersion }?.second?.takeIf { it.isNotBlank() && !isVoiceNoise(it) }
                    // 停顿之后没再开口：停顿时识别的就是整句话。
                    speculation?.takeIf { it.first == result.speechVersion }?.let { speculativeRecognition = it.second; handedOver = true }
                    return result.utterance
                }
                if (result.timedOut) return null
            }
            return null
        } finally {
            detector.close()
            verdicts.close()
            previews.close()
            previewJob?.cancel()
            judges.forEach { it.cancel() }
            if (!handedOver) speculation?.second?.cancel()
        }
    }

    /**
     * 小伴想和说的时候麦克风继续听着：你连续出声一小会儿（比平时更严格，免得把它自己的回声当成你）就打断它，
     * 并把你说的这句话录完，留给下一轮 [listenUtterance]。返回的任务要在这一轮结束时用 [finishBargeWatch] 收尾。
     */
    protected fun watchForBarge(): Job? {
        if (!state.value.bargeIn || stopMic == null) return null
        bargeTriggered = false
        return scope.launch {
            var detector = newDetector(strict = true)
            var nextCheck = BargeMs
            try {
                for (frame in mic) {
                    val r = detector.feed(frame)
                    if (!bargeTriggered && r.speaking && r.voicedMs >= nextCheck) {
                        nextCheck = r.voicedMs + 300
                        val audio = detector.snapshot()
                        if (audio != null && confirmBarge(audio) && !interrupted) {
                            bargeTriggered = true
                            interrupt()
                            update { it.copy(phase = CallPhase.Hearing) }
                        }
                    }
                    if (r.utterance != null || r.timedOut) {
                        if (bargeTriggered && r.utterance != null) { pendingBarge = r.utterance; return@launch }
                        detector.close()
                        detector = newDetector(strict = true)
                        nextCheck = BargeMs
                    }
                }
            } finally {
                detector.close()
            }
        }
    }

    /** 这一轮结束：已经被打断的就等用户把话说完，否则不用再听了。 */
    protected suspend fun finishBargeWatch(watcher: Job?) {
        if (watcher == null) return
        if (bargeTriggered) watcher.join() else watcher.cancel()
    }

    /** 用手机自带的识别听一句话。 */
    protected suspend fun listenSystem(): String {
        interrupted = false
        update { it.copy(phase = CallPhase.Listening, partial = "") }
        return suspendCancellableCoroutine { continuation ->
            val cancel = platform.listenOnce({ partial -> update { it.copy(partial = partial, phase = CallPhase.Hearing) } }) { text, error ->
                if (continuation.isActive) {
                    if (text != null) continuation.resume(text) else continuation.resumeWithException(CallError(error ?: "手机自带的语音识别出错了"))
                }
            }
            if (cancel == null) continuation.resumeWithException(CallError("这台手机没有可用的语音识别服务"))
            continuation.invokeOnCancellation { cancel?.invoke() }
        }
    }

    /** 播放一段音频，播完（或被打断）返回。 */
    protected suspend fun playClip(bytes: ByteArray, mime: String): Boolean {
        val done = CompletableDeferred<Boolean>()
        playback = done
        try {
            if (!platform.playAudio(bytes, mime) { done.complete(true) }) return false
            return done.await()
        } finally {
            platform.stopAudio()
            playback = null
            playedAt = TimeSource.Monotonic.markNow()
        }
    }

    /** 用手机自带的语音合成念一句。 */
    protected suspend fun speakSystem(text: String): Boolean {
        val done = CompletableDeferred<Boolean>()
        playback = done
        try {
            if (!platform.speakText(text) { ok -> done.complete(ok) }) return false
            return done.await()
        } finally {
            platform.stopSpeaking()
            playback = null
            playedAt = TimeSource.Monotonic.markNow()
        }
    }

    protected fun heardSomething() {
        update { it.copy(notice = null) }
    }

    protected suspend fun <T> awaitState(flow: StateFlow<T>, timeoutMs: Long, predicate: (T) -> Boolean): T? =
        withTimeoutOrNull(timeoutMs) { flow.first(predicate) }

    private companion object {
        /** 连续出声这么久才算打断。 */
        const val BargeMs = 400
        /** 喇叭停了以后还有这么久可能录到它的尾音（和电脑端一样）。 */
        val EchoTail = 500.milliseconds
    }
}

/** 这个引擎现在用不了（没装模型、没开通、没联网……）；通话里会换下一个。[cause] 是底下真正的错，排查用。 */
class EngineUnavailable(message: String, cause: Throwable? = null) : Exception(message, cause)

/** 把一句话的录音（16kHz、单声道、16 位小端 PCM）识别成文字。 */
interface RecognizerEngine {
    val name: String
    /** 在手机本地跑（快、不花钱、不联网），可以拿来判断一句话说完没有。 */
    val local: Boolean get() = false
    suspend fun recognize(pcm: ByteArray): String
}

/** 把一句话合成语音（MP3 之类手机能直接播放的音频）。 */
interface SynthesizerEngine {
    val name: String
    suspend fun synthesize(text: String): ByteArray
}

/** 云端小伴 AI 网关的语音识别。 */
class CloudRecognizer(private val api: AiGatewayApi) : RecognizerEngine {
    override val name = "云端识别"
    override suspend fun recognize(pcm: ByteArray): String = try {
        api.transcribe(pcm16ToWav(pcm))
    } catch (e: ApiException) {
        if (e.unauthorized) throw CallError("请先登录小伴账号")
        if (isEngineGone(e)) throw EngineUnavailable(e.message)
        throw e
    }
}

/** 云端小伴 AI 网关的语音合成。 */
class CloudSynthesizer(private val api: AiGatewayApi) : SynthesizerEngine {
    override val name = "云端合成"
    override suspend fun synthesize(text: String): ByteArray = try {
        api.speak(text)
    } catch (e: ApiException) {
        if (e.unauthorized) throw CallError("请先登录小伴账号")
        if (isEngineGone(e)) throw EngineUnavailable(e.message)
        throw e
    }
}

private fun isEngineGone(e: ApiException) =
    e.code in setOf("transcribe_unavailable", "speech_unavailable", "paid_required") || e.status == 503 || e.status == 404

/** 通话里“把话交给谁想”：云端小伴内核，或者电脑上的小伴。 */
interface CallAssistant {
    /** 现在不能用的原因（比如设备不在线）；能用返回 null。 */
    fun unavailableReason(): String?
    /** 把你说的话交给它。 */
    fun ask(text: String)
    fun askWithNote(text: String, note: String) = ask(text)
    fun reply(): AssistantReply
    /** 小伴正等你确认或选择的事；没有就是 null。 */
    fun prompt(): CallPrompt? = null
    /** 回答 [prompt] 的某个选项。 */
    fun answerPrompt(id: String) {}
    /** 让它别说了。 */
    fun cancel()
}

/** 通话时有事需要你处理、又没法在通话里回答时的提示（能在通话里回答的会给出 [CallPrompt]）。 */
const val AttentionNotice = "小伴需要你确认一个操作，请挂断后在对话里处理"

/** 助手此刻的状态：[busy] 还在生成；[attention] 有需要你亲自处理的事（比如确认一个操作）。 */
class AssistantReply(val text: String, val busy: Boolean, val error: String? = null, val attention: String? = null)

/** 云端小伴 AI 对话。 */
class CloudAssistant(private val chat: CloudChatController) : CallAssistant {
    override fun unavailableReason(): String? = null
    override fun ask(text: String) = chat.send(text)
    override fun askWithNote(text: String, note: String) = chat.sendVoice(text, note)
    override fun cancel() { if (chat.state.value.sending) chat.stop() }
    override fun reply(): AssistantReply {
        val state = chat.state.value
        val turn = state.messages.lastOrNull { it.role == "assistant" }
        return AssistantReply(
            turn?.text.orEmpty(), busy = state.sending || turn?.streaming == true, error = turn?.error,
            attention = null,
        )
    }

    override fun prompt(): CallPrompt? {
        chat.state.value.question?.let { question ->
            return CallPrompt(
                title = question.question, detail = if (question.options.isEmpty()) "可在对话页输入回答，或先跳过这个问题。" else "", danger = null, question = true,
                options = question.options.mapIndexed { index, label -> CallPromptOption("answer:${question.id}:$index", label) } +
                    CallPromptOption("dismiss:${question.id}", "跳过", CallPromptOption.Role.Deny),
            )
        }
        val approval = chat.state.value.approval ?: return null
        return CallPrompt(
            title = "小伴想要${toolLabel(approval.tool)}", detail = approval.detail, danger = approval.danger, question = false,
            options = buildList {
                add(CallPromptOption("once", "允许一次", CallPromptOption.Role.Allow))
                if (approval.danger == null) add(CallPromptOption("always", "本对话都允许"))
                add(CallPromptOption("deny", "拒绝", CallPromptOption.Role.Deny))
            },
        )
    }

    override fun answerPrompt(id: String) {
        val question = chat.state.value.question
        if (question != null) {
            if (id == "dismiss:${question.id}") chat.answerQuestion(null)
            else if (id.startsWith("answer:${question.id}:")) {
                val index = id.removePrefix("answer:${question.id}:").toIntOrNull() ?: return
                question.options.getOrNull(index)?.let(chat::answerQuestion)
            }
        } else if (id in setOf("once", "always", "deny")) chat.answerApproval(id)
    }
}

/**
 * 一通“语音对话”：听你说完一句，识别成文字，交给小伴想，把回答一句一句念出来，然后接着听。
 * 识别和合成各有一串引擎，按顺序用，哪个用不了就换下一个；都不行了才用手机自带的。
 */
class SpokenCall(
    scope: CoroutineScope,
    platform: VoicePlatform,
    private val assistant: CallAssistant,
    recognizers: List<RecognizerEngine>,
    synthesizers: List<SynthesizerEngine>,
    private val turnJudge: (suspend (ByteArray) -> Boolean?)? = null,
) : VoiceCall(scope, platform) {
    private val recognizers = recognizers.toMutableList()
    /** 几句话同时在合成，换引擎要一次改完。 */
    private val synthesizers = MutableStateFlow(synthesizers)
    /** 每个合成引擎连着失败了几句；合成成一句就清零。 */
    private val synthFailures = MutableStateFlow<Map<SynthesizerEngine, Int>>(emptyMap())
    private var spokenText = ""
    private var interruptionNote = ""
    private var lastAck: String? = null
    private val ackClips = MutableStateFlow<Map<String, ByteArray>>(emptyMap())

    override fun interrupt() {
        if (!interrupted) interruptionNote = voiceInterruptionNote(spokenText)
        super.interrupt()
        assistant.cancel()
    }

    override fun answerPrompt(id: String) {
        assistant.answerPrompt(id)
        update { it.copy(prompt = null) }
    }

    override suspend fun confirmBarge(audio: ByteArray): Boolean {
        val local = recognizers.firstOrNull { it.local } ?: return true
        val text = try { local.recognize(audio).trim() } catch (e: CancellationException) { throw e } catch (_: Exception) { return false }
        return !isVoiceNoise(text) && !isVoiceEcho(text, spokenText)
    }

    override suspend fun run() {
        assistant.unavailableReason()?.let { throw CallError(it) }
        // 只预热免费的 Edge 朗读，失败不影响正常通话。
        val warm = (synthesizers.value.firstOrNull() as? EdgeTtsSynthesizer)?.let { engine ->
            scope.launch {
                for (phrase in AllVoiceAckPhrases) {
                    try {
                        val clip = withTimeoutOrNull(3_000) { engine.synthesize(phrase) } ?: break
                        ackClips.update { it + (phrase to clip) }
                    } catch (e: CancellationException) { throw e } catch (_: Exception) { break }
                }
            }
        }
        try {
            while (true) {
                val text = hear() ?: continue
                if (isVoiceNoise(text)) continue
                // 小伴说话时或刚说完听到的、和它刚念的一样：是它自己的声音从喇叭漏进了麦克风，不是你说的（和电脑端一样）。
                if (heardOverPlayback && isVoiceEcho(text, spokenText)) continue
                heardSomething()
                if (isVoiceHangUp(text)) {
                    sayGoodbye(text)
                    return
                }
                update { it.copy(userText = text, replyText = "", reading = null, partial = "") }
                phase(CallPhase.Thinking)
                answer(text)
            }
        } finally {
            warm?.cancel()
        }
    }

    /**
     * 合成一句：按顺序试每个引擎，第一个合成出来的算数；都不行返回 null，这一句改用手机自带的朗读。
     * 下一句还是先试原来的引擎（和电脑端一样）：一次没连上，不该让后面整通电话都换了声音。
     * 一个引擎连着 [DropAfter] 句都失败，才在这通电话里不再用它，免得每句都白等它超时。
     */
    private suspend fun synthesize(text: String): ByteArray? {
        for (engine in synthesizers.value) {
            try {
                val audio = engine.synthesize(text)
                synthFailures.update { it - engine }
                return audio
            } catch (e: CancellationException) {
                throw e
            } catch (e: CallError) {
                throw e
            } catch (e: Exception) {
                val count = synthFailures.updateAndGet { it + (engine to (it[engine] ?: 0) + 1) }[engine] ?: 0
                val detail = e.cause?.let { "（${it::class.simpleName}: ${it.message}）" }.orEmpty()
                println("小伴语音：${engine.name}这句没合成出来，改用手机自带的朗读；连续第 $count 句：${e.message}$detail")
                if (count >= DropAfter) synthesizers.update { list -> list - engine }
            }
        }
        return null
    }

    private companion object {
        /** 比同时在合成的句子（3 句）多一句：一次网络抖动让手上的几句一起失败，不至于就换掉声音。 */
        const val DropAfter = 4
    }

    /** 用户说了“挂断”：不交给小伴，说声再见（预先合成好了、也没关朗读才说）就结束通话。 */
    private suspend fun sayGoodbye(text: String) {
        update { it.copy(userText = text, replyText = "", reading = null, partial = "") }
        if (state.value.silent) return
        val phrase = VoiceAckPhrases.getValue(VoiceAckKind.Bye).firstOrNull { it in ackClips.value } ?: return
        update { it.copy(replyText = phrase) }
        phase(CallPhase.Speaking)
        playClip(ackClips.value.getValue(phrase), "audio/mpeg")
    }

    /**
     * “说完了吗”：停顿时把到目前为止的话在本地识别一下，句子以句号、问号、感叹号结尾就算说完；
     * 停在“我想要……”这样的半截话上就多等一会儿（电脑端用 Smart Turn 模型判断，手机上用识别结果的标点）。
     */
    private suspend fun finished(audio: ByteArray): Boolean {
        turnJudge?.invoke(audio)?.let { return it }
        val engine = recognizers.firstOrNull { it.local } ?: return false
        val text = try { engine.recognize(audio).trim() } catch (e: CancellationException) { throw e } catch (e: Exception) { return false }
        return text.isNotEmpty() && text.last() in "。！？!?…"
    }

    /** 听一句话，转成文字；没听清返回 null。 */
    private suspend fun hear(): String? {
        if (recognizers.isEmpty()) return listenSystem().trim().ifEmpty { null }
        val local = recognizers.firstOrNull { it.local }
        val pcm = listenUtterance(
            if (local != null || turnJudge != null) ::finished else null,
            local?.let { engine -> { audio -> engine.recognize(audio).trim() } },
            // 没有 Smart Turn 时判停本身就是本地识别，不用再多识别一遍。
            speculate = turnJudge != null,
        ) ?: return null
        phase(CallPhase.Thinking)
        update { it.copy(partial = "") }
        while (recognizers.isNotEmpty()) {
            val engine = recognizers.first()
            try {
                val early = if (engine.local) cachedRecognition ?: speculativeRecognition?.await() else null
                speculativeRecognition = null
                val recognized = early ?: engine.recognize(pcm)
                return recognized.trim().ifEmpty { null }
            } catch (e: EngineUnavailable) {
                recognizers.remove(engine)
                val next = recognizers.firstOrNull()?.name ?: "手机自带的识别"
                update { it.copy(notice = "${engine.name}暂时用不了（${e.message}），改用$next，请再说一次") }
                return null
            } catch (e: CancellationException) {
                throw e
            } catch (e: CallError) {
                throw e
            } catch (e: Exception) {
                update { it.copy(notice = e.message ?: "没有识别出来") }
                return null
            }
        }
        return null
    }

    private suspend fun answer(text: String) = coroutineScope {
        interrupted = false
        spokenText = ""
        // 先开口说什么要在发出去之前定下来，好告诉模型别再说一遍；声音没准备好就不说，也不告诉模型。
        val ackKind = voiceAckKind(text)?.takeUnless { state.value.silent }
        val ackPhrase = ackKind?.let { kind ->
            val ready = ackClips.value
            chooseVoiceAck(VoiceAckPhrases.getValue(kind).filter { it in ready }, lastAck)
        }
        val ackAudio = ackPhrase?.let { ackClips.value[it] }
        val ack = if (ackKind != null && ackPhrase != null) ackKind to ackPhrase else null
        if (ackPhrase != null) lastAck = ackPhrase
        val sentences = Channel<SpeechSpan>(Channel.UNLIMITED)
        // 和电脑端一样，正在念的这句后面再提前合成两句：通道里放一句，合成这边手里再拿一句。
        val clips = Channel<Pair<SpeechSpan, Deferred<ByteArray?>>>(1)
        val synth = launch {
            try {
                for (sentence in sentences) {
                    // 关了朗读的就不合成了；念到一半重新打开，后面的句子接着念。
                    if (interrupted || state.value.silent) continue
                    clips.send(sentence to async { synthesize(sentence.text) })
                }
            } finally {
                // 合成这边无论怎么结束都要告诉播放那边，否则这一轮永远等不到头，通话既不念也不听。
                clips.close()
            }
        }
        val watcher = watchForBarge()
        var ackStarted = false
        val note = interruptionNote + if (ack != null && ackAudio != null) voiceSpokenNote(ack.second) else ""
        assistant.askWithNote(text, note)
        interruptionNote = ""
        val ackJob = if (ack != null && ackAudio != null) launch {
            val wait = voiceAckDelayMs(ack.first)
            if (wait > 0) delay(wait)
            if (!interrupted && !state.value.silent && assistant.reply().busy && assistant.reply().text.isEmpty()) {
                ackStarted = true
                spokenText = ack.second
                phase(CallPhase.Speaking)
                playClip(ackAudio, "audio/mpeg")
            }
        } else null
        val player = launch {
            ackJob?.join()
            for ((sentence, pending) in clips) {
                if (interrupted || state.value.silent) { pending.cancel(); continue }
                // 合成那边自己放弃（超时之类）也是“取消”：只要这一轮还在，就当这句没合成出来，接着往下念。
                val audio = try { pending.await() } catch (e: CancellationException) { ensureActive(); null }
                if (interrupted || state.value.silent) continue
                update { it.copy(reading = sentence.start until sentence.end) }
                phase(CallPhase.Speaking)
                spokenText += sentence.text
                val ok = if (audio != null) playClip(audio, "audio/mpeg") else false
                if (!ok && !interrupted) speakSystem(sentence.text)
            }
            update { it.copy(reading = null) }
        }
        try {
            val splitter = SpeechSplitter()
            var spoken = 0
            while (true) {
                val reply = assistant.reply()
                if (reply.text.length > spoken) {
                    if (!ackStarted) ackJob?.cancel()
                    for (s in splitter.feedSpans(reply.text.substring(spoken))) sentences.send(s)
                    spoken = reply.text.length
                }
                update { it.copy(replyText = reply.text, prompt = assistant.prompt(), notice = reply.attention ?: it.notice?.takeIf { n -> n != AttentionNotice }) }
                // 不念的时候没有“开始念”这一步，回答一出来就显示。
                if (state.value.silent && reply.text.isNotBlank() && state.value.phase == CallPhase.Thinking) phase(CallPhase.Speaking)
                if (!reply.busy) {
                    update { it.copy(prompt = null) }
                    reply.error?.let { error -> update { it.copy(notice = error) } }
                    break
                }
                delay(80)
            }
            for (s in splitter.flushSpans()) sentences.send(s)
        } finally {
            sentences.close()
        }
        synth.join()
        player.join()
        finishBargeWatch(watcher)
    }
}
