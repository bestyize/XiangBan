package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.plugins.timeout
import io.ktor.client.request.forms.formData
import io.ktor.client.request.forms.submitFormWithBinaryData
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.statement.readRawBytes
import io.ktor.http.Headers
import io.ktor.http.isSuccess
import io.ktor.client.request.preparePost
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsChannel
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.contentType
import io.ktor.utils.io.readUTF8Line
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.roundToLong
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

// ───────────── server-ai 数据模型 ─────────────

@Serializable
data class AiTier(
    val tier: String,
    @SerialName("display_name") val displayName: String,
    val description: String = "",
    @SerialName("is_default") val isDefault: Boolean = false,
    @SerialName("requires_paid") val requiresPaid: Boolean = false,
    val available: Boolean = true,
    @SerialName("reasoning_efforts") val reasoningEfforts: List<String> = emptyList(),
)

@Serializable
data class AiTiersResponse(
    val tiers: List<AiTier> = emptyList(),
)

@Serializable
data class AiBucket(
    val id: Long,
    val kind: String,
    val total: Long,
    val remaining: Long,
    @SerialName("expires_at") val expiresAt: String? = null,
    @SerialName("created_at") val createdAt: String = "",
)

@Serializable
data class AiTrial(
    val available: Boolean = false,
    val quota: Long = 0,
    val days: Int = 0,
)

@Serializable
data class AiProductInfo(
    val name: String = "",
)

@Serializable
data class AiBalance(
    val product: AiProductInfo = AiProductInfo(),
    @SerialName("quota_per_unit") val quotaPerUnit: Long = 500_000,
    val total: Long = 0,
    val paid: Long = 0,
    val buckets: List<AiBucket> = emptyList(),
    val trial: AiTrial = AiTrial(),
)

@Serializable
data class AiRedeemed(
    val kind: String = "",
    val quota: Long = 0,
    @SerialName("expires_at") val expiresAt: String? = null,
    val balance: Long = 0,
)

@Serializable
data class ChatMessage(
    val role: String,
    val content: String,
    val attachments: List<ChatAttachment> = emptyList(),
)

/** 与桌面端 `pickTier` 保持一致：优先保留当前可用选择，否则取默认档位，再退回第一个可用档位。 */
fun pickAiTier(tiers: List<AiTier>, current: String): String {
    val usable = tiers.filter { it.available }
    if (usable.any { it.tier == current }) return current
    return (usable.firstOrNull { it.isDefault } ?: usable.firstOrNull())?.tier.orEmpty()
}

fun bucketKindLabel(kind: String): String = when (kind) {
    "trial" -> "试用额度"
    "monthly" -> "月卡"
    "topup" -> "加油包"
    else -> "额度"
}

/**
 * 与桌面端 `money(points, perDollar)` 保持一致：
 * 额度统一折算成美元显示；大于等于 $100 取整，$1–$100 保留两位小数，小于 $1 保留三位小数（去掉末尾多余的第 3 位 0）。
 */
fun formatAiMoney(points: Long, quotaPerUnit: Long = 500_000): String {
    val unit = max(quotaPerUnit, 1L).toDouble()
    val dollars = points.toDouble() / unit
    val sign = if (dollars < 0) "−" else ""
    val value = abs(dollars)
    if (value >= 100.0) {
        return "$sign$${formatWithCommas(value.roundToLong())}"
    }
    if (value >= 1.0) {
        val cents = (value * 100.0).roundToLong()
        val whole = cents / 100
        val frac = (cents % 100).toString().padStart(2, '0')
        return "$sign$${formatWithCommas(whole)}.$frac"
    }
    val millis = (value * 1000.0).roundToLong()
    val whole = millis / 1000
    val frac3 = (millis % 1000).toString().padStart(3, '0')
    val frac = if (frac3.endsWith('0')) frac3.dropLast(1) else frac3
    return "$sign$$whole.$frac"
}

private fun formatWithCommas(n: Long): String {
    val digits = n.toString()
    val out = StringBuilder()
    for (i in digits.indices) {
        if (i > 0 && (digits.length - i) % 3 == 0) out.append(',')
        out.append(digits[i])
    }
    return out.toString()
}

// ───────────── server-ai 网络客户端 ─────────────

private const val MAX_REDEEM_CODE_LEN = 64

/**
 * 小伴 AI 网关（server-ai）客户端：
 * - `GET /api/ai/tiers`：当前产品可用档位（如“日常”/“深入”），不含真实模型名
 * - `GET /api/ai/balance`：只读查询余额与试用状态
 * - `POST /api/ai/redeem`：兑换码充值
 * - `POST /v1/chat/completions`：流式对话（SSE），同时兼容非流式 JSON 响应
 */
class AiGatewayApi(
    private val client: HttpClient,
    config: CommunityConfig = CommunityConfig(),
    private val tokenProvider: suspend () -> String?,
) {
    private val base = config.aiBase.trimEnd('/')
    private val http = JsonHttp(client, base)

    private suspend fun requireToken(): String =
        tokenProvider() ?: throw ApiException(401, "请先登录小伴账号", "unauthorized")

    suspend fun tiers(): List<AiTier> =
        http.send(HttpMethod.Get, "/api/ai/tiers", AiTiersResponse.serializer(), token = requireToken()).tiers

    suspend fun balance(): AiBalance =
        http.send(HttpMethod.Get, "/api/ai/balance", AiBalance.serializer(), token = requireToken())

    suspend fun redeem(code: String): AiRedeemed {
        val clean = code.trim()
        if (clean.isEmpty()) throw ApiException(400, "请输入兑换码", "code_empty")
        if (clean.length > MAX_REDEEM_CODE_LEN) throw ApiException(400, "兑换码太长，请检查是否复制完整", "code_invalid")
        val body = buildJsonObject { put("code", clean) }.toString()
        return http.send(HttpMethod.Post, "/api/ai/redeem", AiRedeemed.serializer(), body = body, token = requireToken())
    }

    /** 云端语音识别：上传一段 WAV，返回识别出的文字。网关没开这项功能时抛 `transcribe_unavailable`。 */
    suspend fun transcribe(wav: ByteArray, language: String = "zh"): String {
        val token = requireToken()
        val response = try {
            client.submitFormWithBinaryData(
                "$base/v1/audio/transcriptions",
                formData {
                    append("language", language)
                    append("file", wav, Headers.build {
                        append(HttpHeaders.ContentType, "audio/wav")
                        append(HttpHeaders.ContentDisposition, "filename=\"speech.wav\"")
                    })
                },
            ) {
                header(HttpHeaders.Authorization, "Bearer $token")
                timeout { requestTimeoutMillis = 60_000 }
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            throw ApiException(0, "连接小伴 AI 服务失败，请检查网络后重试")
        }
        val text = response.bodyAsText()
        if (!response.status.isSuccess()) throw parseApiError(response.status.value, text)
        val heard = runCatching { CommunityJson.parseToJsonElement(text).jsonObject["text"]?.jsonPrimitive?.contentOrNull }.getOrNull()
        return heard.orEmpty().trim()
    }

    /** 云端语音合成：一句话变成 MP3。网关没开这项功能时抛 `speech_unavailable`。 */
    suspend fun speak(text: String): ByteArray {
        val token = requireToken()
        val response = try {
            client.post("$base/v1/audio/speech") {
                header(HttpHeaders.Authorization, "Bearer $token")
                contentType(ContentType.Application.Json)
                setBody(buildJsonObject { put("input", text.trim()) }.toString())
                timeout { requestTimeoutMillis = 60_000 }
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            throw ApiException(0, "连接小伴 AI 服务失败，请检查网络后重试")
        }
        if (!response.status.isSuccess()) throw parseApiError(response.status.value, response.bodyAsText())
        return response.readRawBytes()
    }

    internal suspend fun postChatRaw(body: String, onLine: suspend (String) -> Unit): ModelResult {
        val token = try {
            requireToken()
        } catch (e: ApiException) {
            return ModelResult(e.status, error = e.message)
        }
        return try {
            client.preparePost("$base/v1/chat/completions") {
                header(HttpHeaders.Authorization, "Bearer $token")
                header(HttpHeaders.Accept, "text/event-stream")
                contentType(ContentType.Application.Json)
                setBody(body)
                timeout {
                    requestTimeoutMillis = 180_000
                    socketTimeoutMillis = 60_000
                }
            }.execute { response ->
                val status = response.status.value
                if (status !in 200..299) return@execute ModelResult(status, body = response.bodyAsText())
                val channel = response.bodyAsChannel()
                while (!channel.isClosedForRead) {
                    val line = channel.readUTF8Line() ?: break
                    onLine(line)
                }
                ModelResult(status)
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            ModelResult(0, error = "连接小伴 AI 服务失败，请检查网络后重试")
        }
    }

    /**
     * 向 `/v1/chat/completions` 发起流式对话，每收到一段增量文字调用一次 [onDelta]，最后返回完整回复。
     */
    suspend fun streamChat(
        messages: List<ChatMessage>,
        tier: String? = null,
        onDelta: (String) -> Unit = {},
    ): String {
        val token = requireToken()
        val payload = buildJsonObject {
            if (!tier.isNullOrBlank()) put("model", tier.trim())
            put("stream", true)
            put(
                "messages",
                buildJsonArray {
                    for (m in messages) {
                        add(buildJsonObject {
                            put("role", m.role)
                            if (m.attachments.isEmpty()) put("content", m.content)
                            else put("content", attachmentParts(m.content, m.attachments, chatCompletions = true))
                        })
                    }
                },
            )
        }.toString()

        val statement = try {
            client.preparePost("$base/v1/chat/completions") {
                header(HttpHeaders.Authorization, "Bearer $token")
                header(HttpHeaders.Accept, "text/event-stream")
                contentType(ContentType.Application.Json)
                setBody(payload)
                timeout {
                    requestTimeoutMillis = 120_000
                    socketTimeoutMillis = 60_000
                }
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            throw ApiException(0, "连接小伴 AI 服务失败，请检查网络后重试")
        }

        val full = StringBuilder()
        try {
            statement.execute { response ->
                val status = response.status.value
                if (status !in 200..299) {
                    throw parseApiError(status, response.bodyAsText())
                }
                val contentType = response.headers[HttpHeaders.ContentType].orEmpty()
                if (!contentType.startsWith("text/event-stream")) {
                    val text = response.bodyAsText()
                    val piece = extractSingleReply(text)
                    if (piece.isNotEmpty()) {
                        full.append(piece)
                        onDelta(piece)
                    }
                    return@execute
                }
                val channel = response.bodyAsChannel()
                while (!channel.isClosedForRead) {
                    val line = channel.readUTF8Line() ?: break
                    if (!line.startsWith("data:")) continue
                    val data = line.removePrefix("data:").trim()
                    if (data.isEmpty()) continue
                    if (data == "[DONE]") break
                    val piece = parseSseData(data)
                    if (piece.isNotEmpty()) {
                        full.append(piece)
                        onDelta(piece)
                    }
                }
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: ApiException) {
            throw e
        } catch (e: Exception) {
            throw ApiException(0, "连接小伴 AI 服务失败，请检查网络后重试")
        }
        return full.toString()
    }
}

/** 一次模型请求的结果：HTTP 状态，失败时的响应文本，或连接层面的错误。 */
data class ModelResult(val status: Int, val body: String? = null, val error: String? = null)

/**
 * 把已经组好的 chat-completions 请求体（流式）发给 server-ai，响应的每一行原样交给 [onLine]。
 * Agent 内核只负责拼请求和解析流，网络请求（地址、令牌、超时）都在这里，所以手机里不用带 TLS 库。
 */
suspend fun AiGatewayApi.postChatLines(body: String, onLine: suspend (String) -> Unit): ModelResult = postChatRaw(body, onLine)

private fun JsonElement?.asStr(): String? =
    this?.takeIf { it !is JsonNull }?.let { runCatching { it.jsonPrimitive.contentOrNull }.getOrNull() }

private fun parseSseData(data: String): String {
    val obj = runCatching { CommunityJson.parseToJsonElement(data) as? JsonObject }.getOrNull() ?: return ""
    if (obj["error"] is JsonObject) {
        throw parseApiError(502, data)
    }
    val choice = (obj["choices"] as? JsonArray)?.firstOrNull() as? JsonObject
    val delta = choice?.get("delta") as? JsonObject
    val content = delta?.get("content").asStr()
        ?: (choice?.get("message") as? JsonObject)?.get("content").asStr()
        ?: obj["delta"].asStr()
    return content.orEmpty()
}

private fun extractSingleReply(body: String): String {
    // 若上游返回的是多行 data: 文本但未标 text/event-stream，也按 SSE 兜底解析。
    if (body.lineSequence().any { it.startsWith("data:") }) {
        val sb = StringBuilder()
        for (line in body.lineSequence()) {
            if (!line.startsWith("data:")) continue
            val data = line.removePrefix("data:").trim()
            if (data.isEmpty() || data == "[DONE]") continue
            sb.append(parseSseData(data))
        }
        return sb.toString()
    }
    val obj = runCatching { CommunityJson.parseToJsonElement(body) as? JsonObject }.getOrNull()
        ?: throw ApiException(200, "小伴 AI 服务返回了无法识别的内容")
    if (obj["error"] is JsonObject) throw parseApiError(502, body)
    val choice = (obj["choices"] as? JsonArray)?.firstOrNull() as? JsonObject
    val message = choice?.get("message") as? JsonObject
    return message?.get("content").asStr().orEmpty()
}

// ───────────── 云端纯聊天状态控制器 ─────────────

@Serializable
data class ChatTurn(
    val id: Long,
    val role: String,
    val text: String,
    val streaming: Boolean = false,
    val error: String? = null,
    /** 这一轮智能体调用的工具（含参数和结果），界面画成工具调用组。 */
    val tools: List<ToolCallInfo> = emptyList(),
    /** 处理过的授权、回答过的问题留下的痕迹。 */
    val records: List<ChatRecord> = emptyList(),
    val attachments: List<ChatAttachment> = emptyList(),
)

/** 智能体要改动东西之前问用户：同意这一次，还是拒绝。 */
data class ChatApproval(
    val id: String,
    val tool: String,
    val detail: String = "",
    val danger: String? = null,
)

data class ChatQuestion(val id: String, val question: String, val options: List<String>)

data class CloudChatState(
    val tiers: List<AiTier> = emptyList(),
    val selectedTier: String = "",
    val balance: AiBalance? = null,
    val loadingMeta: Boolean = false,
    val metaError: String? = null,
    val messages: List<ChatTurn> = emptyList(),
    val sending: Boolean = false,
    val redeeming: Boolean = false,
    val redeemMessage: String? = null,
    val redeemError: String? = null,
    val approval: ChatApproval? = null,
    val question: ChatQuestion? = null,
    /** 当前对话的编号（智能体内核也按它存对话）。 */
    val conversationId: String = "",
    /** 存下来的对话，最近的在前。 */
    val conversations: List<ChatSummary> = emptyList(),
) {
    val activeTier: AiTier? get() = tiers.firstOrNull { it.tier == selectedTier }
}

@OptIn(ExperimentalTime::class)
private fun systemNowMs(): Long = Clock.System.now().toEpochMilliseconds()

/** 内核报的错带着“HTTP 402: ”这样的前缀，给人看时去掉。 */
internal fun friendlyAgentError(raw: String?): String =
    raw.orEmpty().replace(Regex("^HTTP \\d+: "), "").ifBlank { "智能体执行出错" }

/** 给工具卡片和确认框看的一行说明。 */
internal fun summarizeToolArguments(tool: String, arguments: String): String {
    val obj = runCatching { CommunityJson.parseToJsonElement(arguments) as? JsonObject }.getOrNull() ?: return arguments.take(120)
    if (tool == "set_reminder") {
        val title = obj["title"].asStr().orEmpty()
        val minutes = obj["minutes"].asStr()
        return listOfNotNull(title.ifBlank { null }, minutes?.let { "$it 分钟后" }).joinToString(" · ")
    }
    return obj.entries.joinToString("，") { (k, v) -> "$k=${v.asStr() ?: v}" }.take(120)
}

private const val DEFAULT_SYSTEM_PROMPT =
    "你是小伴，用户随身的温和、真诚、靠谱的 AI 伙伴。回答直接清晰、重点突出，默认使用简体中文。"

/**
 * 手机端云端纯聊天控制器：管理档位、额度、兑换码以及与 `server-ai` 的流式多轮对话。
 * 不依赖 Compose，可在 JVM 单元测试中直接验证。
 */
class CloudChatController(
    private val api: AiGatewayApi,
    private val scope: CoroutineScope,
    private val systemPrompt: String = DEFAULT_SYSTEM_PROMPT,
    /** 手机里的智能体内核；为空就是原来的纯云端聊天。 */
    private val agent: AgentRuntime? = null,
    /** 对话存在哪里；为空就只在这次打开应用期间记着。 */
    private val history: ChatHistory? = null,
    private val clock: () -> Long = ::systemNowMs,
    private val newSessionId: () -> String = { newChatId(clock()) },
) {
    private val _state = MutableStateFlow(CloudChatState())
    val state: StateFlow<CloudChatState> = _state.asStateFlow()
    private var nextId = 1L
    private var metaJob: Job? = null
    private var chatJob: Job? = null
    private var agentSession = newSessionId()
        set(value) { field = value; _state.value = _state.value.copy(conversationId = value) }
    private var agentRun: String? = null
    private var agentReplyId: Long = 0

    init {
        _state.value = _state.value.copy(conversationId = agentSession)
        agent?.let { runtime -> scope.launch { runtime.events.collect(::onAgentEvent) } }
    }

    /** 换到某个账号（登录后、换号后调用）：读出这个账号存下的对话列表，当前对话从头开始。 */
    fun useAccount(account: String?) {
        val list = history?.useAccount(account) ?: emptyList()
        clearConversation()
        _state.value = _state.value.copy(conversations = list)
    }

    /** 打开存下来的某个对话，接着聊；智能体内核按同一个编号接上它记着的上下文。 */
    fun openConversation(id: String) {
        if (id == agentSession && _state.value.messages.isNotEmpty()) return
        val saved = history?.load(id)
        if (saved == null) {
            _state.value = _state.value.copy(conversations = history?.list().orEmpty())
            return
        }
        stop()
        agentSession = id
        agentRun = null
        nextId = (saved.maxOfOrNull { it.id } ?: 0L) + 1
        _state.value = _state.value.copy(messages = saved, approval = null, question = null)
    }

    /** 删掉一个存下来的对话；删的是正在看的就换成新对话。 */
    fun deleteConversation(id: String) {
        val store = history ?: return
        // 先离开这个对话（会把没说完的存一下），再删，免得删完又被存回来。
        if (id == agentSession) clearConversation()
        _state.value = _state.value.copy(conversations = store.delete(id))
    }

    /** 把当前对话存一下（发出提问、回答结束、停止时）。 */
    private fun persist() {
        val list = history?.save(agentSession, _state.value.messages, clock()) ?: return
        _state.value = _state.value.copy(conversations = list)
    }

    /** 加载档位与余额，并按桌面端同一规则选定可用默认档位。 */
    fun loadMeta() {
        metaJob?.cancel()
        _state.value = _state.value.copy(loadingMeta = true, metaError = null)
        metaJob = scope.launch {
            try {
                coroutineScope {
                    val balanceDeferred = async { runCatching { api.balance() } }
                    val tiersDeferred = async { runCatching { api.tiers() } }
                    val balanceRes = balanceDeferred.await()
                    val tiersRes = tiersDeferred.await()
                    val current = _state.value
                    val nextTiers = tiersRes.getOrNull() ?: current.tiers
                    val nextBalance = balanceRes.getOrNull() ?: current.balance
                    val err = balanceRes.exceptionOrNull() ?: tiersRes.exceptionOrNull()
                    if (err is CancellationException) throw err
                    _state.value = current.copy(
                        tiers = nextTiers,
                        selectedTier = pickAiTier(nextTiers, current.selectedTier),
                        balance = nextBalance,
                        loadingMeta = false,
                        metaError = if (balanceRes.isFailure && tiersRes.isFailure && err != null) messageOf(err) else null,
                    )
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                _state.value = _state.value.copy(loadingMeta = false, metaError = messageOf(e))
            }
        }
    }

    fun selectTier(tier: String) {
        val found = _state.value.tiers.firstOrNull { it.tier == tier }
        if (found != null && !found.available) return
        _state.value = _state.value.copy(selectedTier = tier)
    }

    fun send(rawPrompt: String) = sendPrompt(rawPrompt, "")

    fun sendWithAttachments(rawPrompt: String, attachments: List<ChatAttachment>) = sendPrompt(rawPrompt, "", attachments)

    fun sendVoice(rawPrompt: String, note: String) = sendPrompt(rawPrompt, note)

    private fun sendPrompt(rawPrompt: String, note: String, attachments: List<ChatAttachment> = emptyList()) {
        val prompt = rawPrompt.trim().ifEmpty { if (attachments.isNotEmpty()) "请查看附件。" else "" }
        if (prompt.isEmpty() || _state.value.sending) return
        validateChatAttachments(attachments)
        val userTurn = ChatTurn(id = nextId++, role = "user", text = prompt, attachments = attachments)
        val replyId = nextId++
        val assistantTurn = ChatTurn(id = replyId, role = "assistant", text = "", streaming = true)
        val history = buildList {
            if (systemPrompt.isNotBlank()) add(ChatMessage("system", systemPrompt))
            for (turn in _state.value.messages) {
                if (turn.error == null && turn.text.isNotBlank()) {
                    add(ChatMessage(turn.role, turn.text, turn.attachments))
                }
            }
            add(ChatMessage("user", note + prompt, attachments))
        }
        val tier = _state.value.selectedTier.ifEmpty { null }
        _state.value = _state.value.copy(
            messages = _state.value.messages + userTurn + assistantTurn,
            sending = true,
            approval = null,
            question = null,
        )
        persist()
        if (agent != null) {
            agentReplyId = replyId
            val session = agentSession
            chatJob = scope.launch {
                try {
                    agent.ensureStarted(tier ?: "")
                    kotlinx.coroutines.currentCoroutineContext().ensureActive()
                    // 先记下这一轮的编号再发出去，免得回来的事件比赋值还早。
                    val runId = agent.nextRunId()
                    agentRun = runId
                    agent.run(session, note + prompt, runId, attachments)
                } catch (e: CancellationException) {
                    throw e
                } catch (e: Exception) {
                    updateTurn(replyId) { it.copy(streaming = false, error = "智能体启动失败：${e.message ?: "未知原因"}") }
                    agentRun = null
                    _state.value = _state.value.copy(sending = false, approval = null, question = null)
                    persist()
                }
            }
            return
        }
        chatJob = scope.launch {
            try {
                api.streamChat(history, tier) { delta ->
                    updateTurn(replyId) { it.copy(text = it.text + delta) }
                }
                updateTurn(replyId) { it.copy(streaming = false) }
                _state.value = _state.value.copy(sending = false)
                persist()
                refreshBalanceQuietly()
            } catch (e: CancellationException) {
                updateTurn(replyId) { it.copy(streaming = false) }
                _state.value = _state.value.copy(sending = false)
                throw e
            } catch (e: Exception) {
                updateTurn(replyId) { it.copy(streaming = false, error = messageOf(e)) }
                _state.value = _state.value.copy(sending = false)
                persist()
                if (e is ApiException && (e.code == "insufficient_quota" || e.code == "paid_required")) {
                    refreshBalanceQuietly()
                }
            }
        }
    }

    /** 上一条回复失败时，原样重试最后一条用户提问。 */
    fun retryLast() {
        val last = _state.value.messages.lastOrNull() ?: return
        if (last.role != "assistant" || last.error == null) return
        regenerate()
    }

    /** 丢掉最后一条回复，把最后一条用户提问再问一遍。 */
    fun regenerate() {
        val current = _state.value
        if (current.sending) return
        val index = current.messages.indexOfLast { it.role == "user" }
        if (index < 0) return
        val turn = current.messages[index]
        _state.value = current.copy(messages = current.messages.take(index))
        sendWithAttachments(turn.text, turn.attachments)
    }

    /** 停止正在生成的流式回复，保留已输出的部分文字。 */
    fun stop() {
        val wasSending = _state.value.sending
        chatJob?.cancel()
        chatJob = null
        val run = agentRun
        agentRun = null
        run?.let { agent?.cancel(it) }
        val updated = _state.value.messages.mapNotNull { turn ->
            if (!turn.streaming) turn
            else if (turn.text.isEmpty() && turn.error == null) null
            else turn.copy(streaming = false, tools = settleTools(turn.tools, false))
        }
        _state.value = _state.value.copy(messages = updated, sending = false, approval = null, question = null)
        if (wasSending) persist()
    }

    fun clearConversation() {
        stop()
        agentSession = newSessionId()
        agentRun = null
        _state.value = _state.value.copy(messages = emptyList(), approval = null, question = null)
    }

    /** 回答智能体的确认：`once` 允许这一次，`deny` 拒绝。 */
    fun answerApproval(decision: String) {
        val pending = _state.value.approval ?: return
        _state.value = _state.value.copy(approval = null, question = null)
        agent?.answerApproval(pending.id, decision)
        val record = ChatRecord("approval", toolLabel(pending.tool), if (decision == "deny") null else "已允许")
        updateTurn(agentReplyId) { it.copy(records = it.records + record) }
        persist()
    }

    fun answerQuestion(answer: String?) {
        val pending = _state.value.question ?: return
        agent?.answerQuestion(pending.id, answer)
        _state.value = _state.value.copy(question = null)
        updateTurn(agentReplyId) { it.copy(records = it.records + ChatRecord("answer", pending.question, answer)) }
        persist()
    }

    @OptIn(ExperimentalTime::class)
    private fun nowMs(): Long = Clock.System.now().toEpochMilliseconds()

    /** 一轮结束时还没有结果的工具调用，按这一轮成败收尾，免得界面一直转圈。 */
    private fun settleTools(tools: List<ToolCallInfo>, failed: Boolean): List<ToolCallInfo> =
        tools.map { if (it.status == ToolStatus.Running) it.copy(status = if (failed) ToolStatus.Error else ToolStatus.Done, endedAtMs = nowMs()) else it }

    private fun onAgentEvent(event: AgentEvent) {
        when (event) {
            is AgentEvent.Text -> if (event.run == agentRun) updateTurn(agentReplyId) { it.copy(text = it.text + event.text) }
            is AgentEvent.ToolStart -> if (event.run == agentRun) {
                val call = ToolCallInfo(event.id, event.name, event.arguments, startedAtMs = nowMs())
                updateTurn(agentReplyId) { it.copy(tools = it.tools + call) }
            }
            is AgentEvent.ToolEnd -> if (event.run == agentRun) {
                updateTurn(agentReplyId) { turn ->
                    turn.copy(tools = turn.tools.map {
                        if (it.id == event.id) it.copy(output = event.output, status = if (event.isError) ToolStatus.Error else ToolStatus.Done, endedAtMs = nowMs()) else it
                    })
                }
            }
            is AgentEvent.Approval -> if (event.run == agentRun && _state.value.sending) _state.value = _state.value.copy(
                approval = ChatApproval(event.id, event.tool, summarizeToolArguments(event.tool, event.arguments.toString()), event.danger),
            )
            is AgentEvent.Question -> if (event.run == agentRun && event.session == agentSession && _state.value.sending) {
                _state.value = _state.value.copy(question = ChatQuestion(event.id, event.question, event.options))
            }
            is AgentEvent.RunEnd -> if (event.run == agentRun) {
                val failure = if (event.ok) null else friendlyAgentError(event.error)
                updateTurn(agentReplyId) { turn ->
                    // 只调工具、没有说话的一轮，回复里留空；失败时把原因挂在这条回复上。
                    turn.copy(streaming = false, error = failure, text = if (turn.text.isEmpty() && event.ok) event.text.orEmpty() else turn.text, tools = settleTools(turn.tools, !event.ok))
                }
                _state.value = _state.value.copy(sending = false, approval = null, question = null)
                agentRun = null
                persist()
                if (!event.ok && (failure.orEmpty().contains("额度") || failure.orEmpty().contains("兑换"))) refreshBalanceQuietly()
                else if (event.ok) refreshBalanceQuietly()
            }
            is AgentEvent.Failure -> if (agentRun != null && _state.value.sending && (event.runs == null || agentRun in event.runs)) {
                updateTurn(agentReplyId) { it.copy(streaming = false, error = event.message, tools = settleTools(it.tools, true)) }
                agentRun = null
                _state.value = _state.value.copy(sending = false, approval = null, question = null)
                persist()
            }
        }
    }

    fun clearRedeemFeedback() {
        _state.value = _state.value.copy(redeemMessage = null, redeemError = null)
    }

    fun redeem(rawCode: String) {
        val code = rawCode.trim()
        if (code.isEmpty() || _state.value.redeeming) return
        _state.value = _state.value.copy(redeeming = true, redeemError = null, redeemMessage = null)
        scope.launch {
            try {
                val done = api.redeem(code)
                val unit = _state.value.balance?.quotaPerUnit ?: 500_000
                _state.value = _state.value.copy(
                    redeeming = false,
                    redeemMessage = "兑换成功，到账 ${formatAiMoney(done.quota, unit)}",
                )
                loadMeta()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                _state.value = _state.value.copy(redeeming = false, redeemError = messageOf(e))
            }
        }
    }

    fun reset() {
        metaJob?.cancel()
        chatJob?.cancel()
        agentRun?.let { agent?.cancel(it) }
        agentRun = null
        _state.value = CloudChatState()
        agentSession = newSessionId()
    }

    private fun updateTurn(id: Long, transform: (ChatTurn) -> ChatTurn) {
        _state.value = _state.value.copy(
            messages = _state.value.messages.map { if (it.id == id) transform(it) else it },
        )
    }

    private fun refreshBalanceQuietly() {
        scope.launch {
            runCatching { api.balance() }.onSuccess { b ->
                _state.value = _state.value.copy(balance = b)
            }
        }
    }
}
