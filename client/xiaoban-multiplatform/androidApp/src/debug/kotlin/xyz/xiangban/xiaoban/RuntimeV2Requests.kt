package xyz.xiangban.xiaoban

import java.net.Inet6Address
import java.net.InetAddress
import java.net.URI
import java.util.Locale

internal object RuntimeV2Requests {
    private const val SUFFIX = ".app.xiangban.xyz"
    fun isWorkHost(host: String?) = host != null && host.lowercase(Locale.ROOT).let {
        it.endsWith(SUFFIX) && it.length > SUFFIX.length
    }

    // URI.rawPath is decoded exactly once. Reject separators, traversal and malformed escapes.
    fun assetPath(url: String): String? = try {
        val uri = URI(url)
        if (uri.scheme != "https" || !isWorkHost(uri.host) || (uri.port != -1 && uri.port != 443)) null
        else {
            val path = uri.path ?: ""
            if (path.contains('\\') || path.split('/').any { it == "." || it == ".." } ||
                Regex("%2f|%5c|%00", RegexOption.IGNORE_CASE).containsMatchIn(uri.rawPath ?: "") || path.contains('\u0000')) null
            else "runtime-v2-spike/" + path.trimStart('/').ifEmpty { "index.html" }
        }
    } catch (_: Exception) { null }

    fun contentType(path: String) = when (path.substringAfterLast('.', "").lowercase(Locale.ROOT)) {
        "html" -> "text/html"
        "js", "mjs" -> "text/javascript"
        "wasm" -> "application/wasm"
        "css" -> "text/css"
        "json" -> "application/json"
        "svg" -> "image/svg+xml"
        else -> "application/octet-stream"
    }

    fun isLocal(host: String?): Boolean {
        val h = host?.lowercase(Locale.ROOT)?.trimEnd('.')?.removePrefix("[")?.removeSuffix("]") ?: return false
        if (h == "localhost" || h.endsWith(".localhost") || h == "local" || h.endsWith(".local")) return true
        if (h.contains(':')) {
            // Only parse literal IPv6, never resolve DNS names.
            if (!h.matches(Regex("[0-9a-f:.]+"))) return false
            val bytes = try { (InetAddress.getByName(h) as? Inet6Address)?.address } catch (_: Exception) { null }
            if (bytes != null) return bytes.take(15).all { it == 0.toByte() } && bytes[15] == 1.toByte() || (bytes[0].toInt() and 0xfe) == 0xfc
            // IPv4-mapped IPv6 can be returned by Java as Inet4Address.
            return try { InetAddress.getByName(h).address.let { it.size == 4 && localV4(it.map { b -> b.toInt() and 255 }) } } catch (_: Exception) { false }
        }
        val octets = h.split('.')
        if (octets.size != 4 || octets.any { it.isEmpty() || it.any { c -> !c.isDigit() } }) return false
        val nums = octets.map { it.toIntOrNull() ?: return false }
        return nums.all { it in 0..255 } && localV4(nums)
    }
    private fun localV4(n: List<Int>) = n[0] == 127 || n[0] == 10 || n[0] == 172 && n[1] in 16..31 || n[0] == 192 && n[1] == 168 || n[0] == 169 && n[1] == 254
}
