package xyz.xiangban.xiaoban.common.community

/** Bound all frames before handing an animation to a native WebView. Never execute image content. */
internal fun validGeneratedGif(bytes: ByteArray): Boolean {
    if (bytes.size < 14 || bytes.decodeToString(0, 6) !in listOf("GIF87a", "GIF89a")) return false
    fun byte(p: Int) = bytes[p].toInt() and 255
    fun number(p: Int) = byte(p) or (byte(p + 1) shl 8)
    val width = number(6)
    val height = number(8)
    if (width !in 1..8192 || height !in 1..8192) return false
    var p = 13 + if (byte(10) and 128 != 0) 3 * (1 shl ((byte(10) and 7) + 1)) else 0
    var frames = 0
    fun blocks(): Boolean {
        while (p < bytes.size) { val n = byte(p++); if (n == 0) return true; p += n }
        return false
    }
    while (p < bytes.size) {
        when (byte(p++)) {
            0x3b -> return frames >= 2
            0x21 -> { p++; if (!blocks()) return false }
            0x2c -> {
                if (p + 9 > bytes.size || number(p + 4) == 0 || number(p + 6) == 0 ||
                    number(p) + number(p + 4) > width || number(p + 2) + number(p + 6) > height) return false
                frames++
                if (frames > 16 || frames.toLong() * width * height > 32 * 1024 * 1024) return false
                val packed = byte(p + 8); p += 9
                if (packed and 128 != 0) p += 3 * (1 shl ((packed and 7) + 1))
                if (p >= bytes.size || byte(p) !in 2..8) return false
                p++; if (!blocks()) return false
            }
            else -> return false
        }
    }
    return false
}
