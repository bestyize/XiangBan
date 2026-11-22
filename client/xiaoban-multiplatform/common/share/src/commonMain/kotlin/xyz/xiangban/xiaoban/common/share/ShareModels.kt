package xyz.xiangban.xiaoban.common.share

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap

/** 分享卡片纸色风格：极简白 / 暖纸笺 / 墨夜。 */
enum class ShareCardStyle(val label: String) {
    White("极简白"),
    Warm("暖纸笺"),
    Ink("墨夜"),
}

/** 预览条左侧缩略图样式。 */
enum class ShareThumbKind {
    Text,
    Cover,
    Repo,
    Daily,
}

/** 卡片保存、分享或复制的执行结果。 */
enum class ShareActionResult {
    Saved,
    Shared,
    Copied,
    Cancelled,
    Unsupported,
    PermissionDenied,
    Failed,
}

/** 分享面板内置图标标识。 */
enum class ShareIconKey {
    SystemShare,
    CopyLink,
    Check,
    ShareCard,
    Sparkles,
    CopyText,
    Browser,
    Delete,
    Desktop,
    Code,
    Download,
    Send,
    Back,
    Close,
    Alert,
    FileText,
    Newspaper,
}

/** 外部调用方传入的纯数据：分享面板和分享卡片只需要这组字段。 */
@Immutable
data class ShareContent(
    val title: String,
    val url: String,
    val summary: String = "",
    val author: String = "",
    val dateLabel: String = "",
    val kindLabel: String = "社区内容",
    val meta: String = "",
    val badge: String = "网页直达",
    val coverUrl: String? = null,
    val thumbKind: ShareThumbKind = if (!coverUrl.isNullOrBlank()) ShareThumbKind.Cover else ShareThumbKind.Text,
    val thumbMonogram: String? = null,
) {
    /** 预览条副标题，未单独指定时自动组合。 */
    val displayMeta: String
        get() = meta.ifBlank {
            listOfNotNull(
                author.takeIf(String::isNotBlank),
                "公开链接",
                "浏览器免登录可看",
            ).joinToString(" · ")
        }

    /** 卡片底部展示的短链接。 */
    val displayUrl: String get() = shareDisplayUrl(url)
}

/** 分享卡片可调参数。 */
@Immutable
data class ShareCardOptions(
    val style: ShareCardStyle = ShareCardStyle.White,
    val showExcerpt: Boolean = true,
    val showQr: Boolean = true,
)

/** 场景化扩展操作（排在「分享卡片」后、「浏览器打开」前）。 */
data class ShareExtraAction(
    val id: String,
    val label: String,
    val icon: ShareIconKey,
    val accentDot: Boolean = false,
    val enabled: Boolean = true,
    val onClick: suspend () -> String?,
)

/** 作者主态删除操作：始终排在单行横滑轨最末尾，点击进入单次内联确认。 */
data class ShareDeleteAction(
    val label: String = "删除",
    val confirmTitle: String = "删除这篇内容？",
    val confirmMessage: String = "删除后内容及对应公开链接都将失效，无法恢复。",
    val confirmButton: String = "确认删除",
    val onConfirm: suspend () -> String?,
)

/** 分享数据的加载态：支持直接传入就绪数据，也支持由外部异步核验公开信息。 */
sealed interface ShareLoadState {
    data object Loading : ShareLoadState
    data class Ready(val content: ShareContent) : ShareLoadState
    data class Unavailable(val message: String, val canRetry: Boolean = true) : ShareLoadState
}

/** 面板与卡片使用的颜色集，默认可与宿主的深浅色和六套主题色对齐。 */
@Immutable
data class ShareSheetPalette(
    val dark: Boolean,
    val paper: Color,
    val surface: Color,
    val ink: Color,
    val ink2: Color,
    val ink3: Color,
    val hairline: Color,
    val fill: Color,
    val line: Color,
    val accent: Color,
    val accentSoft: Color,
    val accentSoftLine: Color,
    val accentInk: Color,
    val success: Color,
    val successSoft: Color,
    val successLine: Color,
    val danger: Color,
    val dangerSoft: Color,
) {
    companion object {
        fun defaults(
            dark: Boolean,
            accent: Color = if (dark) Color(0xFFEE4B5F) else Color(0xFFE3203C),
            accentSoft: Color = if (dark) Color(0xFF3A2225) else Color(0xFFFFF3F4),
            accentInk: Color = if (dark) Color(0xFFFF8C91) else Color(0xFFE3203C),
            accentSoftLine: Color = accent.copy(alpha = if (dark) 0.36f else 0.24f),
        ): ShareSheetPalette = if (dark) {
            ShareSheetPalette(
                dark = true,
                paper = Color(0xFF111111),
                surface = Color(0xFF1C1C1E),
                ink = Color(0xFFECECEC),
                ink2 = Color(0xFF9A9AA0),
                ink3 = Color(0xFF6E6E73),
                hairline = Color(0xFF2A2A2D),
                fill = Color(0xFF2C2C2E),
                line = Color(0xFF363639),
                accent = accent,
                accentSoft = accentSoft,
                accentSoftLine = accentSoftLine,
                accentInk = accentInk,
                success = Color(0xFF45BE83),
                successSoft = Color(0xFF1A3126),
                successLine = Color(0xFF28523D),
                danger = Color(0xFFFF6369),
                dangerSoft = Color(0xFF361E22),
            )
        } else {
            ShareSheetPalette(
                dark = false,
                paper = Color(0xFFFAFAFA),
                surface = Color(0xFFFFFFFF),
                ink = Color(0xFF222222),
                ink2 = Color(0xFF666666),
                ink3 = Color(0xFF9A9A9A),
                hairline = Color(0xFFEDEDED),
                fill = Color(0xFFF2F2F2),
                line = Color(0xFFE4E4E4),
                accent = accent,
                accentSoft = accentSoft,
                accentSoftLine = accentSoftLine,
                accentInk = accentInk,
                success = Color(0xFF2FA36B),
                successSoft = Color(0xFFEAF6F0),
                successLine = Color(0xFFBFE4D1),
                danger = Color(0xFFE3203C),
                dangerSoft = Color(0xFFFFF3F4),
            )
        }
    }
}

/** 宿主平台能力桥接：剪贴板、系统分享、外部浏览器与卡片渲染/导出。 */
interface SharePlatformBridge {
    val systemShareSupported: Boolean get() = true
    val cardExportSupported: Boolean get() = false
    val copyCardSupported: Boolean get() = false

    fun copyText(text: String)
    fun systemShare(title: String, url: String)
    fun openInBrowser(url: String)

    /** 可选：编码可扫描二维码点阵用于卡片实时预览。 */
    suspend fun encodeQrMatrix(text: String): List<BooleanArray>? = null

    /** 可选：将当前风格和开关配置渲染为高清分享卡片图片。 */
    suspend fun renderShareCard(content: ShareContent, options: ShareCardOptions): ImageBitmap? = null
    suspend fun saveShareCard(image: ImageBitmap): ShareActionResult = ShareActionResult.Unsupported
    suspend fun shareCardImage(image: ImageBitmap): ShareActionResult = ShareActionResult.Unsupported
    suspend fun copyCardImage(image: ImageBitmap): ShareActionResult = ShareActionResult.Unsupported
}

/** 单行横滑轨里的单个操作项。 */
internal enum class ShareTileTone { Default, Primary, Copied, Danger }

internal data class ShareRailItem(
    val id: String,
    val label: String,
    val icon: ShareIconKey,
    val tone: ShareTileTone = ShareTileTone.Default,
    val accentDot: Boolean = false,
    val enabled: Boolean = true,
)

/** 按设计稿固定顺序组装单行可横滑图标轨：主分享项 -> 场景扩展项 -> 浏览器打开 -> （主态末尾）删除。 */
internal fun buildShareRailItems(
    systemShareEnabled: Boolean,
    copied: Boolean,
    extraActions: List<ShareExtraAction>,
    deleteAction: ShareDeleteAction?,
): List<ShareRailItem> = buildList {
    add(
        ShareRailItem(
            id = "system",
            label = "系统分享",
            icon = ShareIconKey.SystemShare,
            tone = ShareTileTone.Primary,
            enabled = systemShareEnabled,
        ),
    )
    add(
        ShareRailItem(
            id = "copyLink",
            label = if (copied) "已复制" else "复制链接",
            icon = if (copied) ShareIconKey.Check else ShareIconKey.CopyLink,
            tone = if (copied) ShareTileTone.Copied else ShareTileTone.Default,
        ),
    )
    add(
        ShareRailItem(
            id = "openCard",
            label = "分享卡片",
            icon = ShareIconKey.ShareCard,
        ),
    )
    extraActions.forEach { extra ->
        add(
            ShareRailItem(
                id = "extra:${extra.id}",
                label = extra.label,
                icon = extra.icon,
                accentDot = extra.accentDot,
                enabled = extra.enabled,
            ),
        )
    }
    add(
        ShareRailItem(
            id = "browser",
            label = "浏览器打开",
            icon = ShareIconKey.Browser,
        ),
    )
    if (deleteAction != null) {
        add(
            ShareRailItem(
                id = "delete",
                label = deleteAction.label,
                icon = ShareIconKey.Delete,
                tone = ShareTileTone.Danger,
            ),
        )
    }
}

/** 去掉协议头与尾斜杠，用于卡片底部短链展示。 */
fun shareDisplayUrl(url: String): String =
    url.trim()
        .removePrefix("https://")
        .removePrefix("http://")
        .trimEnd('/')

internal fun shareActionNotice(result: ShareActionResult): String = when (result) {
    ShareActionResult.Saved -> "分享卡片已保存到相册"
    ShareActionResult.Shared -> "已唤起系统分享，可选择好友发送"
    ShareActionResult.Copied -> "卡片图片已复制到剪贴板"
    ShareActionResult.Cancelled -> "已取消操作"
    ShareActionResult.Unsupported -> "当前平台暂不支持卡片导出，可先复制链接"
    ShareActionResult.PermissionDenied -> "未获得保存授权，请重试或直接发送给好友"
    ShareActionResult.Failed -> "操作未完成，请重试"
}
