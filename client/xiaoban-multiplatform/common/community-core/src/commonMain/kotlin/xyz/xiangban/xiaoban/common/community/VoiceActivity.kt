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
)

/**
 * 最朴素的“说完了”判断：按声音大小（RMS）分说话和安静，先用开头一小段估出环境噪声，说话要明显高过它；
 * 连续安静一阵（[endSilenceMs]）就算说完。太短的一声响（咳嗽、碰桌子）不算话。
 * 一个检测器只管一句话，说完或超时后换新的。
 */
class EndpointDetector(
    private val frameMs: Int = 20,
    private val minSpeechMs: Int = 300,
    private val endSilenceMs: Int = 900,
    private val maxSpeechMs: Int = 20_000,
    private val maxWaitMs: Int = 10_000,
    private val preRollMs: Int = 300,
) {
    private var floor = -1.0
    private var calibrated = 0
    private var voiced = 0
    private var silentRun = 0
    private var speechFrames = 0
    private var waited = 0
    private var started = false
    private val preRoll = ArrayDeque<ByteArray>()
    private val captured = ArrayList<ByteArray>()

    fun feed(frame: ByteArray): VadResult {
        val rms = rms(frame)
        // 开头 200ms 只用来估环境噪声（那时一般还没开口）。
        if (calibrated < CalibrationFrames) {
            floor = if (floor < 0) rms else (floor * calibrated + rms) / (calibrated + 1)
            calibrated++
            remember(frame)
            return VadResult(level(rms), false)
        }
        val threshold = maxOf(MinThreshold, floor * 3.0)
        val loud = rms > threshold
        if (!started) {
            if (loud) voiced++ else { voiced = 0; floor = floor * 0.95 + rms * 0.05 }
            remember(frame)
            if (voiced >= StartFrames) {
                started = true
                captured.addAll(preRoll)
                preRoll.clear()
                speechFrames = voiced
                silentRun = 0
            } else {
                waited++
                if (waited * frameMs >= maxWaitMs) return VadResult(level(rms), false, timedOut = true)
            }
            return VadResult(level(rms), started)
        }
        captured.add(frame)
        if (loud) { speechFrames++; silentRun = 0 } else silentRun++
        val done = silentRun * frameMs >= endSilenceMs || captured.size * frameMs >= maxSpeechMs
        if (done) {
            if (speechFrames * frameMs < minSpeechMs) {
                // 只是一声响：当作没发生，接着等。
                captured.clear(); started = false; voiced = 0; speechFrames = 0; silentRun = 0
                return VadResult(level(rms), false)
            }
            val all = ByteArray(captured.sumOf { it.size })
            var at = 0
            for (f in captured) { f.copyInto(all, at); at += f.size }
            return VadResult(level(rms), false, utterance = all)
        }
        return VadResult(level(rms), true)
    }

    private fun remember(frame: ByteArray) {
        preRoll.addLast(frame)
        while (preRoll.size * frameMs > preRollMs) preRoll.removeFirst()
    }

    private fun level(rms: Double): Float = (rms / 6000.0).coerceIn(0.0, 1.0).toFloat()

    private fun rms(frame: ByteArray): Double {
        val n = frame.size / 2
        if (n == 0) return 0.0
        var sum = 0.0
        for (i in 0 until n) {
            val s = ((frame[2 * i + 1].toInt() shl 8) or (frame[2 * i].toInt() and 0xFF)).toShort().toInt()
            sum += s.toDouble() * s
        }
        return sqrt(sum / n)
    }

    private companion object {
        const val CalibrationFrames = 10
        const val StartFrames = 3
        const val MinThreshold = 450.0
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
