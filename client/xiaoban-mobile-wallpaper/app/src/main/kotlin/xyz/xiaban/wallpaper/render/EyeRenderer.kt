package xyz.xiaban.wallpaper.render

import android.graphics.Canvas
import android.graphics.DashPathEffect
import android.graphics.Paint
import android.graphics.Path
import android.graphics.PorterDuff
import android.graphics.PorterDuffXfermode
import xyz.xiaban.wallpaper.companion.Eye
import xyz.xiaban.wallpaper.companion.Glyph
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sqrt

private const val LINE_MIN = 6.0

private fun ellipsePerimeter(a: Double, b: Double) =
    PI * (3 * (a + b) - sqrt(max(0.0, (3 * a + b) * (a + 3 * b))))

internal class EyeRenderer(private val pen: Pen) {
    private val punch = Paint().apply {
        isAntiAlias = true
        xfermode = PorterDuffXfermode(PorterDuff.Mode.CLEAR)
    }

    /** side is -1 for the left eye and 1 for the right one; dash is the spinner rotation in turns. */
    fun draw(canvas: Canvas, e: Eye, side: Int, squash: Double, glyph: Glyph, color: Int, dash: Double, gx: Double, gy: Double) {
        val cx = Geo.FACE_X + side * Geo.EYE_DX + e.x + gx
        val cy = Geo.FACE_Y + Geo.EYE_DY + e.y + gy
        val inner = if (side == -1) 1 else -1
        val w = max(e.w, 1.0)
        val h0 = max(e.h, 0.0)
        val thick = e.thick.coerceIn(0.0, 1.0) * min(w, max(h0, LINE_MIN))

        if (glyph == Glyph.CHEVRON) {
            val hh = max(h0, 18.0) * 0.7 * squash
            val ww = w * 0.62
            pen.stroke(canvas, Path().apply {
                moveTo(cx - ww * inner, cy - hh); lineTo(cx + ww * inner, cy); lineTo(cx - ww * inner, cy + hh)
            }, color, max(thick, LINE_MIN))
            return
        }
        if (glyph == Glyph.HEART) {
            pen.fill(canvas, heartPath(cx, cy + 2, w * 0.82, squash), Palette.pink)
            return
        }

        val sw = max(min(thick, 2 * max(h0 * squash, 0.0)), LINE_MIN)
        val h = max(h0 * squash, sw / 2)
        val rx = max(w - sw / 2, 0.01)
        val ry = max(h - sw / 2, 0.0)
        val oval = rect(cx - rx, cy - max(ry, 0.01), rx * 2, max(ry, 0.01) * 2)
        val span = e.span.coerceIn(0.0, 1.0)
        val ring = Path()
        if (span > 0.995) ring.addOval(oval, Path.Direction.CCW)
        else ring.arcTo(oval, (e.arcMid - span * 180).toFloat(), (span * 360).toFloat(), true)

        val effect = if (e.dash > 0.01) {
            val perimeter = ellipsePerimeter(rx, ry)
            val segment = perimeter / 8
            val on = segment * (1 - 0.55 * e.dash.coerceIn(0.0, 1.0))
            val phase = ((-dash * perimeter) % segment + segment) % segment
            DashPathEffect(floatArrayOf(on.toFloat(), (segment - on).toFloat()), phase.toFloat())
        } else null

        canvas.save()
        if (!(e.lidTop - abs(e.lidTilt) < -1.05 && e.lidBottom > 1.05)) canvas.clipPath(lidPath(e, cx, cy, w, h, inner))
        if (e.pupil > 0.02 && pen.pass != Pass.WIDE) {
            val reach = max(h0, 1.0)
            canvas.saveLayer(rect(cx - w - 16, cy - reach - 16, w * 2 + 32, reach * 2 + 32), null)
            pen.stroke(canvas, ring, color, sw, effect = effect)
            val r = e.pupil * min(w, reach)
            canvas.drawEllipse(cx + e.pupilX * w, cy + e.pupilY * h, r, r * min(1.0, squash * 1.2), punch)
            canvas.restore()
        } else {
            pen.stroke(canvas, ring, color, sw, effect = effect)
        }
        canvas.restore()
    }

    /** The visible window of the eye: the upper lid may slant, the lower lid sits below. */
    private fun lidPath(e: Eye, cx: Double, cy: Double, w: Double, h: Double, inner: Int): Path {
        val x0 = cx - w - 8
        val x1 = cx + w + 8
        fun top(x: Double) = cy + (e.lidTop + e.lidTilt * ((x - cx) / w) * inner) * h
        val bottom = cy + e.lidBottom * h
        return Path().apply {
            moveTo(x0, top(x0)); lineTo(x1, top(x1)); lineTo(x1, bottom); lineTo(x0, bottom); close()
        }
    }
}
