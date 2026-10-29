package androidx.compose.animation

import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.Alignment
import androidx.compose.ui.unit.IntOffset

class EnterTransition { operator fun plus(other: EnterTransition): EnterTransition = TODO() }
class ExitTransition { operator fun plus(other: ExitTransition): ExitTransition = TODO() }
class ContentTransform
infix fun EnterTransition.togetherWith(exit: ExitTransition): ContentTransform = TODO()
interface AnimatedVisibilityScope {
    fun Modifier.animateEnterExit(enter: EnterTransition = fadeIn(), exit: ExitTransition = fadeOut(), label: String = "animateEnterExit"): Modifier
}
@Composable fun AnimatedVisibility(visible: Boolean, modifier: Modifier = Modifier, enter: EnterTransition = fadeIn(), exit: ExitTransition = fadeOut(), label: String = "AnimatedVisibility", content: @Composable AnimatedVisibilityScope.() -> Unit) {}
interface AnimatedContentScope : AnimatedVisibilityScope
interface AnimatedContentTransitionScope<S> { val initialState: S; val targetState: S }
@Composable fun <S> AnimatedContent(targetState: S, modifier: Modifier = Modifier, transitionSpec: AnimatedContentTransitionScope<S>.() -> ContentTransform = { TODO() }, contentAlignment: Alignment = Alignment.TopStart, label: String = "AnimatedContent", contentKey: (S) -> Any? = { it }, content: @Composable AnimatedContentScope.(S) -> Unit) {}
fun fadeIn(animationSpec: FiniteAnimationSpec<Float> = TODO(), initialAlpha: Float = 0f): EnterTransition = TODO()
fun fadeOut(animationSpec: FiniteAnimationSpec<Float> = TODO(), targetAlpha: Float = 0f): ExitTransition = TODO()
fun slideInHorizontally(animationSpec: FiniteAnimationSpec<IntOffset> = TODO(), initialOffsetX: (Int) -> Int = { -it / 2 }): EnterTransition = TODO()
fun slideOutHorizontally(animationSpec: FiniteAnimationSpec<IntOffset> = TODO(), targetOffsetX: (Int) -> Int = { -it / 2 }): ExitTransition = TODO()
fun slideInVertically(animationSpec: FiniteAnimationSpec<IntOffset> = TODO(), initialOffsetY: (Int) -> Int = { -it / 2 }): EnterTransition = TODO()
fun slideOutVertically(animationSpec: FiniteAnimationSpec<IntOffset> = TODO(), targetOffsetY: (Int) -> Int = { -it / 2 }): ExitTransition = TODO()
