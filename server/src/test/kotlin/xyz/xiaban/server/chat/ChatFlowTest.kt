package xyz.xiaban.server.chat

import io.ktor.client.call.body
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.ChatHistoryResponse
import xyz.xiaban.shared.ChatMessageView
import xyz.xiaban.shared.ChatStreamRequest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ChatFlowTest {

    @Test
    fun `history starts empty, records messages upon streaming, and can be cleared`() = apiTest {
        val user = session()
        val token = user.token

        val initial = client.get(ApiPaths.CHAT_MESSAGES) {
            header(HttpHeaders.Authorization, "Bearer $token")
        }.body<ChatHistoryResponse>()
        assertEquals(0, initial.messages.size)

        val streamResp = client.post(ApiPaths.CHAT_STREAM) {
            header(HttpHeaders.Authorization, "Bearer $token")
            contentType(ContentType.Application.Json)
            setBody(ChatStreamRequest(message = "今天好累啊"))
        }
        assertEquals(HttpStatusCode.OK, streamResp.status)
        val streamText = streamResp.bodyAsText()
        assertTrue(streamText.contains("data:"), "SSE chunks are sent")
        assertTrue(streamText.contains("[DONE]"), "Stream terminates with [DONE]")

        val historyAfter = client.get(ApiPaths.CHAT_MESSAGES) {
            header(HttpHeaders.Authorization, "Bearer $token")
        }.body<ChatHistoryResponse>()
        assertEquals(2, historyAfter.messages.size)
        assertEquals("user", historyAfter.messages[0].role)
        assertEquals("今天好累啊", historyAfter.messages[0].content)
        assertEquals("assistant", historyAfter.messages[1].role)
        assertTrue(historyAfter.messages[1].content.isNotBlank())

        val delResp = client.delete(ApiPaths.CHAT_MESSAGES) {
            header(HttpHeaders.Authorization, "Bearer $token")
        }
        assertEquals(HttpStatusCode.OK, delResp.status)

        val cleared = client.get(ApiPaths.CHAT_MESSAGES) {
            header(HttpHeaders.Authorization, "Bearer $token")
        }.body<ChatHistoryResponse>()
        assertEquals(0, cleared.messages.size)
    }

    @Test
    fun `guest chat works without token and rejects empty messages`() = apiTest {
        val guestResp = client.post(ApiPaths.CHAT_STREAM) {
            contentType(ContentType.Application.Json)
            setBody(ChatStreamRequest(
                message = "随便聊聊",
                guestHistory = listOf(ChatMessageView(role = "user", content = "你好"))
            ))
        }
        assertEquals(HttpStatusCode.OK, guestResp.status)
        val text = guestResp.bodyAsText()
        assertTrue(text.contains("data:"))
        assertTrue(text.contains("[DONE]"))

        val bad = client.post(ApiPaths.CHAT_STREAM) {
            contentType(ContentType.Application.Json)
            setBody(ChatStreamRequest(message = "   "))
        }
        assertEquals(HttpStatusCode.BadRequest, bad.status)
    }
}
