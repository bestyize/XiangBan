package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// 发现页里的小图表。都是手绘的 Canvas，不引第三方库；颜色由调用方传入，深浅色下都够对比度。

/** 数据蓝：热榜柱图的当前项、散点图的辅助线。 */
val DataBlue = Color(0xFF2F6BFF)

/** 数据柱图里非当前项的浅蓝。 */
val DataBlueSoft: Color get() = if (ThemeState.dark) Color(0xFF2F6BFF).copy(alpha = 0.35f) else Color(0xFFB9CCFF)

/** 悬赏积分的琥珀色文字。 */
val Amber: Color get() = if (ThemeState.dark) Color(0xFFE0A03A) else Color(0xFFB26A00)

/** 解析 `#RRGGBB`；解析不了用 [fallback]。 */
fun hexColor(hex: String, fallback: Color = Color(0xFF888888)): Color {
    val h = hex.trim().removePrefix("#")
    if (h.length != 6) return fallback
    val v = h.toLongOrNull(16) ?: return fallback
    return Color(0xFF000000L or v)
}

/** 厂商图标块的底色：浅色下用服务端给的浅底，深色下改成品牌色的半透明，免得一块亮底刺眼。 */
fun tintColor(brand: Color, tint: String): Color = if (ThemeState.dark) brand.copy(alpha = 0.22f) else hexColor(tint, brand.copy(alpha = 0.12f))

/** 一行横向柱：[fraction] 是柱长占轨道的比例，柱子从左往右长。 */
@Composable
fun HBar(fraction: Float, color: Color, modifier: Modifier = Modifier, height: Dp = 8.dp, track: Color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.06f)) {
    Box(modifier.fillMaxWidth().height(height).clip(RoundedCornerShape(height / 2)).background(track)) {
        Box(Modifier.fillMaxWidth(fraction.coerceIn(0.02f, 1f)).height(height).clip(RoundedCornerShape(height / 2)).background(color))
    }
}

/** 竖向小柱图（每日新增、重置间隔）。最后一根用 [highlight] 色。 */
@Composable
fun MiniBars(values: List<Double>, modifier: Modifier = Modifier, color: Color = DataBlueSoft, highlight: Color = DataBlue, gap: Dp = 6.dp) {
    Canvas(modifier) {
        if (values.isEmpty()) return@Canvas
        val peak = values.max().coerceAtLeast(1e-9)
        val g = gap.toPx()
        val w = (size.width - g * (values.size - 1)) / values.size
        values.forEachIndexed { i, v ->
            val h = (v / peak * size.height).toFloat().coerceAtLeast(3.dp.toPx())
            drawRoundRect(
                if (i == values.lastIndex) highlight else color,
                topLeft = Offset(i * (w + g), size.height - h), size = Size(w, h), cornerRadius = CornerRadius(3.dp.toPx()),
            )
        }
    }
}

/** 折线（7 日增量趋势），不画坐标轴。 */
@Composable
fun Sparkline(values: List<Long>, color: Color, modifier: Modifier = Modifier) {
    Canvas(modifier) {
        if (values.size < 2) return@Canvas
        val lo = values.min().toFloat()
        val hi = values.max().toFloat()
        val span = (hi - lo).coerceAtLeast(1f)
        val pad = 2.dp.toPx()
        val path = Path()
        values.forEachIndexed { i, v ->
            val x = i * size.width / (values.size - 1)
            val y = size.height - pad - (v - lo) / span * (size.height - 2 * pad)
            if (i == 0) path.moveTo(x, y) else path.lineTo(x, y)
        }
        drawPath(path, color, style = Stroke(width = 1.8.dp.toPx(), cap = StrokeCap.Round))
    }
}

/** 圆环进度。[pct] 为 0～100。 */
@Composable
fun ProgressRing(pct: Int, color: Color, modifier: Modifier = Modifier, size: Dp = 84.dp, stroke: Dp = 8.dp, track: Color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.08f)) {
    Canvas(modifier.size(size)) {
        val s = stroke.toPx()
        val inner = Size(this.size.width - s, this.size.height - s)
        drawArc(track, 0f, 360f, false, Offset(s / 2, s / 2), inner, style = Stroke(s))
        drawArc(color, -90f, 360f * pct.coerceIn(0, 100) / 100f, false, Offset(s / 2, s / 2), inner, style = Stroke(s, cap = StrokeCap.Round))
    }
}

/** 圆点（语言色、状态点）。 */
@Composable
fun Dot(color: Color, size: Dp = 8.dp, modifier: Modifier = Modifier) {
    Box(modifier.size(size).clip(CircleShape).background(color))
}
