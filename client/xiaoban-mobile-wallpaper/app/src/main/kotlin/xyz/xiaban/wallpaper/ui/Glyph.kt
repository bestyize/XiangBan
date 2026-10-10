package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

enum class Glyph { Image, Video, Heart, User, Moon }

/** Rounded line icons drawn on a 24-unit grid, so they match each other in weight and corner. */
@Composable
fun GlyphIcon(glyph: Glyph, tint: Color, size: Dp = 24.dp) {
    val moon = remember { PathParser().parsePathString("M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z").toPath() }
    Canvas(Modifier.size(size)) {
        scale(this.size.minDimension / 24f, pivot = Offset.Zero) {
            val line = Stroke(width = 1.9f, cap = StrokeCap.Round, join = StrokeJoin.Round)
            when (glyph) {
                Glyph.Image -> {
                    drawRoundRect(tint, Offset(3f, 4f), Size(18f, 16f), CornerRadius(4.5f), style = line)
                    drawCircle(tint, 1.6f, Offset(9f, 9.5f), style = line)
                    val hills = Path().apply {
                        moveTo(3.5f, 17f); lineTo(9f, 12.8f); lineTo(12.6f, 15.6f); lineTo(15.6f, 13.2f); lineTo(20.5f, 17.2f)
                    }
                    drawPath(hills, tint, style = line)
                }
                Glyph.Video -> {
                    drawRoundRect(tint, Offset(3f, 5f), Size(18f, 14f), CornerRadius(5f), style = line)
                    val play = Path().apply {
                        moveTo(10.2f, 9.4f); lineTo(15f, 12f); lineTo(10.2f, 14.6f); close()
                    }
                    drawPath(play, tint, style = line)
                    drawPath(play, tint)
                }
                Glyph.Heart -> {
                    val heart = Path().apply {
                        moveTo(12f, 20.4f)
                        cubicTo(5f, 15.6f, 3f, 12.2f, 3f, 8.9f)
                        cubicTo(3f, 6.3f, 5f, 4.5f, 7.3f, 4.5f)
                        cubicTo(9.2f, 4.5f, 11f, 5.6f, 12f, 7.4f)
                        cubicTo(13f, 5.6f, 14.8f, 4.5f, 16.7f, 4.5f)
                        cubicTo(19f, 4.5f, 21f, 6.3f, 21f, 8.9f)
                        cubicTo(21f, 12.2f, 19f, 15.6f, 12f, 20.4f)
                        close()
                    }
                    drawPath(heart, tint, style = line)
                }
                Glyph.Moon -> drawPath(moon, tint, style = line)
                Glyph.User -> {
                    drawCircle(tint, 3.7f, Offset(12f, 8.4f), style = line)
                    val shoulders = Path().apply {
                        moveTo(4.6f, 20f)
                        cubicTo(4.6f, 16.3f, 7.9f, 14.3f, 12f, 14.3f)
                        cubicTo(16.1f, 14.3f, 19.4f, 16.3f, 19.4f, 20f)
                    }
                    drawPath(shoulders, tint, style = line)
                }
            }
        }
    }
}
