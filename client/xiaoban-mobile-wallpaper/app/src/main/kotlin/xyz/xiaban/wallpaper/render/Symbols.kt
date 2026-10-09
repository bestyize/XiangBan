package xyz.xiaban.wallpaper.render

import android.graphics.Canvas
import android.graphics.Path
import xyz.xiaban.wallpaper.companion.Symbol
import kotlin.math.PI
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sin

private fun wave(t: Double, hz: Double, phase: Double = 0.0) = sin((t * hz + phase) * PI * 2)
private fun pulse(t: Double, hz: Double, phase: Double = 0.0) = 0.5 + 0.5 * wave(t, hz, phase)

/** Elastic pop-in curve: 0 -> 1 with a small overshoot. */
internal fun popScale(p: Double): Double {
    if (p <= 0) return 0.0
    if (p >= 1) return 1.0
    val c = 1.9
    val q = p - 1
    return 1 + (c + 1) * q * q * q + c * q * q
}

private inline fun path(build: Path.() -> Unit) = Path().apply(build)

/** Status symbols that pop up at the top right of the screen. */
internal object Symbols {
    fun draw(pen: Pen, canvas: Canvas, symbol: Symbol, appear: Double, t: Double) {
        if (appear <= 0.01) return
        val x = Geo.SYMBOL_X
        val y = Geo.SYMBOL_Y
        val saved = pen.alpha
        pen.alpha = saved * min(1.0, appear * 1.5)
        canvas.save()
        canvas.translate(x, y)
        canvas.scale(popScale(appear), popScale(appear))
        canvas.translate(-x, -y)
        paint(pen, canvas, symbol, t, x, y)
        canvas.restore()
        pen.alpha = saved
    }

    private fun paint(pen: Pen, canvas: Canvas, symbol: Symbol, t: Double, x: Double, y: Double) {
        val line = Palette.line
        val yellow = Palette.yellow
        when (symbol) {
            Symbol.SPARKLE -> {
                val s = 1 + 0.18 * wave(t, 1.3)
                pen.fill(canvas, starPath(x, y, 9 * s), yellow)
                pen.fill(canvas, starPath(x - 13, y + 9, 3.6 * (2 - s)), yellow, 0.8)
            }
            Symbol.SPARKS -> {
                val o = 2 * pulse(t, 2.4)
                pen.stroke(canvas, path {
                    moveTo(x - 6, y - 2 - o); lineTo(x - 3, y - 11 - o)
                    moveTo(x + 2, y + 1 - o); lineTo(x + 9, y - 6 - o)
                }, yellow, 3.2)
                pen.fill(canvas, starPath(x + 9, y - 13, 3.4 + o * 0.6), yellow)
            }
            Symbol.QUESTION -> {
                canvas.save()
                canvas.rotate(9 * wave(t, 0.7), x, y + 10)
                pen.stroke(canvas, path {
                    moveTo(x - 6, y - 6); quadTo(x - 6, y - 13, x, y - 13)
                    quadTo(x + 6.5, y - 13, x + 6.5, y - 7); quadTo(x + 6.5, y - 2, x + 1, y)
                    quadTo(x, y + 1, x, y + 4)
                }, yellow, 3.6)
                pen.fillCircle(canvas, x, y + 10, 2.3, yellow)
                canvas.restore()
            }
            Symbol.EXCLAIM -> {
                val j = 1.2 * wave(t, 6.0) * pulse(t, 0.8)
                pen.stroke(canvas, path { moveTo(x + 1 + j, y - 14); lineTo(x - 0.5 + j, y + 3) }, yellow, 4.4)
                pen.fillCircle(canvas, x - 1 + j, y + 10, 2.6, yellow)
            }
            Symbol.DOTS -> for (i in 0..2) {
                val k = pulse(t, 1.4, -i * 0.18)
                pen.fillCircle(canvas, x - 10 + i * 9, y - 2 * k, 2.6, line, 0.35 + 0.65 * k)
            }
            Symbol.BUBBLE -> {
                val r = 6 + 3.2 * pulse(t, 0.34, -0.25)
                pen.fillCircle(canvas, x - 6, y + 10 - r * 0.4, r, Palette.gray, 0.32)
                pen.strokeCircle(canvas, x - 6, y + 10 - r * 0.4, r, Palette.gray, 1.6, 0.32)
                pen.fillCircle(canvas, x - 6 + r * 0.38, y + 10 - r * 0.8, r * 0.24, 0xFFFFFFFF.toInt(), 0.85)
            }
            Symbol.ZZZ -> for (i in 0..2) {
                val k = (t * 0.45 + i / 3.0) % 1
                val s = 3 + i * 1.6
                val cx = x - 14 + i * 9 + k * 4
                val cy = y + 10 - i * 8 - k * 6
                pen.stroke(canvas, path {
                    moveTo(cx - s, cy - s); lineTo(cx + s, cy - s); lineTo(cx - s, cy + s); lineTo(cx + s, cy + s)
                }, line, 2.4, min(1.0, min(k * 4, (1 - k) * 3)))
            }
            Symbol.ANGER -> {
                val s = 1 + 0.16 * max(0.0, wave(t, 1.6))
                pen.stroke(canvas, path {
                    for ((dx, dy) in listOf(1 to 1, -1 to 1, 1 to -1, -1 to -1)) {
                        moveTo(x + dx * 2.5 * s, y + dy * 8 * s)
                        quadTo(x + dx * 2.5 * s, y + dy * 2.5 * s, x + dx * 8 * s, y + dy * 2.5 * s)
                    }
                }, Palette.red, 3.2)
            }
            Symbol.SOUND -> for (i in 0..1) {
                val k = pulse(t, 2.2, -i * 0.22)
                val r = 9.0 + i * 7
                val cx = x - 12
                val cy = y + 33
                pen.stroke(canvas, path {
                    moveTo(cx + r * 0.6, cy - r * 0.8); quadTo(cx + r * 1.1, cy, cx + r * 0.6, cy + r * 0.8)
                }, yellow, 3.2, 0.35 + 0.65 * k)
            }
            Symbol.WAVE -> for (i in 0..1) {
                val k = pulse(t, 1.6, -i * 0.25)
                val r = 6.0 + i * 6
                pen.stroke(canvas, path {
                    moveTo(x - 4 + r * 0.2, y - r * 0.9); quadTo(x - 4 + r, y - r * 0.5, x - 4 + r, y + r * 0.4)
                }, yellow, 3.0, 0.35 + 0.65 * k)
            }
            Symbol.LISTEN -> for (i in -1..1) {
                val len = 5 + 4 * pulse(t, 1.8, i * 0.15)
                val cx = x + 2
                val cy = y + 22 + i * 7
                pen.stroke(canvas, path { moveTo(cx, cy); lineTo(cx + len, cy + i * 2.4) }, yellow, 2.8)
            }
            Symbol.BAN -> {
                val saved = pen.alpha
                pen.alpha = saved * (0.55 + 0.45 * pulse(t, 0.8))
                pen.strokeCircle(canvas, x, y, 8.0, Palette.orange, 2.8)
                pen.stroke(canvas, path { moveTo(x - 5.6, y + 5.6); lineTo(x + 5.6, y - 5.6) }, Palette.orange, 2.8)
                pen.alpha = saved
            }
            Symbol.BOLT -> pen.fill(canvas, path {
                moveTo(x + 3, y - 12); lineTo(x - 6, y + 1); lineTo(x, y + 1); lineTo(x - 3, y + 12)
                lineTo(x + 6, y - 2); lineTo(x, y - 2); close()
            }, Palette.green, 0.6 + 0.4 * pulse(t, 1.2))
            Symbol.SWEAT -> {
                val k = (t * 0.5) % 1
                val dy = k * 8
                pen.fill(canvas, path {
                    moveTo(x, y - 9 + dy); quadTo(x + 7, y + 1 + dy, x, y + 5 + dy)
                    quadTo(x - 7, y + 1 + dy, x, y - 9 + dy); close()
                }, Palette.tear, min(1.0, (1 - k) * 3))
            }
        }
    }
}
