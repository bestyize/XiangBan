package xyz.xiangban.xiaoban.common.community

import java.io.ByteArrayOutputStream
import java.io.File
import java.io.RandomAccessFile
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import android.system.ErrnoException
import android.system.Os
import android.system.OsConstants

/** Persistent workspace confined to an app-private directory. Symlinks are never followed. */
class AndroidMobileWorkspace(root: File) : ValidatedMobileWorkspace() {
    private val root = root.canonicalFile.also {
        require(!isSymbolic(root.absoluteFile)) { "工作区根目录不能是符号链接" }
        require(it.isDirectory || it.mkdirs()) { "无法创建工作区" }
    }

    private fun isSymbolic(file: File): Boolean = try { OsConstants.S_ISLNK(Os.lstat(file.path).st_mode) } catch (e: ErrnoException) { if (e.errno == OsConstants.ENOENT) false else throw e }

    private fun checked(path: String): File {
        require(root.canonicalFile == root && root.isDirectory && !isSymbolic(root)) { "工作区根目录已变化" }
        var file = root
        for (segment in path.split('/').filter { it.isNotEmpty() }) {
            file = File(file, segment)
            require(!isSymbolic(file) && file.canonicalFile == file.absoluteFile) { "不允许符号链接或工作区外的路径" }
        }
        return file
    }

    override suspend fun <T> gameTransaction(action: suspend () -> T): T = gameGate.withLock {
        withContext(Dispatchers.IO) {
            // The runtime activity can share the same work from a separate Android process.
            val workspace = checked("")
            val lockFile = workspace.resolveSibling(".${workspace.name}-games.lock")
            RandomAccessFile(lockFile, "rw").use { file ->
                file.channel.lock().use {
                    currentCoroutineContext().ensureActive()
                    action()
                }
            }
        }
    }

    override suspend fun entries(path: String, recursive: Boolean): List<WorkspaceEntry> = withContext(Dispatchers.IO) {
        val directory = checked(path)
        require(directory.isDirectory) { "目录不存在：$path" }
        val result = mutableListOf<WorkspaceEntry>()
        fun visit(folder: File, prefix: String, depth: Int) {
            require(depth <= 8) { "工作区目录超过 8 层" }
            val children = folder.listFiles() ?: error("无法读取工作区目录")
            for (child in children) {
                val relative = if (prefix.isEmpty()) child.name else "$prefix/${child.name}"
                MobileWorkspaceLimits.path(relative)
                val safe = checked(relative)
                require(safe.isFile || safe.isDirectory) { "工作区只允许普通文件和目录" }
                require(result.size < MobileWorkspaceLimits.ENTRIES) { "工作区条目数量超过上限" }
                result += WorkspaceEntry(relative, safe.isDirectory, if (safe.isFile) safe.length() else 0)
                if (recursive && safe.isDirectory) visit(safe, relative, depth + 1)
            }
        }
        visit(directory, path, path.count { it == '/' } + if (path.isEmpty()) 0 else 1)
        result
    }

    override suspend fun readData(path: String, limit: Int): ByteArray = withContext(Dispatchers.IO) {
        val file = checked(path)
        require(file.isFile) { "文件不存在：$path" }
        require(file.length() <= limit) { "文件超过大小上限" }
        val bytes = ByteArrayOutputStream()
        file.inputStream().use { input ->
            val buffer = ByteArray(8192)
            while (true) {
                val count = input.read(buffer)
                if (count < 0) break
                require(bytes.size() + count <= limit) { "文件超过大小上限" }
                bytes.write(buffer, 0, count)
            }
        }
        bytes.toByteArray()
    }

    override suspend fun writeData(path: String, content: ByteArray) = withContext(Dispatchers.IO) {
        var file = checked(path)
        require(!file.isDirectory) { "目标是文件夹" }
        val parent = file.parentFile!!
        require(parent.isDirectory || parent.mkdirs()) { "无法创建文件夹" }
        file = checked(path)
        val temporary = File.createTempFile("xb-write-", ".tmp", parent)
        try {
            temporary.writeBytes(content)
            checked(path)
            currentCoroutineContext().ensureActive()
            require(temporary.renameTo(file)) { "无法保存文件" }
        } finally { temporary.delete() }
        Unit
    }

    override suspend fun removeEntry(path: String, recursive: Boolean) = withContext(Dispatchers.IO) {
        val file = checked(path)
        require(file.exists()) { "文件不存在：$path" }
        if (file.isDirectory && recursive) {
            // Inspect the whole tree before deleting anything; never delete through a symlink.
            entries(path, true)
            suspend fun removeTree(target: File) {
                currentCoroutineContext().ensureActive()
                val relative = target.relativeTo(root).invariantSeparatorsPath
                val safe = checked(relative)
                if (safe.isDirectory) (safe.listFiles() ?: error("无法读取目录")).forEach { removeTree(it) }
                require(safe.delete()) { "无法删除：$relative" }
            }
            removeTree(file)
        } else { currentCoroutineContext().ensureActive(); require(file.delete()) { "目录非空；删除整个目录需要 recursive=true" } }
        Unit
    }

    private companion object {
        val gameGate = Mutex()
    }
}

