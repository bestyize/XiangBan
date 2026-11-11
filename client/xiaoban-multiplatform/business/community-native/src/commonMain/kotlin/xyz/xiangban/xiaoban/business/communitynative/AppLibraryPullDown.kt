package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animate
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.*
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.hideFromAccessibility
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlin.math.abs

/** Reveal the library behind the plaza using the vertical drag left over at a scroll edge. */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
internal fun AppLibraryPullDown(
    enabled: Boolean,
    onVisibilityChanged: (Boolean) -> Unit,
    drawer: @Composable (dismiss: () -> Unit) -> Unit,
    content: @Composable (show: () -> Unit, dragHandle: Modifier) -> Unit,
) {
    var reveal by remember { mutableFloatStateOf(0f) }
    var expanded by remember { mutableStateOf(false) }
    var animation by remember { mutableStateOf<Job?>(null) }
    val showing by remember { derivedStateOf { expanded || reveal > 0f } }
    val scope = rememberCoroutineScope()
    val onVisible by rememberUpdatedState(onVisibilityChanged)
    LaunchedEffect(showing) { onVisible(showing) }
    DisposableEffect(Unit) { onDispose { onVisible(false) } }
    LaunchedEffect(enabled) {
        if (!enabled) {
            animation?.cancel()
            expanded = false
            reveal = 0f
        }
    }
    fun settle(open: Boolean) {
        animation?.cancel()
        expanded = open
        animation = scope.launch {
            animate(reveal, if (open) 1f else 0f, animationSpec = tween(230, easing = FastOutSlowInEasing)) { value, _ ->
                reveal = value
            }
        }
    }
    BackHandler(enabled = enabled && showing) { settle(false) }

    BoxWithConstraints(Modifier.fillMaxSize().clipToBounds()) {
        val height = maxHeight * .78f
        val pixels = with(LocalDensity.current) { height.toPx().coerceAtLeast(1f) }
        val threshold = with(LocalDensity.current) { (64.dp.toPx() / pixels).coerceAtMost(.4f) }
        fun drag(delta: Float): Float {
            animation?.cancel()
            val previous = reveal
            val resistance = if (expanded) 1f else .72f
            reveal = (reveal + delta * resistance / pixels).coerceIn(0f, 1f)
            return (reveal - previous) * pixels / resistance
        }
        fun release() {
            settle(if (expanded) reveal > 1f - threshold else reveal >= threshold)
        }
        val scroll = remember(enabled, pixels, threshold) {
            object : NestedScrollConnection {
                private var dragging = false
                override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset {
                    if (!enabled || !dragging || source != NestedScrollSource.UserInput) return Offset.Zero
                    val used = drag(available.y)
                    if (reveal == (if (expanded) 1f else 0f)) dragging = false
                    return Offset(0f, used)
                }
                override fun onPostScroll(consumed: Offset, available: Offset, source: NestedScrollSource): Offset {
                    if (!enabled || source != NestedScrollSource.UserInput ||
                        abs(consumed.x + available.x) > abs(consumed.y + available.y)) return Offset.Zero
                    val towardOtherEdge = if (expanded) available.y < 0f else available.y > 0f
                    if (!towardOtherEdge) return Offset.Zero
                    val used = drag(available.y)
                    if (used != 0f) dragging = true
                    return Offset(0f, used)
                }
                override suspend fun onPreFling(available: Velocity): Velocity {
                    if (!dragging) return Velocity.Zero
                    dragging = false
                    release()
                    return Velocity(0f, available.y)
                }
            }
        }
        val dragHandle = Modifier.draggable(
            state = rememberDraggableState { drag(it) },
            orientation = Orientation.Vertical,
            enabled = enabled,
            onDragStopped = { release() },
        )
        Box(Modifier.fillMaxSize().nestedScroll(scroll)) {
            if (showing) Box(Modifier.fillMaxWidth().height(height)) { drawer { settle(false) } }
            Box(Modifier.fillMaxSize().graphicsLayer { translationY = reveal * pixels }) {
                Box(Modifier.fillMaxSize().semantics { if (showing) hideFromAccessibility() }) {
                    content({ if (enabled) settle(true) }, dragHandle)
                }
                if (showing) Box(
                    Modifier.fillMaxSize().graphicsLayer { alpha = reveal }
                        .background(MaterialTheme.colorScheme.onSurface.copy(alpha = .12f))
                        .then(dragHandle).semantics { contentDescription = "收起最近用过" }
                        .clickable(role = Role.Button, indication = null, interactionSource = null) { settle(false) },
                )
            }
        }
    }
}
