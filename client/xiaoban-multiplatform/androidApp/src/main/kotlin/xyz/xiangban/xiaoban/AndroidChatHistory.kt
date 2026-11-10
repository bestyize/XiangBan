package xyz.xiangban.xiaoban

import android.content.Context
import java.io.File
import xyz.xiangban.xiaoban.common.community.ChatHistoryFiles

/** 小伴AI 的对话历史存在应用私有目录里（`files/chats/…`）；先写临时文件再换名，写一半断电也不会坏掉原来的。 */
class AndroidChatHistoryFiles(context: Context) : ChatHistoryFiles {
    private val root = context.applicationContext.filesDir

    override fun read(name: String): String? = File(root, name).takeIf { it.isFile }?.readText()

    override fun write(name: String, text: String) {
        val file = File(root, name)
        val parent = requireNotNull(file.parentFile)
        check(parent.isDirectory || parent.mkdirs()) { "无法创建对话目录" }
        val temp = File(file.parentFile, file.name + ".tmp")
        try {
            temp.outputStream().use { stream -> stream.write(text.toByteArray(Charsets.UTF_8)); stream.fd.sync() }
            // rename is atomic on Android; never delete the only good copy on failure.
            check(temp.renameTo(file)) { "无法保存对话，原记录已保留" }
        } finally { temp.delete() }
    }

    override fun delete(name: String) {
        val file = File(root, name)
        check(!file.exists() || if (name.startsWith("agent/tool-output/")) file.deleteRecursively() else file.delete()) { "无法删除对话记录" }
    }

    /** 智能体内核把对话存在 `files/agent/s-<编号>.jsonl`（见 MainActivity 里的 agentDir）。 */
    override fun agentCopy(id: String): String = "agent/s-${escapedId(id)}.jsonl"
    override fun agentProgressCopy(id: String): String = "agent/progress/s-${escapedId(id)}.jsonl"
    override fun agentSidecars(id: String): List<String> =
        listOf(agentCopy(id), agentProgressCopy(id), "agent/tool-output/${escapedId(id)}", "agent/trust/$id")

    override fun deletionMarker(folder: String, id: String): String = "agent/deleted/$id"

    override fun deletedIds(folder: String): Set<String> =
        File(root, "agent/deleted").listFiles().orEmpty()
            .filter { it.isFile && it.length() <= 4096 && it.readText() == folder }
            .map { it.name }.toSet()

    private fun escapedId(id: String): String =
        id.map { if (it in 'A'..'Z') "%${it.code.toString(16).uppercase()}" else it }.joinToString("")
}
