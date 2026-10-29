package androidx.compose.animation

import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.IntOffset

class EnterTransition
class ExitTransition
interface AnimatedVisibilityScope {
    fun Modifier.animateEnterExit(enter: EnterTransition = fadeIn(), exit: ExitTransition = fadeOut(), label: String = "animateEnterExit"): Modifier
}
@Composable fun AnimatedVisibility(visible: Boolean, modifier: Modifier = Modifier, enter: EnterTransition = fadeIn(), exit: ExitTransition = fadeOut(), label: String = "AnimatedVisibility", content: @Composable AnimatedVisibilityScope.() -> Unit) {}
fun fadeIn(animationSpec: FiniteAnimationSpec<Float> = TODO(), initialAlpha: Float = 0f): EnterTransition = TODO()
fun fadeOut(animationSpec: FiniteAnimationSpec<Float> = TODO(), targetAlpha: Float = 0f): ExitTransition = TODO()
fun slideInHorizontally(animationSpec: FiniteAnimationSpec<IntOffset> = TODO(), initialOffsetX: (Int) -> Int = { -it / 2 }): EnterTransition = TODO()
fun slideOutHorizontally(animationSpec: FiniteAnimationSpec<IntOffset> = TODO(), targetOffsetX: (Int) -> Int = { -it / 2 }): ExitTransition = TODO()
fun slideInVertically(animationSpec: FiniteAnimationSpec<IntOffset> = TODO(), initialOffsetY: (Int) -> Int = { -it / 2 }): EnterTransition = TODO()
fun slideOutVertically(animationSpec: FiniteAnimationSpec<IntOffset> = TODO(), targetOffsetY: (Int) -> Int = { -it / 2 }): ExitTransition = TODO()
