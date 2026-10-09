package xyz.xiaban.wallpaper.render

import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RectF

internal fun Path.moveTo(x: Double, y: Double) = moveTo(x.toFloat(), y.toFloat())
internal fun Path.lineTo(x: Double, y: Double) = lineTo(x.toFloat(), y.toFloat())
internal fun Path.quadTo(x1: Double, y1: Double, x2: Double, y2: Double) =
    quadTo(x1.toFloat(), y1.toFloat(), x2.toFloat(), y2.toFloat())

internal fun Path.cubicTo(x1: Double, y1: Double, x2: Double, y2: Double, x3: Double, y3: Double) =
    cubicTo(x1.toFloat(), y1.toFloat(), x2.toFloat(), y2.toFloat(), x3.toFloat(), y3.toFloat())

internal fun rect(x: Double, y: Double, w: Double, h: Double) =
    RectF(x.toFloat(), y.toFloat(), (x + w).toFloat(), (y + h).toFloat())

internal fun Canvas.drawCircle(cx: Double, cy: Double, r: Double, paint: Paint) =
    drawCircle(cx.toFloat(), cy.toFloat(), r.toFloat(), paint)

internal fun Canvas.drawEllipse(cx: Double, cy: Double, rx: Double, ry: Double, paint: Paint) =
    drawOval(rect(cx - rx, cy - ry, rx * 2, ry * 2), paint)

internal fun Canvas.drawRoundRect(x: Double, y: Double, w: Double, h: Double, r: Double, paint: Paint) =
    drawRoundRect(rect(x, y, w, h), r.toFloat(), r.toFloat(), paint)

internal fun Canvas.translate(dx: Double, dy: Double) = translate(dx.toFloat(), dy.toFloat())
internal fun Canvas.scale(sx: Double, sy: Double) = scale(sx.toFloat(), sy.toFloat())
internal fun Canvas.rotate(degrees: Double, px: Double, py: Double) =
    rotate(degrees.toFloat(), px.toFloat(), py.toFloat())

internal fun heartPath(x: Double, y: Double, s: Double, sy: Double = 1.0) = Path().apply {
    fun yy(v: Double) = y + v * sy
    moveTo(x, yy(s * 0.85))
    cubicTo(x - s * 1.55, yy(-s * 0.15), x - s * 0.7, yy(-s * 1.25), x, yy(-s * 0.42))
    cubicTo(x + s * 0.7, yy(-s * 1.25), x + s * 1.55, yy(-s * 0.15), x, yy(s * 0.85))
    close()
}

internal fun starPath(x: Double, y: Double, r: Double) = Path().apply {
    val k = r * 0.22
    moveTo(x, y - r)
    quadTo(x + k, y - k, x + r, y)
    quadTo(x + k, y + k, x, y + r)
    quadTo(x - k, y + k, x - r, y)
    quadTo(x - k, y - k, x, y - r)
    close()
}
