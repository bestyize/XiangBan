package xyz.xiangban.xiaoban

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.JsonPrimitive
import xyz.xiangban.xiaoban.common.community.CommunityGameDownload
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.RecentCommunityGame
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URI
import java.nio.file.Files
import java.security.MessageDigest

internal object AndroidRuntimeV2Community {
    const val PACK_ID = "runtime.v2.pack.id"
    const val VERSION = "runtime.v2.pack.version"
    const val DIGEST = "runtime.v2.pack.digest"
    const val KIND = "runtime.v2.pack.kind"
    const val AUTHOR = "runtime.v2.pack.author"
    const val LEGACY_VERSIONS = "runtime.v2.pack.legacy.versions"
    private const val CACHE_BYTES = 1024L * 1024 * 1024
    private val shaPattern = Regex("[a-fA-F0-9]{64}")
    private val lock = Mutex()

    suspend fun needsMobileConfirmation(context: Context, info: CommunityGameDownload): Boolean = withContext(Dispatchers.IO) {
        if (info.size <= 20L * 1024 * 1024) return@withContext false
        val connectivity = context.getSystemService(ConnectivityManager::class.java)
        val mobile = connectivity.getNetworkCapabilities(connectivity.activeNetwork)?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) == true
        mobile && !directory(context, info.packId, info.version, info.sha256).isDirectory
    }

    suspend fun play(activity: Activity, info: CommunityGameDownload, pack: Pack): Boolean {
        require(info.validate(pack, info.version) == 2)
        withContext(Dispatchers.IO) { lock.withLock { prepare(activity, info, pack.slug) } }
        return withContext(Dispatchers.Main) {
            if (activity.isFinishing || activity.isDestroyed) return@withContext false
            val appContext = activity.applicationContext
            val recent = RecentCommunityGame(info.packId, info.version, pack.title.take(200))
            val intent = RuntimeV2Activity.launchIntent(activity) {
                AndroidCommunityGames.opened(appContext, recent)
            }
            val legacyVersions = pack.versions.asSequence()
                .filter { it.version in 1 until info.version }
                .filter { (it.manifest["runtimeProtocol"] ?: JsonPrimitive(1)) == JsonPrimitive(1) }
                .map { it.version }.sortedDescending().toList().toIntArray()
            activity.startActivity(intent
                .putExtra(PACK_ID, info.packId)
                .putExtra(VERSION, info.version)
                .putExtra(DIGEST, info.sha256)
                .putExtra(KIND, info.kind)
                .putExtra(AUTHOR, pack.author?.nickname?.take(80))
                .putExtra(LEGACY_VERSIONS, legacyVersions)
                .putExtra(RuntimeV2Activity.EXTRA_NETWORK_DISABLED, pack.networkDisabled != false))
            true
        }
    }

    fun load(context: Context, intent: Intent): RuntimeV2Package {
        val id = intent.getLongExtra(PACK_ID, 0)
        val version = intent.getIntExtra(VERSION, 0)
        val digest = intent.getStringExtra(DIGEST).orEmpty()
        val kind = intent.getStringExtra(KIND).orEmpty()
        return RuntimeV2Package.loadCommunity(directory(context, id, version, digest), id, version, kind)
    }

    private fun directory(context: Context, id: Long, version: Int, digest: String): File {
        require(id > 0 && version > 0 && shaPattern.matches(digest)) { "作品包信息无效" }
        return context.cacheDir.resolve("runtime-v2-packs/$id-$version-${digest.lowercase()}")
    }

    private suspend fun prepare(context: Context, info: CommunityGameDownload, slug: String) {
        val target = directory(context, info.packId, info.version, info.sha256)
        if (target.isDirectory) {
            val cached = runCatching { RuntimeV2Package.loadCommunity(target, info.packId, info.version, info.kind).manifest }
            if (cached.getOrNull()?.id == slug) {
                target.setLastModified(System.currentTimeMillis())
                return
            }
            check(target.deleteRecursively()) { "无法更新作品缓存" }
        }
        val root = checkNotNull(target.parentFile)
        check(root.isDirectory || root.mkdirs()) { "无法创建作品缓存" }
        root.listFiles().orEmpty().filter { it.name.startsWith("unpack-") }.forEach { it.deleteRecursively() }
        val partial = root.resolve("${target.name}.part")
        trimCache(root, partial, info.size * 2)
        download(info, partial)
        val verified = sha256(partial)
        if (!verified.equals(info.sha256, true)) {
            partial.delete()
            error("作品包校验失败，请重新下载")
        }
        val staging = Files.createTempDirectory(root.toPath(), "unpack-").toFile()
        try {
            val manifest = RuntimeV2Archive.extract(partial, staging, info.kind)
            require(manifest.id == slug) { "作品包标识不匹配" }
            currentCoroutineContext().ensureActive()
            check(staging.renameTo(target)) { "无法保存作品缓存" }
        } finally {
            staging.deleteRecursively()
            partial.delete()
        }
        trimCache(root, target, 0)
    }

    private suspend fun download(info: CommunityGameDownload, file: File) {
        if (file.length() > info.size) file.delete()
        val offset = file.length()
        if (offset == info.size) return
        val uri = runCatching { URI(info.url) }.getOrElse { error("下载地址不受支持") }
        val debugLoopback = BuildConfig.DEBUG && uri.scheme == "http" && uri.host == "127.0.0.1" && uri.port in 1024..65535
        require(uri.userInfo == null && uri.fragment == null && uri.host != null &&
            (debugLoopback || uri.scheme == "https" && uri.port in listOf(-1, 443) && !RuntimeV2Proxy.isBlockedHost(uri.host))) {
            "下载地址不受支持"
        }
        val connection = uri.toURL().openConnection() as HttpURLConnection
        connection.instanceFollowRedirects = false
        connection.connectTimeout = 15_000
        connection.readTimeout = 30_000
        connection.setRequestProperty("Accept-Encoding", "identity")
        if (offset > 0) connection.setRequestProperty("Range", "bytes=$offset-")
        try {
            val start = when (connection.responseCode) {
                200 -> 0L
                206 -> {
                    require(offset > 0 && connection.getHeaderField("Content-Range") == "bytes $offset-${info.size - 1}/${info.size}") {
                        "作品包续传范围不匹配"
                    }
                    offset
                }
                else -> error("作品包下载失败，请稍后重试")
            }
            require(connection.contentLengthLong in listOf(-1, info.size - start)) { "作品包大小不匹配" }
            connection.inputStream.use { input ->
                FileOutputStream(file, start > 0).use { output ->
                    val buffer = ByteArray(32 * 1024)
                    var received = start
                    while (true) {
                        currentCoroutineContext().ensureActive()
                        val count = input.read(buffer)
                        if (count == -1) break
                        received += count
                        if (received > info.size) {
                            file.delete()
                            error("作品包超过声明大小")
                        }
                        output.write(buffer, 0, count)
                    }
                    require(received == info.size) { "作品包未下载完整，请重试" }
                }
            }
        } catch (_: IOException) {
            error("作品包下载中断，请重试")
        } finally {
            connection.disconnect()
        }
    }

    private suspend fun sha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buffer = ByteArray(32 * 1024)
            while (true) {
                currentCoroutineContext().ensureActive()
                val count = input.read(buffer)
                if (count == -1) break
                digest.update(buffer, 0, count)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    private fun trimCache(root: File, keep: File, reserve: Long) {
        val entries = root.listFiles().orEmpty().sortedBy { it.lastModified() }.map { entry ->
            entry to entry.walkTopDown().filter(File::isFile).sumOf(File::length)
        }
        var total = entries.sumOf { it.second }
        for ((entry, size) in entries) {
            if (total + reserve <= CACHE_BYTES) break
            if (entry != keep && entry.deleteRecursively()) total -= size
        }
    }
}
