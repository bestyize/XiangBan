package xyz.xiaban.wallpaper.data

import org.json.JSONException
import org.json.JSONObject
import java.net.URI

/** Allowed media types and size limits, from the desktop delivery protocol. */
object MediaRules {
    const val MAX_SITE_BYTES = 256L * 1024 * 1024

    private val EXTENSIONS = mapOf(
        "image/png" to "png",
        "image/jpeg" to "jpg",
        "image/webp" to "webp",
        "image/gif" to "gif",
        "image/bmp" to "bmp",
        "image/avif" to "avif",
        "video/mp4" to "mp4",
        "video/x-m4v" to "mp4",
        "video/webm" to "webm",
    )

    fun extension(mime: String) = EXTENSIONS[mime.substringBefore(';').trim().lowercase()]

    fun kindOf(mime: String) = when {
        extension(mime) == null -> null
        mime.startsWith("image/") -> MediaKind.IMAGE
        else -> MediaKind.VIDEO
    }

    fun isSha256(value: String) = value.length == 64 && value.all { it in '0'..'9' || it in 'a'..'f' }

    fun isHttps(url: String): Boolean = try {
        val uri = URI(url)
        uri.scheme == "https" && !uri.host.isNullOrEmpty() && uri.userInfo == null && uri.fragment == null
    } catch (_: Exception) {
        false
    }
}

/** "#rrggbb" to opaque ARGB; 0 for anything else. Kept free of android.graphics so it runs in plain JVM tests. */
fun parseHexColor(value: String?): Int {
    val hex = value?.removePrefix("#") ?: return 0
    if (hex.length != 6) return 0
    val rgb = hex.toIntOrNull(16) ?: return 0
    return 0xFF000000.toInt() or rgb
}

/**
 * Turns a link from the site into a full address. The site's own uploads come as "/uploads/..." paths and are completed
 * with [origin]; everything else must already be a plain HTTPS address. Anything else, including "//host/..." that
 * would point somewhere else, becomes empty.
 */
fun resolveLink(value: String?, origin: String): String {
    val link = value.orEmpty()
    if (link.startsWith("/") && !link.startsWith("//") && MediaRules.isHttps(origin)) {
        val joined = origin.trimEnd('/') + link
        return if (MediaRules.isHttps(joined)) joined else ""
    }
    return if (MediaRules.isHttps(link)) link else ""
}

/** Parses the website's JSON. Everything outside the contract is rejected instead of guessed at. */
object SiteParser {
    fun categories(text: String): List<Category> = parse(text) { root ->
        val list = root.getJSONArray("categories")
        (0 until list.length()).map {
            val c = list.getJSONObject(it)
            Category(c.getString("id"), c.getString("name"))
        }
    }

    fun page(text: String, origin: String): SitePage = parse(text) { root ->
        val list = root.getJSONArray("items")
        val items = (0 until list.length()).mapNotNull { item(list.getJSONObject(it), origin) }
        SitePage(items, root.optBoolean("has_more"), root.optInt("page", 1))
    }

    private fun item(o: JSONObject, origin: String): SiteItem? {
        val kind = MediaKind.parse(o.optString("media_type")) ?: return null
        val id = o.getString("id")
        if (id.isBlank() || id.length > 128 || id.contains('/')) return null
        val downloads = o.optJSONObject("downloads")
        fun https(value: String?) = resolveLink(value, origin)
        fun download(key: String) = https(downloads?.optString(key))
        val preview = https(o.optString("preview_url"))
        val regular = https(o.optString("regular_url"))
        val size = o.optJSONObject("resolution")
        val author = o.optJSONObject("author")
        val tags = o.optJSONArray("tags")
        val fb = o.optJSONObject("fallback")
        val fbDownloads = fb?.optJSONObject("downloads")
        fun fbLink(value: String?) = resolveLink(value, origin)
        fun fbDownload(key: String) = fbLink(fbDownloads?.optString(key))
        val cover = if (kind == MediaKind.VIDEO) {
            firstOf(preview, download("small"), download("thumb"))
        } else {
            firstOf(download("small"), preview, download("medium"), download("thumb"))
        }
        val detail = if (kind == MediaKind.VIDEO) cover else firstOf(regular, preview, download("medium"), cover)
        val video = if (kind == MediaKind.VIDEO) {
            listOf(download("medium"), download("large"), download("original"), regular).firstOrNull { VideoFiles.isVideo(it) }.orEmpty()
        } else {
            ""
        }
        return SiteItem(
            id = id,
            kind = kind,
            title = o.optString("title").ifBlank { id },
            author = author?.optString("name").orEmpty(),
            avatarUrl = https(author?.optString("avatar_url")),
            width = size?.optInt("width") ?: 0,
            height = size?.optInt("height") ?: 0,
            color = parseHexColor(o.optString("dominant_color")),
            likes = o.optJSONObject("stats")?.optInt("likes") ?: 0,
            coverUrl = cover,
            detailUrl = detail,
            fullUrl = if (kind == MediaKind.VIDEO) cover else firstOf(download("large"), download("original"), regular, detail),
            videoUrl = video,
            fallbackCoverUrl = if (kind == MediaKind.VIDEO) firstOf(fbLink(fb?.optString("preview_url")), fbDownload("small"), fbDownload("thumb"))
            else firstOf(fbDownload("small"), fbLink(fb?.optString("preview_url")), fbDownload("medium"), fbDownload("thumb")),
            fallbackDetailUrl = if (kind == MediaKind.VIDEO) {
                firstOf(fbLink(fb?.optString("preview_url")), fbDownload("small"), fbDownload("thumb"))
            } else {
                firstOf(fbLink(fb?.optString("regular_url")), fbLink(fb?.optString("preview_url")), fbDownload("medium"), fbDownload("small"))
            },
            fallbackFullUrl = if (kind == MediaKind.VIDEO) "" else firstOf(fbDownload("large"), fbDownload("original"), fbLink(fb?.optString("regular_url"))),
            fallbackVideoUrl = if (kind == MediaKind.VIDEO) {
                listOf(fbDownload("medium"), fbDownload("large"), fbDownload("original")).firstOrNull { VideoFiles.isVideo(it) }.orEmpty()
            } else {
                ""
            },
            tags = if (tags == null) emptyList() else (0 until minOf(tags.length(), 12)).map { tags.optString(it) }.filter { it.isNotBlank() },
        ).takeIf { it.coverUrl.isNotEmpty() }
    }

    private fun firstOf(vararg urls: String) = urls.firstOrNull { it.isNotEmpty() }.orEmpty()

    fun delivery(text: String, origin: String): Delivery = parse(text) { root ->
        if (root.optInt("schemaVersion") != 1) throw SiteException("网站返回了不支持的协议版本")
        val item = root.getJSONArray("items").optJSONObject(0) ?: throw SiteException("网站没有返回壁纸")
        val kind = MediaKind.parse(item.optString("kind")) ?: throw SiteException("壁纸类型无法识别")
        val source = item.optJSONObject("source")?.let { o ->
            val url = resolveLink(o.optString("url"), origin)
            // The provider's address must be a plain web address; a site-relative path here would just be our own copy.
            if (url.isNotEmpty() && MediaRules.isHttps(o.optString("url"))) SourceFile(url, o.optString("mime")) else null
        }
        val copy = item.optJSONObject("resource")?.let { copy(it, kind, origin) }
        if (copy == null && source == null) throw SiteException("网站没有提供可下载的文件")
        Delivery(item.getString("id"), item.optString("title"), kind, copy, source)
    }

    private fun copy(resource: JSONObject, kind: MediaKind, origin: String): CopyFile {
        val mime = resource.getString("mime")
        if (MediaRules.kindOf(mime) != kind) throw SiteException("壁纸格式 $mime 暂不支持")
        val url = resolveLink(resource.getString("url"), origin)
        if (url.isEmpty()) throw SiteException("壁纸下载地址不安全")
        val sha = resource.getString("sha256")
        if (!MediaRules.isSha256(sha)) throw SiteException("壁纸校验值无效")
        val bytes = resource.getLong("bytes")
        if (bytes <= 0 || bytes > MediaRules.MAX_SITE_BYTES) throw SiteException("壁纸大小超出限制")
        return CopyFile(url, sha, bytes, mime)
    }

    private fun <T> parse(text: String, block: (JSONObject) -> T): T = try {
        block(JSONObject(text))
    } catch (e: JSONException) {
        throw SiteException("网站数据无效")
    }
}
