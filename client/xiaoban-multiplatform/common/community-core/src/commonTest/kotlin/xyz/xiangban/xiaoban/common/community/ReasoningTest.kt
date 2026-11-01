package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.content.TextContent
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.*
import kotlin.test.*

class ReasoningTest {
    @Test
    fun agentRequestSetsAndClearsReasoningWithoutChangingTools() {
        val body = """{"model":"tier","tools":[{"type":"function"}],"reasoning_effort":"low"}"""
        val changed = Json.parseToJsonElement(withReasoningEffort(body, "high")).jsonObject
        assertEquals("high", changed["reasoning_effort"]?.jsonPrimitive?.content)
        assertEquals(Json.parseToJsonElement(body).jsonObject["tools"], changed["tools"])
        assertFalse(Json.parseToJsonElement(withReasoningEffort(body, null)).jsonObject.containsKey("reasoning_effort"))
    }

    @Test
    fun streamingRequestIncludesExplicitEffortAndOmitsDefault() = runTest {
        val bodies = mutableListOf<JsonObject>()
        val client = HttpClient(MockEngine { request ->
            bodies += Json.parseToJsonElement((request.body as TextContent).text).jsonObject
            respond("data: [DONE]\n\n")
        })
        try {
            val api = AiGatewayApi(client, CommunityConfig()) { "test-token" }
            api.streamChat(listOf(ChatMessage("user", "hello")), "tier", "high")
            api.streamChat(listOf(ChatMessage("user", "hello")), "tier")
            assertEquals("high", bodies[0]["reasoning_effort"]?.jsonPrimitive?.content)
            assertFalse(bodies[1].containsKey("reasoning_effort"))
        } finally { client.close() }
    }
}
