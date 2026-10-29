package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.Color
import xyz.xiangban.xiaoban.common.community.DesignTokens as Tokens

/** 外观：跟随系统、浅色、深色。[id] 是存下来的值。 */
enum class AppearanceMode(val id: String, val label: String) {
    System("system", "跟随系统"), Light("light", "浅色"), Dark("dark", "深色");

    companion object {
        fun byId(id: String?): AppearanceMode = entries.firstOrNull { it.id == id } ?: System
    }
}

/**
 * 外观和主题色的当前选择。放在全局的可观察状态里，这样 [Brand] 这类颜色在哪儿读都能跟着变，
 * 不用把颜色一层层传下去。启动时由 [load] 从偏好里读回，之后每次修改都会存下来。
 */
object ThemeState {
    var mode by mutableStateOf(AppearanceMode.System)
        private set
    var accent by mutableStateOf(Tokens.Accents.first())
        private set

    /** 当前是不是深色（由 [XiaobanTheme] 按外观和系统设置算出来写入）。 */
    var dark by mutableStateOf(false)
        internal set

    private var prefs: AppPrefs? = null

    fun load(prefs: AppPrefs) {
        this.prefs = prefs
        mode = AppearanceMode.byId(prefs.get(KeyMode))
        accent = Tokens.accentById(prefs.get(KeyAccent))
    }

    fun select(mode: AppearanceMode) {
        this.mode = mode
        prefs?.put(KeyMode, mode.id)
    }

    fun select(accent: Tokens.Accent) {
        this.accent = accent
        prefs?.put(KeyAccent, accent.id)
    }

    val palette: Tokens.AccentPalette get() = if (dark) accent.dark else accent.light

    private const val KeyMode = "appearance"
    private const val KeyAccent = "accent"
}

/** Material 色板里没有的颜色：次要文字（时间、计数）、分隔线、浅灰填充、开关关闭色，以及主题色的浅底和浅底上的文字。 */
@Immutable
data class ExtraColors(val ink3: Color, val hairline: Color, val fill: Color, val line: Color, val brandSoft: Color, val brandInk: Color)

private val LightExtra = ExtraColors(Color(Tokens.Ink3Light), Color(Tokens.HairlineLight), Color(Tokens.FillLight), Color(Tokens.LineLight), Color(Tokens.BrandSoftLight), Color(Tokens.Brand))
private val DarkExtra = ExtraColors(Color(Tokens.Ink3Dark), Color(Tokens.HairlineDark), Color(Tokens.FillDark), Color(Tokens.LineDark), Color(Tokens.BrandSoftDark), Color(0xFFFF8C91L))

val LocalExtraColors = compositionLocalOf { LightExtra }

/** 当前主题色的主色：主按钮、选中态、开关、发送键。在界面里直接读，换主题色时会自动刷新。 */
val Brand: Color get() = Color(ThemeState.palette.btn)

/** 出错、失败、拒绝、高风险用的红色，固定不跟主题色走。 */
val Danger: Color get() = Color(if (ThemeState.dark) Tokens.DangerDark else Tokens.Danger)

/** 出错、高风险、等待授权这类提醒卡片的浅红底。 */
val DangerSoft: Color get() = Color(if (ThemeState.dark) 0xFF3A2225L else 0xFFFFF3F4L)

/** 步骤完成、“已完成”的绿色。 */
val Success = Color(Tokens.Success)

/** 页面底浅灰、卡片纯白（深色下 #111111 和 #1C1C1E），主题色可换。 */
@Composable
fun XiaobanTheme(dark: Boolean = when (ThemeState.mode) {
    AppearanceMode.System -> isSystemInDarkTheme()
    AppearanceMode.Light -> false
    AppearanceMode.Dark -> true
}, content: @Composable () -> Unit) {
    ThemeState.dark = dark
    val palette = ThemeState.palette
    val brand = Color(palette.btn)
    val scheme = if (dark) {
        darkColorScheme(
            primary = brand, onPrimary = Color.White,
            background = Color(Tokens.PaperDark), onBackground = Color(Tokens.InkDark),
            surface = Color(Tokens.SurfaceDark), onSurface = Color(Tokens.InkDark),
            surfaceVariant = Color(Tokens.SurfaceDark), onSurfaceVariant = Color(Tokens.Ink2Dark),
            outline = Color(Tokens.HairlineDark), outlineVariant = Color(Tokens.HairlineDark),
        )
    } else {
        lightColorScheme(
            primary = brand, onPrimary = Color.White,
            background = Color(Tokens.PaperLight), onBackground = Color(Tokens.InkLight),
            surface = Color(Tokens.SurfaceLight), onSurface = Color(Tokens.InkLight),
            surfaceVariant = Color(Tokens.SurfaceLight), onSurfaceVariant = Color(Tokens.Ink2Light),
            outline = Color(Tokens.HairlineLight), outlineVariant = Color(Tokens.HairlineLight),
        )
    }
    val base = if (dark) DarkExtra else LightExtra
    val extra = base.copy(brandSoft = Color(palette.soft), brandInk = Color(palette.ink))
    CompositionLocalProvider(LocalExtraColors provides extra) {
        MaterialTheme(colorScheme = scheme) {
            // Box backgrounds do not provide a content color like Material Surface does.
            // Set the default explicitly so Text without a color follows the active theme.
            CompositionLocalProvider(LocalContentColor provides scheme.onBackground, content = content)
        }
    }
}
