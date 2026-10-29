package xyz.xiangban.xiaoban.common.community

import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.exp
import kotlin.math.ln
import kotlin.math.log10
import kotlin.math.sin
import kotlin.math.sqrt

/** Whisper log-mel input used by the same Smart Turn v3 model as the desktop. */
object SmartTurnFeatures {
    const val Samples = 128_000
    const val Mels = 80
    const val Frames = 800
    private const val Fft = 400
    private const val Hop = 160
    private const val Bins = 201

    private fun hzToMel(hz: Double) = if (hz >= 1000.0) 15.0 + ln(hz / 1000.0) * (27.0 / ln(6.4)) else 3.0 * hz / 200.0
    private fun melToHz(mel: Double) = if (mel >= 15.0) 1000.0 * exp(ln(6.4) / 27.0 * (mel - 15.0)) else 200.0 * mel / 3.0
    private val window = DoubleArray(Fft) { 0.5 - 0.5 * cos(2.0 * PI * it / Fft) }
    private val cosines = Array(Bins) { b -> DoubleArray(Fft) { i -> cos(2.0 * PI * b * i / Fft) } }
    private val sines = Array(Bins) { b -> DoubleArray(Fft) { i -> sin(2.0 * PI * b * i / Fft) } }
    private val filters = run {
        val low = hzToMel(0.0); val high = hzToMel(8000.0)
        val edges = DoubleArray(Mels + 2) { melToHz(low + (high - low) * it / (Mels + 1)) }
        Array(Mels) { m -> DoubleArray(Bins) { b ->
            val freq = 8000.0 * b / (Bins - 1)
            val down = (freq - edges[m]) / (edges[m + 1] - edges[m])
            val up = (edges[m + 2] - freq) / (edges[m + 2] - edges[m + 1])
            minOf(down, up).coerceAtLeast(0.0) * 2.0 / (edges[m + 2] - edges[m])
        } }
    }

    /** Latest eight seconds, padded at the beginning; PCM is 16 kHz signed little-endian. */
    fun fromPcm(pcm: ByteArray): FloatArray {
        val samples = DoubleArray(Samples)
        val count = minOf(pcm.size / 2, Samples)
        val start = pcm.size / 2 - count
        for (i in 0 until count) {
            val at = (start + i) * 2
            val value = ((pcm[at + 1].toInt() shl 8) or (pcm[at].toInt() and 0xff)).toShort()
            samples[Samples - count + i] = value / 32768.0
        }
        val mean = samples.sum() / Samples
        var variance = 0.0
        for (s in samples) variance += (s - mean) * (s - mean)
        val scale = 1.0 / sqrt(variance / Samples + 1e-7)
        for (i in samples.indices) samples[i] = (samples[i] - mean) * scale
        val padded = DoubleArray(Samples + Fft)
        samples.copyInto(padded, Fft / 2)
        for (i in 0 until Fft / 2) {
            padded[Fft / 2 - 1 - i] = samples[i + 1]
            padded[Fft / 2 + Samples + i] = samples[Samples - 2 - i]
        }
        val features = DoubleArray(Mels * Frames)
        val power = DoubleArray(Bins)
        val weighted = DoubleArray(Fft)
        var cachedValue = Double.NaN
        val cached = DoubleArray(Bins)
        var maximum = Double.NEGATIVE_INFINITY
        for (f in 0 until Frames) {
            val offset = f * Hop
            val first = padded[offset]
            var constant = true
            for (i in 0 until Fft) {
                val v = padded[offset + i]
                if (v != first) constant = false
                weighted[i] = v * window[i]
            }
            if (constant && first == cachedValue) cached.copyInto(power) else {
                for (b in 0 until Bins) {
                    var real = 0.0; var imaginary = 0.0
                    val c = cosines[b]; val s = sines[b]
                    for (i in 0 until Fft) { real += weighted[i] * c[i]; imaginary -= weighted[i] * s[i] }
                    power[b] = real * real + imaginary * imaginary
                }
                if (constant) { power.copyInto(cached); cachedValue = first }
            }
            for (m in 0 until Mels) {
                var sum = 0.0
                val filter = filters[m]
                for (b in 0 until Bins) sum += filter[b] * power[b]
                val value = log10(sum.coerceAtLeast(1e-10))
                features[m * Frames + f] = value
                maximum = maxOf(maximum, value)
            }
        }
        return FloatArray(features.size) { ((features[it].coerceAtLeast(maximum - 8.0) + 4.0) / 4.0).toFloat() }
    }
}
