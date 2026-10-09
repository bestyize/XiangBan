package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.nativeCanvas
import xyz.xiaban.wallpaper.companion.Animator
import xyz.xiaban.wallpaper.companion.Expression
import xyz.xiaban.wallpaper.companion.Frame
import xyz.xiaban.wallpaper.render.CompanionRenderer
import xyz.xiaban.wallpaper.render.Geo

private const val FRAME_NANOS = 33_000_000L

/** The 小伴 character, alive: it blinks, glances around and follows the expression it is given. */
@Composable
fun PetFace(expression: Expression, modifier: Modifier = Modifier) {
    val animator = remember { Animator(expression) }
    val renderer = remember { CompanionRenderer() }
    var frame by remember { mutableStateOf<Frame?>(null) }

    LaunchedEffect(expression) { animator.setTarget(expression) }
    LaunchedEffect(Unit) {
        var last = 0L
        while (true) {
            androidx.compose.runtime.withFrameNanos { now ->
                if (last == 0L || now - last >= FRAME_NANOS) {
                    frame = animator.step(if (last == 0L) 1.0 / 30 else (now - last) / 1e9)
                    last = now
                }
            }
        }
    }

    Canvas(modifier.aspectRatio((Geo.VIEW_W / Geo.VIEW_H).toFloat())) {
        val current = frame ?: return@Canvas
        val scale = size.width / Geo.VIEW_W.toFloat()
        drawIntoCanvas { canvas ->
            val native = canvas.nativeCanvas
            native.save()
            native.scale(scale, scale)
            renderer.draw(native, current)
            native.restore()
        }
    }
}
