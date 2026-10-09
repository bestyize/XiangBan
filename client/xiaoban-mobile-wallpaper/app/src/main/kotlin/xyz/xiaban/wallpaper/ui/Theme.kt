package xyz.xiaban.wallpaper.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

/** Colours of the Xiaohongshu-like look: white cards on a pale page, one strong red. */
object Palette {
    val Red = Color(0xFFFF2442)
    val RedSoft = Color(0xFFFFE9EC)
    val Text = Color(0xFF333333)
    val Sub = Color(0xFF999999)
    val Hint = Color(0xFFC4C4C4)
    val Page = Color(0xFFF7F7F7)
    val Card = Color.White
    val Line = Color(0xFFEEEEEE)
    val Field = Color(0xFFF2F2F2)
}

@Composable
fun XiaobanTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = lightColorScheme(
            primary = Palette.Red,
            onPrimary = Color.White,
            background = Palette.Page,
            surface = Palette.Card,
            onSurface = Palette.Text,
            onBackground = Palette.Text,
            surfaceVariant = Palette.Field,
            outline = Palette.Line,
        ),
        content = content,
    )
}
