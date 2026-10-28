package xyz.xiangban.xiaoban.common.community

/**
 * 设计规范变量（原型的视觉规则，克制配色）：黑白灰加品牌红，纯文字标签，图片优先的渐变封面，
 * 通栏白底列表加 0.5dp 发丝线，卡片圆角 8dp。数值与论坛前端 `base.css` 保持一致。
 * 颜色用 ARGB 的 Long，避免核心模块依赖 Compose；界面层再转成 Color。
 */
object DesignTokens {
    // 颜色（浅色 / 深色）
    const val Brand = 0xFFE3203CL
    const val PaperLight = 0xFFF5F5F5L
    const val SurfaceLight = 0xFFFFFFFFL
    const val InkLight = 0xFF222222L
    const val Ink2Light = 0xFF666666L
    const val Ink3Light = 0xFF999999L
    const val HairlineLight = 0xFFEDEDEDL

    const val PaperDark = 0xFF111111L
    const val SurfaceDark = 0xFF1B1B1BL
    const val InkDark = 0xFFF5F5F5L
    const val Ink2Dark = 0xFFCCCCCCL
    const val Ink3Dark = 0xFFA3A3A3L
    const val HairlineDark = 0xFF333333L

    // 尺寸（dp / sp）
    const val RadiusCard = 8f
    const val Hairline = 0.5f
    const val GutterX = 16f
    const val TitleSize = 15f
    const val MetaSize = 12f
    const val BodySize = 15f

    /** 没有封面图时的占位渐变，按内容编号轮换，同一内容永远是同一组。 */
    val CoverGradients: List<Pair<Long, Long>> = listOf(
        0xFF2B2B2EL to 0xFF5A5A60L,
        0xFF3A3A3FL to 0xFF8D8D93L,
        0xFF1F2933L to 0xFF52606DL,
        0xFF44403CL to 0xFFA8A29EL,
        0xFF262626L to 0xFF737373L,
    )

    fun coverGradient(id: Long): Pair<Long, Long> = CoverGradients[(id % CoverGradients.size).toInt().let { if (it < 0) it + CoverGradients.size else it }]
}
