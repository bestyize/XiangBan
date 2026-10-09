package xyz.xiaban.wallpaper.ui

import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.SiteItem

/** Everything the full-screen viewer needs, whether the thing comes from the website or from the phone's gallery. */
data class Viewable(
    val key: String,
    val kind: MediaKind,
    val title: String,
    /** Width over height of the picture as it really is; 0 when unknown. */
    val aspect: Float,
    /** The sharpest picture available: a web address or a content:// address. */
    val image: String,
    /** A picture that is probably already on screen or cached, shown while [image] loads. */
    val poster: String,
    /** A video this phone can play; empty when there is none. */
    val video: String,
    val local: Boolean,
    /** The site's own copies of the above, tried when the first choice fails. Empty when there is nothing to fall back to. */
    val fallbackImage: String = "",
    val fallbackPoster: String = "",
    val fallbackVideo: String = "",
) {
    val playable: Boolean get() = kind == MediaKind.VIDEO && video.isNotEmpty()
}

private fun aspectOf(width: Int, height: Int): Float = if (width > 0 && height > 0) width.toFloat() / height else 0f

fun SiteItem.viewable() = Viewable(
    key = id, kind = kind, title = title, aspect = aspectOf(width, height),
    image = fullUrl, poster = detailUrl,
    // A video only our copy can play is simply played from there.
    video = videoUrl.ifEmpty { fallbackVideoUrl },
    local = false,
    fallbackImage = fallbackFullUrl,
    fallbackPoster = fallbackDetailUrl,
    fallbackVideo = if (videoUrl.isEmpty()) "" else fallbackVideoUrl,
)

fun Downloaded.viewable() =
    Viewable(id, kind, title, aspectOf(width, height), uri, uri, if (kind == MediaKind.VIDEO) uri else "", local = true)
