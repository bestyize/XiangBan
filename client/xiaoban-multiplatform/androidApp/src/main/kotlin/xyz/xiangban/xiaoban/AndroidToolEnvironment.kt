package xyz.xiangban.xiaoban

import android.content.Context
import android.os.Build
import android.system.Os
import android.system.OsConstants
import java.io.File
import java.io.IOException
import java.net.URL
import java.security.MessageDigest
import javax.net.ssl.HttpsURLConnection
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream
import org.json.JSONObject
import org.tukaani.xz.XZInputStream
import xyz.xiangban.xiaoban.business.communitynative.ToolEnvironmentController
import xyz.xiangban.xiaoban.business.communitynative.ToolEnvironmentState
import xyz.xiangban.xiaoban.business.communitynative.ToolEnvironmentStatus as Status

internal class AndroidToolEnvironment(context: Context) : ToolEnvironmentController {
    private data class Manifest(val url: String, val bytes: Long, val sha256: String)

    private val nativeDir = File(context.applicationInfo.nativeLibraryDir)
    private val manifest = runCatching {
        require(Build.VERSION.SDK_INT >= 26)
        val json = context.assets.open("tool-environment.json").bufferedReader().use { JSONObject(it.readText()) }
        val info = Manifest(json.getString("url"), json.getLong("bytes"), json.getString("sha256"))
        require(URL(info.url).protocol == "https" && info.bytes in 1..250_000_000L)
        require(info.sha256.matches(Regex("[0-9a-f]{64}")))
        require(nativeDir.resolve("libproot.so").canExecute() && nativeDir.resolve("libproot-loader.so").canExecute())
        info
    }.getOrNull()
    private val storage = context.noBackupFilesDir.canonicalFile.resolve("tool-environment")
    private val backup = storage.resolve("previous")
    private val root = storage.resolve("root").also {
        if (!it.exists() && backup.isDirectory) backup.renameTo(it)
    }
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val mutable = MutableStateFlow(snapshot(when {
        manifest == null -> Status.Unavailable
        root.isDirectory -> Status.Ready
        else -> Status.Missing
    }))
    override val state = mutable.asStateFlow()
    private var operation: Job? = null

    private fun snapshot(
        status: Status,
        progress: Long = 0,
        versions: Map<String, String> = emptyMap(),
        error: String? = null,
    ) = ToolEnvironmentState(status, manifest?.bytes ?: 0, progress, root.isDirectory, versions, error)

    @Synchronized
    private fun start(status: Status, block: suspend () -> Unit) {
        if (manifest == null || operation?.isCompleted == false) return
        mutable.value = snapshot(status)
        operation = scope.launch(start = CoroutineStart.LAZY) {
            try {
                block()
            } catch (error: CancellationException) {
                mutable.value = snapshot(if (root.isDirectory) Status.Ready else Status.Missing)
                throw error
            } catch (error: Exception) {
                mutable.value = snapshot(Status.Failed, error = when (error) {
                    is IOException -> error.message?.take(180) ?: "请检查网络和可用存储后重试。"
                    else -> "环境准备失败，请重试。"
                })
            }
        }.also { it.start() }
    }

    override fun install() = start(Status.Downloading) {
        val info = checkNotNull(manifest)
        storage.mkdirs()
        val staging = storage.resolve("installing")
        val archive = storage.resolve("runtime.download")
        try {
            if (!root.exists() && backup.isDirectory && !backup.renameTo(root)) {
                throw IOException("无法恢复已有环境。")
            }
            deleteTree(staging)
            deleteTree(backup)
            download(info, archive)
            mutable.value = snapshot(Status.Installing)
            staging.mkdirs()
            extract(archive, staging)
            // Verify the actual tools, not just the presence of files in the archive.
            val versions = probe(staging)
            currentCoroutineContext().ensureActive()
            if (root.exists() && !root.renameTo(backup)) throw IOException("无法替换已有环境。")
            if (!staging.renameTo(root)) {
                backup.renameTo(root)
                throw IOException("无法保存工具环境。")
            }
            deleteTree(backup)
            mutable.value = snapshot(Status.Ready, versions = versions)
        } finally {
            archive.delete()
            deleteTree(staging)
        }
    }

    override fun check() = start(Status.Checking) {
        mutable.value = snapshot(Status.Ready, versions = probe(root))
    }

    @Synchronized
    override fun cancel() {
        if (mutable.value.status in listOf(Status.Downloading, Status.Installing)) operation?.cancel()
    }

    override fun remove() = start(Status.Removing) {
        deleteTree(storage)
        if (storage.exists()) throw IOException("无法完整移除工具环境。")
        mutable.value = snapshot(Status.Missing)
    }

    private suspend fun download(info: Manifest, archive: File) {
        val connection = URL(info.url).openConnection() as HttpsURLConnection
        try {
            connection.connectTimeout = 15_000
            connection.readTimeout = 20_000
            connection.setRequestProperty("Accept-Encoding", "identity")
            val response = connection.responseCode
            if (response != 200) throw IOException("工具包下载失败（HTTP $response），请重试。")
            val digest = MessageDigest.getInstance("SHA-256")
            var total = 0L
            var shown = 0L
            connection.inputStream.buffered().use { input ->
                archive.outputStream().buffered().use { output ->
                    val buffer = ByteArray(64 * 1024)
                    while (true) {
                        currentCoroutineContext().ensureActive()
                        val count = input.read(buffer)
                        if (count < 0) break
                        total += count
                        if (total > info.bytes) throw IOException("工具包大小与发布记录不符。")
                        digest.update(buffer, 0, count)
                        output.write(buffer, 0, count)
                        if (total - shown >= 256 * 1024 || total == info.bytes) {
                            mutable.value = snapshot(Status.Downloading, progress = total)
                            shown = total
                        }
                    }
                }
            }
            val hash = digest.digest().joinToString("") { "%02x".format(it.toInt() and 255) }
            if (total != info.bytes || hash != info.sha256) throw IOException("工具包校验失败，请重新下载。")
        } finally {
            connection.disconnect()
        }
    }

    private data class Link(val path: String, val target: String, val symbolic: Boolean)

    private suspend fun extract(archive: File, target: File) {
        val links = mutableListOf<Link>()
        val paths = HashSet<String>()
        var expanded = 0L
        TarArchiveInputStream(XZInputStream(archive.inputStream().buffered(), 65_536)).use { input ->
            val buffer = ByteArray(64 * 1024)
            while (true) {
                currentCoroutineContext().ensureActive()
                val entry = input.nextEntry ?: break
                require(!entry.name.startsWith('/'))
                require(!entry.isSparse)
                val path = relativePath(entry.name, emptyList())
                if (path.isEmpty()) continue
                require(paths.add(path) && paths.size <= 70_000)
                val file = target.resolve(path)
                when {
                    entry.isDirectory -> require(file.isDirectory || file.mkdirs())
                    entry.isSymbolicLink || entry.isLink -> {
                        val base = if (entry.isSymbolicLink) path.split('/').dropLast(1) else emptyList()
                        links += Link(path, relativePath(entry.linkName, base), entry.isSymbolicLink)
                    }
                    entry.isBlockDevice || entry.isCharacterDevice || entry.isFIFO -> Unit
                    entry.isFile -> {
                        expanded += entry.size
                        require(entry.size >= 0 && expanded <= 1_200_000_000L)
                        file.parentFile?.mkdirs()
                        file.outputStream().buffered().use { output ->
                            while (true) {
                                currentCoroutineContext().ensureActive()
                                val count = input.read(buffer)
                                if (count < 0) break
                                output.write(buffer, 0, count)
                            }
                        }
                        Os.chmod(file.path, entry.mode and 0b111111111)
                    }
                }
            }
        }
        // Write links last so no archive entry can write through a symbolic link.
        for (link in links.sortedBy { it.symbolic }) {
            currentCoroutineContext().ensureActive()
            val file = target.resolve(link.path)
            val parent = checkNotNull(file.parentFile)
            parent.mkdirs()
            require(parent.canonicalFile == parent.absoluteFile)
            if (link.symbolic) {
                val prefix = "../".repeat(link.path.count { it == '/' })
                Os.symlink(prefix + link.target.ifEmpty { "." }, file.path)
            } else {
                val source = target.resolve(link.target)
                require(source.isFile && source.canonicalFile == source.absoluteFile)
                Os.link(source.path, file.path)
            }
        }
    }

    private fun relativePath(value: String, base: List<String>): String {
        val parts = if (value.startsWith('/')) mutableListOf() else base.toMutableList()
        for (part in value.split('/')) {
            when (part) {
                "", "." -> Unit
                ".." -> {
                    require(parts.isNotEmpty())
                    parts.removeAt(parts.lastIndex)
                }
                else -> parts.add(part)
            }
        }
        return parts.joinToString("/")
    }

    private suspend fun probe(directory: File): Map<String, String> {
        if (!directory.isDirectory) throw IOException("本机工具环境尚未安装。")
        val log = File.createTempFile("tool-check-", ".log", storage)
        try {
            val command = listOf(
                nativeDir.resolve("libproot.so").path, "--kill-on-exit", "--link2symlink", "-L", "-0",
                "--rootfs=${directory.path}", "--cwd=/tmp", "-b", "/dev", "-b", "/proc",
                "-b", "/system", "-b", "/apex", "/usr/bin/env", "-i", "HOME=/root", "TMPDIR=/tmp",
                "PATH=/opt/node/bin:/usr/sbin:/usr/bin:/sbin:/bin", "/bin/sh", "-ec",
                "node --version; npm --version; npx --version; python3 --version; python3 -m pip --version; git --version",
            )
            val process = ProcessBuilder(command).redirectErrorStream(true).redirectOutput(log).apply {
                environment()["LD_LIBRARY_PATH"] = nativeDir.path
                environment()["PROOT_LOADER"] = nativeDir.resolve("libproot-loader.so").path
                environment()["PROOT_TMP_DIR"] = storage.resolve("tmp").apply { mkdirs() }.path
            }.start()
            val code = try {
                withTimeoutOrNull(30_000) {
                    var exit = runCatching { process.exitValue() }.getOrNull()
                    while (exit == null) {
                        delay(100)
                        exit = runCatching { process.exitValue() }.getOrNull()
                    }
                    exit
                }
            } finally {
                if (Build.VERSION.SDK_INT >= 26) process.destroyForcibly() else process.destroy()
            }
            if (code != 0) throw IOException(
                if (code == null) "工具启动检测超时。" else "当前设备无法启动工具环境（退出码 $code）。",
            )
            val versions = log.useLines { lines -> lines.filterNot { it.startsWith("proot ") }.take(7).toList() }
            val names = listOf("Node.js", "npm", "npx", "Python", "pip", "Git")
            if (versions.size != names.size) throw IOException("工具版本检测结果不完整。")
            return names.zip(versions.map { it.take(100) }).toMap()
        } finally {
            log.delete()
        }
    }

    // File.deleteRecursively follows symlinked directories on Android.
    private fun deleteTree(file: File) {
        val mode = runCatching { Os.lstat(file.path).st_mode }.getOrNull() ?: return
        if (OsConstants.S_ISDIR(mode)) file.listFiles()?.forEach(::deleteTree)
        file.delete()
    }
}
