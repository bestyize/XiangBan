package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.plugins.timeout
import io.ktor.client.request.header
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
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
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
                            put("content", m.content)
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

data class ChatTurn(
    val id: Long,
    val role: String,
    val text: String,
    val streaming: Boolean = false,
    val error: String? = null,
)

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
) {
    val activeTier: AiTier? get() = tiers.firstOrNull { it.tier == selectedTier }
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
) {
    private val _state = MutableStateFlow(CloudChatState())
    val state: StateFlow<CloudChatState> = _state.asStateFlow()
    private var nextId = 1L
    private var metaJob: Job? = null
    private var chatJob: Job? = null

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

    fun send(rawPrompt: String) {
        val prompt = rawPrompt.trim()
        if (prompt.isEmpty() || _state.value.sending) return
        val userTurn = ChatTurn(id = nextId++, role = "user", text = prompt)
        val replyId = nextId++
        val assistantTurn = ChatTurn(id = replyId, role = "assistant", text = "", streaming = true)
        val history = buildList {
            if (systemPrompt.isNotBlank()) add(ChatMessage("system", systemPrompt))
            for (turn in _state.value.messages) {
                if (turn.error == null && turn.text.isNotBlank()) {
                    add(ChatMessage(turn.role, turn.text))
                }
            }
            add(ChatMessage("user", prompt))
        }
        val tier = _state.value.selectedTier.ifEmpty { null }
        _state.value = _state.value.copy(
            messages = _state.value.messages + userTurn + assistantTurn,
            sending = true,
        )
        chatJob = scope.launch {
            try {
                api.streamChat(history, tier) { delta ->
                    updateTurn(replyId) { it.copy(text = it.text + delta) }
                }
                updateTurn(replyId) { it.copy(streaming = false) }
                _state.value = _state.value.copy(sending = false)
                refreshBalanceQuietly()
            } catch (e: CancellationException) {
                updateTurn(replyId) { it.copy(streaming = false) }
                _state.value = _state.value.copy(sending = false)
                throw e
            } catch (e: Exception) {
                updateTurn(replyId) { it.copy(streaming = false, error = messageOf(e)) }
                _state.value = _state.value.copy(sending = false)
                if (e is ApiException && (e.code == "insufficient_quota" || e.code == "paid_required")) {
                    refreshBalanceQuietly()
                }
            }
        }
    }

    /** 上一条回复失败时，原样重试最后一条用户提问。 */
    fun retryLast() {
        val current = _state.value
        if (current.sending) return
        val last = current.messages.lastOrNull() ?: return
        if (last.role != "assistant" || last.error == null) return
        val withoutFailed = current.messages.dropLast(1)
        val lastUser = withoutFailed.lastOrNull { it.role == "user" } ?: return
        val trimmed = withoutFailed.dropLastWhile { it.id != lastUser.id }.dropLast(1)
        _state.value = current.copy(messages = trimmed)
        send(lastUser.text)
    }

    /** 停止正在生成的流式回复，保留已输出的部分文字。 */
    fun stop() {
        chatJob?.cancel()
        chatJob = null
        val updated = _state.value.messages.mapNotNull { turn ->
            if (!turn.streaming) turn
            else if (turn.text.isEmpty() && turn.error == null) null
            else turn.copy(streaming = false)
        }
        _state.value = _state.value.copy(messages = updated, sending = false)
    }

    fun clearConversation() {
        stop()
        _state.value = _state.value.copy(messages = emptyList())
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
        _state.value = CloudChatState()
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
