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
)

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
    val selectedSessionId: String? = null,
    val loadingSessions: Boolean = false,
    val messages: List<RemoteTurn> = emptyList(),
    val toolProgress: List<RemoteToolProgress> = emptyList(),
    val sending: Boolean = false,
    val pendingApproval: RemoteApprovalRequest? = null,
    val error: String? = null,
) {
    val selectedPeer: PeerSummary? get() = peers.firstOrNull { it.id == selectedPeerId }
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
) {
    private val _state = MutableStateFlow(RemoteChatState())
    val state: StateFlow<RemoteChatState> = _state.asStateFlow()

    private var nextMsgSeq = 1L
    private var eventConsumerJob: Job? = null

    init {
        startListening()
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
        val currentSel = _state.value.selectedPeerId
        val chosen = currentPeers.firstOrNull { it.id == currentSel } ?: currentPeers.firstOrNull()
        _state.value = _state.value.copy(
            peers = currentPeers,
            selectedPeerId = chosen?.id,
            isPeerOnline = chosen?.online ?: false,
        )
        if (chosen != null && chosen.online) {
            sendHello(chosen.id)
            fetchSessions(chosen.id)
        }
    }

    fun selectPeer(peerId: String) {
        val peer = _state.value.peers.firstOrNull { it.id == peerId } ?: return
        val oldPeer = _state.value.selectedPeerId
        val oldSession = _state.value.selectedSessionId
        if (oldPeer != null && oldSession != null) {
            sendJson(oldPeer, buildJsonObject {
                put("t", "session.close")
                put("id", nextId())
                put("session", oldSession)
            })
        }
        _state.value = _state.value.copy(
            selectedPeerId = peer.id,
            isPeerOnline = peer.online,
            sessions = emptyList(),
            selectedSessionId = null,
            messages = emptyList(),
            toolProgress = emptyList(),
            pendingApproval = null,
        )
        if (peer.online) {
            sendHello(peer.id)
            fetchSessions(peer.id)
        }
    }

    fun selectSession(sessionId: String) {
        val peerId = _state.value.selectedPeerId ?: return
        val oldSession = _state.value.selectedSessionId
        if (oldSession != null && oldSession != sessionId) {
            sendJson(peerId, buildJsonObject {
                put("t", "session.close")
                put("id", nextId())
                put("session", oldSession)
            })
        }
        _state.value = _state.value.copy(
            selectedSessionId = sessionId,
            messages = emptyList(),
            toolProgress = emptyList(),
            pendingApproval = null,
            sending = false,
        )
        sendJson(peerId, buildJsonObject {
            put("t", "session.open")
            put("id", nextId())
            put("session", sessionId)
        })
    }

    fun newSession() {
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
            messages = emptyList(),
            toolProgress = emptyList(),
            pendingApproval = null,
            sending = false,
        )
    }

    fun send(rawText: String) {
        val text = rawText.trim()
        val peerId = _state.value.selectedPeerId ?: return
        if (text.isEmpty() || _state.value.sending || !_state.value.isPeerOnline) return

        val clientId = "c-${nowMs()}-${nextMsgSeq++}"
        val userTurn = RemoteTurn(id = clientId, role = "user", text = text)
        val assistantTurn = RemoteTurn(id = "r-$clientId", role = "assistant", text = "", streaming = true)

        _state.value = _state.value.copy(
            messages = _state.value.messages + userTurn + assistantTurn,
            sending = true,
            toolProgress = emptyList(),
        )

        sendJson(peerId, buildJsonObject {
            put("t", "message.send")
            put("id", nextId())
            val sid = _state.value.selectedSessionId
            if (!sid.isNullOrBlank()) put("session", sid)
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
                    if (_state.value.selectedSessionId == null && items.isNotEmpty()) {
                        selectSession(items[0].id)
                    }
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
                }
            }
            "session.snapshot" -> {
                val sid = obj["session"].asStr()
                if (peer == _state.value.selectedPeerId && (sid == null || sid == _state.value.selectedSessionId)) {
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
