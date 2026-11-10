package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.HttpTimeoutCapability
import io.ktor.client.plugins.HttpTimeoutConfig
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.headersOf
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.delay
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.io.encoding.Base64
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlin.test.assertFailsWith

class ImageGenerationTest {
    private val encoded = "R0lGODlhAQABAIAAAP8AAAAA/yH/C05FVFNDQVBFMi4wAwEAAAAh+QQAFAAAACwAAAAAAQABAAACAkQBACH5BAAUAAAALAAAAAABAAEAAAICTAEAOw=="

    @Test fun `background image completion stays with its original conversation`() = runBlocking {
        val started = CompletableDeferred<Unit>()
        val finish = CompletableDeferred<Unit>()
        val png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/lDsAAAAASUVORK5CYII="
        val client = HttpClient(MockEngine { request ->
            when (request.url.encodedPath) {
                "/v1/images/jobs" -> {
                    val id = CommunityJson.parseToJsonElement((request.body as TextContent).text).jsonObject["job_id"]!!.jsonPrimitive.content
                    started.complete(Unit)
                    finish.await()
                    respond("""{"job_id":"$id","status":"succeeded","result":{"data":[{"b64_json":"$png"}]}}""")
                }
                else -> respond("{}")
            }
        })
        val store = ChatHistory(InMemoryChatHistoryFiles(), this)
        val chat = CloudChatController(AiGatewayApi(client) { "token" }, this, history = store)
        try {
            assertTrue(chat.generateImages("poster", 1, null))
            withTimeout(5_000) { started.await() }
            assertTrue(chat.canLeaveForCreation())
            val imageChat = chat.state.value.conversationId
            chat.clearConversation()
            assertFalse(chat.state.value.sending)
            assertEquals(1, chat.state.value.backgroundImageCount)
            chat.openConversation(imageChat)
            assertTrue(chat.state.value.sending)
            chat.clearConversation()
            val newChat = chat.state.value.conversationId
            finish.complete(Unit)
            withTimeout(5_000) { while (chat.state.value.backgroundImageCount > 0) delay(10) }
            assertEquals(newChat, chat.state.value.conversationId)
            assertTrue(chat.state.value.messages.isEmpty())
            chat.openConversation(imageChat)
            assertEquals(1, chat.state.value.messages.last().generatedImages.size)
        } finally {
            chat.reset()
            client.close()
            coroutineContext[kotlinx.coroutines.Job]?.children?.forEach { it.cancel() }
        }
    }

    @Test fun `GIF capability is optional and animated results preserve original bytes`() = runBlocking {
        val old = CommunityJson.decodeFromString<AiTiersResponse>("""{"media":{"image":{"available":true,"sizes":["1024x1024"]}}}""")
        assertEquals(listOf("png"), old.media["image"]!!.formats)
        val requests = mutableListOf<String>()
        val client = HttpClient(MockEngine { request ->
            assertEquals("Bearer test-only-token", request.headers[HttpHeaders.Authorization])
            requests += (request.body as TextContent).text
            respond("""{"data":[{"b64_json":"$encoded"}]}""", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType,"application/json"))
        }) {install(HttpTimeout)}
        try {
            val api = AiGatewayApi(client) {"test-only-token"}
            val result = api.generateImages("猫挥手循环",1,"1024x1024","gif",9,200)
            assertEquals(listOf("data:image/gif;base64,$encoded"), result)
            val payload = CommunityJson.parseToJsonElement(requests.single()).jsonObject
            assertEquals("gif",payload["output_format"]!!.jsonPrimitive.content)
            assertEquals("9",payload["frame_count"]!!.jsonPrimitive.content)
            assertFalse(payload.containsKey("model"))
        } finally {client.close()}
    }

    @Test fun `image requests wait for the first byte as long as the whole request`() = runBlocking {
        var timeouts: HttpTimeoutConfig? = null
        val client = HttpClient(MockEngine { request ->
            timeouts = request.getCapabilityOrNull(HttpTimeoutCapability)
            respond("""{"data":[{"b64_json":"$encoded"}]}""", HttpStatusCode.OK, headersOf(HttpHeaders.ContentType,"application/json"))
        }) {install(HttpTimeout)}
        try {
            AiGatewayApi(client) {"test-token"}.generateImages("cat",1,null)
            val config = checkNotNull(timeouts) {"the request carried no timeout configuration"}
            assertEquals(240_000L, config.requestTimeoutMillis)
            // 图片接口出图前没有任何数据，读超时必须不短于整体超时，否则 OkHttp 默认的 10 秒会先断开。
            assertEquals(240_000L, config.socketTimeoutMillis)
        } finally {client.close()}
    }

    @Test fun `invalid GIF parameters never send a billable request`() = runBlocking {
        var sent = 0
        val client = HttpClient(MockEngine { sent++; respond("{}") }) {install(HttpTimeout)}
        try {
            val api = AiGatewayApi(client) {"test-token"}
            assertFailsWith<IllegalArgumentException> {api.generateImages("cat",2,null,"gif")}
            assertFailsWith<IllegalArgumentException> {api.generateImages("cat",1,null,"gif",17)}
            assertFailsWith<IllegalArgumentException> {api.generateImages("cat",1,null,"gif",9,19)}
            assertEquals(0,sent)
        } finally {client.close()}
    }

    @Test fun `an upstream error is unknown and is never retried automatically`() = runBlocking {
        var sent = 0
        val client = HttpClient(MockEngine { sent++; respond("""{"code":"bad_gateway"}""",HttpStatusCode.BadGateway,headersOf(HttpHeaders.ContentType,"application/json")) }) {install(HttpTimeout)}
        try {
            val api = AiGatewayApi(client) {"test-token"}
            val error = assertFailsWith<ApiException> {api.generateImages("cat",1,null,"gif")}
            assertTrue(error.message.orEmpty().contains("可能已扣除额度"))
            assertEquals(1,sent)
        } finally {client.close()}
    }

    @Test fun `GIF validation bounds frames pixels and malformed data before preview`() {
        val bytes = Base64.decode(encoded)
        assertTrue(validGeneratedGif(bytes))
        assertFalse(validGeneratedGif(bytes.copyOf(45)))
        val giant = bytes.copyOf(); giant[6] = (-1).toByte(); giant[7] = (-1).toByte()
        assertFalse(validGeneratedGif(giant))
    }
}
