package xyz.xiangban.xiaoban.common.community

/**
 * 设计规范变量（见 docs/product/手机端设计规范.md）：页面底浅灰、卡片纯白不描边，红色只给主按钮、选中态和提醒；
 * 深色页面 #111111、卡片 #1C1C1E、卡片里的次要块 #2C2C2E。主题色可换（[Accents]），成功绿和危险红固定不跟主题色走。
 * 颜色用 ARGB 的 Long，避免核心模块依赖 Compose；界面层再转成 Color。
 */
object DesignTokens {
    // 颜色（浅色 / 深色）
    const val Brand = 0xFFE3203CL
    const val PaperLight = 0xFFEDEDEDL
    const val SurfaceLight = 0xFFFFFFFFL
    const val InkLight = 0xFF222222L
    const val Ink2Light = 0xFF666666L
    const val Ink3Light = 0xFF9A9A9AL
    const val HairlineLight = 0xFFEDEDEDL
    /** 浅灰填充（图标底、输入框底）、描边、品牌红的浅底与它的描边。 */
    const val FillLight = 0xFFF2F2F2L
    const val LineLight = 0xFFCFCFCFL
    const val BrandSoftLight = 0xFFFFF3F4L
    const val BrandSoftLineLight = 0xFFF5C7CDL

    const val PaperDark = 0xFF111111L
    const val SurfaceDark = 0xFF1C1C1EL
    const val InkDark = 0xFFECECECL
    const val Ink2Dark = 0xFF9A9AA0L
    const val Ink3Dark = 0xFF6E6E73L
    const val HairlineDark = 0xFF2E2E30L
    const val FillDark = 0xFF2C2C2EL
    const val LineDark = 0xFF3A3A3CL
    const val BrandSoftDark = 0xFF3A1F23L
    const val BrandSoftLineDark = 0xFF5A2B33L

    /** 步骤完成、“完成”的绿色；失败、拒绝、出错的红色（深色下提亮）。这两个不跟主题色走。 */
    const val Success = 0xFF2FA36BL
    const val Danger = 0xFFE3203CL
    const val DangerDark = 0xFFFF6369L

    // 尺寸（dp / sp）
    /** 卡片里的内嵌块圆角；大卡片和弹层用 [RadiusLarge]，按钮药丸全圆。 */
    const val RadiusCard = 8f
    const val RadiusLarge = 16f
    const val Hairline = 0.5f
    const val GutterX = 16f
    const val TitleSize = 15f
    const val MetaSize = 12f
    const val BodySize = 15f

    /** 一套主题色的四个角色：主色（主按钮、选中、开关）、渐变高光、浅底（“允许一次”这类主操作的底）、浅底上的文字与链接。 */
    class AccentPalette(val btn: Long, val hi: Long, val soft: Long, val ink: Long)

    class Accent(val id: String, val name: String, val light: AccentPalette, val dark: AccentPalette)

    /** 六套主题色，沿用桌面端；第一套是默认的小伴红。 */
    val Accents: List<Accent> = listOf(
        Accent("red", "小伴红", AccentPalette(0xFFE3203CL, 0xFFF0506AL, 0xFFFFF3F4L, 0xFFE3203CL), AccentPalette(0xFFEE4B5FL, 0xFFF76B7DL, 0xFF3A2225L, 0xFFFF8C91L)),
        Accent("green", "薄荷绿", AccentPalette(0xFF2E8B5EL, 0xFF3FA172L, 0xFFE3F2E9L, 0xFF1E6A45L), AccentPalette(0xFF34A36DL, 0xFF48B981L, 0xFF1D3328L, 0xFF72D6A2L)),
        Accent("blue", "海湾蓝", AccentPalette(0xFF2F6FDBL, 0xFF4A84E6L, 0xFFE6EEFCL, 0xFF1D53AEL), AccentPalette(0xFF3B7BE8L, 0xFF5A92F0L, 0xFF1E2A42L, 0xFF8DB6FFL)),
        Accent("purple", "暮紫", AccentPalette(0xFF7152D8L, 0xFF8669E3L, 0xFFEDE8FCL, 0xFF5537B8L), AccentPalette(0xFF8466E6L, 0xFF9A80EEL, 0xFF2A2342L, 0xFFBBA8FFL)),
        Accent("amber", "琥珀", AccentPalette(0xFFA8640CL, 0xFFBD7A1CL, 0xFFFAEEDBL, 0xFF87500AL), AccentPalette(0xFFC27A1AL, 0xFFD69130L, 0xFF3A2C17L, 0xFFF0B65AL)),
        Accent("graphite", "石墨", AccentPalette(0xFF2B2B30L, 0xFF4A4A50L, 0xFFE9E9ECL, 0xFF2B2B30L), AccentPalette(0xFF636368L, 0xFF7C7C82L, 0xFF3A3A3DL, 0xFFE5E5EAL)),
    )

    /** 按存下来的编号找主题色；找不到（旧版本、被改坏）就回到默认的小伴红。 */
    fun accentById(id: String?): Accent = Accents.firstOrNull { it.id == id } ?: Accents.first()

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
