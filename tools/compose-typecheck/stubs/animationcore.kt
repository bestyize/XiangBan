package androidx.compose.animation.core

interface FiniteAnimationSpec<T>
class TweenSpec<T> : FiniteAnimationSpec<T>
interface Easing
val FastOutSlowInEasing: Easing = TODO()
fun <T> tween(durationMillis: Int = 300, delayMillis: Int = 0, easing: Easing = FastOutSlowInEasing): TweenSpec<T> = TODO()
