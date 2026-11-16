package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.test.assertFailsWith

class ModelCatalogueTest {
    private fun directory(default: String = "one", models: String = """{"id":"one","name":"One"}""", providers: String? = null): String =
        """{"version":1,"refreshAfterSeconds":300,"providers":[${providers ?: """{"id":"server-new","provider":"来自服务器","endPoint":"https://provider.example/v1","protocol":"OpenAI Chat Completions","type":"third","defaultModel":"$default","models":[$models]}"""}]}"""

    @Test fun `remote updates preserve user settings and retire removed entries`() = runBlocking {
        var body = directory()
        val client = HttpClient(MockEngine { respond(body, headers = headersOf(HttpHeaders.ContentType, "application/json")) })
        val storage = InMemoryCustomModelStorage()
        val api = AiGatewayApi(client, customModels = storage) { null }
        api.saveCustomProvider(CustomModelConfig("https://local.example", "local-key", "local", id = "local"))
        api.refreshModelCatalogue(true)
        assertEquals(listOf("来自服务器"), api.presetViews().map { it.name })
        val initial = assertNotNull(api.presetConfig("server-new"))
        api.saveCustomProvider(initial.copy(baseUrl = "https://proxy.example/v1", apiKey = "my-key", models = listOf("one", "mine"), model = "mine", followRecommended = false))
        body = directory("two", """{"id":"two","name":"Two"}""")
        api.refreshModelCatalogue(true)
        val updated = assertNotNull(api.customProvider(initial.id))
        assertEquals("https://proxy.example/v1", updated.baseUrl)
        assertEquals("my-key", updated.apiKey)
        assertEquals(listOf("two", "one", "mine"), updated.models)
        assertEquals("mine", updated.model)
        body = directory(providers = "")
        api.refreshModelCatalogue(true)
        assertFalse(assertNotNull(api.customProvider(initial.id)).remoteAvailable)
        assertEquals("my-key", api.customProvider(initial.id)?.apiKey)
        assertTrue(api.providerAvailable(assertNotNull(api.customProvider("local"))))
        client.close()
    }

    @Test fun `etag and failed responses retain the cached directory`() = runBlocking {
        var calls = 0
        val client = HttpClient(MockEngine { request ->
            calls++
            when (calls) {
                1 -> respond(directory(), headers = headersOf(HttpHeaders.ETag, "\"v1\""))
                2 -> { assertEquals("\"v1\"", request.headers[HttpHeaders.IfNoneMatch]); respond("", HttpStatusCode.NotModified) }
                else -> respond("{bad json}")
            }
        })
        val storage = InMemoryCustomModelStorage()
        val api = AiGatewayApi(client, customModels = storage) { null }
        api.refreshModelCatalogue(true)
        api.refreshModelCatalogue(true)
        assertFailsWith<Exception> { api.refreshModelCatalogue(true) }
        assertEquals("server-new", api.presetViews().single().id)
        val restarted = AiGatewayApi(client, customModels = storage) { null }
        assertEquals("server-new", restarted.presetViews().single().id)
        client.close()
    }

    @Test fun `recommended selection follows a new server default`() = runBlocking {
        var body = directory()
        val client = HttpClient(MockEngine { respond(body) })
        val api = AiGatewayApi(client) { null }
        val chat = CloudChatController(api, this)
        chat.useAccount(null)
        chat.refreshModelCatalogue(true)
        val draft = assertNotNull(chat.presetDraft("server-new"))
        chat.saveCustomProvider(draft)
        body = directory("two", """{"id":"one","name":"One"},{"id":"two","name":"Two"}""")
        chat.refreshModelCatalogue(true)
        assertEquals(customModelTier(draft.id, "two"), chat.state.value.selectedTier)
        client.close()
    }

    @Test fun `account credentials never survive logout or appear in preset views`() = runBlocking {
        val paid = """{"id":"managed","provider":"Managed","endPoint":"https://provider.example/v1","protocol":"OpenAI Responses","type":"paid","defaultModel":"one","models":[{"id":"one","name":"One"}],"apikey":"provider-secret"}"""
        val client = HttpClient(MockEngine { request ->
            if (request.url.encodedPath.endsWith("/ai/providers")) {
                assertEquals("Bearer account-token", request.headers[HttpHeaders.Authorization])
                respond(directory(providers = paid))
            } else respond("{}")
        })
        val session = SessionManager(AccountApi(client, CommunityConfig()), InMemoryTokenStorage(Tokens("account-token", "refresh", 999_999, "owner-a")), now = { 0 })
        val storage = InMemoryCustomModelStorage()
        val api = AiGatewayApi(client, sessions = session, customModels = storage) { null }
        api.refreshModelCatalogue(true)
        assertFalse(api.presetViews().toString().contains("provider-secret"))
        assertTrue(storage.loadCatalog().presets.isEmpty())
        val config = assertNotNull(api.presetConfig("managed"))
        api.saveCustomProvider(config)
        assertTrue(api.providerAvailable(config))
        session.logout()
        assertFalse(api.providerAvailable(config))
        assertEquals(0, api.postChatRaw("""{"model":"${customModelTier(config.id, config.model)}"}""") {}.status)
        api.accountChanged(null)
        assertEquals("", api.customProvider(config.id)?.apiKey)
        assertNull(api.presetConfig("managed"))
        client.close()
    }
}
