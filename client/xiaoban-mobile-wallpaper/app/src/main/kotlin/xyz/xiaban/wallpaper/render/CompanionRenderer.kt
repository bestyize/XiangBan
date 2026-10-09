package xyz.xiaban.wallpaper.render

import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Path
import xyz.xiaban.wallpaper.companion.Frame
import kotlin.math.PI
import kotlin.math.roundToInt
import kotlin.math.sin

private fun argb(value: String, alpha: Double = 1.0): Int {
    val c = Color.parseColor(value)
    return Color.argb((alpha * 255).roundToInt(), Color.red(c), Color.green(c), Color.blue(c))
}

private fun fillWith(shader: android.graphics.Shader) = Paint(Paint.ANTI_ALIAS_FLAG).apply { this.shader = shader }

private fun fillColor(color: Int) = Paint(Paint.ANTI_ALIAS_FLAG).apply { this.color = color }

/**
 * Draws the robot in its 320x280 viewBox. The caller positions and scales the canvas; this class holds no
 * per-frame state, so one instance can serve the wallpaper and the in-app preview.
 */
class CompanionRenderer {
    private val shell = ShellPainter()
    private val face = FaceRenderer()
    private val screenClip = Path().apply {
        val s = Geo.SCREEN
        addRoundRect(rect(s.x, s.y, s.w, s.h), s.r.toFloat(), s.r.toFloat(), Path.Direction.CW)
    }
    private val glossPaint = fillWith(linear(0.0, Geo.SCREEN.y, 0.0, Geo.SCREEN.y + Geo.SCREEN.h,
        0.0 to argb("#ffffff", 0.09), 0.38 to argb("#ffffff", 0.0)))
    private val glint = fillColor(argb("#ffffff", 0.07))
    private val glintPath = Path().apply {
        moveTo(76.0, 128.0); quadTo(78.0, 104.0, 106.0, 100.0); lineTo(146.0, 99.0); quadTo(108.0, 106.0, 88.0, 124.0); close()
    }

    private val groundPaint = fillWith(android.graphics.RadialGradient(
        0f, 0f, 1f, intArrayOf(argb("#000000"), argb("#000000", 0.0)), null, android.graphics.Shader.TileMode.CLAMP))
    private val halo = fillWith(boxRadial(36.0, 10.0, 112.0, 104.0, 0.5, 0.5, 0.5, 0.0 to argb("#ffbe55", 0.5), 1.0 to argb("#ffbe55", 0.0)))
    private val earShell = fillWith(linear(43 + 0.3 * 102, 13.0, 43 + 0.6 * 102, 124.0, 0.0 to argb("#fbf3e6"), 1.0 to argb("#e2c9a4")))
    private val earLightPaint = fillWith(android.graphics.RadialGradient(98f, 96f, 78f,
        intArrayOf(argb("#fffbe0"), argb("#ffe08a"), argb("#ffb43f"), argb("#f28a1c")), floatArrayOf(0f, 0.3f, 0.62f, 1f),
        android.graphics.Shader.TileMode.CLAMP))
    private val earInner = fillColor(argb("#a8641f"))
    private val earEdge = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeWidth = 1.2f; color = argb("#d8bb94") }
    private val earShine = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        style = Paint.Style.STROKE; strokeWidth = 3f; strokeCap = Paint.Cap.ROUND; color = argb("#fffbe8", 0.55)
    }
    private val earOuterPath = Path().apply {
        moveTo(48.0, 124.0); cubicTo(43.0, 94.0, 52.0, 52.0, 69.0, 22.0); quadTo(75.0, 12.0, 84.0, 17.0)
        cubicTo(106.0, 32.0, 128.0, 56.0, 145.0, 88.0); lineTo(122.0, 114.0); close()
    }
    private val earInnerPath = Path().apply {
        moveTo(58.0, 114.0); cubicTo(55.0, 90.0, 62.0, 58.0, 74.0, 33.0); quadTo(78.0, 26.0, 84.0, 30.0)
        cubicTo(102.0, 44.0, 119.0, 64.0, 133.0, 92.0); lineTo(112.0, 110.0); close()
    }
    private val earShinePath = Path().apply { moveTo(66.0, 100.0); cubicTo(64.0, 80.0, 69.0, 58.0, 78.0, 40.0) }

    fun draw(canvas: Canvas, frame: Frame) {
        val f = frame.face
        shadow(canvas, frame)
        val s = 1 + 0.006 * frame.breath
        canvas.save()
        canvas.translate(0.0, f.lift)
        canvas.rotate(f.tilt, Geo.PIVOT_X, Geo.PIVOT_Y)
        canvas.translate(Geo.PIVOT_X, Geo.PIVOT_Y)
        canvas.scale(2 - s, s)
        canvas.translate(-Geo.PIVOT_X, -Geo.PIVOT_Y)

        val glow = (f.earGlow * (1 + 0.08 * frame.breath) -
            f.glowPulse * (0.5 + 0.5 * sin(frame.t * PI * 2 * 1.1)) * 0.45).coerceIn(0.0, 1.0)
        ear(canvas, -1, f.earL, glow)
        ear(canvas, 1, f.earR, glow)
        shell.draw(canvas)

        canvas.save()
        canvas.clipPath(screenClip)
        face.draw(canvas, frame)
        canvas.restore()

        val sc = Geo.SCREEN
        canvas.drawRoundRect(sc.x, sc.y, sc.w, sc.h, sc.r, glossPaint)
        canvas.drawPath(glintPath, glint)
        canvas.restore()
    }

    private fun shadow(canvas: Canvas, frame: Frame) {
        val up = (-frame.face.lift).coerceIn(0.0, 30.0)
        groundPaint.alpha = (0.3 * (1 - up * 0.015) * 255).roundToInt().coerceIn(0, 255)
        canvas.save()
        canvas.translate(160.0, 250.0)
        canvas.scale(104 * (1 - up * 0.008), 11.0)
        canvas.drawCircle(0.0, 0.0, 1.0, groundPaint)
        canvas.restore()
    }

    private fun ear(canvas: Canvas, side: Int, angle: Double, light: Double) {
        canvas.save()
        if (side == 1) {
            canvas.translate(Geo.VIEW_W, 0.0)
            canvas.scale(-1.0, 1.0)
        }
        canvas.rotate(-angle, 104.0, 98.0)
        halo.alpha = (light * 255).roundToInt().coerceIn(0, 255)
        canvas.drawEllipse(92.0, 62.0, 56.0, 52.0, halo)
        canvas.drawPath(earOuterPath, earShell)
        canvas.drawPath(earOuterPath, earEdge)
        canvas.drawPath(earInnerPath, earInner)
        earLightPaint.alpha = ((0.25 + 0.75 * light) * 255).roundToInt().coerceIn(0, 255)
        canvas.drawPath(earInnerPath, earLightPaint)
        canvas.drawPath(earShinePath, earShine)
        canvas.restore()
    }
}
