package xyz.xiaban.wallpaper.companion

import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sin

/** Everything needed to draw one moment of the robot. */
class Frame(
    val face: Face,
    val glyphL: Glyph,
    val glyphR: Glyph,
    /** Vertical squash of each eye (blinks, glyph swaps), 1 = normal. */
    val squashL: Double,
    val squashR: Double,
    val symbol: Symbol?,
    /** Pop-in progress of the symbol, 0..1. */
    val symbolP: Double,
    /** Seconds, drives looping symbol and tear animations. */
    val t: Double,
    /** Dash rotation of the waiting spinner, in turns. */
    val dash: Double,
    /** Breathing phase, -1..1. */
    val breath: Double,
)

private const val SUBSTEP = 1.0 / 240

/** Heavier body parts lag a little behind the snappy screen features. */
private val SPRINGS: Array<Spring> = Array(FACE_SIZE) { spring(200.0, 0.8) }.also { table ->
    fun set(from: Int, count: Int, s: Spring) { for (i in from until from + count) table[i] = s }
    set(0, EYE_FIELDS * 2, spring(320.0, 0.72))
    set(MOUTH_AT, 7, spring(360.0, 0.7))
    table[SCALAR_AT + 5] = spring(110.0, 0.62)
    table[SCALAR_AT + 6] = spring(140.0, 0.6)
    table[SCALAR_AT + 7] = spring(170.0, 0.5)
    table[SCALAR_AT + 8] = spring(170.0, 0.5)
}

/** Closing then reopening, p in 0..1 -> squash 1 -> floor -> 1. */
private fun closeOpen(p: Double, floor: Double, mid: Double = 0.42): Double {
    if (p <= 0 || p >= 1) return 1.0
    if (p < mid) {
        val k = p / mid
        return 1 - (1 - floor) * k * k
    }
    val k = (p - mid) / (1 - mid)
    return floor + (1 - floor) * (1 - (1 - k) * (1 - k))
}

/**
 * Turns discrete expression targets into continuous motion: every face parameter follows its target on a spring,
 * glyph changes go through a quick squash, and a life layer adds blinks, glances, breathing, ear twitches,
 * hopping and lip movement on top.
 */
class Animator(
    initial: Expression = Expression.IDLE,
    seed: Int = System.nanoTime().toInt(),
    var reducedMotion: Boolean = false,
) {
    /** Where the robot looks; null lets it glance around on its own. */
    var lookAt: Gaze? = null

    private val rand = Mulberry32(seed)::next
    private var targetFace = initial.targetFace
    private var look = initial.targetLook
    private var cur = initial.targetFace
    private val vel = DoubleArray(FACE_SIZE)
    private var shownL = look.glyphL
    private var shownR = look.glyphR
    private var shownSymbol = look.symbol
    private var symbolP = 1.0
    private var swapL: Double? = null
    private var swapR: Double? = null
    private var t = 0.0
    private var blinkAt = 1 + rand() * 3
    private val life = Life(rand, 4 + rand() * 6)
    private var blinkStart: Double? = null
    private var blinkAgain = false
    private var dashPhase = 0.0
    private var bouncePhase = 0.0

    fun setTarget(expression: Expression, instant: Boolean = false) {
        targetFace = expression.targetFace
        look = expression.targetLook
        if (instant) {
            cur = expression.targetFace
            vel.fill(0.0)
            shownL = look.glyphL
            shownR = look.glyphR
            shownSymbol = look.symbol
            symbolP = 1.0
            swapL = null
            swapR = null
        }
    }

    /** Kicks one ear (or both with 0) as if something brushed against it. */
    fun twitchEar(side: Int = 0, strength: Double = 1.0) = life.kick(side, strength)

    fun step(dtIn: Double): Frame {
        val dt = dtIn.coerceIn(0.0, 1.0 / 15)
        t += dt
        val calm = reducedMotion

        var left = dt
        while (left > 1e-6) {
            val h = min(SUBSTEP, left)
            val c = cur.a
            val g = targetFace.a
            for (i in 0 until FACE_SIZE) {
                val s = SPRINGS[i]
                val v = vel[i] + ((g[i] - c[i]) * s.k - vel[i] * s.c) * h
                vel[i] = v
                c[i] += v * h
            }
            life.step(h, t, look.saccade, lookAt, calm)
            left -= SUBSTEP
        }

        val swapDuration = if (calm) 0.12 else 0.24
        var squashL = 1.0
        var squashR = 1.0
        if (shownL != look.glyphL && swapL == null) swapL = t
        if (shownR != look.glyphR && swapR == null) swapR = t + 0.03
        swapL?.let {
            val p = (t - it) / swapDuration
            if (p >= 0.42) shownL = look.glyphL
            squashL = closeOpen(p, 0.06)
            if (p >= 1) swapL = null
        }
        swapR?.let {
            val p = (t - it) / swapDuration
            if (p >= 0.42) shownR = look.glyphR
            squashR = closeOpen(p, 0.06)
            if (p >= 1) swapR = null
        }

        if (shownSymbol != look.symbol) {
            symbolP -= dt * 8
            if (symbolP <= 0 || shownSymbol == null) {
                symbolP = 0.0
                shownSymbol = look.symbol
            }
        } else if (shownSymbol != null) {
            symbolP = min(1.0, symbolP + dt * 3.2)
        }

        var blinkL = 1.0
        var blinkR = 1.0
        if (!calm && look.blink && t >= blinkAt && blinkStart == null) {
            blinkStart = t
            blinkAgain = rand() < 0.2
        }
        blinkStart?.let {
            val p = (t - it) / 0.17
            blinkL = closeOpen(p, 0.08)
            blinkR = closeOpen(p - 0.06, 0.08)
            if (p >= 1.1) {
                if (blinkAgain) {
                    blinkAgain = false
                    blinkStart = t + 0.06
                } else {
                    blinkStart = null
                    blinkAt = t + 2.2 + rand() * 4.2
                }
            }
        }

        val face = cur.copy()
        val breath = if (calm) 0.0 else sin(t / 4.2 * PI * 2)
        face.gazeX += life.gazeX
        face.gazeY += life.gazeY
        face.earL += life.earL
        face.earR += life.earR
        face.lift += breath * 0.8

        if (!calm) {
            bouncePhase += dt * max(cur.bounceHz, 0.0)
            face.lift -= abs(sin(bouncePhase * PI)) * max(cur.bounce, 0.0)
        }
        dashPhase += dt * cur.spin

        val talk = cur.talk
        if (talk > 0.01) {
            val wave = 0.5 + 0.5 * (0.55 * sin(t * 27) + 0.3 * sin(t * 44.6 + 1) + 0.25 * sin(t * 14.5 + 2))
            val amount = talk * wave.coerceIn(0.0, 1.0)
            face.mouth.open = max(face.mouth.open * (1 - talk * 0.5), 0.0) + amount * 8
            face.mouth.tongue = max(face.mouth.tongue, min(1.0, talk * 1.5))
            face.mouth.cat *= 1 - talk * 0.4
        }

        return Frame(face, shownL, shownR, squashL * blinkL, squashR * blinkR, shownSymbol, symbolP, t, dashPhase, breath)
    }
}
