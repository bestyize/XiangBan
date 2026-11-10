package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
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
    fun agentSidecars(id: String): List<String> = listOfNotNull(agentCopy(id), agentProgressCopy(id))
    fun deletionMarker(folder: String, id: String): String = "$folder/deleted/$id"
    fun deletedIds(folder: String): Set<String> = emptySet()
}

/** 不提供存放处时用它：对话只在这次打开应用期间记着。 */
class InMemoryChatHistoryFiles : ChatHistoryFiles {
    private val texts = mutableMapOf<String, String>()
    override fun read(name: String): String? = texts[name]
    override fun write(name: String, text: String) { texts[name] = text }
    override fun delete(name: String) { texts.remove(name) }
    override fun deletedIds(folder: String): Set<String> =
        texts.keys.filter { it.startsWith("$folder/deleted/") }.map { it.substringAfterLast('/') }.toSet()
}

/** 对话列表里的一条。 */
@Serializable
data class ChatSummary(val id: String, val title: String, val updatedAtMs: Long, val workspace: String? = null)

@Serializable
data class StoredChat(val id: String, val messages: List<ChatTurn>, val options: ChatOptions = ChatOptions())

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
class ChatHistory(
    private val files: ChatHistoryFiles,
    private val scope: CoroutineScope,
    private val awaitAgentIdle: suspend (String) -> Unit = {},
) {
    private var folder = ""
    private var index: List<ChatSummary> = emptyList()
    private var groups: List<String> = emptyList()
    private var deleted = emptySet<String>()
    private val cleaning = mutableSetOf<String>()
    private data class Write(val revision: Long, val name: String, val text: String?)
    private val pending = MutableStateFlow<Map<String, Write>>(emptyMap())
    private val writes = Channel<Write>(Channel.UNLIMITED)
    private var revision = 0L
    private val options = mutableMapOf<String, ChatOptions>()
    private val _saveError = MutableStateFlow<String?>(null)
    val saveError = _saveError.asStateFlow()
    private val failed = MutableStateFlow<Set<String>>(emptySet())

    init {
        scope.launch {
            for (write in writes) {
                if (pending.value[write.name] != write) continue
                val result = try {
                    withContext(Dispatchers.Default) {
                        if (write.text == null) files.delete(write.name) else files.write(write.name, write.text)
                    }
                    Result.success(Unit)
                } catch (e: CancellationException) { throw e }
                catch (e: Exception) { Result.failure(e) }
                if (result.isSuccess) {
                    pending.update { if (it[write.name] == write) it - write.name else it }
                    failed.update { it - write.name }
                    if (failed.value.isEmpty()) _saveError.value = null
                } else {
                    failed.update { it + write.name }
                    _saveError.value = "对话尚未保存，请重试保存；退出应用可能丢失未保存内容。"
                }
            }
        }
    }

    fun retryPending() {
        pending.value.values.sortedBy { it.revision }.forEach { writes.trySend(it) }
        deleted.forEach { cleanDeleted(folder, it) }
    }

    /** 换到某个账号的对话；返回它的对话列表（最近的在前）。 */
    fun useAccount(account: String?): List<ChatSummary> {
        options.clear()
        folder = "chats/" + (account ?: "default").map { if (it.isLetterOrDigit() || it == '-' || it == '_') it else '_' }.joinToString("")
        deleted = runCatching { files.deletedIds(folder) }.getOrElse {
            _saveError.value = "无法读取待删除的对话，请重试。"
            emptySet()
        }.filter(::validChatId).toSet()
        index = read(indexName())?.let { runCatching { CommunityJson.decodeFromString<List<ChatSummary>>(it) }.getOrNull() }.orEmpty()
            .filterNot { it.id in deleted }.sortedByDescending { it.updatedAtMs }
        deleted.forEach { cleanDeleted(folder, it) }
        groups = read("$folder/workspaces.json")?.let {
            runCatching { CommunityJson.decodeFromString<List<String>>(it) }.getOrNull()
        }.orEmpty()
        return index
    }

    fun workspaces(): List<String> = groups

    fun addWorkspace(name: String) {
        val clean = name.trim().take(40)
        if (clean.isEmpty() || clean in groups) return
        groups = groups + clean
        persistGroups()
    }

    fun renameWorkspace(old: String, name: String) {
        val clean = name.trim().take(40)
        if (old !in groups || clean.isEmpty() || clean in groups) return
        groups = groups.map { if (it == old) clean else it }
        index = index.map { if (it.workspace == old) it.copy(workspace = clean) else it }
        persistGroups()
    }

    fun deleteWorkspace(name: String) {
        groups = groups - name
        index = index.map { if (it.workspace == name) it.copy(workspace = null) else it }
        persistGroups()
    }

    fun move(id: String, workspace: String?) {
        if (workspace != null && workspace !in groups) return
        index = index.map { if (it.id == id) it.copy(workspace = workspace) else it }
        enqueue(indexName(), CommunityJson.encodeToString(index))
    }

    private fun persistGroups() {
        enqueue("$folder/workspaces.json", CommunityJson.encodeToString(groups))
        enqueue(indexName(), CommunityJson.encodeToString(index))
    }

    fun list(): List<ChatSummary> = index

    /** Missing/corrupt/unreadable records must not silently remove the conversation index. */
    fun loadSnapshot(id: String): StoredChat? = try {
        if (id in deleted) null else read(chatName(id))?.let { CommunityJson.decodeFromString<StoredChat>(it) }?.also { options[id] = it.options }
    } catch (e: Exception) {
        _saveError.value = "无法读取这段对话，原记录已保留，请稍后重试。"
        null
    }

    fun load(id: String): List<ChatTurn>? = loadSnapshot(id)?.messages

    /** 存下一个对话（没有内容就不存），把它挪到列表最前面；返回新的列表。 */
    fun save(id: String, messages: List<ChatTurn>, nowMs: Long, settings: ChatOptions? = null): List<ChatSummary> {
        if (id in deleted) return index
        val kept = messages.filter { it.text.isNotBlank() || it.reasoning.isNotBlank() || it.cancelled || it.error != null || it.tools.isNotEmpty() || it.imageGeneration || it.role == "user" }
            .map {
                when {
                    it.streaming && it.imageGeneration -> it.copy(streaming = false, error = UNKNOWN_IMAGE_RESULT)
                    it.streaming -> it.copy(streaming = false)
                    else -> it
                }
            }
        val selected = settings ?: options[id] ?: ChatOptions()
        selected.validate()
        options[id] = selected
        enqueue(chatName(id), CommunityJson.encodeToString(StoredChat(id, kept, selected)))
        if (kept.none { it.role == "user" }) return index
        index = listOf(ChatSummary(id, chatTitle(kept), nowMs, index.firstOrNull { it.id == id }?.workspace)) + index.filterNot { it.id == id }
        enqueue(indexName(), CommunityJson.encodeToString(index))
        return index
    }

    fun isDeleted(id: String): Boolean = id in deleted

    fun delete(id: String): List<ChatSummary> {
        require(validChatId(id))
        if (id !in deleted) {
            try {
                // Record intent before asking the native run to stop. On restart this also hides an old index.
                files.write(files.deletionMarker(folder, id), folder)
            } catch (e: Exception) {
                _saveError.value = "无法记录删除操作，请重试。"
                return index
            }
            deleted = deleted + id
        }
        options.remove(id)
        enqueue(chatName(id), null)
        index = index.filterNot { it.id == id }
        enqueue(indexName(), CommunityJson.encodeToString(index))
        cleanDeleted(folder, id)
        return index
    }

    private fun cleanDeleted(owner: String, id: String) {
        val key = "$owner/$id"
        if (!cleaning.add(key)) return
        scope.launch {
            try {
                withTimeout(10_000) { awaitAgentIdle(id) }
                withContext(Dispatchers.Default) {
                    files.delete("$owner/$id.json")
                    files.agentSidecars(id).forEach(files::delete)
                }
            } catch (e: kotlinx.coroutines.TimeoutCancellationException) {
                _saveError.value = "对话正在停止，稍后可重试清理。"
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                _saveError.value = "对话文件尚未清理，请重试。"
            } finally {
                cleaning.remove(key)
            }
        }
    }

    private fun validChatId(id: String): Boolean =
        id.length in 1..128 && id.all { it in 'a'..'z' || it in 'A'..'Z' || it in '0'..'9' || it == '-' || it == '_' }

    private fun indexName() = "$folder/index.json"
    private fun chatName(id: String) = "$folder/$id.json"

    private fun read(name: String): String? {
        pending.value[name]?.let { return it.text }
        return try { files.read(name) } catch (e: CancellationException) { throw e }
        catch (e: Exception) { _saveError.value = "无法读取对话存储，原记录已保留，请稍后重试。"; null }
    }

    private fun enqueue(name: String, text: String?) {
        val write = Write(++revision, name, text)
        pending.update { it + (name to write) }
        writes.trySend(write)
    }
}
