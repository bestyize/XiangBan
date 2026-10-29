package xyz.xiangban.xiaoban.common.link

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put

// ───────────── 互联应用协议数据模型 ─────────────

@Serializable
data class RemoteSessionSummary(
    val id: String,
    val title: String,
    @SerialName("updated_at") val updatedAt: Long = 0,
    val state: String = "idle",
    /** 这个对话在电脑上的工作文件夹，也就是它所属的项目；老对话可能为空。 */
    val workspace: String = "",
)

/** 电脑上最近用过、手机可以拿来当项目的文件夹。 */
data class RemoteProject(val path: String, val name: String = folderName(path))

/** 侧栏里的一个项目：文件夹加它下面的对话（新的在前）。 */
data class RemoteProjectGroup(
    val path: String,
    val name: String,
    val sessions: List<RemoteSessionSummary>,
) {
    val running: Boolean get() = sessions.any { it.state == "running" }
}

/** 侧栏要显示的内容：按项目分好的对话，加上没有项目的对话。 */
data class RemoteSidebar(
    val projects: List<RemoteProjectGroup>,
    val loose: List<RemoteSessionSummary>,
)

private fun trimSeparators(path: String): String = path.trim().trimEnd('/', '\\')

/** 文件夹路径的最后一段（项目名）；同时认 `/` 和 `\\`，因为路径来自 Windows 或 macOS 的电脑。 */
fun folderName(path: String): String {
    val trimmed = trimSeparators(path)
    val last = trimmed.substringAfterLast('/').substringAfterLast('\\')
    return last.ifEmpty { path }
}

/**
 * 把一台电脑的对话按项目（工作文件夹）分组。项目按最近一次对话的时间排，`added` 是手机上刚添加、还没有对话的项目，排在最后；
 * 没有文件夹的对话放进 `loose`。两边文件夹只差末尾分隔符时算同一个项目。
 */
fun buildSidebar(sessions: List<RemoteSessionSummary>, added: Collection<String> = emptyList()): RemoteSidebar {
    val byFolder = LinkedHashMap<String, MutableList<RemoteSessionSummary>>()
    val shown = HashMap<String, String>()
    val loose = mutableListOf<RemoteSessionSummary>()
    for (session in sessions.sortedByDescending { it.updatedAt }) {
        if (session.workspace.isBlank()) {
            loose += session
            continue
        }
        val key = trimSeparators(session.workspace)
        shown.getOrPut(key) { session.workspace }
        byFolder.getOrPut(key) { mutableListOf() } += session
    }
    for (path in added) {
        val key = trimSeparators(path)
        if (key.isNotEmpty() && key !in byFolder) {
            shown[key] = path
            byFolder[key] = mutableListOf()
        }
    }
    return RemoteSidebar(
        projects = byFolder.map { (key, items) -> RemoteProjectGroup(shown.getValue(key), folderName(shown.getValue(key)), items) },
        loose = loose,
    )
}

@Serializable
data class RemoteApprovalOption(
    val id: String,
    val label: String,
)

data class RemoteApprovalRequest(
    val approvalId: String,
    val session: String,
    val title: String,
    val detail: JsonObject = JsonObject(emptyMap()),
    val risk: String = "low",
    val options: List<RemoteApprovalOption> = emptyList(),
) {
    val isHighRisk: Boolean get() = risk.equals("high", ignoreCase = true)

    /** 给人看的操作内容：优先取命令、路径这类字段，没有就取整段参数。 */
    val summary: String get() = summarizeArgs(detail)
}

data class RemoteToolProgress(
    val id: String,
    val name: String,
    val running: Boolean = true,
    val isError: Boolean? = null,
    /** 这次工具调用的要点（命令、文件路径…），可能为空。 */
    val detail: String = "",
)

private val SummaryKeys = listOf("command", "cmd", "file_path", "path", "pattern", "query", "url", "text", "detail")

/** 把工具参数压成一行能看懂的文字：常见字段优先，只有一个字段就取它，否则退回整段 JSON（截断）。 */
internal fun summarizeArgs(value: JsonElement?): String {
    if (value == null || value is JsonNull) return ""
    if (value is JsonPrimitive) return value.contentOrNull.orEmpty().take(300)
    if (value is JsonObject) {
        for (key in SummaryKeys) {
            val hit = (value[key] as? JsonPrimitive)?.contentOrNull
            if (!hit.isNullOrBlank()) return hit.take(300)
        }
        if (value.size == 1) summarizeArgs(value.values.first()).takeIf { it.isNotBlank() }?.let { return it }
        if (value.isEmpty()) return ""
    }
    return value.toString().take(300)
}

@Serializable
data class RemoteTurn(
    val id: String,
    val role: String,
    val text: String,
    val streaming: Boolean = false,
    val error: String? = null,
)

data class RemoteChatState(
    val selectedPeerId: String? = null,
    val peers: List<PeerSummary> = emptyList(),
    val isPeerOnline: Boolean = false,
    val sessions: List<RemoteSessionSummary> = emptyList(),
    /** 电脑上最近用过的文件夹，“新项目”里选。 */
    val recentProjects: List<RemoteProject> = emptyList(),
    /** 在手机上添加、还没有对话的项目（只保存在本次打开期间；有了对话就由电脑记着）。 */
    val addedProjects: List<String> = emptyList(),
    /** 下一条新对话要在哪个项目文件夹里开始；空表示沿用电脑上最近用的。 */
    val newChatWorkspace: String? = null,
    val selectedSessionId: String? = null,
    val loadingSessions: Boolean = false,
    val messages: List<RemoteTurn> = emptyList(),
    val toolProgress: List<RemoteToolProgress> = emptyList(),
    val sending: Boolean = false,
    val pendingApproval: RemoteApprovalRequest? = null,
    /** 当前对话在电脑上的权限：`ask` 默认权限（执行前先问），`auto` 完全信任（高风险操作仍然每次都问）。 */
    val permissionMode: String = "ask",
    val error: String? = null,
) {
    val fullTrust: Boolean get() = permissionMode == "auto"

    val selectedPeer: PeerSummary? get() = peers.firstOrNull { it.id == selectedPeerId }
    val selectedSession: RemoteSessionSummary? get() = sessions.firstOrNull { it.id == selectedSessionId }

    /** 当前所处的项目文件夹：正在开始的新对话用选的那个，否则是当前对话所在的。 */
    val currentWorkspace: String
        get() = if (selectedSessionId == null) newChatWorkspace.orEmpty() else selectedSession?.workspace.orEmpty()

    val sidebar: RemoteSidebar get() = buildSidebar(sessions, addedProjects)
}

private val ProtocolJson = Json { ignoreUnknownKeys = true; explicitNulls = false }

private fun JsonElement?.asStr(): String? =
    this?.takeIf { it !is JsonNull }?.let { runCatching { it.jsonPrimitive.contentOrNull }.getOrNull() }

// ───────────── 手机端远程会话与审批控制器 ─────────────

/**
 * 手机端遥控桌面小伴控制器：
 * - 监听 `PhoneLink` 事件，维护同账号电脑的在线状态
 * - 拉取电脑上的会话列表（`sessions.list` / `sessions.page` / `sessions.changed`）
 * - 打开会话查看历史快照与实时流式增量（`event text.delta`）及工具进度（`tool.start`/`tool.end`）
 * - 发送远程指令（`message.send` / `message.accepted`）
 * - 处理审批卡片请求（`approval.request`）并回传决定（`approval.decision`），同步桌面解决状态（`approval.resolved`）
 */
class RemoteChatController(
    val phoneLink: PhoneLink,
    private val scope: CoroutineScope,
    private val cache: RemoteCache = InMemoryRemoteCache(),
    initialAccountId: String? = null,
) {
    private var currentAccountId: String? = initialAccountId
    private val _state = MutableStateFlow(
        RemoteChatState(
            peers = initialAccountId?.let { cache.peers(it) }.orEmpty(),
            selectedPeerId = initialAccountId?.let { cache.peers(it).firstOrNull()?.id },
        )
    )
    val state: StateFlow<RemoteChatState> = _state.asStateFlow()

    private var nextMsgSeq = 1L
    private var eventConsumerJob: Job? = null

    /** 刚选中一台电脑时，第一页对话列表回来后自动打开最近的一个；用户自己点了新对话或别的对话以后就不再抢。 */
    private var openLatestWhenListed = true

    init {
        startListening()
    }

    /** 账号切换时重置当前账号缓存上下文并加载对应缓存 */
    fun changeAccount(newAccountId: String?) {
        currentAccountId = newAccountId
        if (newAccountId == null) {
            _state.value = RemoteChatState()
            return
        }
        val cachedPeers = cache.peers(newAccountId)
        _state.value = RemoteChatState(
            peers = cachedPeers,
            selectedPeerId = cachedPeers.firstOrNull()?.id,
            isPeerOnline = false,
        )
    }

    private fun nextId(prefix: String = "p"): String =
        "$prefix-${nowMs()}-${nextMsgSeq++}"

    private fun nowMs(): Long =
        kotlin.time.Clock.System.now().toEpochMilliseconds()

    private fun startListening() {
        eventConsumerJob?.cancel()
        eventConsumerJob = scope.launch {
            phoneLink.events.collect { event ->
                handleLinkEvent(event)
            }
        }
    }

    suspend fun refreshPeers() {
        // 先向中转要一次最新的电脑列表（新开了遥控的电脑不用等推送），失败就用已有的。
        runCatching { phoneLink.reloadPeers() }
        val currentPeers = phoneLink.peers()
        val peersToUse = if (currentPeers.isNotEmpty()) {
            currentAccountId?.let { acc -> cache.savePeers(acc, currentPeers) }
            currentPeers
        } else {
            val cached = currentAccountId?.let { acc -> cache.peers(acc) }.orEmpty()
            if (cached.isNotEmpty()) cached else currentPeers
        }
        val currentSel = _state.value.selectedPeerId
        val chosen = peersToUse.firstOrNull { it.id == currentSel } ?: peersToUse.firstOrNull()
        _state.value = _state.value.copy(
            peers = peersToUse,
            selectedPeerId = chosen?.id,
            isPeerOnline = chosen?.online ?: false,
        )
        if (chosen != null && chosen.online) {
            sendHello(chosen.id)
            fetchSessions(chosen.id)
            fetchProjects(chosen.id)
        }
    }

    fun selectPeer(peerId: String) {
        val peer = _state.value.peers.firstOrNull { it.id == peerId } ?: return
        openLatestWhenListed = true
        val oldPeer = _state.value.selectedPeerId
        val oldSession = _state.value.selectedSessionId
        if (oldPeer != null && oldSession != null) {
            sendJson(oldPeer, buildJsonObject {
                put("t", "session.close")
                put("id", nextId())
                put("session", oldSession)
            })
        }
        val cachedSessions = currentAccountId?.let { cache.sessions(it, peer.id) }.orEmpty()
        _state.value = _state.value.copy(
            selectedPeerId = peer.id,
            isPeerOnline = peer.online,
            sessions = cachedSessions,
            recentProjects = emptyList(),
            addedProjects = emptyList(),
            newChatWorkspace = null,
            selectedSessionId = null,
            messages = emptyList(),
            toolProgress = emptyList(),
            pendingApproval = null,
        )
        if (peer.online) {
            sendHello(peer.id)
            fetchSessions(peer.id)
            fetchProjects(peer.id)
        } else if (cachedSessions.isNotEmpty()) {
            // 设备不在线、列表回不来：直接打开缓存里最近的对话。
            openLatestWhenListed = false
            selectSession(cachedSessions.maxByOrNull { it.updatedAt }!!.id)
        }
    }

    fun selectSession(sessionId: String) {
        openLatestWhenListed = false
        val peerId = _state.value.selectedPeerId ?: return
        val oldSession = _state.value.selectedSessionId
        if (oldSession != null && oldSession != sessionId) {
            sendJson(peerId, buildJsonObject {
                put("t", "session.close")
                put("id", nextId())
                put("session", oldSession)
            })
        }
        val cachedMsgs = currentAccountId?.let { cache.messages(it, peerId, sessionId) }.orEmpty()
        _state.value = _state.value.copy(
            selectedSessionId = sessionId,
            newChatWorkspace = null,
            permissionMode = "ask",
            messages = cachedMsgs,
            toolProgress = emptyList(),
            pendingApproval = null,
            sending = false,
        )
        if (_state.value.isPeerOnline) {
            sendJson(peerId, buildJsonObject {
                put("t", "session.open")
                put("id", nextId())
                put("session", sessionId)
            })
        }
    }

    /** 开始一个新对话。`workspace` 是要在哪个项目里开始；不传就留在当前所在的项目。 */
    fun newSession(workspace: String? = null) {
        openLatestWhenListed = false
        val keep = workspace ?: _state.value.currentWorkspace
        val peerId = _state.value.selectedPeerId
        val oldSession = _state.value.selectedSessionId
        if (peerId != null && oldSession != null) {
            sendJson(peerId, buildJsonObject {
                put("t", "session.close")
                put("id", nextId())
                put("session", oldSession)
            })
        }
        _state.value = _state.value.copy(
            selectedSessionId = null,
            newChatWorkspace = keep.ifBlank { null },
            permissionMode = "ask",
            messages = emptyList(),
            toolProgress = emptyList(),
            pendingApproval = null,
            sending = false,
        )
    }

    /**
     * 切换权限：`auto` 是完全信任，`ask` 是默认权限。已有对话会让电脑改掉它的模式（电脑回确认后才算数）；
     * 还没发出第一条消息的新对话只记在手机上，随第一条消息一起告诉电脑。
     */
    fun setPermission(mode: String) {
        val normalized = if (mode == "auto") "auto" else "ask"
        val s = _state.value
        if (s.permissionMode == normalized) return
        val peerId = s.selectedPeerId
        val sid = s.selectedSessionId
        _state.value = s.copy(permissionMode = normalized)
        if (peerId != null && sid != null) {
            sendJson(peerId, buildJsonObject {
                put("t", "session.permission")
                put("id", nextId())
                put("session", sid)
                put("mode", normalized)
            })
        }
    }

    /** 把电脑上最近用过的一个文件夹加成项目，并在里面开始一个新对话。 */
    fun addProject(path: String) {
        if (path.isBlank()) return
        val s = _state.value
        val known = s.sidebar.projects.any { trimSeparators(it.path) == trimSeparators(path) }
        if (!known) _state.value = s.copy(addedProjects = s.addedProjects + path)
        newSession(path)
    }

    fun send(rawText: String) {
        val text = rawText.trim()
        val peerId = _state.value.selectedPeerId ?: return
        if (text.isEmpty() || _state.value.sending || !_state.value.isPeerOnline) return

        val clientId = "c-${nowMs()}-${nextMsgSeq++}"
        val userTurn = RemoteTurn(id = clientId, role = "user", text = text)
        val assistantTurn = RemoteTurn(id = "r-$clientId", role = "assistant", text = "", streaming = true)
        val updatedMessages = _state.value.messages + userTurn + assistantTurn

        _state.value = _state.value.copy(
            messages = updatedMessages,
            sending = true,
            toolProgress = emptyList(),
        )
        currentAccountId?.let { acc ->
            val sid = _state.value.selectedSessionId
            if (sid != null) cache.saveMessages(acc, peerId, sid, updatedMessages)
        }

        sendJson(peerId, buildJsonObject {
            put("t", "message.send")
            put("id", nextId())
            val sid = _state.value.selectedSessionId
            if (!sid.isNullOrBlank()) put("session", sid)
            else {
                _state.value.newChatWorkspace?.takeIf { it.isNotBlank() }?.let { put("workspace", it) }
                if (_state.value.fullTrust) put("permission", "auto")
            }
            put("text", text)
            put("client_id", clientId)
        })
    }

    fun answerApproval(approvalId: String, choice: String) {
        val peerId = _state.value.selectedPeerId ?: return
        sendJson(peerId, buildJsonObject {
            put("t", "approval.decision")
            put("id", nextId())
            put("approval_id", approvalId)
            put("choice", choice)
        })
        if (_state.value.pendingApproval?.approvalId == approvalId) {
            _state.value = _state.value.copy(pendingApproval = null)
        }
    }

    /** 把一台电脑从账号的设备列表里移除（它再启动会重新出现）。 */
    fun removePeer(peerId: String) {
        scope.launch {
            try {
                phoneLink.removeDevice(peerId)
                currentAccountId?.let { acc -> cache.removePeer(acc, peerId) }
                refreshPeers()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                _state.value = _state.value.copy(error = e.message)
            }
        }
    }

    // ───────────── 协议收发处理 ─────────────

    private fun sendJson(peer: String, obj: JsonObject, notify: Boolean = false) {
        scope.launch {
            val bytes = obj.toString().encodeToByteArray()
            runCatching { phoneLink.send(peer, bytes, notify) }
        }
    }

    private fun sendHello(peer: String) {
        sendJson(peer, buildJsonObject {
            put("t", "hello")
            put("id", nextId())
            put("v", 1)
            put("caps", buildJsonArray {
                add(JsonPrimitive("sessions"))
                add(JsonPrimitive("stream"))
                add(JsonPrimitive("approvals"))
                add(JsonPrimitive("projects"))
            })
            put("app", "xiaoban-android")
        })
    }

    fun fetchSessions(peerId: String? = _state.value.selectedPeerId) {
        val target = peerId ?: return
        _state.value = _state.value.copy(loadingSessions = true)
        sendJson(target, buildJsonObject {
            put("t", "sessions.list")
            put("id", nextId())
            put("limit", 50)
        })
    }

    fun fetchProjects(peerId: String? = _state.value.selectedPeerId) {
        val target = peerId ?: return
        sendJson(target, buildJsonObject {
            put("t", "projects.list")
            put("id", nextId())
        })
    }

    private suspend fun handleLinkEvent(event: LinkEvent) {
        when (event) {
            is LinkEvent.Connected -> {
                refreshPeers()
            }
            is LinkEvent.Disconnected -> {
                _state.value = _state.value.copy(
                    isPeerOnline = false,
                    sending = false,
                    peers = _state.value.peers.map { it.copy(online = false) },
                )
            }
            is LinkEvent.PeerOnline -> {
                val updated = _state.value.peers.map {
                    if (it.id == event.peer) it.copy(online = event.online) else it
                }
                val isSelectedOnline = if (event.peer == _state.value.selectedPeerId) event.online else _state.value.isPeerOnline
                _state.value = _state.value.copy(peers = updated, isPeerOnline = isSelectedOnline)
                if (event.peer == _state.value.selectedPeerId && event.online) {
                    sendHello(event.peer)
                    fetchSessions(event.peer)
                    fetchProjects(event.peer)
                    val sid = _state.value.selectedSessionId
                    if (sid != null) {
                        sendJson(event.peer, buildJsonObject {
                            put("t", "session.open")
                            put("id", nextId())
                            put("session", sid)
                        })
                    }
                }
            }
            is LinkEvent.PeerAdded, is LinkEvent.PeerRemoved -> {
                refreshPeers()
            }
            is LinkEvent.Message -> {
                handleRemoteMessage(event.peer, event.data)
            }
            else -> {}
        }
    }

    private fun handleRemoteMessage(peer: String, data: ByteArray) {
        val text = data.decodeToString()
        val obj = runCatching { ProtocolJson.parseToJsonElement(text).jsonObject }.getOrNull() ?: return
        val t = obj["t"].asStr() ?: return
        when (t) {
            "sessions.page" -> {
                if (peer == _state.value.selectedPeerId) {
                    val rawItems = obj["items"]?.jsonArray.orEmpty()
                    val items = rawItems.mapNotNull {
                        runCatching { ProtocolJson.decodeFromJsonElement(RemoteSessionSummary.serializer(), it) }.getOrNull()
                    }
                    _state.value = _state.value.copy(sessions = items, loadingSessions = false)
                    currentAccountId?.let { acc -> cache.saveSessions(acc, peer, items) }
                    if (openLatestWhenListed && _state.value.selectedSessionId == null && items.isNotEmpty()) {
                        openLatestWhenListed = false
                        selectSession(items.maxByOrNull { it.updatedAt }!!.id)
                    }
                }
            }
            "projects.page" -> {
                if (peer == _state.value.selectedPeerId) {
                    val items = obj["recent"]?.jsonArray.orEmpty().mapNotNull {
                        val o = it as? JsonObject ?: return@mapNotNull null
                        val path = o["path"].asStr()?.takeIf { p -> p.isNotBlank() } ?: return@mapNotNull null
                        RemoteProject(path, o["name"].asStr()?.takeIf { n -> n.isNotBlank() } ?: folderName(path))
                    }
                    _state.value = _state.value.copy(recentProjects = items)
                }
            }
            "sessions.changed" -> {
                if (peer == _state.value.selectedPeerId) {
                    val rawItems = obj["items"]?.jsonArray.orEmpty()
                    val items = rawItems.mapNotNull {
                        runCatching { ProtocolJson.decodeFromJsonElement(RemoteSessionSummary.serializer(), it) }.getOrNull()
                    }
                    val existingMap = _state.value.sessions.associateBy { it.id }.toMutableMap()
                    for (item in items) {
                        existingMap[item.id] = item
                    }
                    val sorted = existingMap.values.sortedByDescending { it.updatedAt }
                    _state.value = _state.value.copy(sessions = sorted)
                    currentAccountId?.let { acc -> cache.saveSessions(acc, peer, sorted) }
                }
            }
            "session.permission" -> {
                val sid = obj["session"].asStr()
                if (peer == _state.value.selectedPeerId && sid == _state.value.selectedSessionId) {
                    if ((obj["ok"] as? JsonPrimitive)?.booleanOrNull == true) {
                        _state.value = _state.value.copy(permissionMode = if (obj["mode"].asStr() == "auto") "auto" else "ask")
                    } else {
                        // 电脑没答应：回到电脑上实际的模式，并说明原因。
                        val actual = if (_state.value.permissionMode == "auto") "ask" else "auto"
                        _state.value = _state.value.copy(permissionMode = actual, error = obj["error"].asStr() ?: "切换权限失败")
                    }
                }
            }
            "session.snapshot" -> {
                val sid = obj["session"].asStr()
                if (peer == _state.value.selectedPeerId && (sid == null || sid == _state.value.selectedSessionId)) {
                    obj["permission"].asStr()?.let { _state.value = _state.value.copy(permissionMode = if (it == "auto") "auto" else "ask") }
                    val rawMessages = obj["messages"]?.jsonArray.orEmpty()
                    // 桌面按字节预算分页：`after` 为本页起点，`has_more` 为真时用 `cursor` 继续取。
                    val after = obj["after"]?.jsonPrimitive?.intOrNull ?: 0
                    val turns = mutableListOf<RemoteTurn>()
                    for ((index, m) in rawMessages.withIndex()) {
                        val mObj = m.jsonObject
                        val role = mObj["role"].asStr() ?: "assistant"
                        val content = extractMessageText(mObj)
                        turns.add(RemoteTurn(id = "snap-${after + index}", role = role, text = content))
                    }
                    val merged = if (after > 0) {
                        val current = _state.value.messages
                        current.filter { it.id.startsWith("snap-") } + turns + current.filter { !it.id.startsWith("snap-") }
                    } else {
                        turns
                    }
                    _state.value = _state.value.copy(messages = merged)
                    currentAccountId?.let { acc ->
                        val targetSid = sid ?: _state.value.selectedSessionId
                        if (targetSid != null) cache.saveMessages(acc, peer, targetSid, merged)
                    }
                    val hasMore = obj["has_more"]?.jsonPrimitive?.booleanOrNull == true
                    val cursor = obj["cursor"]?.jsonPrimitive?.intOrNull
                    if (hasMore && cursor != null && cursor > after && sid != null) {
                        sendJson(peer, buildJsonObject {
                            put("t", "session.open")
                            put("id", nextId())
                            put("session", sid)
                            put("after", cursor)
                        })
                    } else {
                        // 最后一页：以桌面当前的待审批为准；没有待审批就收起可能已过期的卡片。
                        val pendingObj = obj["pending"] as? JsonObject
                        _state.value = _state.value.copy(pendingApproval = parsePendingApprovals(pendingObj, sid.orEmpty()))
                    }
                }
            }
            "message.accepted" -> {
                val sid = obj["session"].asStr()
                if (sid != null && _state.value.selectedSessionId == null) {
                    _state.value = _state.value.copy(selectedSessionId = sid)
                    fetchSessions(peer)
                }
            }
            "event" -> {
                val sid = obj["session"].asStr()
                if (peer == _state.value.selectedPeerId && (sid.isNullOrBlank() || sid == _state.value.selectedSessionId)) {
                    handleEventPayload(obj)
                }
            }
            "approval.request" -> {
                val approvalId = obj["approval_id"].asStr() ?: return
                val session = obj["session"].asStr().orEmpty()
                val title = obj["title"].asStr() ?: "请确认"
                val detail = (obj["detail"] as? JsonObject) ?: JsonObject(emptyMap())
                val risk = obj["risk"].asStr() ?: "low"
                val rawOpts = obj["options"]?.jsonArray.orEmpty()
                val options = rawOpts.mapNotNull {
                    val opt = it.jsonObject
                    val id = opt["id"].asStr() ?: return@mapNotNull null
                    val label = opt["label"].asStr() ?: id
                    RemoteApprovalOption(id, label)
                }
                _state.value = _state.value.copy(
                    pendingApproval = RemoteApprovalRequest(approvalId, session, title, detail, risk, options),
                )
            }
            "approval.resolved" -> {
                val approvalId = obj["approval_id"].asStr()
                if (approvalId != null && _state.value.pendingApproval?.approvalId == approvalId) {
                    _state.value = _state.value.copy(pendingApproval = null)
                }
            }
            else -> {}
        }
    }

    private fun handleEventPayload(obj: JsonObject) {
        val kind = obj["kind"].asStr() ?: return
        val data = (obj["data"] as? JsonObject) ?: JsonObject(emptyMap())
        when (kind) {
            "text.delta" -> {
                val piece = data["text"].asStr().orEmpty()
                if (piece.isNotEmpty()) {
                    appendAssistantDelta(piece)
                }
            }
            "reasoning.delta" -> {
                // 可选思考过程，暂不打断主正文
            }
            "tool.start" -> {
                val id = data["id"].asStr() ?: nextId("t")
                val name = data["name"].asStr() ?: "工具调用"
                val updated = _state.value.toolProgress + RemoteToolProgress(id, name, running = true, detail = summarizeArgs(data["arguments"]))
                _state.value = _state.value.copy(toolProgress = updated)
            }
            "tool.end" -> {
                val id = data["id"].asStr()
                val isErr = data["is_error"]?.jsonPrimitive?.booleanOrNull
                val updated = _state.value.toolProgress.map {
                    if (it.id == id || (id == null && it.running)) it.copy(running = false, isError = isErr) else it
                }
                _state.value = _state.value.copy(toolProgress = updated)
            }
            "turn.done" -> {
                finalizeAssistant(error = null)
            }
            "error" -> {
                val msg = data["message"].asStr() ?: "执行出错"
                finalizeAssistant(error = msg)
            }
            "permission" -> {
                _state.value = _state.value.copy(permissionMode = if (data["mode"].asStr() == "auto") "auto" else "ask")
            }
            "state" -> {
                val st = data["state"].asStr()
                if (st == "cancelled") {
                    finalizeAssistant(error = "已取消")
                }
            }
        }
    }

    private fun appendAssistantDelta(delta: String) {
        val msgs = _state.value.messages.toMutableList()
        val lastIdx = msgs.indexOfLast { it.role == "assistant" && it.streaming }
        if (lastIdx >= 0) {
            val old = msgs[lastIdx]
            msgs[lastIdx] = old.copy(text = old.text + delta)
        } else {
            msgs.add(RemoteTurn(id = nextId("r"), role = "assistant", text = delta, streaming = true))
        }
        _state.value = _state.value.copy(messages = msgs)
    }

    private fun finalizeAssistant(error: String?) {
        val msgs = _state.value.messages.map { turn ->
            if (turn.role == "assistant" && turn.streaming) {
                turn.copy(streaming = false, error = error)
            } else {
                turn
            }
        }
        _state.value = _state.value.copy(messages = msgs, sending = false, toolProgress = emptyList())
        currentAccountId?.let { acc ->
            val sid = _state.value.selectedSessionId
            val peer = _state.value.selectedPeerId
            if (sid != null && peer != null) {
                cache.saveMessages(acc, peer, sid, msgs)
            }
        }
    }

    private fun extractMessageText(msgObj: JsonObject): String {
        val parts = msgObj["parts"] as? JsonArray
        if (parts != null) {
            val sb = StringBuilder()
            for (p in parts) {
                val pObj = p.jsonObject
                if (pObj["type"].asStr() == "text") {
                    sb.append(pObj["text"].asStr().orEmpty())
                }
            }
            return sb.toString()
        }
        return msgObj["content"].asStr().orEmpty()
    }

    private fun parsePendingApprovals(pending: JsonObject?, session: String): RemoteApprovalRequest? {
        if (pending == null) return null
        val approvals = pending["approvals"] as? JsonArray
        if (!approvals.isNullOrEmpty()) {
            val app = approvals.first().jsonObject
            val id = app["id"].asStr() ?: return null
            val tool = app["tool"].asStr() ?: "操作"
            val danger = app["danger"].asStr()
            val risk = if (danger != null || app["externalPath"]?.jsonPrimitive?.booleanOrNull == true) "high" else "low"
            val title = danger ?: "允许执行 $tool？"
            return RemoteApprovalRequest(
                approvalId = id,
                session = session,
                title = title,
                detail = (app["arguments"] as? JsonObject) ?: JsonObject(emptyMap()),
                risk = risk,
                options = listOf(
                    RemoteApprovalOption("once", "允许一次"),
                    RemoteApprovalOption("deny", "拒绝"),
                ),
            )
        }
        val questions = pending["questions"] as? JsonArray
        if (!questions.isNullOrEmpty()) {
            val q = questions.first().jsonObject
            val id = q["id"].asStr() ?: return null
            val text = q["question"].asStr() ?: "请确认"
            val opts = (q["options"] as? JsonArray).orEmpty().mapNotNull {
                it.asStr()?.let { s -> RemoteApprovalOption(s, s) }
            }
            return RemoteApprovalRequest(
                approvalId = id,
                session = session,
                title = text,
                options = opts + RemoteApprovalOption("__dismiss__", "跳过"),
            )
        }
        return null
    }
}
