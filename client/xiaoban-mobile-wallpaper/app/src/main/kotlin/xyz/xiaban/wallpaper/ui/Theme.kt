package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

/** How the app picks light or dark: follow the phone, or stay fixed. */
enum class ThemeMode(val key: String, val label: String) {
    SYSTEM("system", "跟随系统"),
    LIGHT("light", "浅色"),
    DARK("dark", "深色");

    companion object {
        fun of(key: String?): ThemeMode = entries.firstOrNull { it.key == key } ?: SYSTEM
    }
}

@Composable
@ReadOnlyComposable
fun ThemeMode.isDark(): Boolean = when (this) {
    ThemeMode.SYSTEM -> isSystemInDarkTheme()
    ThemeMode.LIGHT -> false
    ThemeMode.DARK -> true
}

@Immutable
class AppColors(
    val brand: Color,
    val brandSoft: Color,
    val tag: Color,
    val tagSoft: Color,
    val text: Color,
    val sub: Color,
    val hint: Color,
    val page: Color,
    val card: Color,
    val line: Color,
    val field: Color,
    val shadow: Color,
)

/** Xiaohongshu's palette: one red, white cards on a light grey page, #333 text, #999 secondary text, blue only for #tags. */
private val LightColors = AppColors(
    brand = Color(0xFFFF2442),
    brandSoft = Color(0xFFFFE9EC),
    tag = Color(0xFF13386C),
    tagSoft = Color(0xFFE6EDF7),
    text = Color(0xFF333333),
    sub = Color(0xFF999999),
    hint = Color(0xFFC4C4C4),
    page = Color(0xFFF7F7F7),
    card = Color.White,
    line = Color(0xFFEEEEEE),
    field = Color(0xFFF2F2F2),
    shadow = Color(0x33000000),
)

/** The same roles on near-black: the red stays, surfaces step up in lightness instead of being shadowed. */
private val DarkColors = AppColors(
    brand = Color(0xFFFF2442),
    brandSoft = Color(0xFF3A1B22),
    tag = Color(0xFF7FB0EC),
    tagSoft = Color(0xFF1C2A3D),
    text = Color(0xFFE6E6E6),
    sub = Color(0xFF8C8C8C),
    hint = Color(0xFF5A5A5A),
    page = Color(0xFF0F0F0F),
    card = Color(0xFF1C1C1E),
    line = Color(0xFF2C2C2E),
    field = Color(0xFF2A2A2C),
    shadow = Color(0x66000000),
)

private val LocalAppColors = staticCompositionLocalOf { LightColors }

/** Every colour of the interface; each read follows the current theme. */
object Palette {
    val Brand: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.brand
    val BrandSoft: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.brandSoft
    val Heart: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.brand
    val HeartSoft: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.brandSoft
    val Tag: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.tag
    val Sky: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.tag
    val SkySoft: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.tagSoft
    val Text: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.text
    val Sub: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.sub
    val Hint: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.hint
    val Page: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.page
    val Card: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.card
    val Line: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.line
    val Field: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.field
    val Float: Color @Composable @ReadOnlyComposable get() = LocalAppColors.current.shadow
}

@Composable
fun XiaobanTheme(dark: Boolean, content: @Composable () -> Unit) {
    val colors = if (dark) DarkColors else LightColors
    val scheme = if (dark) darkColorScheme() else lightColorScheme()
    CompositionLocalProvider(LocalAppColors provides colors) {
        MaterialTheme(
            colorScheme = scheme.copy(
                primary = colors.brand,
                onPrimary = Color.White,
                background = colors.page,
                surface = colors.card,
                onSurface = colors.text,
                onBackground = colors.text,
                surfaceVariant = colors.field,
                surfaceContainerHigh = colors.card,
                outline = colors.line,
            ),
            content = content,
        )
    }
}
