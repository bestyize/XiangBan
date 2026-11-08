package xyz.xiangban.xiaoban

import org.json.JSONObject
import java.io.File

/** 协议 1 时代保存的存档。只在作品第一次以协议 2 打开时读取一次，由宿主放进 xb.info.legacySave。 */
internal object LegacySaves {
    private const val MAX_SAVE_BYTES = 32 * 1024

    fun builtin(filesDir: File, id: String): JSONObject? = runCatching {
        require(id in xyz.xiangban.xiaoban.common.community.BuiltinMobileGames.ids)
        read(filesDir.canonicalFile.resolve("workspace/saves/$id.json"))
    }.getOrNull()

    fun community(filesDir: File, packId: Long, versions: IntArray): JSONObject? {
        if (packId <= 0) return null
        val directory = filesDir.canonicalFile.resolve("community-game-data/saves")
        return versions.asSequence().filter { it > 0 }.mapNotNull { version ->
            runCatching { read(directory.resolve("pack-$packId-v$version.json")) }.getOrNull()
        }.firstOrNull()
    }

    private fun read(file: File): JSONObject? {
        if (file.canonicalFile != file.absoluteFile || !file.isFile || file.length() > MAX_SAVE_BYTES) return null
        return JSONObject(file.readText(Charsets.UTF_8))
    }
}
