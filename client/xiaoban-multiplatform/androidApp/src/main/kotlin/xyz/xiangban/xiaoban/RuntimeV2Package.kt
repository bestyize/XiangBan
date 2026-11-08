package xyz.xiangban.xiaoban

import android.content.res.AssetManager
import android.net.Uri
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.net.URI
import java.net.URISyntaxException
import java.util.Locale
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonObject

private const val MAX_MANIFEST_BYTES = 64 * 1024
private val WORK_LABEL = Regex("[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?")
private val WORK_VERSION = Regex("[a-zA-Z0-9.-]{1,32}")
private val PATH_SEGMENT = Regex("[a-zA-Z0-9_@-][a-zA-Z0-9._@-]*")
private val ENCODED_SEPARATOR = Regex("%2f|%5c", RegexOption.IGNORE_CASE)

private fun JsonObject.runtimeProtocol(): Int = when (unquoted("runtimeProtocol")) {
    "2" -> 2
    else -> throw IllegalArgumentException("这是旧版作品，已不再支持")
}

private fun JsonObject.unquoted(key: String): String? =
    (this[key] as? JsonPrimitive)?.takeUnless { it.isString }?.content

private fun JsonObject.optionalText(key: String): String? {
    val value = this[key] ?: return null
    require(value is JsonPrimitive && value.isString) { "$key 必须是文本" }
    return value.content
}

private fun JsonObject.requiredText(key: String): String = requireNotNull(optionalText(key)) { "缺少 $key" }

private fun JsonObject.textSet(key: String, allowed: Set<String>): Set<String> {
    val values = this[key] ?: return emptySet()
    require(values is JsonArray) { "$key 必须是文本列表" }
    return values.mapTo(mutableSetOf()) { value ->
        require(value is JsonPrimitive && value.isString && value.content in allowed) { "$key 包含无效值" }
        value.content
    }
}

internal data class RuntimeV2Manifest(
    val id: String,
    val name: String,
    val description: String,
    val version: String,
    val kind: String,
    val category: String,
    val entry: String,
    val icon: String,
    val cover: String,
    val orientation: String,
    val capabilities: Set<String>,
    val input: List<String> = listOf("touch"),
) {
    companion object {
        private val APP_CATEGORIES = setOf("tool", "quiz")
        private val GAME_CATEGORIES = setOf("action", "puzzle", "board", "casual", "story")
        private val CAPABILITIES = setOf("haptics", "share", "clipboard", "screen", "notify", "profile", "camera", "microphone", "contacts", "files")

        fun read(raw: String): RuntimeV2Manifest {
            val json = Json.parseToJsonElement(raw).jsonObject
            require(json.runtimeProtocol() == 2 && json.unquoted("schemaVersion") == "3") { "不是协议 2 作品清单" }
            val id = json.requiredText("id")
            require(id.matches(Regex("[a-z0-9_-]{2,64}"))) { "作品 id 无效" }
            val name = json.requiredText("name").trim()
            require(name.codePointCount(0, name.length) in 1..24) { "作品名称无效" }
            val description = json.requiredText("description").trim()
            require(description.codePointCount(0, description.length) in 1..80) { "作品简介无效" }
            val version = json.requiredText("version").trim()
            require(version.matches(WORK_VERSION)) { "作品版本无效" }
            val kind = json.requiredText("kind")
            require(kind == "app" || kind == "game") { "作品类型无效" }
            val category = json.requiredText("category")
            require(category in (if (kind == "app") APP_CATEGORIES else GAME_CATEGORIES)) { "作品分类无效" }
            val orientation = json.optionalText("orientation") ?: "portrait"
            require(orientation in setOf("portrait", "landscape", "any")) { "作品方向无效" }
            require(json.optionalText("layer") == null) { "该作品仅支持桌面端" }
            val input = json.textSet("input", setOf("touch", "mouse", "keyboard"))
            require(input.isNotEmpty()) { "作品未声明操作方式" }
            val entry = json.optionalText("entry") ?: "index.html"
            val icon = json.requiredText("icon")
            val cover = json.requiredText("cover")
            require(isPackagePath(entry) && packageMimeType(entry) == "text/html") { "作品入口无效" }
            require(listOf(icon, cover).all { isPackagePath(it) && packageMimeType(it)?.startsWith("image/") == true }) {
                "作品图标或封面无效"
            }
            val capabilities = json.textSet("capabilities", CAPABILITIES)
            return RuntimeV2Manifest(id, name, description, version, kind, category, entry, icon, cover, orientation, capabilities, input.toList())
        }
    }
}

internal class RuntimeV2Package private constructor(
    val manifest: RuntimeV2Manifest,
    workHost: String,
    private val version: String,
    private val openResource: (String) -> InputStream,
) {
    val host = "$workHost.app.xiangban.xyz"
    val origin = "https://$host"
    val entryUrl = "$origin/v$version/${manifest.entry}"

    fun ownsOrigin(url: Uri): Boolean = parseWorkUrl(url.toString(), host) != null

    fun pathFor(url: Uri): String? = runtimeV2AssetPath(url.toString(), host, version)

    fun mimeType(path: String): String? = packageMimeType(path)

    fun open(path: String): InputStream? {
        if (!isPackagePath(path) || mimeType(path) == null) return null
        return try {
            openResource(path)
        } catch (_: IOException) {
            null
        }
    }

    companion object {
        // The caller assigns the origin label. A manifest cannot choose another work's origin.
        fun load(assets: AssetManager, root: String, workHost: String, version: String? = null): RuntimeV2Package {
            val manifest = RuntimeV2Manifest.read(readRuntimeManifest(assets, root))
            return create(manifest, workHost, version ?: manifest.version) { path -> assets.open("$root/$path") }
        }

        fun loadCommunity(root: File, id: Long, version: Int, kind: String): RuntimeV2Package {
            require(id > 0 && version > 0)
            val manifest = RuntimeV2Manifest.read(root.resolve("manifest.json").inputStream().use(::readRuntimeManifest))
            require(manifest.kind == kind) { "作品包类型不匹配" }
            return create(manifest, id.toString(), version.toString()) { path -> root.resolve(path).inputStream() }
        }

        /** 手机里自己做的作品，文件在私有工作区 games/<id>/。每次读取都拒绝符号链接和目录外的路径。 */
        fun loadWorkspace(root: File, id: String): RuntimeV2Package {
            val base = root.canonicalFile
            val manifest = base.resolve("manifest.json").inputStream().use(::readRuntimeManifest).let(RuntimeV2Manifest::read)
            require(manifest.id == id && base.name == id) { "作品清单与目录不一致" }
            return create(manifest, "mine-$id", manifest.version) { path ->
                val file = base.resolve(path)
                require(file.canonicalFile.path.startsWith(base.path + File.separator) && file.canonicalFile == file.absoluteFile) { "路径无效" }
                file.inputStream()
            }
        }

        private fun create(manifest: RuntimeV2Manifest, host: String, version: String, open: (String) -> InputStream): RuntimeV2Package {
            require(host.matches(WORK_LABEL)) { "作品源标识无效" }
            require(version.matches(WORK_VERSION)) { "作品版本无效" }
            return RuntimeV2Package(manifest, host, version, open).also { bundle ->
                listOf(manifest.entry, manifest.icon, manifest.cover).forEach { path ->
                    requireNotNull(bundle.open(path)) { "作品缺少 $path" }.close()
                }
            }
        }
    }
}

internal fun readRuntimeManifest(assets: AssetManager, root: String): String {
    require(isPackagePath(root)) { "作品目录无效" }
    return assets.open("$root/manifest.json").use(::readRuntimeManifest)
}

private fun readRuntimeManifest(stream: InputStream): String {
    val bytes = ByteArray(MAX_MANIFEST_BYTES + 1)
    var size = 0
    while (size < bytes.size) {
        val count = stream.read(bytes, size, bytes.size - size)
        if (count == -1) break
        size += count
    }
    require(size <= MAX_MANIFEST_BYTES) { "作品清单超过 64 KB" }
    return bytes.decodeToString(endIndex = size, throwOnInvalidSequence = true)
}

private fun parseWorkUrl(url: String, host: String): URI? {
    val uri = try {
        URI(url)
    } catch (_: URISyntaxException) {
        return null
    }
    if (!"https".equals(uri.scheme, ignoreCase = true)) return null
    val authority = uri.rawAuthority
    return uri.takeIf { host.equals(authority, ignoreCase = true) || "$host:443".equals(authority, ignoreCase = true) }
}

internal fun runtimeV2AssetPath(url: String, host: String, version: String): String? {
    val uri = parseWorkUrl(url, host) ?: return null
    if (ENCODED_SEPARATOR.containsMatchIn(uri.rawPath)) return null
    val prefix = "/v$version/"
    if (!uri.path.startsWith(prefix)) return null
    val path = uri.path.removePrefix(prefix)
    return path.takeIf { isPackagePath(it) && packageMimeType(it) != null }
}

internal fun isPackagePath(path: String): Boolean {
    val segments = path.split('/', limit = 9)
    return segments.size <= 8 && segments.all(PATH_SEGMENT::matches)
}

internal fun packageMimeType(path: String): String? = when (path.substringAfterLast('.', "").lowercase(Locale.ROOT)) {
    "html", "htm" -> "text/html"
    "js", "mjs" -> "text/javascript"
    "css" -> "text/css"
    "json", "map" -> "application/json"
    "svg" -> "image/svg+xml"
    "md", "txt", "atlas", "glsl" -> "text/plain"
    "xml" -> "application/xml"
    "csv" -> "text/csv"
    "png" -> "image/png"
    "jpg", "jpeg" -> "image/jpeg"
    "webp" -> "image/webp"
    "avif" -> "image/avif"
    "gif" -> "image/gif"
    "ico" -> "image/x-icon"
    "mp3" -> "audio/mpeg"
    "ogg" -> "audio/ogg"
    "wav" -> "audio/wav"
    "m4a" -> "audio/mp4"
    "aac" -> "audio/aac"
    "mp4" -> "video/mp4"
    "webm" -> "video/webm"
    "woff" -> "font/woff"
    "woff2" -> "font/woff2"
    "ttf" -> "font/ttf"
    "otf" -> "font/otf"
    "wasm" -> "application/wasm"
    "glb" -> "model/gltf-binary"
    "gltf" -> "model/gltf+json"
    "ktx2" -> "image/ktx2"
    "bin", "data", "basis" -> "application/octet-stream"
    else -> null
}
