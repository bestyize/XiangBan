package xyz.xiaban.wallpaper.data

import java.io.File
import java.io.InputStream
import java.security.MessageDigest
import java.util.concurrent.CancellationException

/** Streams data into a file while hashing it, so size and SHA-256 are those of the bytes actually stored. */
object Verified {
    class Result(val sha256: String, val bytes: Long)

    fun copy(
        input: InputStream,
        target: File,
        limit: Long,
        progress: (Long) -> Unit = {},
        cancelled: () -> Boolean = { false },
    ): Result {
        val digest = MessageDigest.getInstance("SHA-256")
        var total = 0L
        try {
            target.outputStream().use { out ->
                val buffer = ByteArray(64 * 1024)
                while (true) {
                    if (cancelled()) throw CancellationException("已取消")
                    val read = input.read(buffer)
                    if (read < 0) break
                    total += read
                    if (total > limit) throw SiteException("文件超出大小限制")
                    digest.update(buffer, 0, read)
                    out.write(buffer, 0, read)
                    progress(total)
                }
            }
        } catch (e: Throwable) {
            target.delete()
            throw e
        }
        return Result(digest.digest().joinToString("") { "%02x".format(it) }, total)
    }
}

/** Decides what a file really is from its leading bytes; the provider-reported type is never trusted. */
object MediaSniffer {
    fun mime(head: ByteArray): String? {
        fun at(i: Int, vararg v: Int) = head.size >= i + v.size && v.indices.all { (head[i + it].toInt() and 0xff) == v[it] }
        fun text(i: Int, s: String) = head.size >= i + s.length && s.indices.all { head[i + it].toInt().toChar() == s[it] }
        return when {
            at(0, 0x89, 0x50, 0x4e, 0x47) -> "image/png"
            at(0, 0xff, 0xd8, 0xff) -> "image/jpeg"
            text(0, "GIF8") -> "image/gif"
            text(0, "BM") -> "image/bmp"
            text(0, "RIFF") && text(8, "WEBP") -> "image/webp"
            text(4, "ftyp") && (text(8, "avif") || text(8, "avis")) -> "image/avif"
            text(4, "ftyp") -> "video/mp4"
            at(0, 0x1a, 0x45, 0xdf, 0xa3) -> "video/webm"
            else -> null
        }
    }

    fun mime(file: File): String? = file.inputStream().use { input ->
        val head = ByteArray(16)
        val read = input.read(head)
        mime(if (read < head.size) head.copyOf(maxOf(read, 0)) else head)
    }
}
