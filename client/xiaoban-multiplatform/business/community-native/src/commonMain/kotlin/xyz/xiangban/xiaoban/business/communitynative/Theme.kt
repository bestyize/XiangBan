package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.graphics.Color
import xyz.xiangban.xiaoban.common.community.DesignTokens as Tokens

/** Material 色板里没有的两个灰度：次要文字（时间、计数）和发丝线。 */
@Immutable
data class ExtraColors(val ink3: Color, val hairline: Color)

val LocalExtraColors = compositionLocalOf { ExtraColors(Color(Tokens.Ink3Light), Color(Tokens.HairlineLight)) }

val Brand = Color(Tokens.Brand)

/** 取论坛前端同一套克制配色：黑白灰加品牌红，深浅色跟随系统。 */
@Composable
fun XiaobanTheme(dark: Boolean = isSystemInDarkTheme(), content: @Composable () -> Unit) {
    val scheme = if (dark) {
        darkColorScheme(
            primary = Brand, onPrimary = Color.White,
            background = Color(Tokens.PaperDark), onBackground = Color(Tokens.InkDark),
            surface = Color(Tokens.SurfaceDark), onSurface = Color(Tokens.InkDark),
            surfaceVariant = Color(Tokens.SurfaceDark), onSurfaceVariant = Color(Tokens.Ink2Dark),
            outline = Color(Tokens.HairlineDark), outlineVariant = Color(Tokens.HairlineDark),
        )
    } else {
        lightColorScheme(
            primary = Brand, onPrimary = Color.White,
            background = Color(Tokens.PaperLight), onBackground = Color(Tokens.InkLight),
            surface = Color(Tokens.SurfaceLight), onSurface = Color(Tokens.InkLight),
            surfaceVariant = Color(Tokens.SurfaceLight), onSurfaceVariant = Color(Tokens.Ink2Light),
            outline = Color(Tokens.HairlineLight), outlineVariant = Color(Tokens.HairlineLight),
        )
    }
    val extra = if (dark) ExtraColors(Color(Tokens.Ink3Dark), Color(Tokens.HairlineDark)) else ExtraColors(Color(Tokens.Ink3Light), Color(Tokens.HairlineLight))
    CompositionLocalProvider(LocalExtraColors provides extra) {
        MaterialTheme(colorScheme = scheme, content = content)
    }
}
