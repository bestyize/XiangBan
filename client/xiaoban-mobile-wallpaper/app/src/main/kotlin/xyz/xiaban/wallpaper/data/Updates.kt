package xyz.xiaban.wallpaper.data

import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.io.File

/** A newer release of this app than the one installed. */
data class AppUpdate(val version: String, val notes: String)

/**
 * A notice from the website, shown as a dialog at launch.
 *
 * [skippable] ones have a cancel button and are shown once per [revision]; the others have only the confirm button
 * and come back on every launch until the site stops publishing them. [confirmLink] is an HTTPS address the confirm
 * button opens; without one the button just closes the dialog.
 */
data class Announcement(
    val id: String,
    val title: String,
    val body: String,
    val revision: String,
    val skippable: Boolean,
    val confirmText: String,
    val cancelText: String,
    val confirmLink: String?,
)

object UpdateRules {
    const val TARGET = "android-arm64"

    /** Where the website offers the downloads, on the page for the desktop and phone apps. */
    const val DOWNLOADS_FRAGMENT = "#desktop-downloads"

    const val DEFAULT_CONFIRM = "确定"
    const val DEFAULT_CANCEL = "取消"

    private const val MAX_ITEMS = 100
    private const val MAX_TITLE = 160
    private const val MAX_BODY = 16_000
    private const val MAX_LINK = 2048
    private const val MAX_LABEL = 12

    /** The site sends its address with every announcement list for this audience; an older site ignores the ask. */
    const val AUDIENCE = "android"

    /**
     * What the confirm button may open: an HTTPS address with a host and no credentials. Unlike a media address it
     * may carry a fragment, which is how a notice points at a part of a page. Anything else, such as an intent: or
     * javascript: address, is not opened.
     */
    fun isJumpLink(value: String): Boolean = try {
        val uri = java.net.URI(value)
        uri.scheme == "https" && !uri.host.isNullOrEmpty() && uri.userInfo == null
    } catch (_: Exception) {
        false
    }

    fun downloadsLink(origin: String) = origin.trimEnd('/') + "/" + DOWNLOADS_FRAGMENT

    /** What to show at launch, in order: the must-reads first, then skippable ones not yet seen. */
    fun toShow(items: List<Announcement>, seen: (id: String, revision: String) -> Boolean): List<Announcement> {
        val mandatory = items.filter { !it.skippable }
        val fresh = items.filter { it.skippable && !seen(it.id, it.revision) }
        return mandatory + fresh
    }

    fun update(text: String): AppUpdate {
        val o = obj(text)
        if (o.optInt("schemaVersion") != 1) throw SiteException("更新数据无效")
        val version = o.optString("version")
        if (!VERSION.matches(version)) throw SiteException("更新数据无效")
        return AppUpdate(version, o.optString("notes"))
    }

    /**
     * Items that are malformed are skipped rather than failing the list, so one bad notice does not hide the rest.
     * A list that does not say it was built for this app is refused: an older site ignores the audience and would
     * answer with the desktop's notices.
     */
    fun announcements(text: String): List<Announcement> {
        val o = obj(text)
        if (o.optInt("schemaVersion") != 1 || o.optString("audience") != AUDIENCE) throw SiteException("公告数据无效")
        val list = o.optJSONArray("items") ?: throw SiteException("公告数据无效")
        return (0 until minOf(list.length(), MAX_ITEMS)).mapNotNull { announcement(list.optJSONObject(it)) }
    }

    private fun announcement(o: JSONObject?): Announcement? {
        o ?: return null
        val id = o.optString("id")
        val title = o.optString("title").trim()
        val body = o.optString("body")
        val revision = o.optString("revision")
        if (id.isBlank() || id.length > 64 || title.isEmpty() || title.length > MAX_TITLE || body.length > MAX_BODY) return null
        if (!MediaRules.isSha256(revision)) return null
        val link = optional(o, "confirmUrl")?.takeIf { it.length <= MAX_LINK && isJumpLink(it) }
        return Announcement(
            id = id,
            title = title,
            body = body.trim(),
            revision = revision,
            skippable = if (o.has("skippable")) o.optBoolean("skippable", true) else true,
            confirmText = label(optional(o, "confirmText"), DEFAULT_CONFIRM),
            cancelText = label(optional(o, "cancelText"), DEFAULT_CANCEL),
            confirmLink = link,
        )
    }

    /** org.json turns a JSON null into the text "null" through optString, so nulls are checked first. */
    private fun optional(o: JSONObject, key: String): String? =
        if (!o.has(key) || o.isNull(key)) null else o.optString(key).trim().takeIf { it.isNotEmpty() }

    private fun label(value: String?, fallback: String) = value?.takeIf { it.length <= MAX_LABEL } ?: fallback

    private fun obj(text: String) = try {
        JSONObject(text)
    } catch (_: JSONException) {
        throw SiteException("网站数据无效")
    }

    private val VERSION = Regex("^\\d+\\.\\d+\\.\\d+([-+][0-9A-Za-z.-]+)?$")
}

/**
 * Which skippable announcements have already been shown, so each is shown once. An entry is the id plus the
 * revision, so an edited notice counts as new. Only the most recent [LIMIT] are kept.
 */
class SeenAnnouncements(file: File) {
    private val json = JsonFile(file)

    @Synchronized
    fun has(id: String, revision: String) = entries().any { it.first == id && it.second == revision }

    @Synchronized
    fun add(id: String, revision: String) {
        val kept = entries().filter { it.first != id || it.second != revision } + (id to revision)
        json.write(
            JSONArray().also { array ->
                kept.takeLast(LIMIT).forEach { array.put(JSONObject().put("id", it.first).put("rev", it.second)) }
            },
        )
    }

    private fun entries(): List<Pair<String, String>> {
        val array = json.read()
        return (0 until array.length()).mapNotNull {
            val o = array.optJSONObject(it) ?: return@mapNotNull null
            val id = o.optString("id")
            val rev = o.optString("rev")
            if (id.isEmpty() || rev.isEmpty()) null else id to rev
        }
    }

    companion object {
        const val LIMIT = 200
    }
}
