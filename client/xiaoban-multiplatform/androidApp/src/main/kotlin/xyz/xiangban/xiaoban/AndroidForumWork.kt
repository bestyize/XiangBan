package xyz.xiangban.xiaoban

import kotlinx.serialization.json.JsonObject
import org.apache.commons.compress.archivers.zip.ZipFile
import org.apache.commons.compress.utils.SeekableInMemoryByteChannel
import xyz.xiangban.xiaoban.common.community.CommunityJson
import xyz.xiangban.xiaoban.common.community.ContentPackLimits
import xyz.xiangban.xiaoban.common.community.FORUM_MEMORY_MAX_BYTES
import xyz.xiangban.xiaoban.common.community.ForumUploadFile
import xyz.xiangban.xiaoban.common.community.PreparedForumWork
import xyz.xiangban.xiaoban.common.community.forumHtmlWorkFiles
import xyz.xiangban.xiaoban.common.community.forumUploadName
import xyz.xiangban.xiaoban.common.community.manifestValue

private const val ManifestLimit = 64 * 1024

internal fun prepareForumWork(selected: ForumUploadFile): PreparedForumWork {
    val originalName = selected.name.substringAfterLast('/').substringAfterLast('\\').trim()
    val extension = originalName.substringAfterLast('.', "")
    val format = extension.lowercase()
    require(format in setOf("zip", "html", "htm")) { "请选择 ZIP、HTML 或 HTM 作品文件" }
    val base = forumUploadName(originalName.substringBeforeLast('.')).take(119 - extension.length).trimEnd { it.isHighSurrogate() }
    val file = selected.copy(name = "$base.$extension")
    if (format == "zip") return prepareForumZip(file)
    val (game, files) = forumHtmlWorkFiles(file)
    val zip = ForumUploadFile("${game.id}.zip", "application/zip", RuntimeV2Archive.encode(files))
    return PreparedForumWork(zip, file.name, game.kind, game.name, "")
}

/** Read only the root manifest to prefill the form. The resource server validates the full archive. */
private fun prepareForumZip(file: ForumUploadFile): PreparedForumWork {
    require(file.bytes.size.toLong() in 1..FORUM_MEMORY_MAX_BYTES) { "ZIP 文件需不超过 32 MB" }
    val manifest = try {
        ZipFile.builder().setSeekableByteChannel(SeekableInMemoryByteChannel(file.bytes))
            .setMaxNumberOfDisks(1).setIgnoreLocalFileHeader(true).get().use { zip ->
                val entries = zip.getEntries("manifest.json").toList()
                require(entries.size == 1) { "ZIP 根目录需要有且只有一个 manifest.json" }
                val entry = entries.single()
                require(entry.size in 1..ManifestLimit.toLong() && !entry.isDirectory) { "manifest.json 必须是 64 KB 以内的文件" }
                val bytes = ByteArray(entry.size.toInt())
                zip.getInputStream(entry).use { input ->
                    var offset = 0
                    while (offset < bytes.size) {
                        val count = input.read(bytes, offset, bytes.size - offset)
                        require(count > 0) { "无法读取完整的 manifest.json" }
                        offset += count
                    }
                    require(input.read() == -1) { "manifest.json 的大小与 ZIP 清单不一致" }
                }
                CommunityJson.parseToJsonElement(bytes.decodeToString(throwOnInvalidSequence = true)) as? JsonObject
                    ?: throw IllegalArgumentException("manifest.json 必须是 JSON 对象")
            }
    } catch (error: java.io.IOException) {
        throw IllegalArgumentException("无法读取 ZIP 文件，请检查压缩包是否完整", error)
    }
    val kind = manifestValue(manifest, "kind").orEmpty()
    val limit = ContentPackLimits[kind] ?: throw IllegalArgumentException("manifest.json 的作品类型不支持发布")
    require(file.bytes.size <= limit) { "该类作品的 ZIP 不得超过 ${limit / 1024 / 1024} MB" }
    val title = manifestValue(manifest, "name", "title").orEmpty().ifBlank { file.name.substringBeforeLast('.') }
        .trim().take(60).trimEnd { it.isHighSurrogate() || it.isWhitespace() }
    val summary = manifestValue(manifest, "description", "summary").orEmpty().trim().take(500).trimEnd { it.isHighSurrogate() || it.isWhitespace() }
    return PreparedForumWork(file.copy(mime = "application/zip"), file.name, kind, title, summary)
}
