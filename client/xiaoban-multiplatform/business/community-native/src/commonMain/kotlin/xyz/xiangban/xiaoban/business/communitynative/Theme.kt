package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import xyz.xiangban.xiaoban.common.ui.theme.AppearanceMode as UiAppearanceMode
import xyz.xiangban.xiaoban.common.ui.theme.ExtraColors as UiExtraColors
import xyz.xiangban.xiaoban.common.ui.theme.ThemeState as UiThemeState
import xyz.xiangban.xiaoban.common.ui.theme.XType as UiXType
import xyz.xiangban.xiaoban.common.ui.theme.XiaobanTheme as UiXiaobanTheme

typealias AppearanceMode = UiAppearanceMode
typealias ThemeState = UiThemeState
typealias ExtraColors = UiExtraColors
typealias XType = UiXType

val LocalExtraColors get() = xyz.xiangban.xiaoban.common.ui.theme.LocalExtraColors
val Brand: Color get() = xyz.xiangban.xiaoban.common.ui.theme.Brand
val Danger: Color get() = xyz.xiangban.xiaoban.common.ui.theme.Danger
val DangerSoft: Color get() = xyz.xiangban.xiaoban.common.ui.theme.DangerSoft
val Success: Color get() = xyz.xiangban.xiaoban.common.ui.theme.Success

@Composable
fun XiaobanTheme(
    dark: Boolean = when (ThemeState.mode) {
        AppearanceMode.System -> androidx.compose.foundation.isSystemInDarkTheme()
        AppearanceMode.Light -> false
        AppearanceMode.Dark -> true
    },
    content: @Composable () -> Unit,
) = UiXiaobanTheme(dark = dark, content = content)
