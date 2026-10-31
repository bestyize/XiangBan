package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.serialization.encodeToString
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

private fun liveModel(id: String, intelligence: Double? = null, input: Double? = null, output: Double? = null, speed: Double? = null) =
    LiveModel(id, id, id, "vendor", metrics = LiveModelMetrics(intelligence = intelligence),
        pricing = LiveModelPricing(input, output), performance = LiveModelPerformance(speed))
private fun liveBoard(models: List<LiveModel> = listOf(liveModel("first", 131.5, 1.0, 5.0))) = LiveModelBoard(
    source = "Artificial Analysis", sourceUrl = "https://artificialanalysis.ai/", asOf = "2026-10-31T02:00:00Z",
    indexVersion = "5.1", schemaVersion = "aa-free-v2", tier = "free", priceUnit = "USD / 1M tokens",
    vendors = listOf(LiveModelVendor("vendor", "厂商")), models = models,
)
private fun usageBoard() = ModelUsageBoard("OpenRouter", "2026-10-30", "2026-10-31T02:00:00Z", "https://openrouter.ai/rankings",
    "v1", "day", "UTC", "prompt_tokens + completion_tokens", "9007199254740993", "0",
    listOf(ModelUsageEntry("vendor/model:free", "模型", "9007199254740993", 1, 100.0)))

@OptIn(ExperimentalCoroutinesApi::class)
class ModelRankingsTest {
    @Test
    fun `real nullable metrics and prices round trip without old index fabrication`() {
        val board = liveBoard(listOf(liveModel("first", 131.5, input = 1.5)))
        val body = CommunityJson.encodeToString(board)
        assertTrue(body.contains("index_version")); assertTrue(body.contains("vendor_id"))
        val decoded = CommunityJson.decodeFromString<LiveModelBoard>(body)
        assertEquals("5.1", decoded.indexVersion)
        assertEquals(131.5, decoded.models.first().metrics.intelligence)
        assertNull(decoded.models.first().metrics.coding)
        assertNull(decoded.models.first().metrics.agentic)
        assertNull(decoded.models.first().pricing.output)
        assertNull(decoded.vendors.first().country)
        assertEquals(listOf(LiveModelMetric.Intelligence, LiveModelMetric.InputPrice), availableLiveMetrics(decoded))
    }

    @Test
    fun `null models remain unranked after measured zero and ties are stable by id`() {
        val models = listOf(liveModel("missing"), liveModel("z", 50.0), liveModel("a", 50.0), liveModel("zero", 0.0))
        val ranks = rankLiveModels(models, LiveModelMetric.Intelligence)
        assertEquals(listOf("a", "z", "zero", "missing"), ranks.map { it.model.id })
        assertEquals(listOf(1, 2, 3, null), ranks.map { it.rank })
        assertNull(ranks.last().value); assertNull(ranks.last().fraction)
        assertEquals(0.0, ranks[2].value)
        assertEquals(0f, ranks[2].fraction)
        assertEquals(1f, ranks.first().fraction)
    }

    @Test
    fun `prices rank low first and invalid measurements do not become zero`() {
        val models = listOf(liveModel("expensive", input = 20.0), liveModel("cheap", input = 1.0),
            liveModel("free", input = 0.0), liveModel("missing"), liveModel("invalid", input = Double.NaN))
        assertEquals(listOf("free", "cheap", "expensive", "invalid", "missing"), rankLiveModels(models, LiveModelMetric.InputPrice).map { it.model.id })
        assertNull(rankLiveModels(models, LiveModelMetric.InputPrice)[3].rank)
        assertNull(LiveModelMetric.Intelligence.value(liveModel("negative", -1.0)))
        assertNull(LiveModelMetric.Speed.value(liveModel("infinite", speed = Double.POSITIVE_INFINITY)))
        assertEquals(emptyList(), rankLiveModels(emptyList(), LiveModelMetric.Intelligence))
    }

    @Test
    fun `scatter requires real intelligence both prices and positive weighted price`() {
        val board = liveBoard(listOf(liveModel("valid", 131.5, 1.0, 5.0), liveModel("missing-output", 20.0, 2.0),
            liveModel("missing-input", 20.0, output = 1.0), liveModel("missing-score", input = 1.0, output = 1.0),
            liveModel("zero-price", 10.0, 0.0, 0.0), liveModel("zero-score", 0.0, 2.0, 2.0)))
        val points = liveScatterPoints(board)
        assertEquals(listOf("valid", "zero-score"), points.map { it.id })
        assertEquals(2.0, points.first().weightedPrice)
        assertEquals(131.5, points.first().intelligence)
        assertEquals(listOf(LiveModelMetric.Intelligence), availableLiveMetrics(liveBoard(listOf(liveModel("missing")))))
    }

    @Test
    fun `small measured prices retain their precision and never display as free`() {
        assertEquals("$0.001", formatLivePrice(0.001))
        assertEquals("$0", formatLivePrice(0.0))
        assertEquals("$1.5", formatLivePrice(1.5))
        assertEquals("暂无数据", formatLivePrice(null))
        assertEquals("暂无数据", formatLivePrice(Double.NaN))
    }

    @Test
    fun `token formatting and comparisons preserve arbitrary precision including beyond long`() {
        assertEquals("9,007,199,254,740,993", formatTokenCount("9007199254740993"))
        assertEquals("123,456,789,012,345,678,901,234,567,890", formatTokenCount("123456789012345678901234567890"))
        assertTrue(compareTokenCounts("9007199254740993", "9007199254740992") > 0)
        assertTrue(compareTokenCounts("10000000000000000000", "9999999999999999999") > 0)
        assertEquals(0, compareTokenCounts("0", "0"))
        for (bad in listOf("", "01", "-1", "1.5", "1e20", "NaN", "9".repeat(81))) {
            assertFalse(isTokenCount(bad)); assertEquals("暂无数据", formatTokenCount(bad))
        }
    }

    @Test
    fun `daily usage variants and previous day metadata round trip`() {
        val model = usageBoard().models.first().copy(previousRank = 4, rankChange = 3)
        val board = usageBoard().copy(models = listOf(model), previousDateAvailable = true)
        val decoded = CommunityJson.decodeFromString<ModelUsageBoard>(CommunityJson.encodeToString(board))
        assertEquals("9007199254740993", decoded.models.first().tokens)
        assertEquals("vendor/model:free", decoded.models.first().id)
        assertEquals("↑ 3", modelUsageMovement(decoded.models.first(), decoded.previousDateAvailable))
        assertEquals("暂无对比", modelUsageMovement(model.copy(newEntry = true), false))
        assertEquals("新入榜", modelUsageMovement(model.copy(newEntry = true), true))
        assertEquals("↓ 2", modelUsageMovement(model.copy(rankChange = -2), true))
        assertEquals("—", modelUsageMovement(model.copy(rankChange = 0), true))
        assertEquals("暂无对比", modelUsageMovement(model.copy(rankChange = null), true))
    }

    @Test
    fun `api uses three published endpoints with explicit date and no legacy fallback`() = runTest {
        val server = FakeServer()
        server.on("/api/discover/models/v2", body = CommunityJson.encodeToString(liveBoard()))
        server.on({ it.url.encodedPath == "/api/discover/model-usage" }, body = CommunityJson.encodeToString(usageBoard()))
        server.on("/api/discover/model-usage/dates", body = """{"items":[{"date":"2026-10-30","updated_at":"2026-10-31T02:00:00Z"}]}""")
        val api = ForumApi(server.client, CommunityConfig()) { null }
        assertEquals("5.1", api.liveModelBoard().indexVersion)
        assertEquals("2026-10-30", api.modelUsage("2026-10-30").date)
        assertEquals("2026-10-30", server.requests[1].url.parameters["date"])
        assertEquals("2026-10-30", api.modelUsageDates().items.single().date)
        api.modelUsage()
        assertNull(server.requests.last().url.parameters["date"])
        assertEquals(4, server.requests.size)
        server.client.close()
    }

    @Test
    fun `unpublished 404 stays an empty state and does not fetch sample board`() = runTest {
        val server = FakeServer()
        server.on("/api/discover/models/v2", HttpStatusCode.NotFound, """{"message":"尚未发布"}""")
        val api = ForumApi(server.client, CommunityConfig()) { null }
        val loader = Loader(this) { api.liveModelBoard() }
        loader.reload()
        assertTrue(assertIs<Load.Failed>(loader.state.first { it !is Load.Loading }).notFound)
        assertEquals(listOf("/api/discover/models/v2"), server.requests.map { it.url.encodedPath })
        server.on("/api/discover/models/v2", body = CommunityJson.encodeToString(liveBoard()))
        loader.reload()
        assertIs<Load.Ready<LiveModelBoard>>(loader.state.first { it !is Load.Loading })
        server.client.close()
    }

    @Test
    fun `offline load exposes retry and successful retry supplies real data`() = runTest {
        var online = false
        val loader = Loader(this) { if (online) liveBoard() else throw ApiException(0, "网络连接失败，请重试") }
        loader.reload(); advanceUntilIdle()
        assertFalse(assertIs<Load.Failed>(loader.state.value).notFound)
        online = true
        loader.reload(); advanceUntilIdle()
        assertEquals("5.1", assertIs<Load.Ready<LiveModelBoard>>(loader.state.value).value.indexVersion)
    }
}
