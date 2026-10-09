package xyz.xiaban.wallpaper.render

import android.graphics.Canvas
import android.graphics.Color
import android.graphics.DashPathEffect
import android.graphics.Path
import xyz.xiaban.wallpaper.companion.Face
import xyz.xiaban.wallpaper.companion.Frame
import xyz.xiaban.wallpaper.companion.Mouth
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.pow
import kotlin.math.sin

/** Everything on the glowing screen: eyes, mouth, cheeks, tears and the status symbol. */
internal class FaceRenderer {
    private val pen = Pen()
    private val eyes = EyeRenderer(pen)
    private val screenBounds = rect(Geo.SCREEN.x - 24, Geo.SCREEN.y - 24, Geo.SCREEN.w + 48, Geo.SCREEN.h + 48)

    fun draw(canvas: Canvas, frame: Frame) {
        val face = frame.face
        val color = mix(Palette.lineDim, Palette.line, ((face.screen - 0.3) / 0.7).coerceIn(0.0, 1.0))
        val opacity = (0.3 + 0.7 * face.screen).coerceIn(0.0, 1.0)
        val dimmed = opacity < 0.999
        if (dimmed) canvas.saveLayerAlpha(screenBounds, (opacity * 255).toInt())
        for (pass in Pass.entries) {
            pen.pass = pass
            cheeks(canvas, face)
            tears(canvas, face, frame.t)
            eyes.draw(canvas, face.eyeL, -1, frame.squashL, frame.glyphL, color, frame.dash, face.gazeX, face.gazeY)
            eyes.draw(canvas, face.eyeR, 1, frame.squashR, frame.glyphR, color, frame.dash, face.gazeX, face.gazeY)
            mouth(canvas, face.mouth, color, face.gazeX, face.gazeY)
            frame.symbol?.let { Symbols.draw(pen, canvas, it, frame.symbolP, frame.t) }
        }
        pen.pass = Pass.CRISP
        if (dimmed) canvas.restore()
    }

    private fun cheeks(canvas: Canvas, face: Face) {
        for (side in intArrayOf(-1, 1)) {
            val x = Geo.FACE_X + side * Geo.CHEEK_DX + face.gazeX * 0.7
            val y = Geo.FACE_Y + Geo.CHEEK_DY + face.gazeY * 0.7
            if (face.blush > 0.02) pen.fillEllipse(canvas, x, y, 10.0, 5.5, Palette.blush, face.blush.coerceIn(0.0, 1.0) * 0.8)
            if (face.blushLines > 0.02) {
                val marks = Path().apply {
                    for (dx in intArrayOf(-5, 0, 5)) { moveTo(x + dx - 1.8, y + 3.6); lineTo(x + dx + 1.8, y - 3.6) }
                }
                pen.stroke(canvas, marks, Color.parseColor("#ff9ab6"), 2.2, face.blushLines.coerceIn(0.0, 1.0))
            }
        }
    }

    private fun tears(canvas: Canvas, face: Face, t: Double) {
        if (face.tears < 0.02) return
        val k = face.tears.coerceIn(0.0, 1.0)
        val bottom = Geo.SCREEN.y + Geo.SCREEN.h + 6
        for ((side, eye) in listOf(-1 to face.eyeL, 1 to face.eyeR)) {
            val x = Geo.FACE_X + side * Geo.EYE_DX + eye.x + face.gazeX + side * -eye.w * 0.25
            val y0 = Geo.FACE_Y + Geo.EYE_DY + eye.y + face.gazeY + 2
            val yEnd = y0 + (bottom - y0) * k
            val drop = Path().apply { moveTo(x, y0); cubicTo(x - side, y0 + 18, x + side * 2, yEnd - 14, x + side * 3, yEnd) }
            pen.stroke(canvas, drop, Palette.tear, 7 + 4 * k, 0.9)
            val phase = ((-t * 36) % 18 + 18) % 18
            pen.stroke(canvas, drop, Color.parseColor("#d8f3ff"), 2.6, 0.75, DashPathEffect(floatArrayOf(5f, 13f), phase.toFloat()))
        }
    }

    private fun mouth(canvas: Canvas, m: Mouth, color: Int, gx: Double, gy: Double) {
        val mx = Geo.FACE_X + m.x + gx * 0.8
        val my = Geo.FACE_Y + Geo.MOUTH_DY + m.y + gy * 0.8
        val w = max(m.w, 1.0)
        val n = 18
        val upper = Path()
        val lower = ArrayList<Pair<Double, Double>>()
        for (i in 0..n) {
            val t = -1 + 2.0 * i / n
            val base = 1 - t * t
            val y = my + m.curve * base + m.cat * sin(PI * abs(t))
            if (i == 0) upper.moveTo(mx + t * w, y) else upper.lineTo(mx + t * w, y)
            lower += (mx + t * w) to (y + max(m.open, 0.0) * max(base, 0.0).pow(0.7))
        }
        if (m.open < 1.2) {
            pen.stroke(canvas, upper, color, 3.8)
            return
        }
        for ((x, y) in lower.asReversed()) upper.lineTo(x, y)
        upper.close()
        pen.fill(canvas, upper, if (m.tongue > 0.02) Palette.pink else Palette.screen, max(m.tongue, 0.9))
        pen.stroke(canvas, upper, color, 3.2)
    }
}
