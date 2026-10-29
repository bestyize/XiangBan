package androidx.compose.ui.unit
@JvmInline value class Dp(val value: Float) { operator fun plus(o: Dp) = Dp(value + o.value) }
val Int.dp: Dp get() = Dp(this.toFloat())
val Float.dp: Dp get() = Dp(this)
val Double.dp: Dp get() = Dp(this.toFloat())
@JvmInline value class TextUnit(val v: Float)
val Int.sp: TextUnit get() = TextUnit(this.toFloat())
val Float.sp: TextUnit get() = TextUnit(this)
val Double.sp: TextUnit get() = TextUnit(this.toFloat())
