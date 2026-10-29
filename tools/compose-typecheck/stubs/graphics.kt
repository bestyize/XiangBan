package androidx.compose.ui.graphics
@JvmInline value class Color(val v: Long) { constructor(v: Int) : this(v.toLong())
  fun copy(alpha: Float = 1f, red: Float = 0f, green: Float = 0f, blue: Float = 0f): Color = this
  companion object { val White = Color(0L); val Black = Color(0L); val Transparent = Color(0L) } }
open class Brush { companion object { fun linearGradient(colors: List<Color>): Brush = Brush()
  fun verticalGradient(colors: List<Color>): Brush = Brush()
  fun radialGradient(colors: List<Color>, center: androidx.compose.ui.geometry.Offset = androidx.compose.ui.geometry.Offset(0f, 0f), radius: Float = Float.POSITIVE_INFINITY): Brush = Brush() } }
class SolidColor(val value: Color) : Brush()
enum class StrokeCap { Butt, Round, Square }
enum class StrokeJoin { Miter, Round, Bevel }
interface Shape
class PathEffect { companion object { fun dashPathEffect(intervals: FloatArray, phase: Float = 0f): PathEffect = PathEffect() } }
interface GraphicsLayerScope { val size: androidx.compose.ui.geometry.Size; var translationX: Float; var scaleX: Float; var scaleY: Float; var alpha: Float; var rotationZ: Float }
fun androidx.compose.ui.Modifier.graphicsLayer(block: GraphicsLayerScope.() -> Unit): androidx.compose.ui.Modifier = this
