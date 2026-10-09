package xyz.xiaban.wallpaper.companion

internal const val EYE_FIELDS = 14
internal const val MOUTH_AT = EYE_FIELDS * 2
internal const val SCALAR_AT = MOUTH_AT + 7

internal const val FACE_SIZE = SCALAR_AT + 16

/** One glowing eye: an elliptical ring that can thicken into a solid lens, be cut by lids and hollowed by a pupil. */
class Eye internal constructor(private val a: DoubleArray, private val o: Int) {
    var w: Double get() = a[o]; set(v) { a[o] = v }
    var h: Double get() = a[o + 1]; set(v) { a[o + 1] = v }
    var thick: Double get() = a[o + 2]; set(v) { a[o + 2] = v }
    var arcMid: Double get() = a[o + 3]; set(v) { a[o + 3] = v }
    var span: Double get() = a[o + 4]; set(v) { a[o + 4] = v }
    var lidTop: Double get() = a[o + 5]; set(v) { a[o + 5] = v }
    var lidTilt: Double get() = a[o + 6]; set(v) { a[o + 6] = v }
    var lidBottom: Double get() = a[o + 7]; set(v) { a[o + 7] = v }
    var pupil: Double get() = a[o + 8]; set(v) { a[o + 8] = v }
    var pupilX: Double get() = a[o + 9]; set(v) { a[o + 9] = v }
    var pupilY: Double get() = a[o + 10]; set(v) { a[o + 10] = v }
    var dash: Double get() = a[o + 11]; set(v) { a[o + 11] = v }
    var x: Double get() = a[o + 12]; set(v) { a[o + 12] = v }
    var y: Double get() = a[o + 13]; set(v) { a[o + 13] = v }
}

class Mouth internal constructor(private val a: DoubleArray, private val o: Int) {
    var w: Double get() = a[o]; set(v) { a[o] = v }
    var curve: Double get() = a[o + 1]; set(v) { a[o + 1] = v }
    var cat: Double get() = a[o + 2]; set(v) { a[o + 2] = v }
    var open: Double get() = a[o + 3]; set(v) { a[o + 3] = v }
    var tongue: Double get() = a[o + 4]; set(v) { a[o + 4] = v }
    var x: Double get() = a[o + 5]; set(v) { a[o + 5] = v }
    var y: Double get() = a[o + 6]; set(v) { a[o + 6] = v }
}

/**
 * The face is a flat array of continuous parameters, so any two expressions can be blended and animated between.
 * Units are viewBox units of the 320×280 robot. A patch is a face whose unset fields are NaN.
 */
class Face internal constructor(internal val a: DoubleArray) {
    val eyeL = Eye(a, 0)
    val eyeR = Eye(a, EYE_FIELDS)
    val mouth = Mouth(a, MOUTH_AT)

    var blush: Double get() = a[SCALAR_AT]; set(v) { a[SCALAR_AT] = v }
    var blushLines: Double get() = a[SCALAR_AT + 1]; set(v) { a[SCALAR_AT + 1] = v }
    var tears: Double get() = a[SCALAR_AT + 2]; set(v) { a[SCALAR_AT + 2] = v }
    var gazeX: Double get() = a[SCALAR_AT + 3]; set(v) { a[SCALAR_AT + 3] = v }
    var gazeY: Double get() = a[SCALAR_AT + 4]; set(v) { a[SCALAR_AT + 4] = v }
    var tilt: Double get() = a[SCALAR_AT + 5]; set(v) { a[SCALAR_AT + 5] = v }
    var lift: Double get() = a[SCALAR_AT + 6]; set(v) { a[SCALAR_AT + 6] = v }
    var earL: Double get() = a[SCALAR_AT + 7]; set(v) { a[SCALAR_AT + 7] = v }
    var earR: Double get() = a[SCALAR_AT + 8]; set(v) { a[SCALAR_AT + 8] = v }
    var earGlow: Double get() = a[SCALAR_AT + 9]; set(v) { a[SCALAR_AT + 9] = v }
    var glowPulse: Double get() = a[SCALAR_AT + 10]; set(v) { a[SCALAR_AT + 10] = v }
    var screen: Double get() = a[SCALAR_AT + 11]; set(v) { a[SCALAR_AT + 11] = v }
    var spin: Double get() = a[SCALAR_AT + 12]; set(v) { a[SCALAR_AT + 12] = v }
    var bounce: Double get() = a[SCALAR_AT + 13]; set(v) { a[SCALAR_AT + 13] = v }
    var bounceHz: Double get() = a[SCALAR_AT + 14]; set(v) { a[SCALAR_AT + 14] = v }
    var talk: Double get() = a[SCALAR_AT + 15]; set(v) { a[SCALAR_AT + 15] = v }

    fun copy() = Face(a.copyOf())

    fun apply(patch: Face) {
        for (i in a.indices) if (!patch.a[i].isNaN()) a[i] = patch.a[i]
    }

    fun both(block: Eye.() -> Unit) {
        eyeL.block()
        eyeR.block()
    }

    companion object {
        fun blank() = Face(DoubleArray(FACE_SIZE) { Double.NaN })

        fun base(): Face {
            val face = Face(DoubleArray(FACE_SIZE))
            face.both {
                w = 23.0; h = 23.0; thick = 0.36; arcMid = -90.0; span = 1.0
                lidTop = -1.4; lidBottom = 1.4
            }
            face.mouth.w = 10.0
            face.mouth.cat = 4.5
            face.earGlow = 1.0
            face.screen = 1.0
            return face
        }
    }
}

enum class Glyph { LENS, CHEVRON, HEART }

enum class Symbol { SPARKLE, SPARKS, QUESTION, EXCLAIM, DOTS, BUBBLE, ZZZ, ANGER, SOUND, WAVE, LISTEN, BAN, BOLT, SWEAT }

/** Discrete parts of an expression that switch rather than blend. */
data class Look(
    val glyphL: Glyph = Glyph.LENS,
    val glyphR: Glyph = Glyph.LENS,
    val symbol: Symbol? = null,
    val blink: Boolean = true,
    val saccade: Boolean = true,
)
