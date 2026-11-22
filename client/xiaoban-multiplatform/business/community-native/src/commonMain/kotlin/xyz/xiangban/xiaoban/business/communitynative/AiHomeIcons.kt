package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.dp

/** 从小伴 AI 首页设计稿的 SVG 提取，原稿使用 24 × 24 视口。 */
internal object AiHomeIcons {
    val Tasks by lazy { icon(
        "M7 5h10a3 3 0 0 1 3 3v8a3 3 0 0 1 -3 3h-10a3 3 0 0 1 -3 -3v-8a3 3 0 0 1 3 -3Z",
        "M8 10h8M8 14h5",
    ) }
    val More by lazy { icon(
        "M7.4 12a1.4 1.4 0 1 0 -2.8 0a1.4 1.4 0 1 0 2.8 0Z",
        "M13.4 12a1.4 1.4 0 1 0 -2.8 0a1.4 1.4 0 1 0 2.8 0Z",
        "M19.4 12a1.4 1.4 0 1 0 -2.8 0a1.4 1.4 0 1 0 2.8 0Z", filled = true,
    ) }
    val Back by lazy { icon("m15 6-6 6 6 6") }
    val Sliders by lazy { icon(
        "M4 8h9M17 8h3M4 16h3M11 16h9",
        "M17 8a2 2 0 1 0 -4 0a2 2 0 1 0 4 0Z",
        "M11 16a2 2 0 1 0 -4 0a2 2 0 1 0 4 0Z",
    ) }
    val SparkleLine by lazy { icon("M12 3l1.7 4.6L18.3 9.3l-4.6 1.7L12 15.6l-1.7-4.6-4.6-1.7 4.6-1.7Z") }
    val ChevRight by lazy { icon("m9 6 6 6-6 6") }
    val ArrowUp by lazy { icon("M12 19V5M6 11l6-6 6 6", width = 2f) }
    val ArrowUpRight by lazy { icon("M7 17 17 7M9 7h8v8") }
    val Image by lazy { icon(
        "M6 4h12a3 3 0 0 1 3 3v10a3 3 0 0 1 -3 3h-12a3 3 0 0 1 -3 -3v-10a3 3 0 0 1 3 -3Z",
        "M10.5 10a1.5 1.5 0 1 0 -3 0a1.5 1.5 0 1 0 3 0Z",
        "m21 16-5.5-5.5L7 19",
    ) }
    val Game by lazy { icon(
        "M7 7h10a4.5 4.5 0 0 1 4.5 4.5v2a4.5 4.5 0 0 1 -4.5 4.5h-10a4.5 4.5 0 0 1 -4.5 -4.5v-2a4.5 4.5 0 0 1 4.5 -4.5Z",
        "M7.5 12.5h3M9 11v3M15.5 11.5h.01M17.5 13.5h.01",
    ) }
    val Decor by lazy { icon(
        "M12 3l1.8 4.8L18.6 9.6l-4.8 1.8L12 16.2l-1.8-4.8-4.8-1.8 4.8-1.8Z",
        "M5 18.5h14",
    ) }
    val Pc by lazy { icon(
        "M5.5 4h13a2.5 2.5 0 0 1 2.5 2.5v7a2.5 2.5 0 0 1 -2.5 2.5h-13a2.5 2.5 0 0 1 -2.5 -2.5v-7a2.5 2.5 0 0 1 2.5 -2.5Z",
        "M8.5 20h7M12 16v4",
    ) }
    val Target by lazy { icon(
        "M20 12a8 8 0 1 0 -16 0a8 8 0 1 0 16 0Z",
        "M15 12a3 3 0 1 0 -6 0a3 3 0 1 0 6 0Z",
        "M12 4v3M20 12h-3M12 20v-3M4 12h3",
    ) }
    val Book by lazy { icon(
        "M5 4h11a2.5 2.5 0 0 1 2.5 2.5V20H7.5A2.5 2.5 0 0 1 5 17.5Z",
        "M5 17.5A2.5 2.5 0 0 1 7.5 15h11",
    ) }
    val Pen by lazy { icon("M4 20l4.2-1L19.5 7.7a2 2 0 0 0-2.8-2.8L5.4 16.2Z") }
    val Spark by lazy { icon("M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9Z") }
    val Mic by lazy { icon(
        "M12 3h0a3 3 0 0 1 3 3v5a3 3 0 0 1 -3 3h0a3 3 0 0 1 -3 -3v-5a3 3 0 0 1 3 -3Z",
        "M5 11a7 7 0 0 0 14 0M12 18v3",
    ) }
    val Wave by lazy { icon("M4 12v.01M8 8v8M12 5v14M16 8v8M20 12v.01", width = 2f) }
    val Plus by lazy { icon("M12 5v14M5 12h14") }
    val Grid by lazy { icon(
        "M5.6 4h3.3a1.6 1.6 0 0 1 1.6 1.6v3.3a1.6 1.6 0 0 1 -1.6 1.6h-3.3a1.6 1.6 0 0 1 -1.6 -1.6v-3.3a1.6 1.6 0 0 1 1.6 -1.6Z",
        "M15.1 4h3.3a1.6 1.6 0 0 1 1.6 1.6v3.3a1.6 1.6 0 0 1 -1.6 1.6h-3.3a1.6 1.6 0 0 1 -1.6 -1.6v-3.3a1.6 1.6 0 0 1 1.6 -1.6Z",
        "M5.6 13.5h3.3a1.6 1.6 0 0 1 1.6 1.6v3.3a1.6 1.6 0 0 1 -1.6 1.6h-3.3a1.6 1.6 0 0 1 -1.6 -1.6v-3.3a1.6 1.6 0 0 1 1.6 -1.6Z",
        "M15.1 13.5h3.3a1.6 1.6 0 0 1 1.6 1.6v3.3a1.6 1.6 0 0 1 -1.6 1.6h-3.3a1.6 1.6 0 0 1 -1.6 -1.6v-3.3a1.6 1.6 0 0 1 1.6 -1.6Z",
    ) }
    val Compass by lazy { icon(
        "M20.5 12a8.5 8.5 0 1 0 -17 0a8.5 8.5 0 1 0 17 0Z",
        "M15.2 8.8L13.4 13.4 8.8 15.2 10.6 10.6Z",
    ) }
    val User by lazy { icon(
        "M15.5 8.5a3.5 3.5 0 1 0 -7 0a3.5 3.5 0 1 0 7 0Z",
        "M5 19.5c1.2-3.4 3.8-5 7-5s5.8 1.6 7 5",
    ) }
    val Alert by lazy { icon(
        "M20.5 12a8.5 8.5 0 1 0 -17 0a8.5 8.5 0 1 0 17 0Z",
        "M12 8v4.5M12 16h.01",
    ) }
    val Check by lazy { icon("m5 12.5 4.2 4.2L19 7") }
    val Close by lazy { icon("M6 6l12 12M18 6 6 18") }
    val Search by lazy { icon(
        "M17.5 11a6.5 6.5 0 1 0 -13 0a6.5 6.5 0 1 0 13 0Z",
        "m20 20-4.2-4.2",
    ) }
    val Lock by lazy { icon(
        "M7 11h10a2 2 0 0 1 2 2v5a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-5a2 2 0 0 1 2 -2Z",
        "M8 11V8a4 4 0 0 1 8 0v3",
    ) }
    val Star by lazy { icon("m12 3.5 2.5 5.2 5.7.8-4.1 4 1 5.7L12 16.5l-5.1 2.7 1-5.7-4.1-4 5.7-.8Z") }
    val Cpu by lazy { icon(
        "M8 6h8a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-8a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2Z",
        "M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4",
    ) }
    val Vol by lazy { icon(
        "M11 6 6.5 9.5H4v5h2.5L11 18V6Z",
        "M15.5 9a4.5 4.5 0 0 1 0 6M18 6.5a8 8 0 0 1 0 11",
    ) }

    private fun icon(vararg paths: String, filled: Boolean = false, width: Float = 1.8f): ImageVector {
        val vector = ImageVector.Builder(defaultWidth = 24.dp, defaultHeight = 24.dp, viewportWidth = 24f, viewportHeight = 24f)
        val ink = SolidColor(Color.Black)
        paths.forEach { path ->
            vector.addPath(
                pathData = PathParser().parsePathString(path).toNodes(),
                fill = if (filled) ink else null,
                stroke = if (filled) null else ink,
                strokeLineWidth = width,
                strokeLineCap = StrokeCap.Round,
                strokeLineJoin = StrokeJoin.Round,
            )
        }
        return vector.build()
    }
}
