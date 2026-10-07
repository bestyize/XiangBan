package xyz.xiaban.server.tts

import io.ktor.client.HttpClient
import io.ktor.client.plugins.timeout
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.plugins.websocket.webSocket
import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.websocket.Frame
import io.ktor.websocket.readBytes
import io.ktor.websocket.readText
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Streams speech from Microsoft Edge's read-aloud endpoint.
 *
 * The endpoint is unofficial and unsupported, so callers should treat any failure as
 * "audio unavailable" and keep the text reply as the source of truth.
 */
class EdgeTtsClient(private val http: HttpClient) {
    private val json = Json { ignoreUnknownKeys = true }

    suspend fun voices(): List<EdgeTtsVoice> {
        val response = http.get(EdgeTtsProtocol.voicesUrl()) { applyBrowserHeaders() }
        if (response.status != HttpStatusCode.OK) {
            throw EdgeTtsException("Edge voice list failed with HTTP ${response.status.value}")
        }
        return json.decodeFromString<List<VoiceDto>>(response.bodyAsText()).map { it.toPublic() }
    }

    /**
     * Synthesizes [text] and returns the complete MP3.
     *
     * Long text is split before XML escaping so an entity is never cut in half; each split
     * uses its own connection, matching the Edge client's behavior.
     */
    suspend fun synthesize(text: String, options: EdgeTtsVoiceOptions = EdgeTtsVoiceOptions()): ByteArray {
        val chunks = Utf8TextChunker.split(text)
        if (chunks.isEmpty()) return ByteArray(0)

        val output = ArrayList<ByteArray>()
        var receivedAudio = false
        for (chunk in chunks) {
            http.webSocket(
                urlString = EdgeTtsProtocol.webSocketUrl(EdgeTtsProtocol.connectionId()),
                request = {
                    applyBrowserHeaders()
                    header("Pragma", "no-cache")
                    header("Cache-Control", "no-cache")
                },
            ) {
                send(Frame.Text(EdgeTtsProtocol.speechConfig(options)))
                send(Frame.Text(EdgeTtsProtocol.ssmlRequest(EdgeTtsProtocol.connectionId(), options, chunk)))

                for (frame in incoming) {
                    when (frame) {
                        is Frame.Text -> {
                            val response = parseTextResponse(frame.readText())
                            when (response.path) {
                                "turn.end" -> break
                                "audio.metadata", "response", "turn.start" -> Unit
                                else -> throw EdgeTtsException("Unknown text response path: ${response.path}")
                            }
                        }
                        is Frame.Binary -> {
                            val audio = parseBinaryAudioFrame(frame.readBytes())
                            if (audio.data.isEmpty()) continue
                            if (audio.headers["Path"] != "audio") {
                                throw EdgeTtsException("Binary response path is not audio")
                            }
                            receivedAudio = true
                            output += audio.data
                        }
                        is Frame.Close -> break
                        else -> Unit
                    }
                }
            }
        }

        if (!receivedAudio) {
            throw EdgeTtsException("Edge returned no audio for the supplied text")
        }
        val total = output.sumOf { it.size }
        val result = ByteArray(total)
        var offset = 0
        for (bytes in output) {
            bytes.copyInto(result, offset)
            offset += bytes.size
        }
        return result
    }

    private fun io.ktor.client.request.HttpRequestBuilder.applyBrowserHeaders() {
        header(HttpHeaders.UserAgent, EdgeTtsProtocol.USER_AGENT)
        header(HttpHeaders.Accept, "*/*")
        header(HttpHeaders.AcceptLanguage, "en-US,en;q=0.9")
        header(HttpHeaders.Origin, EdgeTtsProtocol.ORIGIN)
        header("Cookie", "muid=${EdgeTtsProtocol.muid()};")
    }

    @Serializable
    private data class VoiceDto(
        @SerialName("ShortName") val shortName: String = "",
        @SerialName("Gender") val gender: String = "",
        @SerialName("Locale") val locale: String = "",
        @SerialName("FriendlyName") val friendlyName: String = "",
    ) {
        fun toPublic() = EdgeTtsVoice(shortName, gender, locale, friendlyName)
    }
}
