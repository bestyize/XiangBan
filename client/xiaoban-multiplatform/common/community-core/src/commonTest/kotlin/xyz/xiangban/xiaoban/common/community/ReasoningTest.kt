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

    @Test
    fun customModelDefaultsToFiveEffortsAndRespectsExplicitSelection() {
        val defaultConfig = CustomModelConfig("https://api.deepseek.com/v1", "", "deepseek-chat", id = "p-1").validated()
        assertEquals(listOf("low", "medium", "high", "xhigh", "max"), defaultConfig.effectiveReasoningEfforts())

        val subset = defaultConfig.copy(reasoningEfforts = listOf("max", "low", "unknown")).validated()
        assertEquals(listOf("low", "max"), subset.effectiveReasoningEfforts())

        val disabled = defaultConfig.copy(reasoningEfforts = emptyList()).validated()
        assertEquals(emptyList(), disabled.effectiveReasoningEfforts())

        val managedFree = CustomModelConfig("https://api.example.com/v1", "", "free-1", id = "p-free", remoteId = "free", remoteType = "free").validated()
        assertEquals(emptyList(), managedFree.effectiveReasoningEfforts())
    }

    @Test
    fun responsesPlainBodyForwardsReasoningEffortWhenSet() {
        val cfg = CustomModelConfig("https://api.openai.com/v1", "", "o3", protocol = "responses", id = "p-resp").validated()
        val withEffort = Json.parseToJsonElement(
            customPlainBody("""{"model":"o3","reasoning_effort":"xhigh","messages":[{"role":"user","content":"hi"}]}""", cfg),
        ).jsonObject
        assertEquals("xhigh", withEffort["reasoning"]?.jsonObject?.get("effort")?.jsonPrimitive?.content)

        val withoutEffort = Json.parseToJsonElement(
            customPlainBody("""{"model":"o3","messages":[{"role":"user","content":"hi"}]}""", cfg),
        ).jsonObject
        assertFalse(withoutEffort.containsKey("reasoning"))
    }
}
