package xyz.xiangban.xiaoban

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import org.json.JSONObject
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import java.io.File
import java.io.RandomAccessFile
import java.nio.file.Files
import java.security.MessageDigest
import java.util.concurrent.ConcurrentHashMap

/** The descriptor is embedded in the app; the server cannot replace a game with different bytes. */
internal object AndroidBuiltinGames {
    internal data class Game(val id: String, val kind: String, val url: String, val sha256: String, val bytes: Long, val sourceSha256: String)
    private val locks = ConcurrentHashMap<String, Mutex>()
    private val hash = Regex("[a-f0-9]{64}")
    private val url = Regex("https://res\\.xiangban\\.xyz/files/[a-f0-9]{32}\\.zip")

    private fun games(context: Context): List<Game> {
        val json = context.assets.open("builtin-games/distribution.json").bufferedReader().use { JSONObject(it.readText()) }
        require(json.getInt("version") == 1)
        val items = json.getJSONArray("games")
        val games = (0 until items.length()).map { index ->
            val item = items.getJSONObject(index)
            Game(item.getString("id"), item.getString("kind"), item.getString("url"), item.getString("sha256"), item.getLong("bytes"), item.getString("sourceSha256"))
        }
        val expected = BuiltinMobileGames.ids.filterNot(BuiltinMobileGames::isBundledTool).toSet()
        require(games.map { it.id }.toSet() == expected && games.size == expected.size)
        require(games.all { it.kind in setOf("game", "app") && url.matches(it.url) && hash.matches(it.sha256) && hash.matches(it.sourceSha256) && it.bytes in 1..RuntimeV2Archive.MAX_ZIP_BYTES })
        return games
    }

    fun ids(context: Context): List<String> = games(context).map { it.id }

    private suspend fun validCache(root: File, game: Game): Boolean {
        var safe = true
        val files = root.walkTopDown().onEnter { dir ->
            if (Files.isSymbolicLink(dir.toPath())) { safe = false; false } else true
        }.filter { it.isFile && it != root.resolve(".complete") }.sortedBy { it.relativeTo(root).invariantSeparatorsPath }.toList()
        if (!safe || files.size > 3_000) return false
        val tree = MessageDigest.getInstance("SHA-256")
        var total = 0L
        for (file in files) {
            if (Files.isSymbolicLink(file.toPath()) || file.length() > 50L * 1024 * 1024) return false
            total += file.length()
            if (total > 100L * 1024 * 1024) return false
            val digest = MessageDigest.getInstance("SHA-256")
            file.inputStream().use { input ->
                val buffer = ByteArray(64 * 1024)
                while (true) {
                    currentCoroutineContext().ensureActive()
                    val count = input.read(buffer)
                    if (count == -1) break
                    digest.update(buffer, 0, count)
                }
            }
            tree.update(file.relativeTo(root).invariantSeparatorsPath.toByteArray(Charsets.UTF_8))
            tree.update(0.toByte())
            tree.update(digest.digest())
        }
        return tree.digest().joinToString("") { "%02x".format(it) } == game.sourceSha256
    }

    suspend fun ensure(context: Context, id: String): File {
        val game = games(context).singleOrNull { it.id == id } ?: error("没有找到内置作品")
        return ensureIn(context.filesDir.resolve("builtin-games"), game) { path ->
            RuntimeV2Package.loadBuiltin(path, id, context.assets)
        }
    }

    internal suspend fun ensureIn(root: File, game: Game, load: (File) -> RuntimeV2Package): File = withContext(Dispatchers.IO) {
        val id = game.id
        locks.computeIfAbsent(id) { Mutex() }.withLock {
            check(root.isDirectory || root.mkdirs()) { "无法创建游戏缓存" }
            // The runner is a separate Android process, so a coroutine mutex alone is insufficient.
            RandomAccessFile(root.resolve("$id.lock"), "rw").channel.use { channel ->
                channel.lock().use {
                    val target = root.resolve("$id-${game.sha256}")
                    if (target.isDirectory && runCatching {
                            target.resolve(".complete").readText() == game.sha256 &&
                                validCache(target, game) && load(target).manifest.id == id
                        }.getOrDefault(false)) return@withLock target
                    val partial = root.resolve("$id-${game.sha256}.part")
                    AndroidRuntimeV2Community.download(game.url, game.bytes, partial)
                    if (AndroidRuntimeV2Community.sha256(partial) != game.sha256) {
                        partial.delete()
                        error("游戏资源校验失败，请重试")
                    }
                    val staging = Files.createTempDirectory(root.toPath(), "unpack-$id-").toFile()
                    try {
                        val manifest = RuntimeV2Archive.extract(partial, staging, game.kind)
                        require(manifest.id == id) { "游戏包标识不匹配" }
                        load(staging)
                        staging.resolve(".complete").writeText(game.sha256)
                        currentCoroutineContext().ensureActive()
                        if (target.exists()) check(target.deleteRecursively()) { "无法更新游戏缓存" }
                        check(staging.renameTo(target)) { "无法保存游戏缓存" }
                        partial.delete()
                        root.listFiles().orEmpty().filter { it.isDirectory && it.name.startsWith("$id-") && it != target }
                            .forEach { it.deleteRecursively() }
                        target
                    } finally {
                        staging.deleteRecursively()
                    }
                }
            }
        }
    }
}
