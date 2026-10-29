package androidx.compose.ui.graphics
@JvmInline value class Color(val v: Long) { constructor(v: Int) : this(v.toLong())
  fun copy(alpha: Float = 1f, red: Float = 0f, green: Float = 0f, blue: Float = 0f): Color = this
  companion object { val White = Color(0L); val Black = Color(0L); val Transparent = Color(0L) } }
open class Brush { companion object { fun linearGradient(colors: List<Color>): Brush = Brush() } }
class SolidColor(val value: Color) : Brush()
enum class StrokeCap { Butt, Round, Square }
enum class StrokeJoin { Miter, Round, Bevel }
interface Shape
