package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.*
import kotlin.io.encoding.Base64
import kotlin.test.*

class ImageAgentToolsTest {
    private val png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/lDsAAAAASUVORK5CYII="
    private fun args(text: String) = Json.parseToJsonElement(text).jsonObject
    private fun capabilities() = """{"tiers":[],"media":{"image":{"available":true,"formats":["png","gif"],"supports_jobs":true}}}"""

    @Test fun `agent generation saves a draft asset and restart recovers without a second paid submission`() = runBlocking {
        val files = InMemoryChatHistoryFiles()
        val workspace = MemoryMobileWorkspace()
        var submitted = 0
        var queried = 0
        var jobId = ""
        val client = HttpClient(MockEngine { request ->
            when {
                request.url.encodedPath == "/api/ai/tiers" -> respond(capabilities())
                request.method == HttpMethod.Post -> {
                    assertNotNull(files.read(imageToolJobFile("session-a")), "receipt must be durable before POST")
                    submitted++
                    jobId = args((request.body as TextContent).text).getValue("job_id").jsonPrimitive.content
                    respond("""{"job_id":"$jobId","status":"succeeded","result":{"data":[{"b64_json":"$png"}]}}""")
                }
                else -> {
                    queried++
                    assertTrue(request.url.encodedPath.endsWith("/$jobId"))
                    respond("""{"job_id":"$jobId","status":"succeeded","result":{"data":[{"b64_json":"$png"}]}}""")
                }
            }
        })
        try {
            val api = AiGatewayApi(client) { "test-only-token" }
            val input = args("""{"request_id":"cover-v1","prompt":"蓝色小鸟","source_dir":"drafts/bird","path":"assets/Hero.PNG"}""")
            withContext(AgentToolContext("session-a", "t1")) {
                val result = imageAgentTools(api, files, { "owner-a" }, workspace).first().run(input)
                assertTrue(result.ok, result.output)
                assertEquals(1, result.generatedImages.size)
                assertFalse(result.output.contains(png), "base64 must not enter model text")
                assertContentEquals(Base64.decode(png), workspace.readBytes("drafts/bird/assets/Hero.PNG"))
                val restored = imageAgentTools(api, files, { "owner-a" }, workspace).first().run(input)
                assertTrue(restored.ok, restored.output)
            }
            assertEquals(1, submitted)
            assertEquals(1, queried)
        } finally { client.close() }
    }

    @Test fun `interrupted submission reuses its stable id and exposes original arguments for recovery`() = runBlocking {
        val files = InMemoryChatHistoryFiles()
        val submittedIds = mutableListOf<String>()
        var first = true
        val client = HttpClient(MockEngine { request ->
            when {
                request.url.encodedPath == "/api/ai/tiers" -> respond(capabilities())
                request.method == HttpMethod.Post -> {
                    val id = args((request.body as TextContent).text).getValue("job_id").jsonPrimitive.content
                    submittedIds += id
                    if (first) { first = false; throw IllegalStateException("connection lost") }
                    respond("""{"job_id":"$id","status":"succeeded","result":{"data":[{"b64_json":"$png"}]}}""")
                }
                else -> respond("""{"error":{"message":"job not found"}}""", HttpStatusCode.NotFound)
            }
        })
        try {
            val api = AiGatewayApi(client) { "test-only-token" }
            withContext(AgentToolContext("session-a", "t1")) {
                val tools = imageAgentTools(api, files, { "owner-a" }).associateBy { it.name }
                val original = args("""{"request_id":"cover","prompt":"蓝色小鸟","format":"png"}""")
                val failed = tools.getValue("image_generate").run(original)
                assertFalse(failed.ok)
                assertContains(failed.output, "request_id")
                val list = tools.getValue("image_jobs").run(args("{}"))
                val recovery = args(list.output).getValue("jobs").jsonArray.single().jsonObject.getValue("arguments").jsonObject
                assertEquals(original, recovery)
                val recovered = imageAgentTools(api, files, { "owner-a" }).first().run(recovery)
                assertTrue(recovered.ok, recovered.output)
                assertEquals(2, submittedIds.size)
                assertEquals(submittedIds[0], submittedIds[1], "a retry must use the same server idempotency key")
            }
        } finally { client.close() }
    }

    @Test fun `invalid asset path changed parameters and another account cannot reuse a paid task`() = runBlocking {
        var submitted = 0
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath == "/api/ai/tiers") respond(capabilities()) else {
                submitted++
                val id = args((request.body as TextContent).text).getValue("job_id").jsonPrimitive.content
                respond("""{"job_id":"$id","status":"succeeded","result":{"data":[{"b64_json":"$png"}]}}""")
            }
        })
        val files = InMemoryChatHistoryFiles()
        try {
            val api = AiGatewayApi(client) { "test-only-token" }
            withContext(AgentToolContext("session-a", "t1")) {
                val tool = imageAgentTools(api, files, { "owner-a" }, MemoryMobileWorkspace()).first()
                assertFailsWith<IllegalArgumentException> {
                    tool.run(args("""{"request_id":"cover","prompt":"cat","source_dir":"../escape","path":"hero.png"}"""))
                }
                assertEquals(0, submitted)
                assertTrue(tool.run(args("""{"request_id":"cover","prompt":"cat"}""")).ok)
                assertFailsWith<IllegalArgumentException> { tool.run(args("""{"request_id":"cover","prompt":"dog"}""")) }
                assertEquals(1, submitted)
                val other = imageAgentTools(api, files, { "owner-b" }).last().run(args("{}"))
                assertTrue(args(other.output).getValue("jobs").jsonArray.isEmpty())
            }
        } finally { client.close() }
    }

    @Test fun `receipt save failure prevents paid submission`() = runBlocking {
        var paid = 0
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath == "/api/ai/tiers") respond(capabilities()) else { paid++; error("must not submit") }
        })
        val files = object : ChatHistoryFiles {
            override fun read(name: String): String? = null
            override fun write(name: String, text: String) = error("disk full")
            override fun delete(name: String) = Unit
        }
        try {
            withContext(AgentToolContext("session-a", "t1")) {
                assertFailsWith<IllegalStateException> {
                    imageAgentTools(AiGatewayApi(client) { "test-only-token" }, files, { "owner-a" }).first()
                        .run(args("""{"request_id":"cover","prompt":"cat"}"""))
                }
            }
            assertEquals(0, paid)
        } finally { client.close() }
    }
}
