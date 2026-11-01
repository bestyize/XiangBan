package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.core.animate
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

/** Drag deltas update placement immediately; only release/programmatic changes animate. */
@Composable
internal fun SwipeChatDrawer(visible: Boolean, onClose: () -> Unit, content: @Composable (Modifier) -> Unit) {
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val width = minOf(336.dp, maxWidth * 0.88f)
        val pixels = with(LocalDensity.current) { width.toPx() }
        val flingThreshold = with(LocalDensity.current) { 400.dp.toPx() }
        var offset by remember(pixels) { mutableFloatStateOf(-pixels) }
        var mounted by remember { mutableStateOf(false) }
        var settling by remember { mutableStateOf<Job?>(null) }
        val scope = rememberCoroutineScope()
        val close by rememberUpdatedState(onClose)
        fun settle(open: Boolean, velocity: Float = 0f, notify: Boolean = false) {
            settling?.cancel()
            settling = scope.launch {
                animate(offset, if (open) 0f else -pixels, initialVelocity = velocity,
                    animationSpec = spring(dampingRatio = 1f, stiffness = 500f)) { value, _ ->
                    offset = value.coerceIn(-pixels, 0f)
                }
                if (!open) {
                    mounted = false
                    if (notify) close()
                }
            }
        }
        LaunchedEffect(visible, pixels) {
            if (visible) mounted = true
            settle(visible)
        }
        DisposableEffect(Unit) { onDispose { settling?.cancel() } }
        if (mounted) {
            val drag = Modifier.draggable(
                state = rememberDraggableState { delta -> offset = (offset + delta).coerceIn(-pixels, 0f) },
                orientation = Orientation.Horizontal,
                startDragImmediately = settling?.isActive == true,
                onDragStarted = { settling?.cancel() },
                onDragStopped = { velocity ->
                    val open = when {
                        velocity < -flingThreshold -> false
                        velocity > flingThreshold -> true
                        else -> offset > -pixels * 0.5f
                    }
                    settle(open, velocity, notify = !open)
                },
            )
            Box(Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.3f * (1f + offset / pixels)))
                .clickable(indication = null, interactionSource = null) { close() }.then(drag))
            Box(Modifier.width(width).fillMaxHeight().offset { IntOffset(offset.roundToInt(), 0) }.then(drag)) {
                content(Modifier)
            }
        }
    }
}
