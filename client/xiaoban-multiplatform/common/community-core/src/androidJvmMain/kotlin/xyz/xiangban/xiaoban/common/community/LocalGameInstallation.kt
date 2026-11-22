package xyz.xiangban.xiaoban.common.community

import java.io.File

/** Called under the workspace game transaction. Staging is outside the user-visible workspace. */
internal class LocalGameInstallation(
    workspace: File,
    private val workspacePath: (String) -> File,
    private val rename: (File, File) -> Boolean = File::renameTo,
) {
    private val pending = workspace.resolveSibling(".${workspace.name}-game-install")

    fun recover() {
        if (!pending.exists()) return
        check(pending.canonicalFile == pending && pending.isDirectory) { "无法读取作品恢复目录" }
        val transactions = pending.listFiles() ?: error("无法读取作品恢复目录")
        for (transaction in transactions) {
            MobileGameFormat.id(transaction.name)
            recover(transaction, workspacePath("games/${transaction.name}"))
        }
    }

    private fun recover(transaction: File, target: File) {
        check(transaction.canonicalFile == transaction && transaction.isDirectory) { "无法读取作品恢复目录" }
        val previous = transaction.resolve("previous")
        if (!target.exists() && previous.isDirectory) {
            check(previous.canonicalFile == previous && rename(previous, target)) { "无法恢复旧版作品" }
        }
        check(transaction.deleteRecursively()) { "无法清理作品临时目录" }
    }

    fun install(id: String, files: List<MobileGameFile>, entries: List<WorkspaceEntry>) {
        MobileGameFormat.id(id)
        val directory = "games/$id"
        val target = workspacePath(directory)
        val retained = entries.filterNot { it.path == directory || it.path.startsWith("$directory/") }
        val paths = retained.mapTo(mutableSetOf()) { it.path }
        for (file in files) {
            val path = MobileWorkspaceLimits.path("$directory/${file.path}")
            paths.addAll(path.split('/').runningReduce { prefix, part -> "$prefix/$part" })
        }
        require(retained.sumOf { it.size } + files.sumOf { it.bytes.size.toLong() } <= MobileWorkspaceLimits.TOTAL_BYTES) { "工作区总量不能超过 64 MB" }
        require(paths.size <= MobileWorkspaceLimits.ENTRIES) { "工作区文件和目录数量已达上限" }

        val transaction = pending.resolve(id)
        val next = transaction.resolve("next")
        val previous = transaction.resolve("previous")
        check(pending.isDirectory || pending.mkdir()) { "无法创建作品临时目录" }
        check(pending.canonicalFile == pending && transaction.mkdir()) { "无法创建作品临时目录" }
        try {
            check(next.mkdir()) { "无法创建作品临时目录" }
            for (file in files) {
                val destination = next.resolve(file.path)
                val parent = destination.parentFile!!
                check(parent.isDirectory || parent.mkdirs()) { "无法创建作品文件夹" }
                destination.writeBytes(file.bytes)
            }
            val parent = target.parentFile!!
            check(parent.isDirectory || parent.mkdirs()) { "无法创建作品文件夹" }
            if (target.exists()) check(rename(target, previous)) { "无法保留旧版作品" }
            check(rename(next, target)) { "无法替换作品" }
        } catch (failure: Exception) {
            try { recover(transaction, target) } catch (rollback: Exception) { failure.addSuppressed(rollback) }
            throw failure
        }
        // Once the new directory is visible it is already complete. Recovery can retry this cleanup.
        transaction.deleteRecursively()
    }
}
