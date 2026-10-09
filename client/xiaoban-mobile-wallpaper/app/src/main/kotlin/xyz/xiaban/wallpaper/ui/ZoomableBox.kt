package xyz.xiaban.wallpaper.ui

import androidx.compose.animation.core.animate
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateCentroid
import androidx.compose.foundation.gestures.calculateCentroidSize
import androidx.compose.foundation.gestures.calculatePan
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerInputScope
import androidx.compose.ui.input.pointer.changedToUp
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.input.pointer.positionChanged
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.IntSize
import kotlinx.coroutines.launch
import kotlin.math.abs

/** What the viewer is showing right now. Held outside the composable so tests and callers can read and set it. */
class ZoomState {
    var zoom by mutableStateOf(Zoom())

    /** How far the picture has been pulled down or up while trying to close the viewer. */
    var dragY by mutableFloatStateOf(0f)

    suspend fun animateTo(target: Zoom) {
        val from = zoom
        animate(0f, 1f, animationSpec = tween(ZOOM_MILLIS)) { t, _ ->
            zoom = Zoom(lerp(from.scale, target.scale, t), lerp(from.x, target.x, t), lerp(from.y, target.y, t))
        }
    }

    suspend fun settle() {
        animate(dragY, 0f, animationSpec = tween(ZOOM_MILLIS)) { value, _ -> dragY = value }
    }

    private fun lerp(a: Float, b: Float, t: Float) = a + (b - a) * t

    private companion object {
        const val ZOOM_MILLIS = 220
    }
}

@Composable
fun rememberZoomState() = remember { ZoomState() }

const val ZOOMABLE_TAG = "zoomable"

/**
 * A full-screen area whose [content] can be pinched, dragged and double-tapped. With the content at its normal size,
 * dragging up or down pulls it away and lets go of the viewer ([onDismiss]); a plain tap calls [onTap].
 * [aspect] is the content's width over height, used to keep it from being dragged out of view.
 */
@Composable
fun ZoomableBox(
    aspect: Float,
    state: ZoomState,
    onTap: () -> Unit,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    var size by remember { mutableStateOf(IntSize.Zero) }
    val scope = rememberCoroutineScope()
    val frame = remember(size, aspect) {
        val (w, h) = ZoomMath.fitted(size.width.toFloat(), size.height.toFloat(), aspect)
        ZoomMath.Frame(size.width.toFloat(), size.height.toFloat(), w, h)
    }
    val currentFrame by rememberUpdatedState(frame)
    val tap by rememberUpdatedState(onTap)
    val dismiss by rememberUpdatedState(onDismiss)

    Box(
        modifier
            .fillMaxSize()
            .testTag(ZOOMABLE_TAG)
            .onSizeChanged { size = it }
            .drawBehind { drawRect(Color.Black.copy(alpha = ZoomMath.backdropAlpha(state.dragY, this.size.height.coerceAtLeast(1f)))) }
            .pointerInput(Unit) {
                detectTapGestures(
                    onTap = { tap() },
                    onDoubleTap = { at ->
                        scope.launch { state.animateTo(ZoomMath.doubleTap(state.zoom, at.x, at.y, currentFrame)) }
                    },
                )
            }
            .pointerInput(Unit) {
                viewerGestures(
                    onChange = { centroid, pan, zoomChange, fingers ->
                        val now = state.zoom
                        if (fingers >= 2 || !now.isFit) {
                            state.dragY = 0f
                            state.zoom = ZoomMath.transform(now, centroid.x, centroid.y, pan.x, pan.y, zoomChange, currentFrame)
                        } else {
                            state.dragY += pan.y
                        }
                    },
                    onEnd = {
                        if (ZoomMath.shouldDismiss(state.dragY, currentFrame.viewH)) dismiss() else scope.launch { state.settle() }
                    },
                )
            },
        contentAlignment = Alignment.Center,
    ) {
        Box(
            Modifier.fillMaxSize().graphicsLayer {
                val z = state.zoom
                scaleX = z.scale
                scaleY = z.scale
                translationX = z.x
                translationY = z.y + state.dragY
            },
        ) { content() }
    }
}

/**
 * Like the standard transform detector, but it only takes over the touch once the fingers have really moved,
 * so a plain tap or double tap still reaches the tap detector.
 */
private suspend fun PointerInputScope.viewerGestures(
    onChange: (centroid: Offset, pan: Offset, zoom: Float, fingers: Int) -> Unit,
    onEnd: () -> Unit,
) {
    awaitEachGesture {
        awaitFirstDown(requireUnconsumed = false)
        val slop = viewConfiguration.touchSlop
        var pastSlop = false
        var moved = false
        var zoomTotal = 1f
        var panTotal = Offset.Zero
        do {
            val event = awaitPointerEvent()
            val cancelled = event.changes.any { it.isConsumed }
            if (!cancelled) {
                val zoomChange = event.calculateZoom()
                val panChange = event.calculatePan()
                if (!pastSlop) {
                    zoomTotal *= zoomChange
                    panTotal += panChange
                    val zoomMotion = abs(1 - zoomTotal) * event.calculateCentroidSize(useCurrent = false)
                    if (zoomMotion > slop || panTotal.getDistance() > slop) pastSlop = true
                }
                if (pastSlop) {
                    moved = true
                    val fingers = event.changes.count { it.pressed && !it.changedToUp() }
                    if (zoomChange != 1f || panChange != Offset.Zero) {
                        onChange(event.calculateCentroid(useCurrent = false), panChange, zoomChange, fingers)
                    }
                    event.changes.forEach { if (it.positionChanged()) it.consume() }
                }
            }
        } while (!cancelled && event.changes.any { it.pressed })
        if (moved) onEnd()
    }
}
