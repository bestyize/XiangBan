package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.content.TextContent
import kotlinx.coroutines.flow.first
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
    fun officialDefaultsPersistAndRemainSeparateFromConversationOverrides() = runTest {
        val server = FakeServer()
        server.on("/api/ai/tiers", body = """{"tiers":[
            {"tier":"auto","display_name":"Auto","is_default":true,"reasoning_efforts":["low","medium","high"]},
            {"tier":"other","display_name":"Other"}
        ]}""")
        server.on("/api/ai/balance", body = """{"total":10000}""")
        val storage = InMemoryCustomModelStorage()
        fun controller() = CloudChatController(AiGatewayApi(server.client, customModels = storage) { "access" }, this)
        val chat = controller()
        chat.useAccount("owner")
        chat.loadMeta()
        assertEquals(2, chat.state.first { !it.loadingMeta }.tiers.size)
        chat.configureOfficialModel("auto", "high")
        assertEquals("high", chat.state.value.reasoningByTier["auto"])
        chat.selectReasoning("auto", "low")
        assertEquals("low", chat.state.value.reasoningByTier["auto"])
        assertEquals("high", chat.state.value.officialReasoningByTier["auto"])
        chat.configureOfficialModel("other")
        assertEquals("other", chat.state.value.defaultOfficialModel?.tier)
        chat.configureOfficialModel("auto")
        assertEquals("high", chat.state.value.reasoningByTier["auto"])
        val restarted = controller()
        restarted.useAccount("owner")
        restarted.loadMeta()
        assertEquals(2, restarted.state.first { !it.loadingMeta }.tiers.size)
        assertEquals("auto", restarted.state.value.selectedTier)
        assertEquals("high", restarted.state.value.reasoningByTier["auto"])
        restarted.configureOfficialModel("auto", null)
        assertNull(storage.loadCatalog().officialReasoningByTier["auto"])
        assertNull(restarted.state.value.reasoningByTier["auto"])
        server.client.close()
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
