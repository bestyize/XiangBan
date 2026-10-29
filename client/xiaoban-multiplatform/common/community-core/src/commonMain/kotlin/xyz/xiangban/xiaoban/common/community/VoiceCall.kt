package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withTimeoutOrNull
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

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
    /** 要让你知道的事：出错原因、需要你去确认操作、好久没人说话。 */
    val notice: String? = null,
    val ended: Boolean = false,
    /** 小伴说话时，你直接开口是否能打断它（有回声的环境可以关掉）。 */
    val bargeIn: Boolean = true,
)

/** 这次通话没法继续，原因要告诉用户。 */
class CallError(message: String) : Exception(message)

/**
 * 一通“语音对话”：听你说完一句，交给小伴想，再把回答念出来，然后接着听。可以随时 [interrupt] 打断它念，[stop] 挂断。
 * 子类决定一句话怎么变成回答、怎么念出来。
 */
abstract class VoiceCall(protected val scope: CoroutineScope, protected val platform: VoicePlatform) {
    private val bargeSafe = platform.bargeInSafe()
    private val _state = MutableStateFlow(
        CallState(
            bargeIn = bargeSafe,
            notice = if (bargeSafe) null else "这台手机的回声消除不可用，已关闭“直接开口打断”，请用“打断”按钮；戴耳机时可以在下面打开",
        ),
    )
    val state: StateFlow<CallState> = _state.asStateFlow()
    private var job: Job? = null
    private var playback: CompletableDeferred<Boolean>? = null
    private var idleRounds = 0
    private val mic = Channel<ByteArray>(Channel.UNLIMITED)
    private var stopMic: (() -> Unit)? = null
    @Volatile private var pendingBarge: ByteArray? = null
    @Volatile private var bargeTriggered = false

    @Volatile protected var interrupted = false

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

    /** 打开/关闭“小伴说话时直接开口打断”。 */
    fun setBargeIn(on: Boolean) = update { it.copy(bargeIn = on) }

    protected abstract suspend fun run()

    protected fun update(change: (CallState) -> CallState) {
        _state.update(change)
    }

    protected fun phase(phase: CallPhase) = update { it.copy(phase = phase, level = if (phase == CallPhase.Listening || phase == CallPhase.Hearing) it.level else 0f) }

    /** 麦克风在整通电话里一直开着（这样小伴说话时也能听见你开口）；第一次需要时才打开。 */
    private suspend fun openMic() {
        if (stopMic != null) return
        stopMic = platform.streamRecording { mic.trySend(it) } ?: throw CallError("没有麦克风权限，没法通话")
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
    protected suspend fun listenUtterance(judge: (suspend (ByteArray) -> Boolean)? = null): ByteArray? {
        pendingBarge?.let { pendingBarge = null; interrupted = false; return it }
        interrupted = false
        openMic()
        drainMic()
        update { it.copy(phase = CallPhase.Listening, partial = "") }
        val detector = newDetector(strict = false, useJudge = judge != null)
        val verdicts = Channel<Pair<Int, Boolean>>(Channel.UNLIMITED)
        try {
            var counter = 0
            for (frame in mic) {
                while (true) { val v = verdicts.tryReceive().getOrNull() ?: break; detector.verdict(v.first, v.second) }
                val result = detector.feed(frame)
                // 声音大小不用每 20ms 都刷新界面。
                if (++counter % 5 == 0) update { it.copy(level = result.level, phase = if (result.speaking) CallPhase.Hearing else CallPhase.Listening) }
                if (result.ask != null && judge != null) {
                    val ticket = result.askTicket
                    val audio = result.ask
                    scope.launch { verdicts.trySend(ticket to (try { judge(audio) } catch (e: CancellationException) { throw e } catch (e: Exception) { false })) }
                }
                if (result.utterance != null) return result.utterance
                if (result.timedOut) return null
            }
            return null
        } finally {
            detector.close()
            verdicts.close()
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
            try {
                for (frame in mic) {
                    val r = detector.feed(frame)
                    if (!bargeTriggered && r.speaking && r.voicedMs >= BargeMs) {
                        bargeTriggered = true
                        interrupt()
                        update { it.copy(phase = CallPhase.Hearing) }
                    }
                    if (r.utterance != null || r.timedOut) {
                        if (bargeTriggered && r.utterance != null) { pendingBarge = r.utterance; return@launch }
                        detector.close()
                        detector = newDetector(strict = true)
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
        }
    }

    /** 这一轮没听到话；连续几轮没人说话就结束通话。返回是否应该结束。 */
    protected fun noSpeech(): Boolean {
        idleRounds++
        if (idleRounds >= MaxIdleRounds) {
            update { it.copy(notice = "好一会儿没听到声音，已结束通话") }
            return true
        }
        return false
    }

    protected fun heardSomething() {
        idleRounds = 0
        update { it.copy(notice = null) }
    }

    protected suspend fun <T> awaitState(flow: StateFlow<T>, timeoutMs: Long, predicate: (T) -> Boolean): T? =
        withTimeoutOrNull(timeoutMs) { flow.first(predicate) }

    private companion object {
        const val MaxIdleRounds = 3
        /** 连续出声这么久才算打断。 */
        const val BargeMs = 400
    }
}

/** 这个引擎现在用不了（没装模型、没开通、没联网……）；通话里会换下一个。 */
class EngineUnavailable(message: String) : Exception(message)

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
    fun reply(): AssistantReply
    /** 让它别说了。 */
    fun cancel()
}

/** 通话时小伴要你确认操作的提示（确认只能回到对话里做）。 */
const val AttentionNotice = "小伴需要你确认一个操作，请挂断后在对话里处理"

/** 助手此刻的状态：[busy] 还在生成；[attention] 有需要你亲自处理的事（比如确认一个操作）。 */
class AssistantReply(val text: String, val busy: Boolean, val error: String? = null, val attention: String? = null)

/** 云端小伴 AI 对话。 */
class CloudAssistant(private val chat: CloudChatController) : CallAssistant {
    override fun unavailableReason(): String? = null
    override fun ask(text: String) = chat.send(text)
    override fun cancel() { if (chat.state.value.sending) chat.stop() }
    override fun reply(): AssistantReply {
        val state = chat.state.value
        val turn = state.messages.lastOrNull { it.role == "assistant" }
        return AssistantReply(
            turn?.text.orEmpty(), busy = state.sending || turn?.streaming == true, error = turn?.error,
            attention = if (state.approval != null) AttentionNotice else null,
        )
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
) : VoiceCall(scope, platform) {
    private val recognizers = recognizers.toMutableList()
    private val synthesizers = synthesizers.toMutableList()

    override fun interrupt() {
        super.interrupt()
        assistant.cancel()
    }

    override suspend fun run() {
        assistant.unavailableReason()?.let { throw CallError(it) }
        while (true) {
            val text = hear() ?: if (noSpeech()) return else continue
            heardSomething()
            update { it.copy(userText = text, replyText = "", partial = "") }
            phase(CallPhase.Thinking)
            answer(text)
        }
    }

    /**
     * “说完了吗”：停顿时把到目前为止的话在本地识别一下，句子以句号、问号、感叹号结尾就算说完；
     * 停在“我想要……”这样的半截话上就多等一会儿（电脑端用 Smart Turn 模型判断，手机上用识别结果的标点）。
     */
    private suspend fun finished(audio: ByteArray): Boolean {
        val engine = recognizers.firstOrNull { it.local } ?: return false
        val text = try { engine.recognize(audio).trim() } catch (e: CancellationException) { throw e } catch (e: Exception) { return false }
        return text.isNotEmpty() && text.last() in "。！？!?…"
    }

    /** 听一句话，转成文字；没听清返回 null。 */
    private suspend fun hear(): String? {
        if (recognizers.isEmpty()) return listenSystem().trim().ifEmpty { null }
        val pcm = listenUtterance(if (recognizers.any { it.local }) ::finished else null) ?: return null
        phase(CallPhase.Thinking)
        update { it.copy(partial = "") }
        while (recognizers.isNotEmpty()) {
            val engine = recognizers.first()
            try {
                return engine.recognize(pcm).trim().ifEmpty { null }
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
        val sentences = Channel<String>(Channel.UNLIMITED)
        val clips = Channel<Pair<String, ByteArray?>>(2)
        val synth = launch {
            for (sentence in sentences) {
                if (interrupted) continue
                var audio: ByteArray? = null
                while (audio == null && synthesizers.isNotEmpty()) {
                    val engine = synthesizers.first()
                    try {
                        audio = engine.synthesize(sentence)
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: EngineUnavailable) {
                        synthesizers.remove(engine)
                    } catch (e: CallError) {
                        throw e
                    } catch (e: Exception) {
                        break
                    }
                }
                clips.send(sentence to audio)
            }
            clips.close()
        }
        val watcher = watchForBarge()
        val player = launch {
            for ((sentence, audio) in clips) {
                if (interrupted) continue
                phase(CallPhase.Speaking)
                val ok = if (audio != null) playClip(audio, "audio/mpeg") else false
                if (!ok && !interrupted) speakSystem(sentence)
            }
        }
        try {
            assistant.ask(text)
            val splitter = SpeechSplitter()
            var spoken = 0
            while (true) {
                val reply = assistant.reply()
                if (reply.text.length > spoken) {
                    for (s in splitter.feed(reply.text.substring(spoken))) sentences.send(s)
                    spoken = reply.text.length
                }
                update { it.copy(replyText = reply.text, notice = reply.attention ?: it.notice?.takeIf { n -> n != AttentionNotice }) }
                if (!reply.busy) {
                    reply.error?.let { error -> update { it.copy(notice = error) } }
                    break
                }
                delay(80)
            }
            for (s in splitter.flush()) sentences.send(s)
        } finally {
            sentences.close()
        }
        synth.join()
        player.join()
        finishBargeWatch(watcher)
    }
}
