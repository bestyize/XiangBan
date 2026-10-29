package androidx.compose.ui.graphics.vector
import androidx.compose.ui.graphics.*
import androidx.compose.ui.unit.Dp
class PathNode
fun addPathNodes(pathStr: String?): List<PathNode> = TODO()
class ImageVector { class Builder(name: String = "", defaultWidth: Dp, defaultHeight: Dp, viewportWidth: Float, viewportHeight: Float) {
  fun addPath(pathData: List<PathNode>, name: String = "", fill: Brush? = null, fillAlpha: Float = 1f, stroke: Brush? = null, strokeAlpha: Float = 1f, strokeLineWidth: Float = 0f, strokeLineCap: StrokeCap = StrokeCap.Butt, strokeLineJoin: StrokeJoin = StrokeJoin.Miter, strokeLineMiter: Float = 4f): Builder = this
  fun build(): ImageVector = ImageVector() } }
