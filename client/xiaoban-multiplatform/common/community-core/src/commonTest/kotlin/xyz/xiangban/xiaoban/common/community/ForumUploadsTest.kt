package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.OutgoingContent
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import io.ktor.utils.io.ByteChannel
import io.ktor.utils.io.readRemaining
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.test.runTest
import kotlinx.io.readByteArray
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val ASSET = "00112233-4455-6677-8899-aabbccddeeff"
private val PNG_BYTES = byteArrayOf(0x89.toByte(), 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A)
private fun imageFile() = ForumUploadFile("photo.png", "image/png", PNG_BYTES)
private const val IMAGE_RESPONSE = """{"id":"00112233-4455-6677-8899-aabbccddeeff","url":"/api/images/00112233-4455-6677-8899-aabbccddeeff","size":8}"""
private const val ATTACHMENT_RESPONSE = """{"id":"00112233-4455-6677-8899-aabbccddeeff","name":"note.txt","size":3,"price":0,"purchased":true}"""

private suspend fun OutgoingContent.multipartBytes(): ByteArray = coroutineScope {
    val channel = ByteChannel(autoFlush = true)
    val reader = async { channel.readRemaining().readByteArray() }
    (this@multipartBytes as OutgoingContent.WriteChannelContent).writeTo(channel)
    channel.flushAndClose()
    reader.await()
}

class ForumUploadsTest {
    @Test
    fun `image sends multipart file and freshest bearer without local uri or token in URL`() = runTest {
        var multipart = ""
        var binary = byteArrayOf()
        val client = HttpClient(MockEngine { request ->
            assertEquals("/api/images", request.url.encodedPath)
            assertEquals("Bearer fresh-access", request.headers[HttpHeaders.Authorization])
            assertEquals("", request.url.encodedQuery)
            assertTrue(request.body.contentType.toString().startsWith("multipart/form-data; boundary="))
            binary = request.body.multipartBytes()
            multipart = binary.decodeToString()
            respond(IMAGE_RESPONSE, headers = headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val api = ForumApi(client, CommunityConfig()) { "fresh-access" }
        val image = api.uploadImage(imageFile(), FORUM_IMAGE_MAX_BYTES)
        assertTrue(multipart.contains("name=file") || multipart.contains("name=\"file\""))
        assertTrue(multipart.contains("filename=\"photo.png\""))
        assertTrue(multipart.contains("Content-Type: image/png", ignoreCase = true))
        assertFalse(multipart.contains("fresh-access"))
        assertTrue(binary.asList().windowed(PNG_BYTES.size).any { it == PNG_BYTES.toList() })
        assertEquals("\n![photo](/api/images/$ASSET)\n", forumImageMarkdown(image, "photo.png"))
        client.close()
    }

    @Test
    fun `limits use authenticated forum session and zero forbids images before any upload`() = runTest {
        val server = FakeServer()
        server.on("/api/session", body = """{"user":{"uid":"u","nickname":"昵称","trust_level":0},"limits":{"image_max_bytes":0,"upload_max_bytes":5242880}}""")
        val api = ForumApi(server.client, CommunityConfig()) { "access" }
        val session = api.uploadSession()
        assertEquals(0, session.user.trustLevel)
        assertEquals(5_242_880L, session.limits.uploadMaxBytes)
        assertEquals("Bearer access", server.requests.single().headers[HttpHeaders.Authorization])
        assertEquals(403, assertFailsWith<ApiException> { api.uploadImage(imageFile(), session.limits.imageMaxBytes) }.status)
        assertEquals(1, server.requests.size)
        server.client.close()
    }

    @Test
    fun `over limit empty and unsupported files fail locally without multipart request`() = runTest {
        val server = FakeServer()
        val api = ForumApi(server.client, CommunityConfig()) { "access" }
        assertFailsWith<ApiException> { api.uploadImage(imageFile(), 7) }
        assertFailsWith<ApiException> { api.uploadImage(imageFile().copy(mime = "image/svg+xml"), 100) }
        assertFailsWith<ApiException> { api.uploadAttachment(ForumUploadFile("empty", "text/plain", byteArrayOf()), 100) }
        assertEquals(0, server.requests.size)
        server.client.close()
    }

    @Test
    fun `image markdown rejects local or external URLs and escaped filenames cannot inject markup`() {
        val image = ForumImage(ASSET, "/api/images/$ASSET", 8)
        for (url in listOf("file:///private/photo.png", "content://provider/photo", "https://evil.example/photo", "/api/images/$ASSET)\n[evil]"))
            assertFailsWith<ApiException> { forumImageMarkdown(image.copy(url = url), "photo") }
        val markdown = forumImageMarkdown(image, "path/evil[link](x)!\\photo\n.png")
        assertFalse(markdown.contains("file:")); assertFalse(markdown.contains("content:"))
        assertEquals(2, markdown.count { it == '\n' })
        assertEquals("\n![photo](/api/images/$ASSET)\n", markdown)
        assertEquals("quote-name.txt", forumUploadName("../quote-\"name.txt\r\n"))
    }

    @Test
    fun `upload failure returns server reason and explicit retry reuses selected bytes`() = runTest {
        val server = FakeServer()
        server.on("/api/images", HttpStatusCode.TooManyRequests, """{"message":"今天图片已达上限","retry_after":3600}""")
        val api = ForumApi(server.client, CommunityConfig()) { "access" }
        val error = assertFailsWith<ApiException> { api.uploadImage(imageFile(), FORUM_IMAGE_MAX_BYTES) }
        assertEquals(429, error.status); assertEquals(3600L, error.retryAfter?.toLong())
        assertEquals(1, server.requests.size)
        server.on("/api/images", body = IMAGE_RESPONSE)
        assertEquals(ASSET, api.uploadImage(imageFile(), FORUM_IMAGE_MAX_BYTES).id)
        assertEquals(2, server.requests.size)
        server.client.close()
    }

    @Test
    fun `401 is not automatically replayed and missing credentials never send anonymous upload`() = runTest {
        val server = FakeServer()
        server.on("/api/images", HttpStatusCode.Unauthorized, """{"message":"登录已过期"}""")
        val api = ForumApi(server.client, CommunityConfig()) { "expired" }
        assertEquals(401, assertFailsWith<ApiException> { api.uploadImage(imageFile(), 100) }.status)
        assertEquals(1, server.requests.size)
        val anonymous = ForumApi(server.client, CommunityConfig()) { null }
        assertEquals(401, assertFailsWith<ApiException> { anonymous.uploadImage(imageFile(), 100) }.status)
        assertEquals(1, server.requests.size)
        server.client.close()
    }

    @Test
    fun `upload refreshes near expiry before multipart and failed refresh sends no upload`() = runTest {
        val server = FakeServer()
        server.on("/api/auth/refresh", body = """{"access_token":"new","refresh_token":"next","expires_in":900}""")
        server.on("/api/images", body = IMAGE_RESPONSE)
        val store = InMemoryTokenStorage(Tokens("old", "refresh", 10000))
        val manager = SessionManager(AccountApi(server.client, CommunityConfig()), store, now = { 5000 })
        val api = ForumApi(server.client, CommunityConfig()) { manager.accessToken() }
        api.uploadImage(imageFile(), 100)
        assertEquals(listOf("/api/auth/refresh", "/api/images"), server.requests.map { it.url.encodedPath })
        assertEquals("Bearer new", server.requests.last().headers[HttpHeaders.Authorization])
        server.client.close()

        val offline = FakeServer(); offline.onThrow("/api/auth/refresh")
        val kept = InMemoryTokenStorage(Tokens("old", "refresh", 10000))
        val offlineManager = SessionManager(AccountApi(offline.client, CommunityConfig()), kept, now = { 5000 })
        val offlineApi = ForumApi(offline.client, CommunityConfig()) { offlineManager.accessToken() }
        assertFailsWith<ApiException> { offlineApi.uploadImage(imageFile(), 100) }
        assertEquals(listOf("/api/auth/refresh"), offline.requests.map { it.url.encodedPath })
        assertTrue(offlineManager.state.value.loggedIn)
        assertEquals("refresh", kept.load()?.refreshToken)
        offline.client.close()
    }

    @Test
    fun `attachment upload links existing topic ids and prices and pending removal uses delete`() = runTest {
        val server = FakeServer()
        server.on("/api/attachments", body = ATTACHMENT_RESPONSE)
        server.on("/api/topics", body = """{"id":7}""")
        val api = ForumApi(server.client, CommunityConfig()) { "access" }
        val attachment = api.uploadAttachment(ForumUploadFile("note.txt", "text/plain", "abc".encodeToByteArray()), 100)
        assertEquals(7L, api.createTopic(1, "标题", "正文足够十个字以上的示例", attachments = listOf(attachment)))
        val payload = CommunityJson.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertEquals(ASSET, payload["attachment_ids"]!!.jsonArray.single().jsonPrimitive.content)
        assertEquals(ASSET, payload["attachment_prices"]!!.jsonArray.single().jsonObject["id"]!!.jsonPrimitive.content)
        assertEquals("0", payload["attachment_prices"]!!.jsonArray.single().jsonObject["price"]!!.jsonPrimitive.content)
        assertNull(payload["attachments"])
        server.on("/api/attachments/$ASSET", body = """{"ok":true}""")
        api.deletePendingAttachment(ASSET)
        assertEquals("DELETE", server.requests.last().method.value)
        assertEquals("/api/attachments/$ASSET", server.requests.last().url.encodedPath)
        server.client.close()
    }

    @Test
    fun `failed topic publish retains attachment ids for retry without reupload or deletion`() = runTest {
        val server = FakeServer()
        server.on("/api/attachments", body = ATTACHMENT_RESPONSE)
        server.on("/api/topics", HttpStatusCode.ServiceUnavailable, """{"message":"稍后重试"}""")
        val api = ForumApi(server.client, CommunityConfig()) { "access" }
        val attachment = api.uploadAttachment(ForumUploadFile("note.txt", "text/plain", "abc".encodeToByteArray()), 100)
        assertEquals(503, assertFailsWith<ApiException> {
            api.createTopic(1, "标题", "正文足够十个字以上的示例", attachments = listOf(attachment))
        }.status)
        server.on("/api/topics", body = """{"id":7}""")
        api.createTopic(1, "标题", "正文足够十个字以上的示例", attachments = listOf(attachment))
        assertEquals(listOf("/api/attachments", "/api/topics", "/api/topics"), server.requests.map { it.url.encodedPath })
        server.requests.drop(1).forEach { request ->
            val payload = CommunityJson.parseToJsonElement((request.body as TextContent).text).jsonObject
            assertEquals(ASSET, payload["attachment_ids"]!!.jsonArray.single().jsonPrimitive.content)
        }
        server.client.close()
    }

    @Test
    fun `attachment validation rejects duplicate ids oversized lists bad prices and non UUID cleanup`() = runTest {
        val attachment = ForumAttachment(ASSET, "file", 3)
        assertFailsWith<ApiException> { validateTopicAttachments(listOf(attachment, attachment)) }
        assertFailsWith<ApiException> { validateTopicAttachments(List(6) { attachment }) }
        assertFailsWith<ApiException> { validateTopicAttachments(listOf(attachment.copy(price = 11))) }
        val server = FakeServer()
        val api = ForumApi(server.client, CommunityConfig()) { "access" }
        assertFailsWith<ApiException> { api.deletePendingAttachment("../images/123") }
        assertEquals(0, server.requests.size)
        server.client.close()
    }

    @Test
    fun `reply sends only successful site image markdown without unsupported attachment protocol`() = runTest {
        val server = FakeServer()
        server.on("/api/images", body = IMAGE_RESPONSE)
        server.on("/api/topics/7/posts", body = """{"id":8}""")
        val api = ForumApi(server.client, CommunityConfig()) { "access" }
        val image = api.uploadImage(imageFile(), 100)
        val body = "我的回复" + forumImageMarkdown(image, "photo.png")
        api.reply(7, body)
        val payload = CommunityJson.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertEquals(body, payload["body"]!!.jsonPrimitive.content)
        assertNull(payload["attachment_ids"])
        server.client.close()
    }

    @Test
    fun `cancelled upload propagates cancellation instead of reporting upload failure`() = runTest {
        val client = HttpClient(MockEngine { throw CancellationException("cancelled") })
        val api = ForumApi(client, CommunityConfig()) { "access" }
        assertFailsWith<CancellationException> { api.uploadImage(imageFile(), 100) }
        client.close()
    }
}
