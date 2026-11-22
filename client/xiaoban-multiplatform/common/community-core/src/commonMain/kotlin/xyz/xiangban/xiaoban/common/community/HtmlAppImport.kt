package xyz.xiangban.xiaoban.common.community

import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

private const val ReadyScript = """<script>
document.addEventListener('DOMContentLoaded', () => {
  requestAnimationFrame(() => window.xb?.ready?.());
}, { once: true });
</script>
"""

val MaxImportedHtmlBytes = MobileGameFormat.FILE_BYTES - ReadyScript.length

private const val Icon = """<svg xmlns="http://www.w3.org/2000/svg" width="192" height="192" viewBox="0 0 192 192">
  <rect width="192" height="192" rx="36" fill="#324475"/>
  <path d="M70 71 43 96l27 25m52-50 27 25-27 25m-17-57-18 64" fill="none" stroke="#e4ebff" stroke-width="10" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"""

/** Installs a local document using the same validation, storage and export path as a created work. */
suspend fun MobileGameStore.importHtml(file: ForumUploadFile): MobileGame {
    val (game, files) = prepareHtmlApp(file)
    return create(game, files)
}

/** Prepares a protocol-2 archive without installing or executing the selected page. */
fun forumHtmlWorkFiles(file: ForumUploadFile): Pair<MobileGame, List<MobileGameFile>> {
    val (game, files) = prepareHtmlApp(file)
    MobileGameFormat.files(files)
    return game to (listOf(MobileGameFile("manifest.json", game.json().toString())) + files)
}

@OptIn(ExperimentalUuidApi::class)
private fun prepareHtmlApp(file: ForumUploadFile): Pair<MobileGame, List<MobileGameFile>> {
    val filename = file.name.substringAfterLast('/').substringAfterLast('\\')
    require(filename.substringAfterLast('.', "").lowercase() in setOf("html", "htm")) { "请选择 .html 或 .htm 文件" }
    require(file.bytes.size <= MaxImportedHtmlBytes) { "单个 HTML 文件不能超过 1 MB" }
    val html = try {
        file.bytes.decodeToString(throwOnInvalidSequence = true)
    } catch (_: CharacterCodingException) {
        throw IllegalArgumentException("请选择 UTF-8 编码的 HTML 文件")
    }
    require(html.removePrefix("\uFEFF").isNotBlank() && '\u0000' !in html) { "请选择非空的 UTF-8 HTML 文件" }
    var count = 0
    val name = filename.substringBeforeLast('.').filter { !it.isISOControl() && it != '\uFFFE' && it != '\uFFFF' }
        .trim().takeWhile {
            if (!it.isLowSurrogate()) count++
            count <= 24
        }.trimEnd().ifEmpty { "HTML 作品" }
    val id = "html-" + Uuid.random().toHexString()
    val title = name.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
    val cover = """<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360" viewBox="0 0 640 360">
  <rect width="640" height="360" fill="#324475"/>
  <path d="M286 106 250 140l36 34m68-68 36 34-36 34m-22-78-24 88" fill="none" stroke="#e4ebff" stroke-width="12" stroke-linecap="round" stroke-linejoin="round"/>
  <text x="320" y="265" fill="#fff" font-family="sans-serif" font-size="23" text-anchor="middle">$title</text>
</svg>"""
    val offset = prologEnd(html)
    val game = MobileGameFormat.metadata(MobileGame(id, name, "从 HTML 导入的作品", orientation = "any", kind = "app", category = "tool"))
    return game to listOf(
        MobileGameFile("index.html", html.substring(0, offset) + ReadyScript + html.substring(offset)),
        MobileGameFile("icon.svg", Icon),
        MobileGameFile("cover.svg", cover),
    )
}

private fun Char?.isHtmlSpace() = this == ' ' || this == '\t' || this == '\n' || this == '\r' || this == '\u000C'

// Keep the author's doctype and head attributes. Register before the page's own scripts.
private fun prologEnd(html: String): Int {
    var offset = if (html.startsWith('\uFEFF')) 1 else 0
    val tags = listOf("<!doctype", "<html", "<head")
    while (true) {
        while (html.getOrNull(offset).isHtmlSpace()) offset++
        if (html.startsWith("<!--", offset)) {
            val end = html.indexOf("-->", offset + 4)
            if (end < 0) return offset
            offset = end + 3
            continue
        }
        val tag = tags.firstOrNull {
            if (!html.startsWith(it, offset, ignoreCase = true)) return@firstOrNull false
            val next = html.getOrNull(offset + it.length)
            next.isHtmlSpace() || next == '>' || next == '/'
        } ?: return offset
        var quote: Char? = null
        var end = offset + tag.length
        while (end < html.length) {
            val ch = html[end++]
            if (quote != null) {
                if (ch == quote) quote = null
            } else if (ch == '\'' || ch == '"') {
                quote = ch
            } else if (ch == '>') {
                break
            }
        }
        if (quote != null || html.getOrNull(end - 1) != '>') return offset
        offset = end
    }
}
