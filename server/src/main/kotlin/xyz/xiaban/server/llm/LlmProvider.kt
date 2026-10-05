package xyz.xiaban.server.llm

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.timeout
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.post
import io.ktor.client.request.preparePost
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsChannel
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.utils.io.readUTF8Line
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import xyz.xiaban.server.config.LlmEndpoint
import xyz.xiaban.server.http.AppJson
import java.time.Duration

/** Vendor-neutral seam (PRD 9.1): swap or add vendors without touching the conversation logic. */
interface LlmProvider {
    val name: String
    suspend fun complete(system: String, user: String, timeout: Duration): String
    suspend fun stream(
        messages: List<Pair<String, String>>,
        timeout: Duration,
        maxTokens: Int = 300,
        temperature: Double = 0.8,
        onChunk: suspend (String) -> Unit,
    ): String = complete(
        system = messages.firstOrNull { it.first == "system" }?.second.orEmpty(),
        user = messages.lastOrNull { it.first == "user" }?.second.orEmpty(),
        timeout = timeout,
    ).also { onChunk(it) }
}

class OpenAiCompatibleProvider(private val endpoint: LlmEndpoint, private val http: HttpClient) : LlmProvider {
    override val name: String = endpoint.name

    override suspend fun complete(system: String, user: String, timeout: Duration): String {
        val response = http.post("${endpoint.baseUrl.trimEnd('/')}/chat/completions") {
            bearerAuth(endpoint.apiKey)
            contentType(ContentType.Application.Json)
            timeout { requestTimeoutMillis = timeout.toMillis() }
            setBody(
                ChatRequest(
                    model = endpoint.model,
                    messages = listOf(ChatMessage("system", system), ChatMessage("user", user)),
                    maxTokens = 80,
                    temperature = 0.8,
                ),
            )
        }
        check(response.status.isSuccess()) { "$name answered ${response.status}" }
        return response.body<ChatResponse>().choices.firstOrNull()?.message?.content ?: error("$name returned no choices")
    }

    override suspend fun stream(
        messages: List<Pair<String, String>>,
        timeout: Duration,
        maxTokens: Int,
        temperature: Double,
        onChunk: suspend (String) -> Unit,
    ): String {
        val full = StringBuilder()
        val requestBody = ChatRequest(
            model = endpoint.model,
            messages = messages.map { ChatMessage(it.first, it.second) },
            maxTokens = maxTokens,
            temperature = temperature,
            stream = true,
        )
        http.preparePost("${endpoint.baseUrl.trimEnd('/')}/chat/completions") {
            bearerAuth(endpoint.apiKey)
            contentType(ContentType.Application.Json)
            timeout { requestTimeoutMillis = timeout.toMillis() }
            setBody(requestBody)
        }.execute { response ->
            check(response.status.isSuccess()) { "$name answered ${response.status}" }
            val channel = response.bodyAsChannel()
            while (!channel.isClosedForRead) {
                val line = channel.readUTF8Line() ?: break
                val trimmed = line.trim()
                if (trimmed.isEmpty() || !trimmed.startsWith("data:")) continue
                val data = trimmed.removePrefix("data:").trim()
                if (data == "[DONE]") break
                val chunk = runCatching { AppJson.decodeFromString<StreamChunkResponse>(data) }.getOrNull()
                val delta = chunk?.choices?.firstOrNull()?.delta?.content
                if (!delta.isNullOrEmpty()) {
                    full.append(delta)
                    onChunk(delta)
                }
            }
        }
        return full.toString()
    }

    @Serializable
    private data class ChatRequest(
        val model: String,
        val messages: List<ChatMessage>,
        @SerialName("max_tokens") val maxTokens: Int,
        val temperature: Double,
        val stream: Boolean = false,
    )

    @Serializable
    private data class ChatMessage(val role: String, val content: String? = null)

    @Serializable
    private data class ChatResponse(val choices: List<Choice> = emptyList()) {
        @Serializable
        data class Choice(val message: ChatMessage? = null)
    }

    @Serializable
    private data class StreamChunkResponse(val choices: List<StreamChoice> = emptyList()) {
        @Serializable
        data class StreamChoice(val delta: StreamDelta? = null)
        @Serializable
        data class StreamDelta(val content: String? = null)
    }
}
