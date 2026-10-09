package xyz.xiaban.wallpaper.data

import org.json.JSONArray
import org.json.JSONObject

enum class MediaKind(val wire: String) {
    IMAGE("image"),
    VIDEO("video"),
    ;

    companion object {
        fun parse(value: String?) = entries.find { it.wire == value }
    }
}

class Category(val id: String, val name: String)

/** One card of the website's public search: everything needed to draw it, open it and favourite it. */
data class SiteItem(
    val id: String,
    val kind: MediaKind,
    val title: String,
    val author: String,
    val avatarUrl: String,
    val width: Int,
    val height: Int,
    /** Dominant colour as ARGB, 0 when the site did not give one. Shown while the picture loads. */
    val color: Int,
    val likes: Int,
    /** Picture used in the grid. */
    val coverUrl: String,
    /** Larger picture used on the detail page; for a video this is its poster. */
    val detailUrl: String,
    /** The sharpest picture worth loading for full-screen viewing; same as [detailUrl] when the site offers nothing bigger. */
    val fullUrl: String,
    /** A video file this phone can stream; empty for pictures and for formats it cannot play. */
    val videoUrl: String,
    /** The same pictures and video as held by our own server; tried when the provider's link above fails. Empty when there is no copy. */
    val fallbackCoverUrl: String = "",
    val fallbackDetailUrl: String = "",
    val fallbackFullUrl: String = "",
    val fallbackVideoUrl: String = "",
    val tags: List<String>,
) {
    fun toJson(): JSONObject = JSONObject()
        .put("id", id).put("kind", kind.wire).put("title", title).put("author", author).put("avatar", avatarUrl)
        .put("w", width).put("h", height).put("color", color).put("likes", likes)
        .put("cover", coverUrl).put("detail", detailUrl).put("full", fullUrl).put("video", videoUrl)
        .put("fbCover", fallbackCoverUrl).put("fbDetail", fallbackDetailUrl).put("fbFull", fallbackFullUrl).put("fbVideo", fallbackVideoUrl)
        .put("tags", JSONArray(tags))

    companion object {
        fun fromJson(o: JSONObject): SiteItem? {
            val kind = MediaKind.parse(o.optString("kind")) ?: return null
            val id = o.optString("id")
            if (id.isEmpty()) return null
            val tags = o.optJSONArray("tags")
            val cover = o.optString("cover")
            // Older saves of a video kept the video file itself as "detail"; a file is never a picture.
            val detail = o.optString("detail").takeUnless { kind == MediaKind.VIDEO && VideoFiles.isVideo(it) } ?: cover
            return SiteItem(
                id = id, kind = kind, title = o.optString("title"), author = o.optString("author"),
                avatarUrl = o.optString("avatar"), width = o.optInt("w"), height = o.optInt("h"),
                color = o.optInt("color"), likes = o.optInt("likes"), coverUrl = cover,
                detailUrl = detail,
                fullUrl = o.optString("full").ifEmpty { detail },
                videoUrl = o.optString("video"),
                fallbackCoverUrl = o.optString("fbCover"), fallbackDetailUrl = o.optString("fbDetail"),
                fallbackFullUrl = o.optString("fbFull"), fallbackVideoUrl = o.optString("fbVideo"),
                tags = if (tags == null) emptyList() else (0 until tags.length()).map { tags.getString(it) },
            )
        }
    }
}

/** Which links are video files this phone's player can stream. */
object VideoFiles {
    private val PLAYABLE = listOf(".mp4", ".m4v", ".webm")

    fun isVideo(url: String): Boolean = url.substringBefore('?').lowercase().let { path -> PLAYABLE.any { path.endsWith(it) } }
}

class SitePage(val items: List<SiteItem>, val hasMore: Boolean, val page: Int)

/** Our own stored copy of the file; its hash and size are those of this copy. */
class CopyFile(val url: String, val sha256: String, val bytes: Long, val mime: String)

/** The provider's own link. Nothing is known about the file behind it, so it is checked by what it really is. */
class SourceFile(val url: String, val mime: String)

/**
 * The website's single-item delivery manifest: the provider's link and/or our verified copy.
 * Either may be missing, but never both.
 */
class Delivery(val id: String, val title: String, val kind: MediaKind, val copy: CopyFile?, val source: SourceFile?)

/** A wallpaper this app saved into the phone's gallery. */
data class Downloaded(
    val id: String,
    val kind: MediaKind,
    val title: String,
    /** content:// address of the gallery entry. */
    val uri: String,
    val sha256: String,
    val width: Int,
    val height: Int,
    val savedAt: Long,
)

/** A favourite: a snapshot of the card, kept only on this phone. */
data class Favorite(val item: SiteItem, val savedAt: Long)

class SiteException(message: String) : Exception(message)
