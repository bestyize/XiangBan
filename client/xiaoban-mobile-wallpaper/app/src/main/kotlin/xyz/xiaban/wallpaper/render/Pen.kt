package xyz.xiaban.wallpaper.render

import android.graphics.BlurMaskFilter
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Path
import android.graphics.PathEffect
import kotlin.math.roundToInt

/** The screen glow is three passes of the same drawing: a wide tinted halo, a tight halo, then the crisp art. */
internal enum class Pass { WIDE, NEAR, CRISP }

internal object Geo {
    class Box(val x: Double, val y: Double, val w: Double, val h: Double, val r: Double)

    const val VIEW_W = 320.0
    const val VIEW_H = 280.0
    val HEAD = Box(38.0, 76.0, 244.0, 164.0, 66.0)
    val BEZEL = Box(53.0, 90.0, 214.0, 137.0, 50.0)
    val SCREEN = Box(58.0, 95.0, 204.0, 127.0, 45.0)
    const val FACE_X = 160.0
    const val FACE_Y = 158.0
    const val EYE_DX = 50.0
    const val EYE_DY = -7.0
    const val MOUTH_DY = 18.0
    const val CHEEK_DX = 80.0
    const val CHEEK_DY = 20.0
    const val SYMBOL_X = 236.0
    const val SYMBOL_Y = 117.0
    const val PIVOT_X = 160.0
    const val PIVOT_Y = 246.0
}

internal object Palette {
    val line = Color.parseColor("#eaf9ff")
    val lineDim = Color.parseColor("#5fb6ff")
    val pink = Color.parseColor("#ff6f9a")
    val blush = Color.parseColor("#ff6f91")
    val yellow = Color.parseColor("#ffd64d")
    val red = Color.parseColor("#ff5a6e")
    val orange = Color.parseColor("#ff8a4c")
    val green = Color.parseColor("#7be07b")
    val tear = Color.parseColor("#5ec8ff")
    val gray = Color.parseColor("#b9c3cf")
    val screen = Color.parseColor("#050608")
}

internal fun mix(a: Int, b: Int, k: Double): Int {
    fun ch(from: Int, to: Int) = (from + (to - from) * k).roundToInt().coerceIn(0, 255)
    return Color.rgb(ch(Color.red(a), Color.red(b)), ch(Color.green(a), Color.green(b)), ch(Color.blue(a), Color.blue(b)))
}

/** Draws with the current glow pass; alpha scales everything like a group opacity. */
internal class Pen {
    var pass = Pass.CRISP
    var alpha = 1.0

    private val paint = Paint()
    private val wide = BlurMaskFilter(SIGMA_WIDE_RADIUS, BlurMaskFilter.Blur.NORMAL)
    private val near = BlurMaskFilter(SIGMA_NEAR_RADIUS, BlurMaskFilter.Blur.NORMAL)

    private fun prepare(color: Int, opacity: Double, style: Paint.Style): Paint {
        paint.reset()
        paint.isAntiAlias = true
        paint.style = style
        paint.color = if (pass == Pass.WIDE) glow(color) else color
        paint.alpha = (opacity * alpha * 255).roundToInt().coerceIn(0, 255)
        paint.maskFilter = when (pass) {
            Pass.WIDE -> wide
            Pass.NEAR -> near
            Pass.CRISP -> null
        }
        return paint
    }

    fun stroke(canvas: Canvas, path: Path, color: Int, width: Double, opacity: Double = 1.0, effect: PathEffect? = null) {
        val p = prepare(color, opacity, Paint.Style.STROKE)
        p.strokeWidth = width.toFloat()
        p.strokeCap = Paint.Cap.ROUND
        p.strokeJoin = Paint.Join.ROUND
        p.pathEffect = effect
        canvas.drawPath(path, p)
    }

    fun fill(canvas: Canvas, path: Path, color: Int, opacity: Double = 1.0) =
        canvas.drawPath(path, prepare(color, opacity, Paint.Style.FILL))

    fun fillCircle(canvas: Canvas, cx: Double, cy: Double, r: Double, color: Int, opacity: Double = 1.0) =
        canvas.drawCircle(cx, cy, r, prepare(color, opacity, Paint.Style.FILL))

    fun strokeCircle(canvas: Canvas, cx: Double, cy: Double, r: Double, color: Int, width: Double, opacity: Double = 1.0) {
        val p = prepare(color, opacity, Paint.Style.STROKE)
        p.strokeWidth = width.toFloat()
        canvas.drawCircle(cx, cy, r, p)
    }

    fun fillEllipse(canvas: Canvas, cx: Double, cy: Double, rx: Double, ry: Double, color: Int, opacity: Double = 1.0) =
        canvas.drawEllipse(cx, cy, rx, ry, prepare(color, opacity, Paint.Style.FILL))

    /** The wide halo of the desktop art shifts colours toward cyan (feColorMatrix 0.55 / 0.85+0.04 / 1+0.12). */
    private fun glow(c: Int) = Color.rgb(
        (Color.red(c) * 0.55).roundToInt(),
        (Color.green(c) * 0.85 + 0.04 * 255).roundToInt().coerceAtMost(255),
        (Color.blue(c) + 0.12 * 255).roundToInt().coerceAtMost(255),
    )

    private companion object {
        // Android blur radius -> sigma is 0.57735 * radius + 0.5; these give sigma 5.5 and 1.5 like the SVG filter.
        const val SIGMA_WIDE_RADIUS = 8.66f
        const val SIGMA_NEAR_RADIUS = 1.73f
    }
}
