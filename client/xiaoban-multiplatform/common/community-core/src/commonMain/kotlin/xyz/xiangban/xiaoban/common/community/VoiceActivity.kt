package xyz.xiangban.xiaoban.common.community

import kotlin.math.sqrt

// 语音对话用的几个纯逻辑：判断一句话说完没有、把录音包成 WAV、把边生成边到的回答切成一句一句念。

private const val SampleRate = 16_000

/** 把 16kHz、单声道、16 位小端 PCM 包成 WAV 文件（云端识别接口要带文件头的音频）。 */
fun pcm16ToWav(pcm: ByteArray, sampleRate: Int = SampleRate): ByteArray {
    val out = ByteArray(44 + pcm.size)
    fun put(at: Int, text: String) { text.forEachIndexed { i, c -> out[at + i] = c.code.toByte() } }
    fun int(at: Int, value: Int) { for (i in 0 until 4) out[at + i] = (value shr (8 * i)).toByte() }
    fun short(at: Int, value: Int) { out[at] = value.toByte(); out[at + 1] = (value shr 8).toByte() }
    put(0, "RIFF"); int(4, 36 + pcm.size); put(8, "WAVE"); put(12, "fmt ")
    int(16, 16); short(20, 1); short(22, 1); int(24, sampleRate); int(28, sampleRate * 2); short(32, 2); short(34, 16)
    put(36, "data"); int(40, pcm.size)
    pcm.copyInto(out, 44)
    return out
}

/** 一帧（20ms）里有没有人在说话。不同平台可以换更准的实现（安卓上是 Silero VAD）。 */
interface SpeechGate {
    /** [strict] 为真时只认很明确的人声：小伴自己在说话时，它的回声不能被当成用户开口。 */
    fun voiced(frame: ByteArray, strict: Boolean): Boolean
    /** 声音大小，0 到 1，给界面画呼吸动画。 */
    fun level(frame: ByteArray): Float
    /** 用完了，释放占用的资源。 */
    fun release() {}
}

/** 按声音大小（RMS）判断：先用开头一小段估出环境噪声，说话要明显高过它。没有 VAD 模型时的备选。 */
class EnergyGate : SpeechGate {
    private var floor = -1.0
    private var calibrated = 0

    override fun level(frame: ByteArray): Float = (rms(frame) / 6000.0).coerceIn(0.0, 1.0).toFloat()

    override fun voiced(frame: ByteArray, strict: Boolean): Boolean {
        val value = rms(frame)
        if (calibrated < CalibrationFrames) {
            floor = if (floor < 0) value else (floor * calibrated + value) / (calibrated + 1)
            calibrated++
            return false
        }
        // 用户可能一打开通话就说话；不能把校准时的整段人声当成环境噪声。
        if (calibrated == CalibrationFrames) { floor = floor.coerceAtMost(MinThreshold / 3.0); calibrated++ }
        val threshold = maxOf(MinThreshold, floor * 3.0) * (if (strict) 1.8 else 1.0)
        val loud = value > threshold
        if (!loud) floor = floor * 0.95 + value * 0.05
        return loud
    }

    private fun rms(frame: ByteArray): Double {
        val n = frame.size / 2
        if (n == 0) return 0.0
        var sum = 0.0
        for (i in 0 until n) {
            val sample = ((frame[2 * i + 1].toInt() shl 8) or (frame[2 * i].toInt() and 0xFF)).toShort().toInt()
            sum += sample.toDouble() * sample
        }
        return sqrt(sum / n)
    }

    private companion object {
        const val CalibrationFrames = 10
        const val MinThreshold = 450.0
    }
}

/** 一帧（20ms）喂进去之后的结果。 */
class VadResult(
    /** 当前声音大小，0 到 1，给界面画呼吸动画。 */
    val level: Float,
    /** 此刻是不是在说话。 */
    val speaking: Boolean,
    /** 一句话说完了：这句话的完整录音（含开头一点点余量）；还没说完是 null。 */
    val utterance: ByteArray? = null,
    /** 等了很久一直没人说话。 */
    val timedOut: Boolean = false,
    /** 这句话里已经有多少毫秒的人声。 */
    val voicedMs: Int = 0,
    /** 停顿够长了，该问一句“说完了吗”：到目前为止的录音。回答用 [EndpointDetector.verdict] 交回来。 */
    val ask: ByteArray? = null,
    val askTicket: Int = 0,
)

/**
 * “这句话说完了没有”的判断，思路和电脑端 `voice/turn/segmenter.ts` 一样：
 * 人声门（Silero 或声音大小）分出说话和安静；太短的一声响不算话；说完话后停顿一会儿就算说完。
 * 有“判停”的话（[useJudge]）：停顿 [askAfterMs] 就问一次“说完了吗”，说没说完就再等，隔 [askAgainAfterMs] 再问一次，
 * 无论如何停顿到 [giveUpAfterMs] 就结束；没有判停就固定停顿 [endSilenceMs] 结束。
 * 一个检测器只管一句话，说完或超时后换新的。
 */
class EndpointDetector(
    private val gate: SpeechGate = EnergyGate(),
    private val strict: Boolean = false,
    private val useJudge: Boolean = false,
    private val frameMs: Int = 20,
    private val minSpeechMs: Int = if (strict) 400 else 300,
    private val endSilenceMs: Int = 900,
    private val askAfterMs: Int = 350,
    private val askAgainAfterMs: Int = 900,
    private val giveUpAfterMs: Int = 1_600,
    private val maxSpeechMs: Int = 20_000,
    private val maxWaitMs: Int = 10_000,
    private val preRollMs: Int = 300,
) {
    private var voiced = 0
    private var silentRun = 0
    private var speechFrames = 0
    private var waited = 0
    private var started = false
    private var asked = 0
    private var ticket = 0
    private var verdictComplete = false
    private val preRoll = ArrayDeque<ByteArray>()
    private val captured = ArrayList<ByteArray>()

    /** 这个检测器不用了。 */
    fun close() = gate.release()

    /** 判停的回答。[ticket] 是 [VadResult.askTicket]；中间又开口说话了，旧的回答就作废。 */
    fun verdict(ticket: Int, complete: Boolean) {
        if (ticket == this.ticket && started) verdictComplete = complete
    }

    fun feed(frame: ByteArray): VadResult {
        val level = gate.level(frame)
        val loud = gate.voiced(frame, strict)
        if (!started) {
            if (loud) voiced++ else voiced = 0
            remember(frame)
            if (voiced >= StartFrames) {
                started = true
                captured.addAll(preRoll)
                preRoll.clear()
                speechFrames = voiced
                silentRun = 0
                asked = 0
                ticket++
            } else {
                waited++
                if (waited * frameMs >= maxWaitMs) return VadResult(level, false, timedOut = true)
            }
            return VadResult(level, started, voicedMs = speechFrames * frameMs)
        }
        captured.add(frame)
        if (loud) {
            speechFrames++
            silentRun = 0
            // 又开口了：之前对停顿的判断作废。
            if (asked > 0 || verdictComplete) { asked = 0; verdictComplete = false; ticket++ }
        } else silentRun++
        val silenceMs = silentRun * frameMs
        val tooLong = captured.size * frameMs >= maxSpeechMs
        val done = when {
            tooLong -> true
            useJudge -> verdictComplete && speechFrames * frameMs >= minSpeechMs || silenceMs >= giveUpAfterMs
            else -> silenceMs >= endSilenceMs
        }
        if (done) {
            if (speechFrames * frameMs < minSpeechMs) {
                // 只是一声响：当作没发生，接着等。
                captured.clear(); started = false; voiced = 0; speechFrames = 0; silentRun = 0; asked = 0; verdictComplete = false
                return VadResult(level, false)
            }
            return VadResult(level, false, utterance = join(captured), voicedMs = speechFrames * frameMs)
        }
        var ask: ByteArray? = null
        if (useJudge && !verdictComplete && speechFrames * frameMs >= minSpeechMs) {
            val due = when (asked) { 0 -> askAfterMs; 1 -> askAgainAfterMs; else -> Int.MAX_VALUE }
            if (silenceMs >= due) { asked++; ask = join(captured) }
        }
        return VadResult(level, true, voicedMs = speechFrames * frameMs, ask = ask, askTicket = ticket)
    }

    private fun join(frames: List<ByteArray>): ByteArray {
        val all = ByteArray(frames.sumOf { it.size })
        var at = 0
        for (f in frames) { f.copyInto(all, at); at += f.size }
        return all
    }

    private fun remember(frame: ByteArray) {
        preRoll.addLast(frame)
        while (preRoll.size * frameMs > preRollMs) preRoll.removeFirst()
    }

    private companion object {
        const val StartFrames = 3
    }
}

/**
 * 回答是一小段一小段流出来的；这里把它切成一句一句，方便边生成边念。
 * 代码块不念，Markdown 符号去掉，太碎的句子并到下一句。总共念不超过 [maxChars] 字。
 */
class SpeechSplitter(private val maxChars: Int = 1200) {
    private var pending = StringBuilder()
    private var inCode = false
    private var carry = ""
    private var emitted = 0

    fun feed(delta: String): List<String> {
        pending.append(delta)
        val out = ArrayList<String>()
        while (true) {
            val cut = boundary(pending) ?: break
            val piece = pending.substring(0, cut)
            pending = StringBuilder(pending.substring(cut))
            take(piece, out)
        }
        return out
    }

    /** 回答结束，把没切完的尾巴也吐出来。 */
    fun flush(): List<String> {
        val out = ArrayList<String>()
        if (pending.isNotEmpty()) take(pending.toString(), out)
        pending = StringBuilder()
        if (carry.isNotBlank() && emitted < maxChars) { out += carry; emitted += carry.length }
        carry = ""
        return out
    }

    private fun boundary(text: CharSequence): Int? {
        for (i in text.indices) {
            val c = text[i]
            if (c == '\n' || c in "。！？!?；;…") return i + 1
        }
        if (text.length >= 90) {
            val comma = (text.length - 1 downTo 40).firstOrNull { text[it] in "，、,：:" }
            if (comma != null) return comma + 1
        }
        return null
    }

    private fun take(piece: String, out: MutableList<String>) {
        val line = piece.trim()
        if (line.startsWith("```") || line.startsWith("~~~")) { inCode = !inCode; return }
        if (inCode || emitted >= maxChars) return
        val clean = cleanForSpeech(line)
        if (clean.isBlank()) return
        val joined = if (carry.isEmpty()) clean else "$carry$clean"
        if (joined.length < MinSentence && !piece.endsWith("\n")) { carry = joined; return }
        carry = ""
        out += joined
        emitted += joined.length
    }

    private companion object {
        const val MinSentence = 8
    }
}

private val LinkRe = Regex("""\[([^\]]*)]\([^)]*\)""")

/** 去掉标题、列表、引用、粗体、代码的符号和链接地址，只留念出来的话。 */
internal fun cleanForSpeech(line: String): String {
    var s = line.trim()
    if (s.all { it in "-*_= " }) return ""
    s = s.trimStart('#', '>', ' ')
    s = s.removePrefix("- ").removePrefix("* ").removePrefix("+ ")
    s = LinkRe.replace(s) { it.groupValues[1] }
    return s.replace("*", "").replace("`", "").replace("~", "").trim()
}
