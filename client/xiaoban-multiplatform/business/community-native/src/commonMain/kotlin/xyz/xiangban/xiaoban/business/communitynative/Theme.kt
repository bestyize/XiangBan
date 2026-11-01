package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
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
import androidx.compose.ui.unit.sp
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

/**
 * 字号规范（见 docs/product/手机端设计规范.md 4.1）：六档字号，每档带固定行高，不加字距。
 * 页面里写 `style = XType.Body`，不直接写 fontSize；字重按需要另写 fontWeight。
 */
object XType {
    // 以 Material 的正文样式为底，保留平台字体设置，只去掉它默认的 0.5 字距。
    private val Base = Typography().bodyLarge.copy(letterSpacing = 0.sp)

    /** 22/30：首页问候语、“我的”页昵称、醒目的数字。 */
    val Headline = Base.copy(fontSize = 22.sp, lineHeight = 30.sp)
    /** 18/24：顶栏标题、一级 tab、弹层标题。 */
    val Title = Base.copy(fontSize = 18.sp, lineHeight = 24.sp)
    /** 16/22：分区标题、卡片标题。 */
    val Section = Base.copy(fontSize = 16.sp, lineHeight = 22.sp)
    /** 15/22：列表主文字、输入框、帖子摘要。 */
    val Body = Base.copy(fontSize = 15.sp, lineHeight = 22.sp)
    /** 13/18：说明、作者和时间、按钮小字。 */
    val Secondary = Base.copy(fontSize = 13.sp, lineHeight = 18.sp)
    /** 12/16：标签、计数、角标。 */
    val Caption = Base.copy(fontSize = 12.sp, lineHeight = 16.sp)
    /** 10/13：只给底部导航的文字用。 */
    val Nav = Base.copy(fontSize = 10.sp, lineHeight = 13.sp)
}

/** 没写 style 的文字和 Material 组件也落到这几档上。 */
private val XTypography = Typography(
    headlineSmall = XType.Headline, titleLarge = XType.Title, titleMedium = XType.Section,
    bodyLarge = XType.Body, bodyMedium = XType.Secondary, bodySmall = XType.Caption, labelSmall = XType.Caption,
)

/** 当前主题色的主色：主按钮、选中态、开关、发送键。在界面里直接读，换主题色时会自动刷新。 */
val Brand: Color get() = Color(ThemeState.palette.btn)

/** 出错、失败、拒绝、高风险用的红色，固定不跟主题色走。 */
val Danger: Color get() = Color(if (ThemeState.dark) Tokens.DangerDark else Tokens.Danger)

/** 出错、高风险、等待授权这类提醒卡片的浅红底。 */
val DangerSoft: Color get() = Color(if (ThemeState.dark) 0xFF3A2225L else 0xFFFFF3F4L)

/** 步骤完成、“已完成”的绿色。 */
val Success = Color(Tokens.Success)

/** 页面底 #FAFAFA、卡片纯白（深色下 #111111 和 #1C1C1E），主题色可换。 */
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
        MaterialTheme(colorScheme = scheme, typography = XTypography) {
            // Box backgrounds do not provide a content color like Material Surface does.
            // Set the default explicitly so Text without a color follows the active theme.
            CompositionLocalProvider(LocalContentColor provides scheme.onBackground, content = content)
        }
    }
}
