package xyz.xiaban.server.chat

import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.server.auth.authenticate
import io.ktor.server.plugins.ratelimit.rateLimit
import io.ktor.server.request.receive
import io.ktor.server.response.header
import io.ktor.server.response.respond
import io.ktor.server.response.respondBytes
import io.ktor.server.response.respondTextWriter
import io.ktor.server.routing.Route
import io.ktor.server.routing.delete
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import xyz.xiaban.contracts.common.OkResponse
import xyz.xiaban.contracts.companion.chat.ChatApiPaths
import xyz.xiaban.contracts.companion.chat.ChatHistoryResponse
import xyz.xiaban.contracts.companion.chat.ChatMessageView
import xyz.xiaban.contracts.companion.chat.ChatSpeakRequest
import xyz.xiaban.contracts.companion.chat.ChatStreamRequest
import xyz.xiaban.contracts.companion.personality.Personality
import xyz.xiaban.server.Services
import xyz.xiaban.server.auth.AUTH_USER
import xyz.xiaban.server.auth.userId
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.RateLimits
import xyz.xiaban.server.http.badRequest

fun Route.chatRoutes(services: Services) {
    get(ChatApiPaths.CHAT_MESSAGES) {
        val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
        val userId = bearer?.let { services.tokens.userIdOrNull(it) }
        val messages = if (userId != null) services.chat.list(userId) else emptyList()
        call.respond(ChatHistoryResponse(messages))
    }

    rateLimit(RateLimits.CHAT) {
        post(ChatApiPaths.CHAT_STREAM) {
            val req = runCatching { call.receive<ChatStreamRequest>() }.getOrElse {
                badRequest("请求格式不对")
            }
            if (req.message.isBlank()) {
                badRequest("消息内容不能为空")
            }

            val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
            val userId = bearer?.let { services.tokens.userIdOrNull(it) }

            call.respondTextWriter(contentType = ContentType.Text.EventStream) {
                services.chat.stream(userId, req.message, req.guestHistory) { delta ->
                    val obj = buildJsonObject { put("delta", delta) }
                    write("data: ${obj}\n\n")
                    flush()
                }
                write("data: [DONE]\n\n")
                flush()
            }
        }
    }

    delete(ChatApiPaths.CHAT_MESSAGES) {
        val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
        val userId = bearer?.let { services.tokens.userIdOrNull(it) }
        if (userId != null) {
            services.chat.clear(userId)
        }
        call.respond(OkResponse())
    }

    rateLimit(RateLimits.TTS) {
        post(ChatApiPaths.CHAT_SPEAK) {
            val req = runCatching { call.receive<ChatSpeakRequest>() }.getOrElse { badRequest("请求格式不对") }
            if (req.text.isBlank()) badRequest("要朗读的内容不能为空")

            val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
            val userId = bearer?.let { services.tokens.userIdOrNull(it) }
            val personality = if (userId != null) services.profiles.user(userId).personality else Personality.Gentle

            val audio = services.tts.speak(req.text, personality)
                ?: throw ApiException(HttpStatusCode.ServiceUnavailable, "tts_unavailable", "小伴现在发不出声音，稍后再试")

            call.response.header(HttpHeaders.CacheControl, "private, max-age=604800")
            call.respondBytes(audio.bytes, ContentType.parse(audio.mimeType))
        }
    }
}
