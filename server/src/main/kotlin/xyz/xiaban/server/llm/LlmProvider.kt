package xyz.xiaban.server.llm

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.timeout
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import xyz.xiaban.server.config.LlmEndpoint
import java.time.Duration

/** Vendor-neutral seam (PRD 9.1): swap or add vendors without touching the polishing logic. */
interface LlmProvider {
    val name: String
    suspend fun complete(system: String, user: String, timeout: Duration): String
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

    @Serializable
    private data class ChatRequest(
        val model: String,
        val messages: List<ChatMessage>,
        @SerialName("max_tokens") val maxTokens: Int,
        val temperature: Double,
    )

    @Serializable
    private data class ChatMessage(val role: String, val content: String? = null)

    @Serializable
    private data class ChatResponse(val choices: List<Choice> = emptyList()) {
        @Serializable
        data class Choice(val message: ChatMessage? = null)
    }
}
