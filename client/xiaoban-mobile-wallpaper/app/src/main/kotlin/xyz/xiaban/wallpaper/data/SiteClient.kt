package xyz.xiaban.wallpaper.data

import org.json.JSONObject
import java.io.File
import java.io.IOException
import java.net.HttpURLConnection
import java.net.SocketTimeoutException
import java.net.URL
import java.net.URLEncoder
import java.net.UnknownHostException

/** Talks to the wallpaper website's public API. Downloads are verified before they are ever used. */
class SiteClient(val origin: String = DEFAULT_ORIGIN) : Remote {
    fun categories(): List<Category> = SiteParser.categories(text("$origin/api/v1/categories", READ_MS))

    fun search(query: String, category: String, kind: MediaKind?, page: Int, orientation: Orientation = Orientation.ALL): SitePage {
        val params = buildString {
            append("page=").append(page.coerceAtLeast(1)).append("&per_page=").append(PAGE_SIZE)
            append("&q=").append(enc(query)).append("&category=").append(enc(category))
            if (kind != null) append("&media_type=").append(kind.wire)
            if (orientation != Orientation.ALL) append("&orientation=").append(orientation.wire)
        }
        return SiteParser.page(text("$origin/api/v1/wallpapers/search?$params", READ_MS), origin)
    }

    /** The release newer than [currentVersion] for this app, or null when this is the newest (the site answers 204). */
    fun update(currentVersion: String): AppUpdate? {
        val url = "$origin/api/v1/desktop/updates?currentVersion=${enc(currentVersion)}&target=${UpdateRules.TARGET}"
        return optionalText(url, READ_MS)?.let(UpdateRules::update)
    }

    /** Notices for this app. A site that predates them answers without naming the audience, which is refused. */
    fun announcements(): List<Announcement> =
        UpdateRules.announcements(text("$origin/api/v1/desktop/announcements?audience=${UpdateRules.AUDIENCE}", READ_MS))

    /** Without [preferSource] the site builds and hashes its own copy first, which can take a while. */
    override fun delivery(id: String, preferSource: Boolean): Delivery {
        val query = if (preferSource) "?prefer=source" else ""
        return SiteParser.delivery(text("$origin/api/v1/desktop/wallpapers/${enc(id)}$query", DELIVERY_MS), origin)
    }

    override fun downloadCopy(copy: CopyFile, target: File, progress: (Long, Long) -> Unit, cancelled: () -> Boolean) {
        guard {
            val conn = open(copy.url, DOWNLOAD_MS, identity = true)
            try {
                if (conn.contentLengthLong > copy.bytes) throw SiteException("下载内容与清单不符")
                val result = conn.inputStream.use { Verified.copy(it, target, copy.bytes, { done -> progress(done, copy.bytes) }, cancelled) }
                if (result.bytes != copy.bytes || result.sha256 != copy.sha256) {
                    target.delete()
                    throw SiteException("下载内容校验失败，已丢弃")
                }
            } finally {
                conn.disconnect()
            }
        }
    }

    override fun downloadSource(
        source: SourceFile,
        target: File,
        onTotal: (Long) -> Unit,
        progress: (Long, Long) -> Unit,
        cancelled: () -> Boolean,
    ): Verified.Result = guard {
        val conn = open(source.url, DOWNLOAD_MS, identity = true)
        try {
            val total = conn.contentLengthLong
            if (total > MediaRules.MAX_SOURCE_BYTES) throw SiteException("文件超出大小限制")
            // The download and its copy in the gallery exist side by side until the download is deleted.
            if (total > 0 && target.parentFile?.usableSpace?.let { it < total * 2 } == true) throw SiteException("手机存储空间不足，需要至少 ${sizeMb(total * 2)} 可用空间")
            if (total > 0) onTotal(total)
            conn.inputStream.use { Verified.copy(it, target, MediaRules.MAX_SOURCE_BYTES, { done -> progress(done, total) }, cancelled) }
        } finally {
            conn.disconnect()
        }
    }

    fun thumbnail(url: String): ByteArray = guard {
        val conn = open(url, READ_MS, identity = false)
        try {
            conn.inputStream.use { input ->
                val out = java.io.ByteArrayOutputStream()
                val buffer = ByteArray(16 * 1024)
                while (true) {
                    val n = input.read(buffer)
                    if (n < 0) break
                    out.write(buffer, 0, n)
                    if (out.size() > MAX_THUMB_BYTES) throw SiteException("缩略图过大")
                }
                out.toByteArray()
            }
        } finally {
            conn.disconnect()
        }
    }

    private fun text(url: String, readMs: Int): String = guard {
        val conn = open(url, readMs, identity = false)
        try {
            conn.inputStream.bufferedReader().use { it.readText() }
        } finally {
            conn.disconnect()
        }
    }

    /** Like [text], but a 204 (nothing to report) gives null. */
    private fun optionalText(url: String, readMs: Int): String? = guard {
        val conn = open(url, readMs, identity = false, allowNoContent = true)
        try {
            if (conn.responseCode == 204) null else conn.inputStream.bufferedReader().use { it.readText() }
        } finally {
            conn.disconnect()
        }
    }

    private fun open(start: String, readMs: Int, identity: Boolean, allowNoContent: Boolean = false): HttpURLConnection {
        var current = start
        repeat(MAX_REDIRECTS + 1) {
            if (!MediaRules.isHttps(current)) throw SiteException("只允许使用 HTTPS 地址")
            val conn = URL(current).openConnection() as HttpURLConnection
            conn.instanceFollowRedirects = false
            conn.connectTimeout = CONNECT_MS
            conn.readTimeout = readMs
            conn.setRequestProperty("User-Agent", USER_AGENT)
            if (identity) conn.setRequestProperty("Accept-Encoding", "identity")
            val code = conn.responseCode
            if (code in 301..308 && code != 304) {
                val next = conn.getHeaderField("Location")
                conn.disconnect()
                current = URL(URL(current), next ?: throw SiteException("重定向缺少地址")).toString()
            } else if (code == 200 || (allowNoContent && code == 204)) {
                return conn
            } else {
                val reason = reasonOf(conn)
                conn.disconnect()
                throw SiteException(reason ?: "网站返回错误（$code）")
            }
        }
        throw SiteException("重定向次数过多")
    }

    private fun reasonOf(conn: HttpURLConnection): String? = try {
        conn.errorStream?.bufferedReader()?.use { JSONObject(it.readText()).optString("error") }?.takeIf { it.isNotBlank() }?.let(::phoneReason)
    } catch (_: Exception) {
        null
    }

    private fun <T> guard(block: () -> T): T = try {
        block()
    } catch (e: SiteException) {
        throw e
    } catch (e: SocketTimeoutException) {
        throw SiteException("网站响应超时，请稍后重试")
    } catch (e: UnknownHostException) {
        throw SiteException("无法连接网站，请检查网络")
    } catch (e: IOException) {
        throw SiteException("网络出错：${e.message ?: "连接中断"}")
    }

    private fun sizeMb(bytes: Long) = "${bytes / (1024 * 1024)} MB"

    private fun enc(value: String) = URLEncoder.encode(value, "UTF-8")

    companion object {
        const val DEFAULT_ORIGIN = "https://wallpaper.xiangban.xyz"
        const val PAGE_SIZE = 24
        private const val MAX_REDIRECTS = 4
        private const val CONNECT_MS = 15_000
        private const val READ_MS = 20_000
        private const val DELIVERY_MS = 75_000
        private const val DOWNLOAD_MS = 60_000
        private const val MAX_THUMB_BYTES = 3 * 1024 * 1024
        private const val USER_AGENT = "XiaoBanMobile/0.1"
    }
}
