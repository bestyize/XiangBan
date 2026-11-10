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
import io.ktor.utils.io.readAvailable
import io.ktor.utils.io.readUTF8Line
import io.ktor.client.request.prepareGet
import io.ktor.http.Url
import kotlin.io.encoding.Base64
import kotlinx.io.Buffer
import kotlinx.io.readByteArray
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import kotlin.uuid.Uuid
import kotlin.uuid.ExperimentalUuidApi
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.delay
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
    @SerialName("context_window_tokens") val contextWindowTokens: Int? = null,
    @SerialName("supports_tools") val supportsTools: Boolean? = null,
    @SerialName("supports_vision") val supportsVision: Boolean? = null,
    @SerialName("max_output_tokens") val maxOutputTokens: Int? = null,
)

@Serializable
data class AiTiersResponse(
    val tiers: List<AiTier> = emptyList(),
    val media: Map<String, AiMediaCapability> = emptyMap(),
)

@Serializable
data class AiMediaCapability(
    val available: Boolean = false,
    val sizes: List<String> = emptyList(),
    val formats: List<String> = listOf("png"),
    @SerialName("supports_jobs") val supportsJobs: Boolean = false,
)

@Serializable
data class ImageJobReceipt(val id: String, val count: Int, val format: String)

@Serializable
private data class ImageJobError(val code: String? = null, val message: String)

@Serializable
private data class ImageJobResponse(
    @SerialName("job_id") val id: String,
    val status: String,
    val result: ImageGenerationResponse? = null,
    val error: ImageJobError? = null,
)

internal data class ImageJobResult(
    val images: List<String> = emptyList(),
    val error: String? = null,
    val canRecover: Boolean = false,
)

@Serializable
private data class ImageGenerationResponse(val data: List<ImageGenerationItem> = emptyList())

@Serializable
private data class ImageGenerationItem(
    @SerialName("b64_json") val base64: String? = null,
    val url: String? = null,
)

private const val MAX_GENERATED_IMAGE_BYTES = 16 * 1024 * 1024
internal const val UNKNOWN_IMAGE_RESULT = "生图结果尚未确认，可能已扣除额度，请勿直接重复生成。"
internal const val RECOVER_IMAGE_RESULT = "可查回原生图任务的结果。"

private fun generatedImageMime(bytes: ByteArray): String? = when {
    bytes.size >= 8 && bytes.copyOf(8).contentEquals(byteArrayOf(-119, 80, 78, 71, 13, 10, 26, 10)) -> "image/png"
    bytes.size >= 3 && bytes[0] == (-1).toByte() && bytes[1] == (-40).toByte() && bytes[2] == (-1).toByte() -> "image/jpeg"
    bytes.size >= 12 && bytes.decodeToString(0, 4) == "RIFF" && bytes.decodeToString(8, 12) == "WEBP" -> "image/webp"
    validGeneratedGif(bytes) -> "image/gif"
    else -> null
}

private suspend fun io.ktor.client.statement.HttpResponse.readBoundedBytes(limit: Int): ByteArray {
    val channel = bodyAsChannel()
    val buffer = Buffer()
    val chunk = ByteArray(64 * 1024)
    while (true) {
        val count = channel.readAvailable(chunk, 0, chunk.size)
        if (count < 0) return buffer.readByteArray()
        check(buffer.size + count <= limit) { "响应过大" }
        buffer.write(chunk, 0, count)
    }
}

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
    "trial" -> "试用点数"
    "monthly" -> "月卡"
    "topup" -> "加油包"
    else -> "点数"
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
class AiAccountRequest internal constructor(
    internal val identity: SessionIdentity?,
    private val current: () -> Boolean,
) : kotlin.coroutines.AbstractCoroutineContextElement(Key) {
    companion object Key : kotlin.coroutines.CoroutineContext.Key<AiAccountRequest>
    fun isCurrent(): Boolean = current()
    fun checkCurrent() {
        if (!isCurrent()) throw ApiException(401, "账号已切换，请重新发起请求", "account_changed")
    }
}

class AiGatewayApi(
    private val client: HttpClient,
    config: CommunityConfig = CommunityConfig(),
    private val sessions: SessionManager? = null,
    private val tokenProvider: suspend () -> String?,
) {
    private val base = config.aiBase.trimEnd('/')
    private val http = JsonHttp(client, base)

    internal fun bindAccount(account: String?, current: () -> Boolean): AiAccountRequest {
        val owner = sessions?.identity()?.takeIf { it.accountId == account }
        return AiAccountRequest(owner) { current() && (sessions == null || owner != null && sessions.identity() == owner) }
    }

    private suspend fun requireToken(): String {
        val request = kotlinx.coroutines.currentCoroutineContext()[AiAccountRequest]
        request?.checkCurrent()
        val owner = request?.identity ?: sessions?.identity()
        val token = if (sessions != null && owner != null) sessions.accessToken(owner) else tokenProvider()
        request?.checkCurrent()
        return token ?: throw ApiException(401, "请先登录小伴账号", "unauthorized")
    }

    suspend fun tiers(): List<AiTier> = capabilities().tiers

    suspend fun capabilities(): AiTiersResponse =
        http.send(HttpMethod.Get, "/api/ai/tiers", AiTiersResponse.serializer(), token = requireToken())

    internal fun imageParameters(prompt: String, count: Int, size: String?, format: String, frames: Int, frameDurationMs: Int): JsonObject {
        require(prompt.isNotBlank() && prompt.length <= 4000) { "请输入 1–4000 字的图片描述" }
        require(count in 1..4)
        require(format in listOf("png", "gif")) { "图片格式无效" }
        if (format == "gif") {
            require(count == 1 && frames in 2..16 && frameDurationMs in 20..2000) { "GIF 参数无效" }
        }
        return buildJsonObject {
            put("prompt", prompt)
            put("n", count)
            size?.let { put("size", it) }
            if (format == "gif") {
                put("output_format", "gif")
                put("frame_count", frames)
                put("frame_duration_ms", frameDurationMs)
            }
        }
    }

    /** 兼容旧网关的同步接口。连接中断后不重发付费请求。 */
    suspend fun generateImages(prompt: String, count: Int, size: String?, format: String = "png", frames: Int = 9, frameDurationMs: Int = 200): List<String> {
        val payload = imageParameters(prompt, count, size, format, frames, frameDurationMs)
        val token = requireToken()
        try {
            val items = client.preparePost("$base/v1/images/generations") {
                header(HttpHeaders.Authorization, "Bearer $token")
                contentType(ContentType.Application.Json)
                setBody(payload.toString())
                // 出图要等 20 秒左右才有第一个字节；不单独设读超时时，Android 的 OkHttp 默认 10 秒就会断开，图已生成并扣费却拿不到。
                timeout {
                    requestTimeoutMillis = 240_000
                    socketTimeoutMillis = 240_000
                }
            }.execute { response ->
                if (!response.status.isSuccess()) {
                    val error = parseApiError(response.status.value, response.readBoundedBytes(64 * 1024).decodeToString())
                    throw when (error.code) {
                        "image_unavailable" -> ApiException(error.status, "小伴 AI 暂未开放生图", error.code)
                        "paid_required" -> ApiException(error.status, "图片生成需要先兑换付费额度", error.code)
                        "insufficient_quota" -> ApiException(error.status, "小伴 AI 额度不足，请兑换后再生成", error.code)
                        else -> if (error.status >= 500) ApiException(error.status, UNKNOWN_IMAGE_RESULT) else error
                    }
                }
                val body = response.readBoundedBytes(48 * 1024 * 1024)
                withContext(Dispatchers.Default) { CommunityJson.decodeFromString<ImageGenerationResponse>(body.decodeToString()).data }
            }
            val images = decodeImages(items, count, format)
            check(images.isNotEmpty())
            return images
        } catch (e: CancellationException) {
            throw e
        } catch (e: ApiException) {
            throw e
        } catch (_: Exception) {
            throw ApiException(0, UNKNOWN_IMAGE_RESULT)
        }
    }

    internal suspend fun awaitImageJob(receipt: ImageJobReceipt, parameters: JsonObject? = null): ImageJobResult {
        val job = withTimeoutOrNull(27 * 60_000L) {
            var status = requestImageJob(receipt.id, parameters)
            while (status.status == "queued" || status.status == "running") {
                delay(2_000)
                status = requestImageJob(receipt.id, null)
            }
            status
        } ?: return ImageJobResult(error = "等待生图超时。$RECOVER_IMAGE_RESULT", canRecover = true)

        val items = job.result?.data.orEmpty().take(receipt.count)
        val images = decodeImages(items, receipt.count, receipt.format)
        val missing = images.size != items.size
        return when (job.status) {
            "succeeded", "partial" -> ImageJobResult(images,
                error = if (missing || images.isEmpty()) "部分生图结果未下载成功。$RECOVER_IMAGE_RESULT" else null,
                canRecover = missing || images.isEmpty())
            "failed" -> ImageJobResult(images, job.error?.message ?: "生图任务失败")
            "unknown" -> ImageJobResult(images, job.error?.message ?: UNKNOWN_IMAGE_RESULT)
            "expired" -> ImageJobResult(error = "生图结果已超过 7 天保留期")
            else -> ImageJobResult(images, "无法识别生图任务状态。$RECOVER_IMAGE_RESULT", canRecover = true)
        }
    }

    private suspend fun requestImageJob(id: String, parameters: JsonObject?): ImageJobResponse {
        val token = requireToken()
        val statement = if (parameters != null) client.preparePost("$base/v1/images/jobs") {
            header(HttpHeaders.Authorization, "Bearer $token")
            contentType(ContentType.Application.Json)
            setBody(buildJsonObject { put("job_id", id); put("parameters", parameters) }.toString())
        } else client.prepareGet("$base/v1/images/jobs/$id") {
            header(HttpHeaders.Authorization, "Bearer $token")
            timeout { requestTimeoutMillis = 60_000; socketTimeoutMillis = 60_000 }
        }
        return statement.execute { response ->
            if (!response.status.isSuccess()) throw parseApiError(response.status.value, response.readBoundedBytes(64 * 1024).decodeToString())
            val body = response.readBoundedBytes(48 * 1024 * 1024 + 16 * 1024)
            withContext(Dispatchers.Default) {
                CommunityJson.decodeFromString<ImageJobResponse>(body.decodeToString()).also {
                    check(it.id == id) { "生图任务编号不匹配" }
                }
            }
        }
    }

    private suspend fun decodeImages(items: List<ImageGenerationItem>, count: Int, format: String): List<String> {
        val images = mutableListOf<String>()
        for (item in items.take(count)) {
            try {
                val bytes = item.base64?.let { encoded ->
                    check(encoded.length <= ((MAX_GENERATED_IMAGE_BYTES + 2) / 3) * 4)
                    withContext(Dispatchers.Default) { Base64.decode(encoded) }
                } ?: downloadGeneratedImage(checkNotNull(item.url))
                check(bytes.size <= MAX_GENERATED_IMAGE_BYTES)
                val mime = checkNotNull(generatedImageMime(bytes))
                check(format != "gif" || mime == "image/gif")
                images += "data:$mime;base64," + (item.base64 ?: Base64.encode(bytes))
            } catch (e: CancellationException) {
                throw e
            } catch (_: Exception) {
                // Preserve usable images if a later result cannot be loaded.
            }
        }
        return images
    }

    private suspend fun downloadGeneratedImage(raw: String): ByteArray {
        val url = Url(raw)
        check(url.protocol.name == "https" && url.host.isNotBlank() && url.user == null && url.password == null)
        // Download without the AI gateway bearer token.
        return client.prepareGet(raw).execute { response ->
            check(response.status.isSuccess())
            response.readBoundedBytes(MAX_GENERATED_IMAGE_BYTES)
        }
    }

    suspend fun balance(): AiBalance =
        http.send(HttpMethod.Get, "/api/ai/balance", AiBalance.serializer(), token = requireToken())

    internal suspend fun webSearch(query: String, count: Int): WebSearchReply =
        webRequest("/v1/search", buildJsonObject {
            put("query", query)
            put("count", count)
            put("need_content", false)
        }, WebSearchReply.serializer())

    internal suspend fun webFetch(url: String): WebFetchReply =
        webRequest("/v1/fetch", buildJsonObject { put("url", url) }, WebFetchReply.serializer())

    private suspend fun <T> webRequest(
        path: String, payload: JsonObject, serializer: kotlinx.serialization.DeserializationStrategy<T>,
    ): T {
        val token = requireToken()
        try {
            return client.preparePost("$base$path") {
                header(HttpHeaders.Authorization, "Bearer $token")
                contentType(ContentType.Application.Json)
                setBody(payload.toString())
                timeout { requestTimeoutMillis = 60_000 }
            }.execute { response ->
                val body = response.readBoundedBytes(1024 * 1024).decodeToString()
                if (!response.status.isSuccess()) throw parseApiError(response.status.value, body)
                CommunityJson.decodeFromString(serializer, body)
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: ApiException) {
            throw e
        } catch (_: Exception) {
            throw ApiException(0, "联网请求失败或返回了无效结果")
        }
    }

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
        if (body.encodeToByteArray().size > CHAT_BODY_BYTES) return ModelResult(413, body = "{\"error\":{\"message\":\"request_too_large: 当前请求超过 8 MiB，请减少附件或缩短文本\"}}")
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
                    connectTimeoutMillis = 15_000
                    requestTimeoutMillis = Long.MAX_VALUE
                    socketTimeoutMillis = 90_000
                }
            }.execute { response ->
                val status = response.status.value
                if (status !in 200..299) return@execute ModelResult(status, body = response.bodyAsText())
                val verified = response.headers["x-xiaoban-stream-terminal-version"] == "1"
                if (response.headers[HttpHeaders.ContentType]?.startsWith("application/json") == true) {
                    return@execute ModelResult(status, body = response.readBoundedBytes(CHAT_BODY_BYTES).decodeToString(), verifiedToolTerminal = verified)
                }
                val channel = response.bodyAsChannel()
                while (!channel.isClosedForRead) {
                    val line = channel.readUTF8Line() ?: break
                    onLine(line)
                    if (line.startsWith("data:") && line.removePrefix("data:").trim() == "[DONE]") break
                }
                ModelResult(status, verifiedToolTerminal = verified)
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
        reasoningEffort: String? = null,
        contextTokens: Int = 256_000,
        onDelta: (String) -> Unit = {},
    ): String {
        val token = requireToken()
        val payload = boundedChatBody(messages, tier, reasoningEffort, contextTokens)

        val statement = try {
            client.preparePost("$base/v1/chat/completions") {
                header(HttpHeaders.Authorization, "Bearer $token")
                header(HttpHeaders.Accept, "text/event-stream")
                contentType(ContentType.Application.Json)
                setBody(payload)
                timeout {
                    connectTimeoutMillis = 15_000
                    requestTimeoutMillis = Long.MAX_VALUE
                    socketTimeoutMillis = 90_000
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
                val stream = ChatTextStream { piece -> full.append(piece); onDelta(piece) }
                val channel = response.bodyAsChannel()
                while (!channel.isClosedForRead) {
                    val line = channel.readUTF8Line() ?: break
                    if (stream.line(line)) break
                }
                stream.finish()
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
data class ModelResult(val status: Int, val body: String? = null, val error: String? = null, val verifiedToolTerminal: Boolean = true)

/**
 * 把已经组好的 chat-completions 请求体（流式）发给 server-ai，响应的每一行原样交给 [onLine]。
 * Agent 内核只负责拼请求和解析流，网络请求（地址、令牌、超时）都在这里，所以手机里不用带 TLS 库。
 */
suspend fun AiGatewayApi.postChatLines(body: String, onLine: suspend (String) -> Unit): ModelResult = postChatRaw(body, onLine)

private fun JsonElement?.asStr(): String? =
    this?.takeIf { it !is JsonNull }?.let { runCatching { it.jsonPrimitive.contentOrNull }.getOrNull() }

private class ChatTextStream(private val onText: (String) -> Unit) {
    private var finishReason: String? = null
    private var done = false

    fun line(line: String): Boolean {
        if (!line.startsWith("data:")) return false
        val data = line.removePrefix("data:").trim()
        if (data.isEmpty()) return false
        if (data == "[DONE]") { done = true; return true }
        val obj = runCatching { CommunityJson.parseToJsonElement(data) as? JsonObject }.getOrNull()
            ?: throw ApiException(502, "模型返回了无法识别的流式内容", "invalid_stream")
        if (obj["error"] is JsonObject) throw parseApiError(502, data)
        val choice = (obj["choices"] as? JsonArray)?.firstOrNull() as? JsonObject
        choice?.get("finish_reason").asStr()?.let { finishReason = it }
        val content = (choice?.get("delta") as? JsonObject)?.get("content").asStr()
            ?: (choice?.get("message") as? JsonObject)?.get("content").asStr()
            ?: obj["delta"].asStr()
        if (!content.isNullOrEmpty()) onText(content)
        return false
    }

    fun finish() {
        if (finishReason != "stop" && !(finishReason == null && done)) {
            throw ApiException(502, "模型回复未完整结束，可重试", "stream_interrupted")
        }
    }
}

private fun extractSingleReply(body: String): String {
    if (body.lineSequence().any { it.startsWith("data:") }) {
        val text = StringBuilder()
        val stream = ChatTextStream { text.append(it) }
        for (line in body.lineSequence()) if (stream.line(line)) break
        stream.finish()
        return text.toString()
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
    val reasoning: String = "",
    val activity: String = "",
    val promptTokens: Long = 0,
    val completionTokens: Long = 0,
    val truncated: Boolean = false,
    val nativeNotStarted: Boolean = false,
    val cancelled: Boolean = false,
    val imageGeneration: Boolean = false,
    val generatedImages: List<String> = emptyList(),
    val imageJob: ImageJobReceipt? = null,
    val imageJobRecoverable: Boolean = false,
)

/** 智能体要改动东西之前问用户：同意这一次，还是拒绝。 */
data class ChatApproval(
    val id: String,
    val tool: String,
    val detail: String = "",
    val danger: String? = null,
    val canRemember: Boolean = true,
)

data class ChatQuestion(val id: String, val question: String, val options: List<String>)

data class CloudChatState(
    val tiers: List<AiTier> = emptyList(),
    val selectedTier: String = "",
    val reasoningByTier: Map<String, String> = emptyMap(),
    val balance: AiBalance? = null,
    val contextTokens: Int = 256_000,
    val permissionMode: String = "ask",
    val grants: List<String> = emptyList(),
    val budget: AgentExecutionBudget = AgentExecutionBudget(),
    val saveError: String? = null,
    val nativeAgentAvailable: Boolean = false,
    val loadingMeta: Boolean = false,
    val metaError: String? = null,
    val inputError: String? = null,
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
    val workspaces: List<String> = emptyList(),
    val media: Map<String, AiMediaCapability> = emptyMap(),
) {
    val imageCapability: AiMediaCapability? get() = media["image"]
    val activeTier: AiTier? get() = tiers.firstOrNull { it.tier == selectedTier }
}

@OptIn(ExperimentalTime::class)
private fun systemNowMs(): Long = Clock.System.now().toEpochMilliseconds()

/** 内核报的错带着“HTTP 402: ”这样的前缀，给人看时去掉。 */
internal fun friendlyAgentError(raw: String?): String {
    val text = raw.orEmpty()
    return when {
        "tool_outcome_needs_review" in text || "Execution outcome unknown" in text -> "操作已中断，结果尚未确认。请先核对文件、提醒或作品的实际状态，再继续任务。"
        "resources_busy" in text -> "共享资源正在被其他任务使用，当前任务已暂停；等前一个操作结束后再继续。"
        "invalid_model_terminal_or_call_identity" in text -> "模型返回了无效的工具调用，本轮已暂停，未执行这些无效调用。"
        "output_length_limit" in text -> "模型连续输出不完整，已暂停；可缩小单次任务或调整模型后继续。"
        "no_progress" in text -> "任务连续没有取得进展，已暂停。可以补充信息后继续。"
        "BudgetExceeded" in text || "budget exceeded" in text -> "已达到本次任务预算，可调整预算后继续。"
        "turn limit" in text -> "已达到你设置的任务轮数预算，可调整后继续。"
        "cannot retry" in text || "retry requires" in text || "unanswered user message" in text || "cannot be replayed" in text -> "上一轮已有执行记录，请继续任务，不要重新执行原请求。"
        else -> text.replace(Regex("^HTTP \\d+: "), "").ifBlank { "智能体执行出错" }
    }
}

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
    private var openJob: Job? = null
    private var checkpointJob: Job? = null
    private var sendEpoch = 0L
    @kotlin.concurrent.Volatile private var accountGeneration = 0L
    private var accountId: String? = null
    private var balanceJob: Job? = null
    private var redeemJob: Job? = null
    private var runAuthorization: AiAccountRequest? = null

    private fun accountRequest(): AiAccountRequest {
        val generation = accountGeneration
        return api.bindAccount(accountId) { generation == accountGeneration }
    }
    private var agentSession = newSessionId()
        set(value) { field = value; _state.value = _state.value.copy(conversationId = value) }
    private var agentRun: String? = null
    private var agentReplyId: Long = 0
    private val stoppingRuns = mutableMapOf<String, Pair<String, Long>>()

    init {
        _state.value = _state.value.copy(conversationId = agentSession, nativeAgentAvailable = agent != null)
        history?.let { store -> scope.launch { store.saveError.collect { error -> _state.value = _state.value.copy(saveError = error) } } }
        agent?.let { runtime -> scope.launch { runtime.events.collect(::onAgentEvent) } }
    }

    /** 换到某个账号（登录后、换号后调用）：读出这个账号存下的对话列表，当前对话从头开始。 */
    fun useAccount(account: String?) {
        ++accountGeneration
        metaJob?.cancel()
        balanceJob?.cancel()
        redeemJob?.cancel()
        // Stop and save under the OLD account before changing the storage namespace.
        clearConversation()
        stoppingRuns.clear()
        accountId = account
        runAuthorization = null
        val list = history?.useAccount(account) ?: emptyList()
        _state.value = _state.value.copy(tiers = emptyList(), media = emptyMap(), balance = null, selectedTier = "", reasoningByTier = emptyMap(), loadingMeta = false, metaError = null, redeeming = false, redeemMessage = null, redeemError = null, conversations = list, workspaces = history?.workspaces().orEmpty())
    }

    /** 打开存下来的某个对话，接着聊；智能体内核按同一个编号接上它记着的上下文。 */
    fun openConversation(id: String) {
        openJob?.cancel()
        if (id == agentSession && _state.value.messages.isNotEmpty()) return
        val store = history ?: return
        if (store.backgroundReads) {
            openJob = scope.launch {
                val saved = store.loadInBackground(id)
                openJob = null
                showConversation(id, saved)
            }
        } else showConversation(id, store.loadSnapshot(id))
    }

    private fun showConversation(id: String, saved: StoredChat?) {
        if (saved == null) {
            _state.value = _state.value.copy(conversations = history?.list().orEmpty())
            return
        }
        stop()
        agentSession = id
        agentRun = null
        nextId = (saved.messages.maxOfOrNull { it.id } ?: 0L) + 1
        applyOptions(saved.options)
        _state.value = _state.value.copy(messages = saved.messages, approval = null, question = null, inputError = modelInputError(emptyList()))
    }

    suspend fun resolveImage(conversation: String, value: String): String? {
        if (conversation != agentSession) return null
        val generation = accountGeneration
        val source = try {
            if (isChatMediaReference(value)) history?.mediaFor(conversation)?.resolve(value)
            else value.takeIf { it.startsWith("data:image/") }
        } catch (e: CancellationException) { throw e }
        catch (_: Exception) { null }
        return source?.takeIf { conversation == agentSession && generation == accountGeneration }
    }

    /** 删掉一个存下来的对话；删的是正在看的就换成新对话。 */
    fun addWorkspace(name: String) { history?.addWorkspace(name); refreshWorkspaces() }
    fun renameWorkspace(old: String, name: String) { history?.renameWorkspace(old, name); refreshWorkspaces() }
    fun deleteWorkspace(name: String) { history?.deleteWorkspace(name); refreshWorkspaces() }
    fun moveConversation(id: String, workspace: String?) { history?.move(id, workspace); refreshWorkspaces() }
    private fun refreshWorkspaces() {
        _state.value = _state.value.copy(conversations = history?.list().orEmpty(), workspaces = history?.workspaces().orEmpty())
    }

    fun deleteConversation(id: String) {
        val store = history ?: return
        val remaining = store.delete(id)
        _state.value = _state.value.copy(conversations = remaining)
        if (store.isDeleted(id) && id == agentSession) clearConversation()
    }

    /** 把当前对话存一下（发出提问、回答结束、停止时）。 */
    private fun persist() {
        val list = history?.save(agentSession, _state.value.messages, clock(), options()) ?: return
        _state.value = _state.value.copy(conversations = list, workspaces = history?.workspaces().orEmpty())
    }

    private fun options(): ChatOptions = _state.value.let {
        ChatOptions(it.selectedTier, it.reasoningByTier, it.contextTokens, it.permissionMode, it.grants, it.budget)
    }

    private fun applyOptions(value: ChatOptions) {
        _state.value = _state.value.copy(selectedTier = value.tier, reasoningByTier = value.reasoningByTier,
            contextTokens = value.contextTokens, permissionMode = value.permissionMode, grants = value.grants, budget = value.budget)
    }

    fun setPermission(mode: String) {
        require(mode in setOf("ask", "auto"))
        val next = options().copy(permissionMode = mode)
        agent?.configureSession(agentSession, next, _state.value.activeTier)
        applyOptions(next); persist()
    }

    fun revokeGrant(tool: String) {
        val next = options().copy(grants = _state.value.grants - tool)
        agent?.configureSession(agentSession, next, _state.value.activeTier)
        applyOptions(next); persist()
    }

    fun setExecution(contextTokens: Int, budget: AgentExecutionBudget) {
        check(!_state.value.sending) { "请先停止当前任务再修改预算" }
        val next = options().copy(contextTokens = contextTokens, budget = budget)
        next.validate(); applyOptions(next); persist()
    }

    fun retrySave() { history?.retryPending() }

    fun acknowledgeOutcome() {
        check(!_state.value.sending)
        agent?.acknowledgeOutcome(agentSession)
        send("我已核对上一轮操作的实际结果。请先读取当前状态，再继续未完成部分，避免重复操作。")
    }

    /** 加载档位与余额，并按桌面端同一规则选定可用默认档位。 */
    fun loadMeta() {
        metaJob?.cancel()
        _state.value = _state.value.copy(loadingMeta = true, metaError = null)
        val request = accountRequest()
        metaJob = scope.launch(request) {
            try {
                coroutineScope {
                    val balanceDeferred = async { runCatching { api.balance() } }
                    val tiersDeferred = async { runCatching { api.capabilities() } }
                    val balanceRes = balanceDeferred.await()
                    val tiersRes = tiersDeferred.await()
                    if (!request.isCurrent()) return@coroutineScope
                    val current = _state.value
                    val nextTiers = tiersRes.getOrNull()?.tiers ?: current.tiers
                    val nextBalance = balanceRes.getOrNull() ?: current.balance
                    val err = balanceRes.exceptionOrNull() ?: tiersRes.exceptionOrNull()
                    if (err is CancellationException) throw err
                    _state.value = current.copy(
                        tiers = nextTiers,
                        media = tiersRes.getOrNull()?.media ?: current.media,
                        selectedTier = current.selectedTier.ifBlank { pickAiTier(nextTiers, "") },
                        balance = nextBalance,
                        loadingMeta = false,
                        metaError = if (balanceRes.isFailure && tiersRes.isFailure && err != null) messageOf(err) else null,
                    )
                    val inputError = modelInputError(emptyList())
                    if (inputError != null) _state.value = _state.value.copy(inputError = inputError)
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                if (request.isCurrent()) _state.value = _state.value.copy(loadingMeta = false, metaError = messageOf(e))
            }
        }
    }

    fun selectReasoning(tier: String, effort: String?) {
        val found = _state.value.tiers.firstOrNull { it.tier == tier } ?: return
        if (!found.available || _state.value.sending) return
        if (effort != null && effort !in found.reasoningEfforts) return
        val choices = _state.value.reasoningByTier - tier
        _state.value = _state.value.copy(selectedTier = tier, inputError = null,
            reasoningByTier = if (effort == null) choices else choices + (tier to effort))
        persist()
    }

    fun selectTier(tier: String) {
        if (_state.value.sending) return
        val found = _state.value.tiers.firstOrNull { it.tier == tier }
        if (found != null && !found.available) return
        _state.value = _state.value.copy(selectedTier = tier, inputError = null)
        persist()
    }

    fun send(rawPrompt: String) { sendPrompt(rawPrompt, "") }

    fun sendWithAttachments(rawPrompt: String, attachments: List<ChatAttachment>) = sendPrompt(rawPrompt, "", attachments)

    @OptIn(ExperimentalUuidApi::class)
    fun generateImages(rawPrompt: String, count: Int, size: String?, format: String = "png", frames: Int = 9, frameDurationMs: Int = 200): Boolean {
        val current = _state.value
        if (current.sending) return false
        val prompt = rawPrompt.trim()
        if (format == "gif") require(current.imageCapability?.formats?.contains("gif") == true) { "当前图片服务尚未开放 GIF" }
        val parameters = api.imageParameters(prompt, count, size, format, frames, frameDurationMs)
        val receipt = if (current.imageCapability?.supportsJobs != false) ImageJobReceipt(Uuid.random().toString(), count, format) else null
        val userTurn = ChatTurn(id = nextId++, role = "user", text = prompt, imageGeneration = true)
        val reply = ChatTurn(nextId++, "assistant", "", streaming = true, imageGeneration = true,
            activity = "正在生成图片…", imageJob = receipt, imageJobRecoverable = receipt != null)
        val owner = agentSession
        _state.value = current.copy(messages = current.messages + userTurn + reply)
        runImageWork(reply.id, count, receipt) {
            if (receipt == null) ImageJobResult(api.generateImages(prompt, count, size, format, frames, frameDurationMs))
            else if (history?.awaitSaved(owner) == false) ImageJobResult(error = "未能保存生图任务编号，请重试保存后重新提交。")
            else api.awaitImageJob(receipt, parameters)
        }
        return true
    }

    fun recoverImageJob(replyId: Long) {
        if (_state.value.sending) return
        val receipt = _state.value.messages.firstOrNull { it.id == replyId && it.imageJobRecoverable }?.imageJob ?: return
        updateTurn(replyId) { it.copy(streaming = true, error = null, activity = "正在查回图片…") }
        runImageWork(replyId, receipt.count, receipt) { api.awaitImageJob(receipt) }
    }

    private fun runImageWork(replyId: Long, count: Int, receipt: ImageJobReceipt?, work: suspend () -> ImageJobResult) {
        openJob?.cancel()
        val epoch = ++sendEpoch
        val ownerSession = agentSession
        val media = history?.mediaFor(ownerSession)
        val request = accountRequest()
        _state.value = _state.value.copy(sending = true)
        persist()
        chatJob = scope.launch(request) {
            fun stillHere() = request.isCurrent() && epoch == sendEpoch && ownerSession == agentSession
            try {
                if (!stillHere()) return@launch
                val result = work()
                if (!stillHere()) return@launch
                val images = media?.save(result.images) ?: result.images
                if (!stillHere()) return@launch
                updateTurn(replyId) {
                    val available = if (images.size >= it.generatedImages.size) images else it.generatedImages
                    val text = when (available.size) {
                        count -> "已生成 ${available.size} 张图片"
                        0 -> it.text
                        else -> "已收到 ${available.size} / $count 张图片。"
                    }
                    it.copy(text = text, streaming = false, generatedImages = available, error = result.error,
                        imageJobRecoverable = result.canRecover)
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                if (!stillHere()) return@launch
                val error = if (receipt == null) messageOf(e)
                    else if (e is ApiException && e.code == "image_job_not_found") "暂未查到这次生图任务。$RECOVER_IMAGE_RESULT"
                    else "${messageOf(e)} $RECOVER_IMAGE_RESULT"
                updateTurn(replyId) { it.copy(streaming = false, error = error) }
            }
            _state.value = _state.value.copy(sending = false)
            persist()
            refreshBalanceQuietly()
        }
    }

    fun sendVoice(rawPrompt: String, note: String) { sendPrompt(rawPrompt, note) }

    private fun modelInputError(attachments: List<ChatAttachment>): String? {
        val state = _state.value
        val tier = state.activeTier
        return when {
            state.selectedTier.isNotBlank() && state.tiers.isNotEmpty() && tier == null -> "历史模型已下架，请选择可用模型后发送。"
            tier?.available == false -> "当前档位不可用，请检查点数或选择其他模型。"
            tier?.supportsVision == false && attachments.any { it.imageDataUrl != null } ->
                "当前模型不支持图片，请选择支持图片的模型。"
            else -> null
        }
    }

    private fun sendPrompt(rawPrompt: String, note: String, attachments: List<ChatAttachment> = emptyList(), nativeRetry: Boolean = false, reuseUser: Boolean = false): Boolean {
        val prompt = rawPrompt.trim().ifEmpty { if (attachments.isNotEmpty()) "请查看附件。" else "" }
        if (prompt.isEmpty() || _state.value.sending) return false
        modelInputError(attachments)?.let { error ->
            _state.value = _state.value.copy(inputError = error)
            return false
        }
        validateChatAttachments(attachments, allowStoredImages = history != null)
        openJob?.cancel()
        val userTurn = ChatTurn(id = nextId++, role = "user", text = prompt, attachments = attachments)
        val replyId = nextId++
        val assistantTurn = ChatTurn(id = replyId, role = "assistant", text = "", streaming = true)
        val history = buildList {
            if (systemPrompt.isNotBlank()) add(ChatMessage("system", systemPrompt))
            for (turn in _state.value.messages) {
                if (turn.error == null && turn.text.isNotBlank()) {
                    val pastAttachments = if (_state.value.activeTier?.supportsVision == false) {
                        turn.attachments.filter { it.imageDataUrl == null }
                    } else turn.attachments
                    add(ChatMessage(turn.role, turn.text, pastAttachments))
                }
            }
            add(ChatMessage("user", note + prompt, attachments))
        }
        val epoch = ++sendEpoch
        val ownerSession = agentSession
        val media = this.history?.mediaFor(ownerSession)
        val request = accountRequest()
        val tier = _state.value.selectedTier.ifEmpty { null }
        val reasoning = _state.value.reasoningByTier[tier]?.takeIf { it in _state.value.activeTier?.reasoningEfforts.orEmpty() }
        _state.value = _state.value.copy(
            messages = _state.value.messages + (if (nativeRetry || reuseUser) emptyList() else listOf(userTurn)) + assistantTurn,
            sending = true,
            inputError = null,
            approval = null,
            question = null,
        )
        persist()
        if (agent != null) {
            val snapshot = options().copy(reasoningByTier = reasoning?.let { mapOf(tier.orEmpty() to it) }.orEmpty())
            val tierSnapshot = _state.value.activeTier
            agentReplyId = replyId
            val session = agentSession
            chatJob = scope.launch(request) {
                try {
                    val resolved = resolveInput(attachments, media)
                    agent.ensureStarted(tier ?: "", reasoning)
                    kotlinx.coroutines.currentCoroutineContext().ensureActive()
                    // 先记下这一轮的编号再发出去，免得回来的事件比赋值还早。
                    request.checkCurrent()
                    val runId = agent.nextRunId()
                    runAuthorization = request
                    agentRun = runId
                    agent.run(session, note + prompt, runId, resolved, snapshot, tierSnapshot, retry = nativeRetry, account = request)
                } catch (e: CancellationException) {
                    throw e
                } catch (e: Exception) {
                    if (!request.isCurrent() || epoch != sendEpoch || ownerSession != agentSession) return@launch
                    updateTurn(replyId) { it.copy(streaming = false, error = "智能体启动失败：${e.message ?: "未知原因"}", nativeNotStarted = true) }
                    agentRun = null
                    _state.value = _state.value.copy(sending = false, approval = null, question = null)
                    persist()
                }
            }
            return true
        }
        val contextTokens = _state.value.contextTokens
        val tierContext = _state.value.activeTier?.contextWindowTokens
        chatJob = scope.launch(request) {
            try {
                val messages = resolveHistory(history, media)
                api.streamChat(messages, tier, reasoning, tierContext?.let { minOf(it, contextTokens) } ?: contextTokens) { delta ->
                    if (request.isCurrent() && epoch == sendEpoch && ownerSession == agentSession) updateTurn(replyId) { it.copy(text = it.text + delta) }
                }
                if (!request.isCurrent() || epoch != sendEpoch || ownerSession != agentSession) return@launch
                updateTurn(replyId) { it.copy(streaming = false) }
                _state.value = _state.value.copy(sending = false)
                persist()
                refreshBalanceQuietly()
            } catch (e: CancellationException) {
                if (!request.isCurrent() || epoch != sendEpoch || ownerSession != agentSession) throw e
                updateTurn(replyId) { it.copy(streaming = false) }
                _state.value = _state.value.copy(sending = false)
                throw e
            } catch (e: Exception) {
                if (!request.isCurrent() || epoch != sendEpoch || ownerSession != agentSession) return@launch
                updateTurn(replyId) { it.copy(streaming = false, error = messageOf(e)) }
                _state.value = _state.value.copy(sending = false)
                persist()
                if (e is ApiException && (e.code == "insufficient_quota" || e.code == "paid_required")) {
                    refreshBalanceQuietly()
                }
            }
        }
        return true
    }

    private suspend fun resolveInput(attachments: List<ChatAttachment>, media: ChatMedia?): List<ChatAttachment> {
        val resolved = attachments.map { attachment ->
            val reference = attachment.imageDataUrl?.takeIf(::isChatMediaReference) ?: return@map attachment
            val image = media?.resolve(reference) ?: throw ApiException(0, "找不到引用的图片，请重新选择。")
            attachment.copy(imageDataUrl = image)
        }
        if (resolved != attachments) withContext(Dispatchers.Default) { validateChatAttachments(resolved) }
        return resolved
    }

    private suspend fun resolveHistory(messages: List<ChatMessage>, media: ChatMedia?): List<ChatMessage> {
        val current = resolveInput(messages.last().attachments, media)
        // Preserve the most recent old images without reading every large sidecar into memory.
        var remaining = 4 * 1024 * 1024
        val earlier = messages.dropLast(1).asReversed().map { message ->
            message.copy(attachments = message.attachments.map images@{ attachment ->
                val image = attachment.imageDataUrl ?: return@images attachment
                val resolved = if (remaining > 0 && isChatMediaReference(image)) media?.resolve(image) else image
                if (resolved != null && !isChatMediaReference(resolved) && resolved.length <= remaining) {
                    remaining -= resolved.length
                    attachment.copy(imageDataUrl = resolved)
                } else ChatAttachment(attachment.name, text = "[历史图片 ${attachment.name} 已省略，可重新引用。]")
            })
        }.asReversed()
        return earlier + messages.last().copy(attachments = current)
    }

    /** 上一条回复失败时，原样重试最后一条用户提问。 */
    fun retryLast() {
        val last = _state.value.messages.lastOrNull() ?: return
        if (last.role != "assistant" || last.error == null || last.imageGeneration) return
        regenerate()
    }

    /** Never rewind only the UI while the native append-only history still contains tool effects. */
    fun regenerate() {
        val current = _state.value
        if (current.sending) return
        val index = current.messages.indexOfLast { it.role == "user" }
        if (index < 0) return
        val turn = current.messages[index]
        modelInputError(turn.attachments)?.let {
            _state.value = current.copy(inputError = it)
            return
        }
        val last = current.messages.lastOrNull() ?: return
        if (turn.imageGeneration || last.imageGeneration) return
        if (agent != null) {
            val safe = last.role == "assistant" && last.error != null && last.text.isBlank() && last.reasoning.isBlank() && last.tools.isEmpty() && last.records.isEmpty() && !last.error.orEmpty().contains("请继续任务")
            if (safe) {
                _state.value = current.copy(messages = current.messages.dropLast(1))
                sendPrompt(turn.text, "", turn.attachments, nativeRetry = !last.nativeNotStarted, reuseUser = true)
            } else {
                send(if (last.error != null) "请接着上一次任务继续，先核对已执行操作的真实结果，再处理未完成部分，不要重复已完成的操作。"
                    else "请结合上文重新回答上一条问题；此前已执行的操作不要重复执行。")
            }
            return
        }
        _state.value = current.copy(messages = current.messages.take(index))
        sendWithAttachments(turn.text, turn.attachments)
    }

    /** 停止正在生成的流式回复，保留已输出的部分文字。 */
    fun stop() {
        openJob?.cancel()
        openJob = null
        checkpointJob?.cancel()
        checkpointJob = null
        ++sendEpoch
        val wasSending = _state.value.sending
        chatJob?.cancel()
        chatJob = null
        val run = agentRun
        agentRun = null
        run?.let { stoppingRuns[it] = agentSession to agentReplyId; agent?.cancel(it) }
        val updated = _state.value.messages.mapNotNull { turn ->
            if (!turn.streaming) turn
            else if (turn.imageGeneration) turn.copy(streaming = false,
                error = "已停止等待。" + if (turn.imageJobRecoverable) RECOVER_IMAGE_RESULT else UNKNOWN_IMAGE_RESULT)
            else if (turn.text.isEmpty() && turn.error == null && turn.tools.isEmpty() && turn.reasoning.isEmpty() && run == null) null
            else turn.copy(streaming = false, cancelled = true, activity = "已停止", tools = settleTools(turn.tools, true))
        }
        _state.value = _state.value.copy(messages = updated, sending = false, approval = null, question = null)
        if (wasSending) persist()
    }

    fun clearConversation() {
        stop()
        agentSession = newSessionId()
        agentRun = null
        applyOptions(ChatOptions(tier = pickAiTier(_state.value.tiers, "")))
        _state.value = _state.value.copy(messages = emptyList(), approval = null, question = null, inputError = null)
    }

    /** 回答智能体的确认：`once` 允许这一次，`deny` 拒绝。 */
    fun answerApproval(decision: String) {
        val pending = _state.value.approval ?: return
        _state.value = _state.value.copy(approval = null, question = null)
        agent?.answerApproval(pending.id, decision)
        if (decision == "always" && pending.canRemember && pending.tool !in _state.value.grants) {
            _state.value = _state.value.copy(grants = _state.value.grants + pending.tool)
        }
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
        if (runAuthorization?.isCurrent() == false) return
        if (event is AgentEvent.ToolEnd && event.run != agentRun) {
            stoppingRuns[event.run]?.let { stopped ->
                fun update(turn: ChatTurn) = turn.copy(tools = turn.tools.map { if (it.id == event.id) it.copy(output = event.output, status = if (event.isError) ToolStatus.Error else ToolStatus.Done, endedAtMs = nowMs()) else it })
                if (stopped.first == agentSession) { updateTurn(stopped.second, ::update); persist() }
                else history?.loadSnapshot(stopped.first)?.let { saved ->
                    history.save(stopped.first, saved.messages.map { if (it.id == stopped.second) update(it) else it }, clock(), saved.options)
                }
            }
        }
        if (event is AgentEvent.RunEnd && event.run != agentRun) {
            val stopped = stoppingRuns.remove(event.run)
            if (stopped != null && event.error.orEmpty().contains("tool_outcome_needs_review")) {
                val message = friendlyAgentError(event.error)
                if (stopped.first == agentSession) { updateTurn(stopped.second) { it.copy(error = message, cancelled = false, activity = "结果需要核对") }; persist() }
                else history?.loadSnapshot(stopped.first)?.let { saved ->
                    history.save(stopped.first, saved.messages.map { if (it.id == stopped.second) it.copy(error = message, cancelled = false, activity = "结果需要核对") else it }, clock(), saved.options)
                }
            }
        }
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
                approval = ChatApproval(event.id, event.tool, summarizeToolArguments(event.tool, event.arguments.toString()), event.danger, event.canRemember),
            )
            is AgentEvent.Question -> if (event.run == agentRun && event.session == agentSession && _state.value.sending) {
                _state.value = _state.value.copy(question = ChatQuestion(event.id, event.question, event.options))
            }
            is AgentEvent.RunEnd -> if (event.run == agentRun) {
                val failure = if (event.ok) null else friendlyAgentError(event.error)
                updateTurn(agentReplyId) { turn ->
                    // 只调工具、没有说话的一轮，回复里留空；失败时把原因挂在这条回复上。
                    turn.copy(streaming = false, activity = "", error = failure, text = if (turn.text.isEmpty() && event.ok) event.text.orEmpty() else turn.text, tools = settleTools(turn.tools, !event.ok), truncated = turn.truncated || event.finish == "length", promptTokens = maxOf(turn.promptTokens, event.promptTokens), completionTokens = maxOf(turn.completionTokens, event.completionTokens))
                }
                _state.value = _state.value.copy(sending = false, approval = null, question = null)
                agentRun = null
                persist()
                if (!event.ok && (failure.orEmpty().contains("额度") || failure.orEmpty().contains("兑换"))) refreshBalanceQuietly()
                else if (event.ok) refreshBalanceQuietly()
            }
            is AgentEvent.Progress -> if (event.run == agentRun) {
                if (event.kind == "approval_resolved" && _state.value.approval?.id == event.detail["id"].asStr()) _state.value = _state.value.copy(approval = null)
                updateTurn(agentReplyId) { turn ->
                fun str(name: String) = event.detail[name].asStr().orEmpty()
                fun num(name: String) = str(name).toLongOrNull() ?: 0
                when (event.kind) {
                    "reasoning_delta" -> turn.copy(reasoning = turn.reasoning + str("text"))
                    "usage" -> turn.copy(promptTokens = turn.promptTokens + num("prompt_tokens"), completionTokens = turn.completionTokens + num("completion_tokens"))
                    "retrying" -> turn.copy(reasoning = "", activity = "正在重试（${str("attempt")}）")
                    "continuing" -> turn.copy(activity = "正在继续未完成任务")
                    "truncated" -> turn.copy(truncated = true)
                    "tool_waiting" -> turn.copy(activity = if (str("waiting") == "true") "正在等待资源" else "")
                    "tool_input" -> turn.copy(activity = "正在准备 ${str("name")}（${str("bytes")} 字节）")
                    else -> turn
                }
                }
            }
            is AgentEvent.Failure -> if (agentRun != null && _state.value.sending && (event.runs == null || agentRun in event.runs)) {
                val rejectedBeforeRun = event.message.let {
                    "invalid message" in it || "needs exactly one of" in it || "line too long" in it || "cannot start the run" in it || "must be 1 to" in it
                }
                updateTurn(agentReplyId) {
                    val unstarted = rejectedBeforeRun && it.text.isEmpty() && it.reasoning.isEmpty() && it.tools.isEmpty() && it.records.isEmpty()
                    it.copy(streaming = false, error = event.message, nativeNotStarted = it.nativeNotStarted || unstarted, tools = settleTools(it.tools, true))
                }
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
        val request = accountRequest()
        redeemJob = scope.launch(request) {
            try {
                val done = api.redeem(code)
                if (!request.isCurrent()) return@launch
                val unit = _state.value.balance?.quotaPerUnit ?: 500_000
                _state.value = _state.value.copy(
                    redeeming = false,
                    redeemMessage = "兑换成功，到账 ${formatAiCredits(done.quota, unit)}",
                )
                loadMeta()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                if (request.isCurrent()) _state.value = _state.value.copy(redeeming = false, redeemError = messageOf(e))
            }
        }
    }

    fun reset() {
        openJob?.cancel()
        checkpointJob?.cancel()
        ++accountGeneration
        metaJob?.cancel()
        balanceJob?.cancel()
        redeemJob?.cancel()
        chatJob?.cancel()
        agentRun?.let { agent?.cancel(it) }
        agentRun = null
        _state.value = CloudChatState(nativeAgentAvailable = agent != null)
        agentSession = newSessionId()
    }

    private fun updateTurn(id: Long, transform: (ChatTurn) -> ChatTurn) {
        val messages = _state.value.messages.map { if (it.id == id) transform(it) else it }
        _state.value = _state.value.copy(messages = messages)
        val store = history ?: return
        val last = messages.lastOrNull()?.takeIf { it.id == id && it.streaming && !it.imageGeneration } ?: return
        if (checkpointJob?.isActive == true) return
        val owner = agentSession
        val epoch = sendEpoch
        checkpointJob = scope.launch {
            delay(750)
            if (epoch == sendEpoch && owner == agentSession) {
                _state.value.messages.lastOrNull()?.takeIf { it.id == last.id }?.let { store.checkpoint(owner, it) }
            }
        }
    }

    private fun refreshBalanceQuietly() {
        balanceJob?.cancel()
        val request = accountRequest()
        balanceJob = scope.launch(request) {
            runCatching { api.balance() }.onSuccess { b ->
                if (request.isCurrent()) _state.value = _state.value.copy(balance = b)
            }
        }
    }
}
