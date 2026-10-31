package xyz.xiangban.xiaoban.common.community

import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.charset.CodingErrorAction
import java.security.MessageDigest
import java.util.zip.ZipInputStream

data class CommunityGameArchive(val game: MobileGame, val files: Map<String, ByteArray>) {
    companion object {
        fun read(bytes: ByteArray, expected: CommunityGameDownload): CommunityGameArchive {
            require(bytes.size.toLong() == expected.size && bytes.size in 1..CommunityGameDownload.MAX_ZIP_BYTES) { "游戏包下载不完整" }
            val hash = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }
            require(hash.equals(expected.sha256, true)) { "游戏包校验失败，请重新下载" }
            val contents = linkedMapOf<String, ByteArray>()
            val seen = mutableSetOf<String>()
            var total = 0
            var entries = 0
            ZipInputStream(ByteArrayInputStream(bytes)).use { zip ->
                while (true) {
                    val entry = zip.nextEntry ?: break
                    require(++entries <= 161) { "游戏包文件过多" }
                    if (entry.isDirectory) {
                        val dir = entry.name.removeSuffix("/")
                        MobileGameFormat.path("$dir/check.txt")
                        zip.closeEntry()
                        continue
                    }
                    MobileGameFormat.path(entry.name)
                    require(seen.add(entry.name.lowercase())) { "游戏包存在重复路径" }
                    require(contents.size < 41) { "游戏包文件过多" }
                    val max = if (entry.name == "manifest.json") 64 * 1024 else MobileGameFormat.FILE_BYTES
                    val output = ByteArrayOutputStream()
                    val buffer = ByteArray(8192)
                    while (true) {
                        val count = zip.read(buffer)
                        if (count < 0) break
                        total += count
                        require(output.size() + count <= max && total <= MobileGameFormat.TOTAL_BYTES + 64 * 1024) { "游戏包展开后过大" }
                        output.write(buffer, 0, count)
                    }
                    contents[entry.name] = output.toByteArray()
                    zip.closeEntry()
                }
            }
            fun text(bytes: ByteArray): String = Charsets.UTF_8.newDecoder().onMalformedInput(CodingErrorAction.REPORT)
                .onUnmappableCharacter(CodingErrorAction.REPORT).decode(ByteBuffer.wrap(bytes)).toString()
            val game = readMobileGameManifest(text(requireNotNull(contents["manifest.json"]) { "游戏包缺少清单" }))
            MobileGameFormat.files(contents.filterKeys { it != "manifest.json" }.map { (path, bytes) -> MobileGameFile(path, text(bytes)) })
            return CommunityGameArchive(game, contents)
        }
    }
}
