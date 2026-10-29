package xyz.xiangban.xiaoban.common.community

// 聊天回答里的 Markdown：标题、列表、引用、代码块、粗体、斜体、行内代码、链接。
// 表格按等宽文字显示（不画表格线）。不支持的写法原样显示，不会丢字。

sealed interface MdInline {
    data class Text(val text: String) : MdInline
    data class Bold(val text: String) : MdInline
    data class Italic(val text: String) : MdInline
    data class Code(val text: String) : MdInline
    data class Link(val label: String, val url: String) : MdInline
}

sealed interface MdBlock {
    data class Heading(val level: Int, val inlines: List<MdInline>) : MdBlock
    data class Paragraph(val inlines: List<MdInline>) : MdBlock
    data class Bullets(val items: List<List<MdInline>>) : MdBlock
    data class Numbered(val start: Int, val items: List<List<MdInline>>) : MdBlock
    data class Quote(val inlines: List<MdInline>) : MdBlock
    data class Code(val language: String, val text: String) : MdBlock
    data object Rule : MdBlock
}

private val BulletRe = Regex("""^\s{0,3}[-*+]\s+(.*)$""")
private val NumberRe = Regex("""^\s{0,3}(\d{1,3})[.)]\s+(.*)$""")
private val HeadingRe = Regex("""^(#{1,6})\s+(.*?)\s*#*\s*$""")
private val RuleRe = Regex("""^\s{0,3}([-*_])(\s*\1){2,}\s*$""")
private val FenceRe = Regex("""^\s{0,3}(```+|~~~+)\s*([\w+#.-]*)\s*$""")

fun parseMarkdown(source: String): List<MdBlock> {
    val lines = source.replace("\r\n", "\n").split('\n')
    val out = mutableListOf<MdBlock>()
    var i = 0
    val paragraph = StringBuilder()
    fun flushParagraph() {
        if (paragraph.isNotBlank()) out += MdBlock.Paragraph(parseInlines(paragraph.toString().trim()))
        paragraph.clear()
    }
    while (i < lines.size) {
        val line = lines[i]
        val fence = FenceRe.matchEntire(line)
        when {
            fence != null -> {
                flushParagraph()
                val mark = fence.groupValues[1]
                val body = StringBuilder()
                i++
                // 没有结尾的围栏（回答还在流式输出）就一直算到文末。
                while (i < lines.size && !lines[i].trimStart().startsWith(mark.take(3))) { body.append(lines[i]).append('\n'); i++ }
                out += MdBlock.Code(fence.groupValues[2], body.toString().trimEnd('\n'))
                i++
                continue
            }
            line.isBlank() -> { flushParagraph(); i++; continue }
            HeadingRe.matches(line) -> {
                flushParagraph()
                val m = HeadingRe.matchEntire(line)!!
                out += MdBlock.Heading(m.groupValues[1].length, parseInlines(m.groupValues[2]))
            }
            RuleRe.matches(line) -> { flushParagraph(); out += MdBlock.Rule }
            line.trimStart().startsWith(">") -> {
                flushParagraph()
                val quote = StringBuilder()
                while (i < lines.size && lines[i].trimStart().startsWith(">")) { quote.append(lines[i].trimStart().removePrefix(">").trimStart()).append(' '); i++ }
                out += MdBlock.Quote(parseInlines(quote.toString().trim()))
                continue
            }
            BulletRe.matches(line) -> {
                flushParagraph()
                val items = mutableListOf<List<MdInline>>()
                while (i < lines.size && BulletRe.matches(lines[i])) { items += parseInlines(BulletRe.matchEntire(lines[i])!!.groupValues[1]); i++ }
                out += MdBlock.Bullets(items)
                continue
            }
            NumberRe.matches(line) -> {
                flushParagraph()
                val start = NumberRe.matchEntire(line)!!.groupValues[1].toInt()
                val items = mutableListOf<List<MdInline>>()
                while (i < lines.size && NumberRe.matches(lines[i])) { items += parseInlines(NumberRe.matchEntire(lines[i])!!.groupValues[2]); i++ }
                out += MdBlock.Numbered(start, items)
                continue
            }
            line.trimStart().startsWith("|") -> {
                // 表格：整段当作等宽文字。
                flushParagraph()
                val table = StringBuilder()
                while (i < lines.size && lines[i].trimStart().startsWith("|")) { table.append(lines[i].trim()).append('\n'); i++ }
                out += MdBlock.Code("", table.toString().trimEnd('\n'))
                continue
            }
            else -> {
                if (paragraph.isNotEmpty()) paragraph.append('\n')
                paragraph.append(line)
            }
        }
        i++
    }
    flushParagraph()
    return out
}

/** 行内：`代码`、**粗体**、*斜体*、[文字](网址)。其余原样保留。 */
fun parseInlines(text: String): List<MdInline> {
    val out = mutableListOf<MdInline>()
    val plain = StringBuilder()
    fun flush() { if (plain.isNotEmpty()) { out += MdInline.Text(plain.toString()); plain.clear() } }
    var i = 0
    while (i < text.length) {
        val c = text[i]
        when {
            c == '`' -> {
                val end = text.indexOf('`', i + 1)
                if (end > i + 1) { flush(); out += MdInline.Code(text.substring(i + 1, end)); i = end + 1; continue }
            }
            c == '*' && text.startsWith("**", i) -> {
                val end = text.indexOf("**", i + 2)
                if (end > i + 2) { flush(); out += MdInline.Bold(text.substring(i + 2, end)); i = end + 2; continue }
            }
            (c == '*' || c == '_') && i + 1 < text.length && !text[i + 1].isWhitespace() && (i == 0 || !text[i - 1].isLetterOrDigit()) -> {
                val end = text.indexOf(c, i + 1)
                if (end > i + 1 && !text[end - 1].isWhitespace() && (end + 1 >= text.length || !text[end + 1].isLetterOrDigit())) {
                    flush(); out += MdInline.Italic(text.substring(i + 1, end)); i = end + 1; continue
                }
            }
            c == '[' -> {
                val close = text.indexOf("](", i + 1)
                val end = if (close > i) text.indexOf(')', close + 2) else -1
                if (close > i + 1 && end > close + 2) {
                    val url = text.substring(close + 2, end).trim()
                    if (url.startsWith("http://") || url.startsWith("https://")) {
                        flush(); out += MdInline.Link(text.substring(i + 1, close), url); i = end + 1; continue
                    }
                }
            }
        }
        plain.append(c)
        i++
    }
    flush()
    return out
}
