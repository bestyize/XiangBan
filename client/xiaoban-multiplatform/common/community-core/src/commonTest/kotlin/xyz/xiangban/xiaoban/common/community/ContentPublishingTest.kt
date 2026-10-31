package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.OutgoingContent
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.*

private val ZIP_FILE = ForumUploadFile("demo.zip", "application/zip", byteArrayOf(80, 75, 3, 4, 1, 2, 3, 4))
private val PUBLISH_DRAFT = ContentPublishDraft("effect", "作品", "简介", listOf("标签"), true)
private val TICKET = """{"ticket":"RESOURCE-ONLY-TICKET","upload_url":"https://resource.example/upload","file_key":"packs/effect/key.zip","expires_at":9999999999}"""
private const val CONTENT_KEY = "packs/effect/key.zip"

class ContentPublishingTest {
    @Test
    fun `direct resource PUT has only upload ticket and byte payload while forum request has bearer`() = runTest {
        val requests = mutableListOf<String>()
        val client = HttpClient(MockEngine { request ->
            requests += request.url.host + request.url.encodedPath
            if (request.url.host == "resource.example") {
                assertEquals("PUT", request.method.value)
                assertNull(request.headers[HttpHeaders.Authorization])
                assertEquals("RESOURCE-ONLY-TICKET", request.headers["X-Upload-Ticket"])
                assertContentEquals(ZIP_FILE.bytes, (request.body as OutgoingContent.ByteArrayContent).bytes())
                respond("{}", headers = headersOf(HttpHeaders.ContentType, "application/json"))
            } else {
                assertEquals("Bearer FORUM-ACCESS", request.headers[HttpHeaders.Authorization])
                assertNull(request.headers["X-Upload-Ticket"])
                val body = CommunityJson.parseToJsonElement((request.body as TextContent).text).jsonObject
                assertEquals("effect", body["kind"]!!.jsonPrimitive.content)
                assertEquals(ZIP_FILE.bytes.size.toString(), body["size"]!!.jsonPrimitive.content)
                assertEquals(Sha256().apply { update(ZIP_FILE.bytes) }.hex(), body["sha256"]!!.jsonPrimitive.content)
                respond(TICKET, headers = headersOf(HttpHeaders.ContentType, "application/json"))
            }
        })
        val api = ContentPublishingApi(client, CommunityConfig()) { "FORUM-ACCESS" }
        assertEquals(CONTENT_KEY, api.upload("effect", ZIP_FILE).fileKey)
        assertEquals(listOf("chat.xiangban.xyz/api/packs/upload-ticket", "resource.example/upload"), requests)
        client.close()
    }

    @Test
    fun `resource 307 is rejected without following to another origin`() = runTest {
        var requests = 0
        val client = HttpClient(MockEngine { request ->
            requests++
            if (request.url.encodedPath == "/api/packs/upload-ticket") respond(TICKET, headers = headersOf(HttpHeaders.ContentType, "application/json"))
            else respond("", HttpStatusCode.TemporaryRedirect, headersOf(HttpHeaders.Location, "https://evil.example/steal"))
        })
        val api = ContentPublishingApi(client, CommunityConfig()) { "ACCESS" }
        assertFailsWith<ApiException> { api.upload("effect", ZIP_FILE) }
        assertEquals(2, requests)
        client.close()
    }

    @Test
    fun `resource not configured is a friendly 503 and makes no resource upload`() = runTest {
        val server = FakeServer()
        server.on("/api/packs/upload-ticket", HttpStatusCode.ServiceUnavailable, """{"message":"资源服务器尚未接入"}""")
        val api = ContentPublishingApi(server.client, CommunityConfig()) { "access" }
        val error = assertFailsWith<ApiException> { api.upload("effect", ZIP_FILE) }
        assertEquals(503, error.status); assertEquals("资源服务器尚未接入", error.message)
        assertEquals(1, server.requests.size)
        server.client.close()
    }

    @Test
    fun `known package kinds have server limits and scheme is never a zip kind`() = runTest {
        assertEquals(1_048_576L, ContentPackLimits["effect"])
        assertEquals(8_388_608L, ContentPackLimits["skill"])
        assertEquals(33_554_432L, ContentPackLimits["wallpaper"])
        assertNull(ContentPackLimits["scheme"])
        val server = FakeServer(); val api = ContentPublishingApi(server.client, CommunityConfig()) { "access" }
        assertFailsWith<ApiException> { api.upload("scheme", ZIP_FILE) }
        assertFailsWith<ApiException> { api.upload("effect", ZIP_FILE.copy(name = "bad.txt")) }
        assertFailsWith<ApiException> { api.upload("effect", ZIP_FILE.copy(bytes = byteArrayOf(1, 2, 3, 4))) }
        assertFailsWith<ApiException> { api.upload("effect", ZIP_FILE.copy(bytes = ByteArray(1_048_577))) }
        assertEquals(0, server.requests.size)
        server.client.close()
    }

    @Test
    fun `rights must be explicitly acknowledged and invalid fields never reach publish endpoint`() = runTest {
        val server = FakeServer(); val api = ContentPublishingApi(server.client, CommunityConfig()) { "access" }
        assertNotNull(contentDraftError(PUBLISH_DRAFT.copy(rightsAck = false)))
        assertNotNull(contentDraftError(PUBLISH_DRAFT.copy(title = "")))
        assertNotNull(contentDraftError(PUBLISH_DRAFT.copy(tags = List(9) { "标签$it" })))
        assertNotNull(contentDraftError(PUBLISH_DRAFT.copy(summary = "x".repeat(501))))
        assertFailsWith<ApiException> { api.publish(PUBLISH_DRAFT.copy(rightsAck = false), CONTENT_KEY) }
        assertFailsWith<ApiException> { api.publish(PUBLISH_DRAFT.copy(rightsAck = false), CONTENT_KEY, 7) }
        assertEquals(0, server.requests.size)
        server.client.close()
    }

    @Test
    fun `published payload and new version follow existing endpoints without inventing execution`() = runTest {
        val server = FakeServer()
        server.on("/api/packs", body = """{"id":7,"slug":"demo","trust_level":2}""")
        server.on("/api/packs/7/versions", body = """{"id":7,"version":2,"trust_level":2}""")
        val api = ContentPublishingApi(server.client, CommunityConfig()) { "access" }
        assertEquals(2, api.publish(PUBLISH_DRAFT, CONTENT_KEY).trustLevel)
        val create = CommunityJson.parseToJsonElement((server.requests[0].body as TextContent).text).jsonObject
        assertEquals("true", create["rights_ack"]!!.jsonPrimitive.content)
        assertEquals(CONTENT_KEY, create["file_key"]!!.jsonPrimitive.content)
        assertEquals("标签", create["tags"]!!.jsonArray.single().jsonPrimitive.content)
        assertNull(create["execute"]); assertNull(create["apply"])
        api.publish(PUBLISH_DRAFT.copy(changelog = "版本更新"), CONTENT_KEY, 7)
        val version = CommunityJson.parseToJsonElement((server.requests[1].body as TextContent).text).jsonObject
        assertEquals(setOf("file_key", "changelog"), version.keys)
        assertEquals("版本更新", version["changelog"]!!.jsonPrimitive.content)
        server.client.close()
    }

    @Test
    fun `verification waits finitely and never repeatedly posts publish`() = runTest {
        var reads = 0
        val client = HttpClient(MockEngine { request ->
            assertEquals("GET", request.method.value)
            assertEquals(CONTENT_KEY, request.url.parameters["file_key"])
            reads++
            respond("""{"file_key":"$CONTENT_KEY","verified":false,"used":false}""", headers = headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val api = ContentPublishingApi(client, CommunityConfig()) { "access" }
        assertFalse(api.awaitVerified(CONTENT_KEY, pauseMillis = 0))
        assertEquals(5, reads)
        assertFailsWith<IllegalArgumentException> { api.awaitVerified(CONTENT_KEY, attempts = 6) }
        client.close()
    }

    @Test
    fun `verified upload returns immediately while used upload asks user to inspect works`() = runTest {
        val server = FakeServer()
        server.on("/api/packs/upload-status", body = """{"file_key":"$CONTENT_KEY","verified":true,"used":false}""")
        val api = ContentPublishingApi(server.client, CommunityConfig()) { "access" }
        assertTrue(api.awaitVerified(CONTENT_KEY, pauseMillis = 0)); assertEquals(1, server.requests.size)
        server.on("/api/packs/upload-status", body = """{"file_key":"$CONTENT_KEY","verified":true,"used":true}""")
        assertEquals(409, assertFailsWith<ApiException> { api.awaitVerified(CONTENT_KEY, pauseMillis = 0) }.status)
        server.client.close()
    }

    @Test
    fun `unavailable scheme references omit private metadata and scheme detail 404 is preserved`() = runTest {
        val server = FakeServer()
        server.on("/api/schemes/5", body = """{"id":5,"title":"方案","summary":"说明","author":{"uid":"u","nickname":"作者"},"items":[{"pack_id":7,"version":1,"available":false}]}""")
        val api = ContentPublishingApi(server.client, CommunityConfig()) { null }
        val ref = api.scheme(5).items.single()
        assertFalse(ref.available); assertEquals("", ref.kind); assertEquals("", ref.slug); assertNull(ref.title)
        server.on("/api/schemes/5", HttpStatusCode.NotFound, """{"message":"方案不存在"}""")
        assertEquals(404, assertFailsWith<ApiException> { api.scheme(5) }.status)
        server.client.close()
    }

    @Test
    fun `owner methods require a token and backend refusal is preserved`() = runTest {
        val server = FakeServer(); val anonymous = ContentPublishingApi(server.client, CommunityConfig()) { null }
        assertEquals(401, assertFailsWith<ApiException> { anonymous.myPacks() }.status)
        assertEquals(401, assertFailsWith<ApiException> { anonymous.removeOwn(7) }.status)
        assertEquals(0, server.requests.size)
        server.on("/api/packs/7/remove", HttpStatusCode.NotFound, """{"message":"内容包不存在"}""")
        val api = ContentPublishingApi(server.client, CommunityConfig()) { "access" }
        assertEquals(404, assertFailsWith<ApiException> { api.removeOwn(7) }.status)
        assertEquals("Bearer access", server.requests.single().headers[HttpHeaders.Authorization])
        server.client.close()
    }

    @Test
    fun `work states and manifests preserve real values without inferred permissions or prompt`() {
        val base = Pack(7, "expert", title = "作品", trustLevel = 2)
        assertEquals("已发布 · 未审核", contentWorkState(base))
        assertEquals("审核中 · 未审核", contentWorkState(base.copy(inReview = true)))
        assertEquals("未通过审核", contentWorkState(base.copy(status = "rejected", inReview = true)))
        assertEquals("已下架", contentWorkState(base.copy(status = "removed")))
        assertEquals("已审核", contentWorkState(base.copy(trustLevel = 1)))
        assertNull(manifestValue(JsonObject(emptyMap()), "prompt", "permissions"))
        val manifest = JsonObject(mapOf("prompt" to JsonPrimitive("完整提示词\n第二行"), "runtime" to JsonPrimitive("desktop")))
        assertEquals("完整提示词\n第二行", manifestValue(manifest, "prompt"))
        assertEquals("desktop", manifestValue(manifest, "runtime"))
        assertNull(manifestValue(manifest, "permissions"))
        val pack = CommunityJson.decodeFromString<Pack>("""{"id":7,"kind":"expert","title":"作品","in_review":true,"versions":[{"version":1,"manifest":{"prompt":"实际提示词"}}]}""")
        assertTrue(pack.inReview); assertEquals("实际提示词", manifestValue(pack.versions.single().manifest, "prompt"))
    }
}
