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
        file.parentFile?.mkdirs()
        val temp = File(file.parentFile, file.name + ".tmp")
        temp.writeText(text)
        if (!temp.renameTo(file)) { file.delete(); temp.renameTo(file) }
    }

    override fun delete(name: String) { File(root, name).delete() }

    /** 智能体内核把对话存在 `files/agent/s-<编号>.jsonl`（见 MainActivity 里的 agentDir）。 */
    override fun agentCopy(id: String): String = "agent/s-$id.jsonl"
    override fun agentProgressCopy(id: String): String = "agent/progress/s-$id.jsonl"
}
