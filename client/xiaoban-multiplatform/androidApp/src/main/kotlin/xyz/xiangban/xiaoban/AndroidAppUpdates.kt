package xyz.xiangban.xiaoban

import android.content.Intent
import android.content.pm.PackageInfo
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.os.SystemClock
import android.provider.Settings
import androidx.core.content.FileProvider
import androidx.core.content.pm.PackageInfoCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import org.json.JSONObject
import xyz.xiangban.xiaoban.business.communitynative.AppUpdateState
import xyz.xiangban.xiaoban.business.communitynative.AppUpdates
import java.io.File
import java.io.IOException
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest
import kotlin.coroutines.coroutineContext

class UpdateFileProvider : FileProvider()

internal class AndroidAppUpdates(private val activity: MainActivity) : AppUpdates {
    override val currentVersion = BuildConfig.VERSION_NAME
    private val prefs = activity.getSharedPreferences("app-updates", 0)
    private val mutableState = MutableStateFlow(AppUpdateState(automatic = prefs.getBoolean("automatic", true)))
    override val state = mutableState.asStateFlow()
    private var job: Job? = null
    private var lastAttempt: Long? = null
    private var release: Release? = null
    private var showAutomatically = false

    override fun setAutomatic(enabled: Boolean) {
        prefs.edit().putBoolean("automatic", enabled).apply()
        mutableState.update { it.copy(automatic = enabled) }
        if (enabled) checkOnStart()
    }

    fun checkOnStart() {
        if (!state.value.automatic || job?.isActive == true) return
        val elapsed = SystemClock.elapsedRealtime()
        if (lastAttempt?.let { elapsed - it < RETRY_INTERVAL } == true) return
        val last = prefs.getLong("last-check", 0L)
        val now = System.currentTimeMillis()
        if (now >= last && now - last < CHECK_INTERVAL) return
        checkRelease(automatic = true)
    }

    override fun check() {
        mutableState.update { it.copy(visible = true) }
        if (job?.isActive != true) checkRelease(automatic = false)
    }

    override fun dismiss() {
        showAutomatically = false
        mutableState.update { it.copy(visible = false) }
    }

    private fun checkRelease(automatic: Boolean) {
        showAutomatically = automatic
        lastAttempt = SystemClock.elapsedRealtime()
        release = null
        mutableState.update { it.copy(busy = true, version = null, notes = "", message = "正在检查更新…", percent = null) }
        job = activity.lifecycleScope.launch {
            try {
                val found = withContext(Dispatchers.IO) { fetchRelease() }
                release = found
                val show = found != null && showAutomatically && state.value.automatic &&
                    prefs.getLong("last-prompt", 0) < found.versionCode
                prefs.edit().putLong("last-check", System.currentTimeMillis()).apply {
                    if (show) putLong("last-prompt", found.versionCode)
                }.apply()
                mutableState.update {
                    it.copy(
                        visible = it.visible || show,
                        version = found?.version,
                        notes = found?.notes.orEmpty(),
                        message = if (found == null) "当前已是最新版本" else "下载后由 Android 确认安装。更新会保留现有数据。",
                    )
                }
            } catch (e: CancellationException) {
                throw e
            } catch (_: Exception) {
                mutableState.update { it.copy(message = "暂时无法检查更新，请稍后重试。") }
            } finally {
                mutableState.update { it.copy(busy = false) }
            }
        }
    }

    override fun install() {
        val target = release ?: return
        if (job?.isActive == true) return
        mutableState.update { it.copy(busy = true, message = "正在准备安装包…", percent = null) }
        job = activity.lifecycleScope.launch {
            try {
                val apk = withTimeoutOrNull(DOWNLOAD_TIMEOUT) {
                    withContext(Dispatchers.IO) { prepareApk(target) }
                } ?: throw UpdateException("下载更新超时，请重试。")
                val canOpenInstaller = state.value.visible && activity.lifecycle.currentState.isAtLeast(Lifecycle.State.RESUMED)
                if (!canOpenInstaller) {
                    mutableState.update { it.copy(message = "下载完成，打开检查更新即可继续安装。") }
                } else if (Build.VERSION.SDK_INT >= 26 && !activity.packageManager.canRequestPackageInstalls()) {
                    activity.startActivity(Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, Uri.parse("package:${activity.packageName}")))
                    mutableState.update { it.copy(message = "允许小伴安装应用后，返回并再次点击「安装更新」。") }
                } else {
                    val uri = FileProvider.getUriForFile(activity, "${activity.packageName}.app-updates", apk)
                    activity.startActivity(Intent(Intent.ACTION_VIEW).apply {
                        setDataAndType(uri, "application/vnd.android.package-archive")
                        addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    })
                    mutableState.update { it.copy(message = "请在系统安装界面确认更新。") }
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: UpdateException) {
                mutableState.update { it.copy(message = e.message.orEmpty()) }
            } catch (_: Exception) {
                mutableState.update { it.copy(message = "下载或启动安装失败，请检查网络和剩余存储空间后重试。") }
            } finally {
                mutableState.update { it.copy(busy = false, percent = null) }
            }
        }
    }

    private fun fetchRelease(): Release? {
        val connection = open(URL(FEED))
        try {
            if (connection.responseCode == 404 || connection.responseCode == 204) return null
            check(connection.responseCode == 200)
            val text = connection.inputStream.buffered().use { input ->
                val data = ByteArray(MAX_MANIFEST + 1)
                var read = 0
                while (read < data.size) {
                    val size = input.read(data, read, data.size - read)
                    if (size == -1) break
                    read += size
                }
                check(read <= MAX_MANIFEST)
                String(data, 0, read, Charsets.UTF_8)
            }
            val manifest = JSONObject(text)
            check(manifest.getInt("schemaVersion") == 1)
            val version = manifest.getString("version")
            val versionCode = manifest.getLong("versionCode")
            val sha256 = manifest.getString("sha256")
            val bytes = manifest.getLong("bytes")
            check(version.matches(VERSION) && versionCode in 1..MAX_VERSION_CODE)
            check(sha256.matches(SHA256) && bytes in 1..MAX_APK)
            val url = URL(manifest.getString("url"))
            val packagePath = "/bestyize/XiangBan/releases/download/xiaoban-wallpaper-v"
            check(url.host == "github.com" && url.path.startsWith(packagePath) &&
                url.path.substringAfter(packagePath).matches(APK_PATH) &&
                url.path.endsWith("-$sha256.apk") && url.query == null && url.ref == null)
            return Release(version, versionCode, url, sha256, bytes, manifest.optString("notes").take(4_000))
                .takeIf { versionCode > BuildConfig.VERSION_CODE }
        } finally {
            connection.disconnect()
        }
    }

    private suspend fun prepareApk(target: Release): File {
        val directory = activity.cacheDir.resolve("app-updates")
        val apk = directory.resolve("update.apk")
        if (apk.isFile && apk.length() == target.bytes &&
            apk.inputStream().buffered().use { input ->
                val digest = MessageDigest.getInstance("SHA-256")
                val buffer = ByteArray(64 * 1024)
                while (true) {
                    coroutineContext.ensureActive()
                    val count = input.read(buffer)
                    if (count == -1) break
                    digest.update(buffer, 0, count)
                }
                digest.hex() == target.sha256
            }
        ) {
            verifyPackage(apk, target)
            return apk
        }
        directory.mkdirs()
        apk.delete()
        val partial = directory.resolve("update.pending.apk")
        val connection = open(target.url)
        try {
            check(connection.responseCode == 200)
            val digest = MessageDigest.getInstance("SHA-256")
            connection.inputStream.buffered().use { input ->
                partial.outputStream().buffered().use { output ->
                    val buffer = ByteArray(64 * 1024)
                    var received = 0L
                    while (true) {
                        coroutineContext.ensureActive()
                        val count = input.read(buffer)
                        if (count == -1) break
                        received += count
                        if (received > target.bytes) throw UpdateException("安装包大小不符，请重新检查更新。")
                        digest.update(buffer, 0, count)
                        output.write(buffer, 0, count)
                        val percent = (received * 100 / target.bytes).toInt()
                        if (state.value.percent != percent) {
                            mutableState.update { it.copy(message = "正在下载并校验安装包…", percent = percent) }
                        }
                    }
                    if (received != target.bytes || digest.hex() != target.sha256) {
                        throw UpdateException("安装包校验失败，请重新检查更新。")
                    }
                }
            }
            verifyPackage(partial, target)
            check(partial.renameTo(apk))
            return apk
        } finally {
            connection.disconnect()
            partial.delete()
        }
    }

    @Suppress("DEPRECATION")
    private fun verifyPackage(file: File, target: Release) {
        val manager = activity.packageManager
        val flags = if (Build.VERSION.SDK_INT >= 28) PackageManager.GET_SIGNING_CERTIFICATES else PackageManager.GET_SIGNATURES
        val candidate = manager.getPackageArchiveInfo(file.absolutePath, flags)
        if (candidate == null || candidate.packageName != activity.packageName ||
            candidate.versionName != target.version || PackageInfoCompat.getLongVersionCode(candidate) != target.versionCode ||
            target.versionCode <= BuildConfig.VERSION_CODE
        ) throw UpdateException("安装包应用或版本不符。")
        val current = manager.getPackageInfo(activity.packageName, flags)
        fun signers(info: PackageInfo) =
            if (Build.VERSION.SDK_INT >= 28) info.signingInfo?.apkContentsSigners?.toSet().orEmpty()
            else info.signatures?.toSet().orEmpty()
        val expected = signers(current)
        if (expected.isEmpty() || signers(candidate) != expected) {
            throw UpdateException("安装包签名与当前小伴不一致，无法覆盖安装。")
        }
    }

    private fun open(initial: URL): HttpURLConnection {
        var url = initial
        repeat(5) {
            check(url.protocol == "https" && url.host in RELEASE_HOSTS && url.userInfo == null && url.port == -1)
            val connection = (url.openConnection() as HttpURLConnection).apply {
                connectTimeout = 10_000
                readTimeout = 20_000
                instanceFollowRedirects = false
                useCaches = false
            }
            try {
                if (connection.responseCode !in REDIRECTS) return connection
                url = URL(url, connection.getHeaderField("Location") ?: error("Missing redirect"))
            } catch (e: Exception) {
                connection.disconnect()
                throw e
            }
            connection.disconnect()
        }
        throw IOException("Too many redirects")
    }

    private fun MessageDigest.hex() = digest().joinToString("") { "%02x".format(it) }

    private class UpdateException(message: String) : IOException(message)
    private data class Release(val version: String, val versionCode: Long, val url: URL, val sha256: String, val bytes: Long, val notes: String)

    private companion object {
        const val FEED = "https://raw.githubusercontent.com/bestyize/XiangBan/client-updates/updates/android-arm64-v8a.json"
        const val CHECK_INTERVAL = 24 * 60 * 60 * 1000L
        const val RETRY_INTERVAL = 60 * 60 * 1000L
        const val DOWNLOAD_TIMEOUT = 10 * 60 * 1000L
        const val MAX_MANIFEST = 32 * 1024
        const val MAX_APK = 512 * 1024 * 1024L
        const val MAX_VERSION_CODE = 2_100_000_000L
        val VERSION = Regex("[0-9]+\\.[0-9]+\\.[0-9]+")
        val SHA256 = Regex("[0-9a-f]{64}")
        val APK_PATH = Regex("[0-9]+\\.[0-9]+\\.[0-9]+/XiaoBan-Update-android-arm64-v8a-[0-9a-f]{64}\\.apk")
        val RELEASE_HOSTS = setOf("raw.githubusercontent.com", "github.com", "release-assets.githubusercontent.com", "objects.githubusercontent.com")
        val REDIRECTS = setOf(301, 302, 303, 307, 308)
    }
}
