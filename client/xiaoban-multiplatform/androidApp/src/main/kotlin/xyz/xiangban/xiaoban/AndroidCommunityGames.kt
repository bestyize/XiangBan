package xyz.xiangban.xiaoban

import android.app.Activity
import android.content.Context
import android.content.Intent
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.common.community.*
import java.io.ByteArrayOutputStream
import java.io.File
import java.net.URL
import javax.net.ssl.HttpsURLConnection
import java.util.UUID

/** Community packages never enter the creator workspace. Only the bounded ZIP cache is disposable. */
internal object AndroidCommunityGames {
    private val lock = Mutex()
    private val history = MutableStateFlow<List<RecentCommunityGame>>(emptyList())
    private var initialized = false
    private val sessions = mutableMapOf<String, PlaySession>()
    data class PlaySession(val archive: CommunityGameArchive, val recent: RecentCommunityGame,
        val bookmark: suspend (Boolean?) -> Boolean, val login: () -> Unit)
    @Synchronized fun recent(context: Context): MutableStateFlow<List<RecentCommunityGame>> {
        if (!initialized) {
            history.value = CommunityGameHistory.decode(context.getSharedPreferences("community_game_history", Context.MODE_PRIVATE).getString("recent", null))
            initialized = true
        }
        return history
    }
    fun opened(context: Context, session: PlaySession) = opened(context, session.recent)

    @Synchronized fun opened(context: Context, recentGame: RecentCommunityGame) {
        recent(context)
        history.value = CommunityGameHistory.opened(history.value, recentGame)
        context.getSharedPreferences("community_game_history", Context.MODE_PRIVATE).edit()
            .putString("recent", CommunityGameHistory.encode(history.value)).apply()
    }
    @Synchronized fun session(key: String?) = sessions[key]
    @Synchronized fun forget(key: String?) { sessions.remove(key) }

    suspend fun play(activity: Activity, info: CommunityGameDownload, title: String,
        bookmark: suspend (Boolean?) -> Boolean, login: () -> Unit): Boolean {
        val archive = withContext(Dispatchers.IO) { lock.withLock { cached(activity, info) } }
        return withContext(Dispatchers.Main) {
            if (activity.isFinishing || activity.isDestroyed) return@withContext false
            val key = UUID.randomUUID().toString()
            synchronized(this@AndroidCommunityGames) {
                sessions[key] = PlaySession(archive, RecentCommunityGame(info.packId, info.version, title.take(200)), bookmark, login)
            }
            try {
                activity.startActivity(Intent(activity, MobileGameActivity::class.java)
                    .putExtra(MobileGameActivity.EXTRA_GAME_ID, "pack-${info.packId}-v${info.version}")
                    .putExtra(MobileGameActivity.EXTRA_COMMUNITY_SESSION, key))
                true
            } catch (e: Exception) { forget(key); throw e }
        }
    }

    private suspend fun cached(context: Context, info: CommunityGameDownload): CommunityGameArchive {
        require(info.packId > 0 && info.version > 0 && info.kind == "game" && Regex("[a-fA-F0-9]{64}").matches(info.sha256)
            && info.size in 1..CommunityGameDownload.MAX_ZIP_BYTES.toLong()) { "游戏包信息无效" }
        val root = File(context.cacheDir, "community-games").apply { mkdirs() }
        val file = File(root, "${info.packId}-${info.version}-${info.sha256.lowercase()}.zip")
        if (file.isFile && file.length() == info.size) {
            val archive = runCatching { CommunityGameArchive.read(file.readBytes(), info) }.getOrNull()
            if (archive != null) { file.setLastModified(System.currentTimeMillis()); return archive }
        }
        file.delete()
        val url = URL(info.url)
        require(url.protocol == "https" && url.userInfo == null && url.ref == null && url.port in listOf(-1, 443)) { "下载地址不受支持" }
        val connection = url.openConnection() as HttpsURLConnection
        connection.instanceFollowRedirects = false
        connection.connectTimeout = 15_000
        connection.readTimeout = 30_000
        // This separate connection carries no forum tokens, cookies or native bridge.
        val bytes = try {
            require(connection.responseCode == 200) { "游戏下载失败，请稍后重试" }
            require(connection.contentLengthLong == -1L || connection.contentLengthLong == info.size) { "游戏包大小不匹配" }
            connection.inputStream.use { input ->
                val output = ByteArrayOutputStream()
                val buffer = ByteArray(8192)
                while (true) {
                    currentCoroutineContext().ensureActive()
                    val count = input.read(buffer)
                    if (count < 0) break
                    require(output.size().toLong() + count <= info.size) { "游戏包超过声明大小" }
                    output.write(buffer, 0, count)
                }
                output.toByteArray()
            }
        } finally { connection.disconnect() }
        val archive = CommunityGameArchive.read(bytes, info)
        currentCoroutineContext().ensureActive()
        val temp = File.createTempFile("download-", ".tmp", root)
        try { temp.writeBytes(bytes); check(temp.renameTo(file)) { "游戏缓存保存失败" } }
        finally { temp.delete() }
        // Eight packages, at most 40 MiB. Saves/history/collections are elsewhere.
        root.listFiles().orEmpty().filter { it.extension == "zip" }.sortedByDescending { it.lastModified() }.drop(8).forEach { it.delete() }
        return archive
    }
}
