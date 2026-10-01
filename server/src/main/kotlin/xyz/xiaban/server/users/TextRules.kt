package xyz.xiaban.server.users

import xyz.xiaban.server.http.badRequest

/**
 * Names typed by users end up on share cards and in model prompts, so keep them short and free of
 * contact details or links. This is a minimal filter, not a substitute for a moderation service.
 */
object TextRules {
    private val forbidden = Regex("""(https?|www\.|\.com|\.cn|\.xyz|@|微信|vx|qq|加我)""", RegexOption.IGNORE_CASE)
    private val longDigits = Regex("""\d{5,}""")
    private val control = Regex("""[\p{Cc}\p{Cf}]""")

    fun companionName(raw: String): String = clean(raw, min = 2, max = 8, what = "名字")

    fun nickname(raw: String): String? = raw.trim().ifEmpty { null }?.let { clean(it, min = 1, max = 8, what = "称呼") }

    private fun clean(raw: String, min: Int, max: Int, what: String): String {
        val text = raw.replace(control, "").trim().replace(Regex("""\s+"""), " ")
        val length = text.codePointCount(0, text.length)
        if (length < min || length > max) badRequest("${what}要 $min–$max 个字", "invalid_text")
        if (forbidden.containsMatchIn(text) || longDigits.containsMatchIn(text)) {
            badRequest("这个${what}小伴叫不出口，换一个吧", "invalid_text")
        }
        return text
    }
}
