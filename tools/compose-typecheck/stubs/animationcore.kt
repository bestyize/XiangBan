package androidx.compose.animation.core

interface FiniteAnimationSpec<T>
interface DurationBasedAnimationSpec<T> : FiniteAnimationSpec<T>
class TweenSpec<T> : DurationBasedAnimationSpec<T>
class InfiniteRepeatableSpec<T>
enum class RepeatMode { Restart, Reverse }
val LinearEasing: Easing = TODO()
class InfiniteTransition
@androidx.compose.runtime.Composable fun rememberInfiniteTransition(label: String = "InfiniteTransition"): InfiniteTransition = TODO()
fun <T> infiniteRepeatable(animation: DurationBasedAnimationSpec<T>, repeatMode: RepeatMode = RepeatMode.Restart): InfiniteRepeatableSpec<T> = TODO()
@androidx.compose.runtime.Composable fun InfiniteTransition.animateFloat(initialValue: Float, targetValue: Float, animationSpec: InfiniteRepeatableSpec<Float>, label: String = "FloatAnimation"): androidx.compose.runtime.State<Float> = TODO()
interface Easing
val FastOutSlowInEasing: Easing = TODO()
fun <T> tween(durationMillis: Int = 300, delayMillis: Int = 0, easing: Easing = FastOutSlowInEasing): TweenSpec<T> = TODO()

@androidx.compose.runtime.Composable fun animateFloatAsState(targetValue: Float, animationSpec: FiniteAnimationSpec<Float> = TODO(), label: String = "FloatAnimation"): androidx.compose.runtime.State<Float> = TODO()
