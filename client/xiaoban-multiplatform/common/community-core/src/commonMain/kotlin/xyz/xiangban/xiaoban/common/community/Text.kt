package xyz.xiangban.xiaoban.common.community

/** 内容类型的中文名，顺序就是信息流二级 Tab 的顺序。 */
val PackKinds: List<Pair<String, String>> = listOf(
    "effect" to "特效", "wallpaper" to "壁纸", "widget" to "组件", "game" to "游戏",
    "reminder" to "提醒", "routine" to "例行", "task" to "任务", "goal" to "目标",
    "skill" to "技能", "expert" to "专家", "workflow" to "工作流",
)

fun kindLabel(kind: String): String = PackKinds.firstOrNull { it.first == kind }?.second ?: kind

/** 视觉类内容用双列封面卡片，文字为主的能力类用单列。 */
val VisualKinds = setOf("effect", "wallpaper", "widget", "game")

private val tag = Regex("<[^>]*>")
private val blankLines = Regex("\n{3,}")

/**
 * 把服务端渲染好的 HTML 正文转成纯文字显示（手机原生页先不嵌 WebView）：段落和换行保留，去掉标签，还原常见实体。
 * 服务端的 HTML 已经过净化，这里只做展示用的转换，不依赖它来保证安全。
 */
fun htmlToPlain(html: String): String =
    html.replace(Regex("(?i)<br\\s*/?>"), "\n")
        .replace(Regex("(?i)</(p|div|li|h[1-6]|pre|blockquote)>"), "\n\n")
        .replace(Regex("(?i)<li[^>]*>"), "· ")
        .replace(tag, "")
        .replace("&nbsp;", " ").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&amp;", "&")
        .replace(blankLines, "\n\n")
        .trim()

fun formatSize(bytes: Long): String = when {
    bytes < 1024 -> "$bytes B"
    bytes < 1024 * 1024 -> "${(bytes * 10 / 1024) / 10.0} KB"
    else -> "${(bytes * 10 / (1024 * 1024)) / 10.0} MB"
}

/** 下载大小：MB 以上取整数（12 MB），KB 以上也取整数；不到 1 KB 写字节。 */
fun formatDownloadSize(bytes: Long): String = when {
    bytes < 1024 -> "$bytes B"
    bytes < 1024 * 1024 -> "${(bytes + 512) / 1024} KB"
    else -> "${(bytes + 512 * 1024) / (1024 * 1024)} MB"
}
