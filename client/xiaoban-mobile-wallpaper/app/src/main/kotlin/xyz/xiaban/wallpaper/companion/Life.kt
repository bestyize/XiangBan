package xyz.xiaban.wallpaper.companion

import kotlin.math.sqrt

internal class Spring(val k: Double, val c: Double)

internal fun spring(k: Double, zeta: Double) = Spring(k, 2 * zeta * sqrt(k))

/** Deterministic PRNG so tests (and two robots seeded alike) behave reproducibly. */
class Mulberry32(seed: Int) {
    private var a = seed

    fun next(): Double {
        a += 0x6d2b79f5
        var t = a
        t = (t xor (t ushr 15)) * (t or 1)
        t = t xor (t + (t xor (t ushr 7)) * (t or 61))
        return ((t xor (t ushr 14)).toLong() and 0xFFFFFFFFL).toDouble() / 4294967296.0
    }
}

/** Where the robot looks, -1..1 on each axis relative to itself. */
class Gaze(val x: Double, val y: Double)

/** Idle glances and ear twitches layered on top of the expression springs. */
internal class Life(private val rand: () -> Double, private var twitchNext: Double) {
    var gazeX = 0.0; private set
    var gazeY = 0.0; private set
    var earL = 0.0; private set
    var earR = 0.0; private set

    private var glanceNext = 0.0
    private var targetX = 0.0
    private var targetY = 0.0
    private var vx = 0.0
    private var vy = 0.0
    private var vl = 0.0
    private var vr = 0.0
    private val glanceSpring = spring(520.0, 0.86)
    private val twitchSpring = spring(420.0, 0.28)

    fun kick(side: Int, strength: Double) {
        if (side <= 0) vl -= 300 * strength
        if (side >= 0) vr -= 300 * strength
    }

    fun step(h: Double, t: Double, saccade: Boolean, lookAt: Gaze?, calm: Boolean) {
        if (lookAt != null) {
            targetX = lookAt.x.coerceIn(-1.0, 1.0) * 11
            targetY = lookAt.y.coerceIn(-1.0, 1.0) * 7
        } else if (!saccade || calm) {
            targetX = 0.0
            targetY = 0.0
        } else if (t >= glanceNext) {
            if (rand() < 0.35) {
                targetX = 0.0
                targetY = 0.0
            } else {
                targetX = (rand() * 2 - 1) * 7
                targetY = (rand() * 2 - 1) * 4
            }
            glanceNext = t + 0.8 + rand() * 2.8
        }
        vx += ((targetX - gazeX) * glanceSpring.k - vx * glanceSpring.c) * h
        vy += ((targetY - gazeY) * glanceSpring.k - vy * glanceSpring.c) * h
        gazeX += vx * h
        gazeY += vy * h

        if (!calm && t >= twitchNext) {
            kick(if (rand() < 0.5) -1 else 1, 0.7 + rand() * 0.5)
            twitchNext = t + 5 + rand() * 9
        }
        vl += (-earL * twitchSpring.k - vl * twitchSpring.c) * h
        vr += (-earR * twitchSpring.k - vr * twitchSpring.c) * h
        earL += vl * h
        earR += vr * h
    }
}
