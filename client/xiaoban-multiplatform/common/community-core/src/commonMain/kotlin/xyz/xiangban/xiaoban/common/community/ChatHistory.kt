package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import kotlin.random.Random

/** 对话历史放在手机的什么地方（各平台自己实现）：按名字存取一段文字，名字里的 `/` 是子文件夹。 */
interface ChatHistoryFiles {
    fun read(name: String): String?
    fun write(name: String, text: String)
    fun delete(name: String)
    /** 智能体内核自己也存了一份对话，这里给出它的名字；删对话时一起删。没有就是 null。 */
    fun agentCopy(id: String): String? = null
    /** 内核持久任务清单，随会话一起删除。 */
    fun agentProgressCopy(id: String): String? = null
}

/** 不提供存放处时用它：对话只在这次打开应用期间记着。 */
class InMemoryChatHistoryFiles : ChatHistoryFiles {
    private val texts = mutableMapOf<String, String>()
    override fun read(name: String): String? = texts[name]
    override fun write(name: String, text: String) { texts[name] = text }
    override fun delete(name: String) { texts.remove(name) }
}

/** 对话列表里的一条。 */
@Serializable
data class ChatSummary(val id: String, val title: String, val updatedAtMs: Long)

@Serializable
private data class StoredChat(val id: String, val messages: List<ChatTurn>)

/** 新对话的编号：时间加随机数，重开应用也不会和以前的对话撞上（智能体内核按这个编号存对话）。 */
fun newChatId(nowMs: Long, random: Random = Random.Default): String =
    "c" + nowMs.toString(36) + random.nextLong(0, 36L * 36 * 36 * 36).toString(36).padStart(4, '0')

/** 对话的标题：第一句提问，太长就截断。 */
fun chatTitle(messages: List<ChatTurn>): String {
    val first = messages.firstOrNull { it.role == "user" }?.text?.trim().orEmpty()
    return if (first.isEmpty()) "新对话" else if (first.length > 14) first.take(14) + "…" else first
}

/**
 * 小伴AI 的对话历史：每个账号一份对话列表，每个对话一个文件。列表在内存里，写文件放到后台按顺序做，不卡界面。
 * 换账号只是换一份列表，不删别的账号的对话。和 [CloudChatController] 一样只在 [scope] 的线程上调用。
 */
class ChatHistory(private val files: ChatHistoryFiles, scope: CoroutineScope) {
    private var folder = ""
    private var index: List<ChatSummary> = emptyList()
    /** 已经交给后台、还没写完的内容；这时读要以它为准。 */
    private val pending = mutableMapOf<String, String?>()
    private val writes = Channel<Pair<String, String?>>(Channel.UNLIMITED)

    init {
        // 只有读写文件在后台线程；[pending] 只在 [scope] 的线程上碰。
        scope.launch {
            for ((name, text) in writes) {
                withContext(Dispatchers.Default) { runCatching { if (text == null) files.delete(name) else files.write(name, text) } }
                if (pending[name] == text) pending.remove(name)
            }
        }
    }

    /** 换到某个账号的对话；返回它的对话列表（最近的在前）。 */
    fun useAccount(account: String?): List<ChatSummary> {
        folder = "chats/" + (account ?: "default").map { if (it.isLetterOrDigit() || it == '-' || it == '_') it else '_' }.joinToString("")
        index = read(indexName())?.let { runCatching { CommunityJson.decodeFromString<List<ChatSummary>>(it) }.getOrNull() }.orEmpty()
            .sortedByDescending { it.updatedAtMs }
        return index
    }

    fun list(): List<ChatSummary> = index

    /** 读出一个对话；文件没了或坏了就从列表里去掉，返回 null。 */
    fun load(id: String): List<ChatTurn>? {
        val chat = read(chatName(id))?.let { runCatching { CommunityJson.decodeFromString<StoredChat>(it) }.getOrNull() }
        if (chat == null && index.any { it.id == id }) {
            index = index.filterNot { it.id == id }
            enqueue(indexName(), CommunityJson.encodeToString(index))
        }
        return chat?.messages
    }

    /** 存下一个对话（没有内容就不存），把它挪到列表最前面；返回新的列表。 */
    fun save(id: String, messages: List<ChatTurn>, nowMs: Long): List<ChatSummary> {
        val kept = messages.filter { it.text.isNotBlank() || it.error != null || it.tools.isNotEmpty() || it.role == "user" }
            .map { if (it.streaming) it.copy(streaming = false) else it }
        if (kept.none { it.role == "user" }) return index
        enqueue(chatName(id), CommunityJson.encodeToString(StoredChat(id, kept)))
        index = listOf(ChatSummary(id, chatTitle(kept), nowMs)) + index.filterNot { it.id == id }
        enqueue(indexName(), CommunityJson.encodeToString(index))
        return index
    }

    fun delete(id: String): List<ChatSummary> {
        enqueue(chatName(id), null)
        files.agentCopy(id)?.let { enqueue(it, null) }
        files.agentProgressCopy(id)?.let { enqueue(it, null) }
        index = index.filterNot { it.id == id }
        enqueue(indexName(), CommunityJson.encodeToString(index))
        return index
    }

    private fun indexName() = "$folder/index.json"
    private fun chatName(id: String) = "$folder/$id.json"

    private fun read(name: String): String? {
        if (name in pending) return pending[name]
        return runCatching { files.read(name) }.getOrNull()
    }

    private fun enqueue(name: String, text: String?) {
        pending[name] = text
        writes.trySend(name to text)
    }
}
