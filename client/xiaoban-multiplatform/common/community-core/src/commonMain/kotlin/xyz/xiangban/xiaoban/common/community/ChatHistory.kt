package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.Serializable
import kotlin.io.encoding.Base64
import kotlin.random.Random

/** 对话历史放在手机的什么地方（各平台自己实现）：按名字存取一段文字，名字里的 `/` 是子文件夹。 */
interface ChatHistoryFiles {
    fun read(name: String): String?
    fun write(name: String, text: String)
    fun delete(name: String)
    val backgroundReads: Boolean get() = false
    fun readMedia(name: String): ByteArray? = read(name)?.let(Base64::decode)
    fun writeMedia(name: String, bytes: ByteArray) = write(name, Base64.encode(bytes))
    fun hasMedia(name: String): Boolean = read(name) != null
    fun deleteTree(name: String) = delete(name)
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
    override fun deleteTree(name: String) { texts.keys.removeAll { it.startsWith("$name/") } }
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
    private data class Write(
        val revision: Long,
        val name: String,
        val text: String? = null,
        val snapshot: StoredChat? = null,
        val checkpoint: ChatTurn? = null,
        val media: ChatMedia? = null,
        val deleteTree: String? = null,
        val result: CompletableDeferred<Boolean> = CompletableDeferred(),
    )
    private val pending = MutableStateFlow<Map<String, Write>>(emptyMap())
    private val writes = Channel<Unit>(Channel.CONFLATED)
    private val io = Mutex()
    internal val backgroundReads get() = files.backgroundReads
    private var revision = 0L
    private val options = mutableMapOf<String, ChatOptions>()
    private val _saveError = MutableStateFlow<String?>(null)
    val saveError = _saveError.asStateFlow()
    private val failed = MutableStateFlow<Set<String>>(emptySet())

    init {
        scope.launch {
            for (signal in writes) {
                for (write in pending.value.values.sortedBy { it.revision }) {
                    if (pending.value[write.name] != write) continue
                    val saved = try {
                        withContext(Dispatchers.Default) {
                            io.withLock {
                                val text = when {
                                    write.snapshot != null -> CommunityJson.encodeToString(
                                        write.snapshot.copy(messages = write.snapshot.messages.map(requireNotNull(write.media)::compact)),
                                    )
                                    write.checkpoint != null -> CommunityJson.encodeToString(write.checkpoint)
                                    else -> write.text
                                }
                                if (text == null) files.delete(write.name) else files.write(write.name, text)
                                write.deleteTree?.let(files::deleteTree)
                            }
                        }
                        true
                    } catch (e: CancellationException) { throw e }
                    catch (_: Exception) { false }
                    if (saved) {
                        pending.update { if (it[write.name] == write) it - write.name else it }
                        failed.update { it - write.name }
                        if (failed.value.isEmpty()) _saveError.value = null
                    } else {
                        failed.update { it + write.name }
                        _saveError.value = "对话尚未保存，请重试保存；退出应用可能丢失未保存内容。"
                    }
                    write.result.complete(saved)
                }
            }
        }
    }

    fun retryPending() {
        writes.trySend(Unit)
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
    private fun readSnapshot(owner: String, id: String): StoredChat? = try {
        val name = "$owner/$id.json"
        val queuedSnapshot = pending.value[name]
        val saved = if (queuedSnapshot != null) queuedSnapshot.snapshot
            else read(name)?.let { CommunityJson.decodeFromString<StoredChat>(it) }
        val last = saved?.messages?.lastOrNull()
        if (saved != null && last?.error == INTERRUPTED_REPLY) {
            val checkpointName = "$owner/$id.stream.json"
            val queued = pending.value[checkpointName]
            val checkpoint = if (queued != null) queued.checkpoint
                else read(checkpointName)?.let { runCatching { CommunityJson.decodeFromString<ChatTurn>(it) }.getOrNull() }
            if (checkpoint?.id == last.id) saved.copy(messages = saved.messages.dropLast(1) + checkpoint) else saved
        } else saved
    } catch (e: CancellationException) { throw e }
    catch (_: Exception) {
        _saveError.value = "无法读取这段对话，原记录已保留，请稍后重试。"
        null
    }

    fun loadSnapshot(id: String): StoredChat? =
        if (!validChatId(id) || id in deleted) null else readSnapshot(folder, id)?.also { options[id] = it.options }

    internal suspend fun loadInBackground(id: String): StoredChat? {
        if (!validChatId(id) || id in deleted) return null
        val owner = folder
        val media = mediaFor(id)
        val saved = withContext(Dispatchers.Default) {
            io.withLock {
                readSnapshot(owner, id)?.let { snapshot ->
                    try { snapshot.copy(messages = snapshot.messages.map(media::compact)) }
                    catch (_: Exception) { snapshot } // An old inline record remains readable if storage is full.
                }
            }
        }
        return saved?.takeIf { folder == owner && id !in deleted }?.also { options[id] = it.options }
    }

    fun load(id: String): List<ChatTurn>? = loadSnapshot(id)?.messages

    internal fun mediaFor(id: String): ChatMedia {
        require(validChatId(id))
        return ChatMedia(files, io, folder, id)
    }

    internal fun checkpoint(id: String, turn: ChatTurn) {
        if (id in deleted || !turn.streaming || turn.imageGeneration) return
        enqueue(Write(++revision, "$folder/$id.stream.json", checkpoint = interrupted(turn)))
    }

    internal suspend fun awaitSaved(id: String): Boolean {
        val writes = pending.value
        return writes[chatName(id)]?.result?.await() != false && writes[indexName()]?.result?.await() != false
    }

    private fun interrupted(turn: ChatTurn): ChatTurn = turn.copy(
        streaming = false,
        activity = "回复已中断",
        error = INTERRUPTED_REPLY,
        tools = turn.tools.map { if (it.status == ToolStatus.Running) it.copy(status = ToolStatus.Error) else it },
    )

    /** 存下一个对话（没有内容就不存），把它挪到列表最前面；返回新的列表。 */
    fun save(id: String, messages: List<ChatTurn>, nowMs: Long, settings: ChatOptions? = null): List<ChatSummary> {
        if (id in deleted) return index
        val kept = messages.filter { it.text.isNotBlank() || it.reasoning.isNotBlank() || it.cancelled || it.error != null || it.tools.isNotEmpty() || it.streaming || it.imageGeneration || it.role == "user" }
            .map {
                when {
                    it.streaming && it.imageGeneration -> it.copy(streaming = false,
                        error = if (it.imageJobRecoverable) RECOVER_IMAGE_RESULT else UNKNOWN_IMAGE_RESULT)
                    it.streaming -> interrupted(it)
                    else -> it
                }
            }
        val selected = settings ?: options[id] ?: ChatOptions()
        selected.validate()
        options[id] = selected
        enqueue(Write(++revision, chatName(id), snapshot = StoredChat(id, kept, selected), media = mediaFor(id)))
        if (messages.none { it.streaming && !it.imageGeneration }) enqueue("$folder/$id.stream.json", null)
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
        enqueue(Write(++revision, chatName(id), deleteTree = "$folder/media/$id"))
        enqueue("$folder/$id.stream.json", null)
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
                    io.withLock {
                        files.delete("$owner/$id.json")
                        files.delete("$owner/$id.stream.json")
                        files.deleteTree("$owner/media/$id")
                        files.agentSidecars(id).forEach(files::delete)
                    }
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
        enqueue(Write(++revision, name, text))
    }

    private fun enqueue(write: Write) {
        val previous = pending.value[write.name]
        pending.update { it + (write.name to write) }
        previous?.result?.complete(false)
        writes.trySend(Unit)
    }
}

internal const val INTERRUPTED_REPLY = "回复已中断，已保留收到的内容。"
