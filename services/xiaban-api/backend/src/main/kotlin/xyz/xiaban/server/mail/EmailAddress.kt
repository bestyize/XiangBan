package xyz.xiaban.server.mail

import xyz.xiaban.server.http.badRequest

private val emailPattern = Regex("""^[^\s@]+@[^\s@]+\.[^\s@]+$""")

/** Trims and lowercases an address typed by a person; rejects anything that is not a plausible email. */
fun normalizeEmail(raw: String): String {
    val email = raw.trim().lowercase()
    if (email.length > 254 || !emailPattern.matches(email)) badRequest("邮箱格式不对", "invalid_email")
    return email
}
