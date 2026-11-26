package xyz.xiangban.xiaoban.common.ui.theme

import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.Easing
import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.ui.unit.IntOffset

/**
 * 手机端统一动效规格：入场标准减速、退场更快收束、不做无意义弹性过冲。
 * 页面切换动画应仅作用于 graphicsLayer 位移，不触发重排布局。
 */
object XMotion {
    const val Fast = 140
    const val Normal = 200
    const val DialogEnter = 220
    const val DialogExit = 160
    const val SheetEnter = 260
    const val SheetExit = 200
    const val PagePush = 260
    const val PagePop = 240
    const val DrawerEnter = 240
    const val DrawerExit = 190
    const val ToastEnter = 180
    const val ToastExit = 150
    const val ToastHoldMs = 2400L

    /** 标准减速曲线（入场与页面切换），与 Web `--ease: cubic-bezier(0.22, 1, 0.36, 1)` 对齐。 */
    val StandardEasing: Easing = CubicBezierEasing(0.22f, 1f, 0.36f, 1f)

    /** 加速离场曲线（弹层收起），与 Web `--ease-exit: cubic-bezier(0.4, 0, 1, 1)` 对齐。 */
    val ExitEasing: Easing = CubicBezierEasing(0.4f, 0f, 1f, 1f)

    /** 抽屉 / 底部弹层手势松手归位用的无过冲临界阻尼弹簧。 */
    fun <T> gestureSpring(): FiniteAnimationSpec<T> = spring(dampingRatio = 1f, stiffness = 500f)

    fun pageFloatSpec(popping: Boolean = false): FiniteAnimationSpec<Float> =
        tween(if (popping) PagePop else PagePush, easing = StandardEasing)

    fun pageOffsetSpec(popping: Boolean = false): FiniteAnimationSpec<IntOffset> =
        tween(if (popping) PagePop else PagePush, easing = StandardEasing)

    fun sheetEnterSpec(): FiniteAnimationSpec<IntOffset> = tween(SheetEnter, easing = StandardEasing)
    fun sheetExitSpec(): FiniteAnimationSpec<IntOffset> = tween(SheetExit, easing = ExitEasing)
    fun drawerEnterSpec(): FiniteAnimationSpec<IntOffset> = tween(DrawerEnter, easing = StandardEasing)
    fun drawerExitSpec(): FiniteAnimationSpec<IntOffset> = tween(DrawerExit, easing = ExitEasing)
    fun scrimEnterSpec(): FiniteAnimationSpec<Float> = tween(Normal, easing = StandardEasing)
    fun scrimExitSpec(): FiniteAnimationSpec<Float> = tween(DialogExit, easing = ExitEasing)
}
