package androidx.compose.ui.graphics.drawscope
import androidx.compose.ui.geometry.*
import androidx.compose.ui.graphics.*
import androidx.compose.ui.unit.Dp
interface DrawStyle
object Fill : DrawStyle
class Stroke(val width: Float = 0f, val miter: Float = 4f, val cap: StrokeCap = StrokeCap.Butt, val join: StrokeJoin = StrokeJoin.Miter, val pathEffect: PathEffect? = null) : DrawStyle
interface DrawScope {
    val center: Offset
    val size: Size
    fun Dp.toPx(): Float
    fun drawCircle(color: Color, radius: Float = 0f, center: Offset = this.center, alpha: Float = 1f, style: DrawStyle = Fill)
    fun drawCircle(brush: Brush, radius: Float = 0f, center: Offset = this.center, alpha: Float = 1f, style: DrawStyle = Fill)
    fun drawRoundRect(color: Color, topLeft: Offset = Offset(0f, 0f), size: Size = this.size, cornerRadius: CornerRadius = CornerRadius(0f, 0f), style: DrawStyle = Fill, alpha: Float = 1f)
    fun drawLine(color: Color, start: Offset, end: Offset, strokeWidth: Float = 0f, cap: StrokeCap = StrokeCap.Butt, alpha: Float = 1f)
}
fun DrawScope.rotate(degrees: Float, pivot: Offset = center, block: DrawScope.() -> Unit) {}
