package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import kotlin.coroutines.resume
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import xyz.xiangban.xiaoban.common.community.EdgeTtsSynthesizer
import xyz.xiangban.xiaoban.common.community.SpeechRates
import xyz.xiangban.xiaoban.common.community.SpeechSettings
import xyz.xiangban.xiaoban.common.community.SpeechSource
import xyz.xiangban.xiaoban.common.community.SpeechSplitter
import xyz.xiangban.xiaoban.common.community.SynthesizerEngine
import xyz.xiangban.xiaoban.common.community.speechText

/**
 * 声音设置的当前值。放在全局的可观察状态里，设置页改了，聊天里的朗读、通话马上按新的来；
 * 启动时由 [load] 从偏好里读回，之后每次修改都会存下来。
 */
object SpeechState {
    var settings by mutableStateOf(SpeechSettings())
        private set
    private var prefs: AppPrefs? = null

    fun load(prefs: AppPrefs) {
        this.prefs = prefs
        settings = SpeechSettings.decode { prefs.get(it) }
    }

    fun update(change: (SpeechSettings) -> SpeechSettings) {
        val next = change(settings)
        settings = next
        prefs?.let { store -> next.encode().forEach { (key, value) -> store.put(key, value) } }
    }
}

/** 通话用的合成引擎：Edge 在线音色（按设置的音色和语速）；选了系统朗读就没有，通话会改用手机自带的朗读。 */
fun CommunityEnv.synthesizers(): List<SynthesizerEngine> {
    val s = SpeechState.settings
    return if (s.source == SpeechSource.Edge) listOf(EdgeTtsSynthesizer(client, s.voice, SpeechRates.percent(s.rateStep))) else emptyList()
}

/**
 * 把回复念出来：一句一句排队，按声音设置选 Edge 在线音色或系统朗读；Edge 连不上就改用系统朗读。
 * [speak] 念一整段（先停掉正在念的），[enqueue] 接着往后排（自动朗读边生成边念）。
 */
class ReplyReader(private val env: CommunityEnv) {
    var speaking by mutableStateOf(false)
        private set

    private class Item(val text: String, val voice: String?)

    private val pending = ArrayDeque<Item>()
    private var runner: Job? = null

    /** 念这一段，先停掉正在念的。[voice] 不为空时用这个音色（试听音色用）。 */
    fun speak(text: String, voice: String? = null) {
        stop()
        enqueue(text, voice)
    }

    fun enqueue(text: String, voice: String? = null) {
        val clean = speechText(text)
        if (clean.isEmpty() || env.platform == null) return
        pending.addLast(Item(clean, voice))
        speaking = true
        if (runner == null) {
            lateinit var job: Job
            job = env.scope.launch {
                try {
                    while (true) {
                        val next = pending.removeFirstOrNull() ?: break
                        playOne(next)
                    }
                } finally {
                    if (runner === job) { runner = null; speaking = pending.isNotEmpty() }
                }
            }
            runner = job
        }
    }

    fun stop() {
        pending.clear()
        runner?.cancel()
        runner = null
        speaking = false
        env.platform?.stopAudio()
        env.platform?.stopSpeaking()
    }

    private suspend fun playOne(item: Item) {
        val platform = env.platform ?: return
        val s = SpeechState.settings
        if (s.source == SpeechSource.Edge) {
            val audio = try {
                EdgeTtsSynthesizer(env.client, item.voice ?: s.voice, SpeechRates.percent(s.rateStep)).synthesize(item.text)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                null
            }
            if (audio != null) {
                suspendCancellableCoroutine { cont ->
                    val started = platform.playAudio(audio, "audio/mpeg") { if (cont.isActive) cont.resume(Unit) }
                    if (!started && cont.isActive) cont.resume(Unit)
                    cont.invokeOnCancellation { platform.stopAudio() }
                }
                return
            }
        }
        suspendCancellableCoroutine { cont ->
            val started = platform.speakReply(item.text) { if (cont.isActive) cont.resume(Unit) }
            if (!started && cont.isActive) cont.resume(Unit)
            cont.invokeOnCancellation { platform.stopSpeaking() }
        }
    }
}

val LocalReplyReader = compositionLocalOf<ReplyReader?> { null }

private class AutoReadTrack {
    var turn: Any? = null
    var fed = 0
    var splitter = SpeechSplitter()
    var finished = false
}

/**
 * 自动朗读回复：开着的时候，新的一条回复一边生成一边念（一句一句）。打开聊天时已经有的历史回复不念；
 * 你发了新消息，或者离开这个页面，会停止朗读。传入最后一条消息的编号、角色、文字和是否还在生成。
 * [paused]：正在实时对话，回答由通话自己念，这里什么都不做，也不能去停声音（会把通话正在播的那句掐掉）；通话里的回答之后也不补念。
 */
@Composable
fun AutoRead(reader: ReplyReader, turnId: Any?, role: String?, text: String, streaming: Boolean, paused: Boolean = false) {
    val track = remember(reader) { AutoReadTrack() }
    val inCall by rememberUpdatedState(paused)
    DisposableEffect(reader) { onDispose { if (!inCall) reader.stop() } }
    LaunchedEffect(turnId, role, text, streaming, paused) {
        when {
            paused -> { track.turn = turnId; track.finished = true }
            role == "user" -> { reader.stop(); track.turn = null }
            role != "assistant" -> Unit
            streaming && track.turn != turnId -> {
                reader.stop()
                track.turn = turnId; track.fed = 0; track.splitter = SpeechSplitter(); track.finished = false
                feed(reader, track, text)
            }
            track.turn == turnId && !track.finished -> {
                feed(reader, track, text)
                if (!streaming) { track.splitter.flush().forEach { reader.enqueue(it) }; track.finished = true }
            }
        }
    }
}

private fun feed(reader: ReplyReader, track: AutoReadTrack, text: String) {
    if (!SpeechState.settings.autoRead) { track.fed = text.length; return }
    if (text.length > track.fed) {
        track.splitter.feed(text.substring(track.fed)).forEach { reader.enqueue(it) }
        track.fed = text.length
    }
}
