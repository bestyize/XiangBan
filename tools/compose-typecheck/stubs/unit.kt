package androidx.compose.ui.unit
data class IntOffset(val x: Int, val y: Int)
@JvmInline value class Dp(val value: Float) { operator fun plus(o: Dp) = Dp(value + o.value); operator fun times(o: Int) = Dp(value * o); operator fun times(o: Float) = Dp(value * o) }
val Int.dp: Dp get() = Dp(this.toFloat())
val Float.dp: Dp get() = Dp(this)
val Double.dp: Dp get() = Dp(this.toFloat())
@JvmInline value class TextUnit(val v: Float)
val Int.sp: TextUnit get() = TextUnit(this.toFloat())
val Float.sp: TextUnit get() = TextUnit(this)
val Double.sp: TextUnit get() = TextUnit(this.toFloat())
