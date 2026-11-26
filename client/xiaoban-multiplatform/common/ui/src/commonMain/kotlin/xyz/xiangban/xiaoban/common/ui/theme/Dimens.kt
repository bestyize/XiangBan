package xyz.xiangban.xiaoban.common.ui.theme

import androidx.compose.ui.unit.dp

/**
 * 手机端统一尺寸阶梯（与设计稿 ui-kit-overlays.html 及手机端设计规范对齐）。
 * 间距遵循 4dp 网格，圆角收敛为 6 档，避免各页面手写散落数值。
 */
object XDimens {
    // 圆角阶梯（6 档）
    val RadiusInner = 8.dp
    val RadiusSmall = 12.dp
    val RadiusCard = 16.dp
    val RadiusSheet = 20.dp
    val RadiusDialog = 24.dp

    // 间距网格（4dp 倍数）
    val Space4 = 4.dp
    val Space8 = 8.dp
    val Space12 = 12.dp
    val Space16 = 16.dp
    val Space24 = 24.dp

    // 控件与触控尺寸
    val TouchTarget = 44.dp
    val ButtonPrimaryHeight = 48.dp
    val ButtonHeight = 40.dp
    val ChipHeight = 34.dp
    val InputHeight = 48.dp
    val ActionSheetRowHeight = 52.dp
    val TopBarSubHeight = 48.dp
    val TopBarHomeHeight = 52.dp
    val BottomNavHeight = 56.dp

    // 图标四档与头像阶梯
    val IconExtraSmall = 14.dp
    val IconSmall = 16.dp
    val IconMedium = 20.dp
    val IconLarge = 24.dp
    val AvatarXS = 24.dp
    val AvatarSM = 32.dp
    val AvatarMD = 44.dp
    val AvatarLG = 68.dp
    val Hairline = 0.5.dp
}

/** 全局浮层层叠顺序（数值越大越靠上，Toast 永远高于弹窗）。 */
object XZIndex {
    const val Sticky = 10f
    const val Dock = 20f
    const val Popover = 30f
    const val Drawer = 40f
    const val Sheet = 50f
    const val Dialog = 60f
    const val Toast = 70f
}
