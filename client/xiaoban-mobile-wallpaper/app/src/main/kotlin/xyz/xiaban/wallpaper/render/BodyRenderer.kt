package xyz.xiaban.wallpaper.render

import android.graphics.Canvas
import android.graphics.Color
import android.graphics.LinearGradient
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RadialGradient
import android.graphics.Shader
import kotlin.math.roundToInt

private fun hex(value: String, alpha: Double = 1.0): Int {
    val c = Color.parseColor(value)
    return Color.argb((alpha * 255).roundToInt(), Color.red(c), Color.green(c), Color.blue(c))
}

private fun stops(vararg s: Pair<Double, Int>) = s.map { it.first.toFloat() }.toFloatArray() to s.map { it.second }.toIntArray()

/** A radial gradient in SVG objectBoundingBox units: centre and radius are fractions of the box. */
internal fun boxRadial(x: Double, y: Double, w: Double, h: Double, cx: Double, cy: Double, r: Double, vararg s: Pair<Double, Int>): Shader {
    val (at, colors) = stops(*s)
    return RadialGradient(0f, 0f, 1f, colors, at, Shader.TileMode.CLAMP).apply {
        setLocalMatrix(Matrix().apply { setScale((w * r).toFloat(), (h * r).toFloat()); postTranslate((x + cx * w).toFloat(), (y + cy * h).toFloat()) })
    }
}

internal fun linear(x0: Double, y0: Double, x1: Double, y1: Double, vararg s: Pair<Double, Int>): Shader {
    val (at, colors) = stops(*s)
    return LinearGradient(x0.toFloat(), y0.toFloat(), x1.toFloat(), y1.toFloat(), colors, at, Shader.TileMode.CLAMP)
}

private fun solid(color: Int) = Paint(Paint.ANTI_ALIAS_FLAG).apply { this.color = color }

private fun line(color: Int, width: Double) = Paint(Paint.ANTI_ALIAS_FLAG).apply {
    style = Paint.Style.STROKE; strokeWidth = width.toFloat(); strokeCap = Paint.Cap.ROUND; this.color = color
}

private fun shaded(shader: Shader) = Paint(Paint.ANTI_ALIAS_FLAG).apply { this.shader = shader }

/** The cream shell: feet, side pods, head, bezel and the dark screen. Static, so everything is built once. */
internal class ShellPainter {
    private val head = Geo.HEAD
    private val bezel = Geo.BEZEL
    private val screen = Geo.SCREEN
    private val shell = shaded(boxRadial(head.x, head.y, head.w, head.h, 0.36, 0.2, 0.95,
        0.0 to hex("#fffdf8"), 0.45 to hex("#f6ebdb"), 0.82 to hex("#e4cfae"), 1.0 to hex("#c9aa80")))
    private val shade = shaded(linear(0.0, head.y, 0.0, head.y + head.h, 0.5 to hex("#6e4c27", 0.0), 1.0 to hex("#6e4c27", 0.24)))
    private val bezelPaint = shaded(linear(0.0, bezel.y, 0.0, bezel.y + bezel.h,
        0.0 to hex("#f4e2c3"), 0.5 to hex("#d9b682"), 1.0 to hex("#b98c55")))
    private val screenPaint = shaded(boxRadial(screen.x, screen.y, screen.w, screen.h, 0.5, 0.18, 0.95,
        0.0 to hex("#1d222c"), 0.55 to hex("#090a0e"), 1.0 to hex("#020203")))
    private val headEdge = line(hex("#d2b085"), 1.5)
    private val screenEdge = line(hex("#000000", 0.7), 1.6)
    private val podEdge = line(hex("#c9a273"), 1.4)
    private val foot = solid(hex("#3a3029"))
    private val footSole = solid(hex("#5a4a3e"))
    private val highlight = solid(hex("#ffffff", 0.55))
    private val headClip = Path().apply { addRoundRect(rect(head.x, head.y, head.w, head.h), head.r.toFloat(), head.r.toFloat(), Path.Direction.CW) }

    private val pods = listOf(19.0, 265.0).map { x ->
        Triple(
            x,
            shaded(linear(x, 0.0, x + 36, 0.0, 0.0 to hex("#d9bf98"), 0.45 to hex("#fbf4e8"), 1.0 to hex("#dcc29c"))),
            shaded(linear(x + 6, 0.0, x + 30, 0.0, 0.0 to hex("#c9a87b"), 0.5 to hex("#efe0c8"), 1.0 to hex("#c49f70"))),
        )
    }

    fun draw(canvas: Canvas) {
        for (x in doubleArrayOf(98.0, 182.0)) {
            canvas.drawRoundRect(x, 226.0, 40.0, 24.0, 10.0, foot)
            canvas.drawRoundRect(x + 6, 240.0, 28.0, 4.0, 2.0, footSole)
        }
        for ((x, pod, core) in pods) {
            canvas.drawRoundRect(x, 120.0, 36.0, 88.0, 18.0, pod)
            canvas.drawRoundRect(x, 120.0, 36.0, 88.0, 18.0, podEdge)
            core.alpha = 230
            canvas.drawRoundRect(x + 6, 129.0, 24.0, 70.0, 12.0, core)
        }
        canvas.drawRoundRect(head.x, head.y, head.w, head.h, head.r, shell)
        canvas.drawRoundRect(head.x, head.y, head.w, head.h, head.r, headEdge)
        canvas.save()
        canvas.clipPath(headClip)
        canvas.drawRect(rect(head.x, head.y, head.w, head.h), shade)
        canvas.drawEllipse(118.0, 84.0, 70.0, 9.0, highlight)
        canvas.restore()
        canvas.drawRoundRect(bezel.x, bezel.y, bezel.w, bezel.h, bezel.r, bezelPaint)
        canvas.drawRoundRect(screen.x, screen.y, screen.w, screen.h, screen.r, screenPaint)
        canvas.drawRoundRect(screen.x, screen.y, screen.w, screen.h, screen.r, screenEdge)
    }
}
