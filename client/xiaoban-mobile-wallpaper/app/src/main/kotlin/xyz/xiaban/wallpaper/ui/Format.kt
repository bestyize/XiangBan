package xyz.xiaban.wallpaper.ui

import java.util.Locale

/** 9999 stays as is, 12345 becomes 1.2万, 100000000 becomes 1亿. */
fun likesText(count: Int): String = when {
    count < 10_000 -> count.coerceAtLeast(0).toString()
    count < 100_000_000 -> String.format(Locale.ROOT, "%.1f", count / 10_000.0).removeSuffix(".0") + "万"
    else -> String.format(Locale.ROOT, "%.1f", count / 100_000_000.0).removeSuffix(".0") + "亿"
}

private const val MIN_ASPECT = 0.25f
private const val MAX_ASPECT = 4f
private const val UNKNOWN_ASPECT = 1.5f

/**
 * Width over height of a grid cover, exactly as the picture is shaped, so a waterfall column is as tall as its
 * pictures really are. Only absurd values are bounded, to keep a bad record from breaking the layout.
 * Unknown sizes get the usual shape of the catalogue.
 */
fun coverAspect(width: Int, height: Int): Float =
    if (width <= 0 || height <= 0) UNKNOWN_ASPECT else (width.toFloat() / height).coerceIn(MIN_ASPECT, MAX_ASPECT)

fun sizeText(bytes: Long): String = when {
    bytes < 1024 -> "$bytes B"
    bytes < 1024 * 1024 -> String.format(Locale.ROOT, "%.0f KB", bytes / 1024.0)
    else -> String.format(Locale.ROOT, "%.1f MB", bytes / 1024.0 / 1024.0)
}

/** "下载中 35% · 38.5 MB / 110.0 MB"; only the percentage when the total is not known yet. */
fun downloadLabel(progress: Float, totalBytes: Long): String {
    val done = progress.coerceIn(0f, 1f)
    val percent = "下载中 ${(done * 100).toInt()}%"
    return if (totalBytes > 0) "$percent · ${sizeText((done * totalBytes).toLong())} / ${sizeText(totalBytes)}" else percent
}
