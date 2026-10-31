package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpMethod
import io.ktor.http.content.TextContent
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.Json
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private val CONFIG = CommunityConfig()

private fun model(key: String, iq: Double, price: Double = 1.0, speed: Double = 100.0, reasoning: Boolean = false, open: Boolean = false, cn: Boolean = false) =
    ModelEntry(key = key, name = key, vendor = "v", iq = iq, code = iq - 3, math = iq - 6, agent = iq - 2, price = price, speed = speed, reasoning = reasoning, open = open, cn = cn)

private val TIERS = listOf(TierDef("S", "第一梯队", 70.0), TierDef("A", "第二梯队", 63.0), TierDef("B", "第三梯队", 56.0), TierDef("C", "第四梯队", null))

class LeaderboardTest {
    private val models = listOf(
        model("a", 72.0, price = 4.0, speed = 90.0, reasoning = true),
        model("b", 66.0, price = 1.2, speed = 140.0, reasoning = true, cn = true),
        model("c", 58.0, price = 0.3, speed = 210.0, open = true),
        model("d", 51.0, price = 0.15, speed = 260.0, cn = true),
    )

    @Test
    fun `scores rank high to low and bar length follows the score on a 0 to 100 scale`() {
        val rows = rankModels(models.shuffled(), Metric.Iq)
        assertEquals(listOf("a", "b", "c", "d"), rows.map { it.model.key })
        assertEquals(listOf(1, 2, 3, 4), rows.map { it.rank })
        assertEquals(0.72f, rows[0].fraction, 1e-6f)
        assertEquals(0.51f, rows[3].fraction, 1e-6f)
        assertTrue(rows.zipWithNext().all { (x, y) -> x.fraction > y.fraction })
    }

    @Test
    fun `price ranks cheap first and speed ranks fast first, longest bar fills the width`() {
        val price = rankModels(models, Metric.Price)
        assertEquals(listOf("d", "c", "b", "a"), price.map { it.model.key })
        assertEquals(1f, price.last().fraction, 1e-6f)
        val speed = rankModels(models, Metric.Speed)
        assertEquals(listOf("d", "c", "b", "a"), speed.map { it.model.key })
        assertEquals(1f, speed.first().fraction, 1e-6f)
    }

    @Test
    fun `filters keep the ranking order and renumber`() {
        assertEquals(listOf("a", "b"), rankModels(models, Metric.Iq, ModelFilter.Reasoning).map { it.model.key })
        assertEquals(listOf("c"), rankModels(models, Metric.Iq, ModelFilter.Open).map { it.model.key })
        val cn = rankModels(models, Metric.Iq, ModelFilter.Cn)
        assertEquals(listOf("b", "d"), cn.map { it.model.key })
        assertEquals(listOf(1, 2), cn.map { it.rank })
        assertTrue(rankModels(emptyList(), Metric.Price).isEmpty())
    }

    @Test
    fun `tiers use the thresholds from the server and drop empty tiers`() {
        val groups = groupByTier(rankModels(models, Metric.Iq), TIERS)
        assertEquals(listOf("S", "A", "B", "C"), groups.map { it.tier.id })
        assertEquals(listOf("a"), groups[0].rows.map { it.model.key })
        assertEquals(listOf("b"), groups[1].rows.map { it.model.key })
        val partial = groupByTier(rankModels(models.take(1) + models.last(), Metric.Iq), TIERS)
        assertEquals(listOf("S", "C"), partial.map { it.tier.id })
        assertTrue(groupByTier(rankModels(models, Metric.Iq), emptyList()).isEmpty())
        // 边界值落在上一档。
        val edge = groupByTier(rankModels(listOf(model("x", 70.0), model("y", 63.0), model("z", 56.0)), Metric.Iq), TIERS)
        assertEquals(listOf("S", "A", "B"), edge.map { it.tier.id })
    }

    @Test
    fun `axis positions clamp to the plot`() {
        assertEquals(0f, logFraction(0.1, 0.1, 10.0), 1e-6f)
        assertEquals(1f, logFraction(10.0, 0.1, 10.0), 1e-6f)
        assertEquals(0.5f, logFraction(1.0, 0.1, 10.0), 1e-6f)
        assertEquals(0f, logFraction(0.01, 0.1, 10.0))
        assertEquals(1f, logFraction(99.0, 0.1, 10.0))
        assertEquals(0f, logFraction(-1.0, 0.1, 10.0))
        assertEquals(0.5f, linearFraction(60.0, 45.0, 75.0), 1e-6f)
        assertEquals(1f, linearFraction(80.0, 45.0, 75.0))
    }

    @Test
    fun `scatter axes default to the design range and grow with the data`() {
        val normal = listOf(model("a", 72.0, price = 4.0), model("b", 50.0, price = 0.15))
        assertEquals(AxisRange(45.0, 75.0), scoreAxis(normal))
        assertEquals(AxisRange(0.1, 10.0), priceAxis(normal))
        assertEquals(listOf(0.1, 0.3, 1.0, 3.0, 10.0), priceTicks(priceAxis(normal)).map { (it * 1000).toLong() / 1000.0 })
        val wide = listOf(model("a", 88.0, price = 40.0), model("b", 38.0, price = 0.02))
        assertEquals(AxisRange(35.0, 90.0), scoreAxis(wide))
        val range = priceAxis(wide)
        assertEquals(0.01, range.lo, 1e-9)
        assertEquals(100.0, range.hi, 1e-9)
        assertEquals(AxisRange(45.0, 75.0), scoreAxis(emptyList()))
        assertEquals("$0.1", formatTick(0.1))
        assertEquals("$0.03", formatTick(0.03))
        assertEquals("$3", formatTick(3.0))
        assertEquals("$10", formatTick(10.0))
    }

    @Test
    fun `numbers are formatted like the design`() {
        assertEquals("$0.15", formatPrice(0.15))
        assertEquals("$0.30", formatPrice(0.3))
        assertEquals("$3.4", formatPrice(3.4))
        assertEquals("$10.0", formatPrice(10.0))
        assertEquals("72", formatScore(72.0))
        assertEquals("72.4", formatScore(72.4))
        assertEquals("+3,214", formatGain(3214))
        assertEquals("+86,420", formatGain(86420))
        assertEquals("-12", formatGain(-12))
        assertEquals("+0", formatGain(0))
        assertEquals("12.4k", formatStars(12400))
        assertEquals("999", formatStars(999))
        assertEquals("12.0万", formatStars(120_000))
    }
}

class DiscoverApiTest {
    private fun scope() = CoroutineScope(Dispatchers.Unconfined)

    @Test
    fun `home bundle parses banners, news, featured packs and the top repos`() = runTest {
        val server = FakeServer()
        server.on(
            "/api/discover/home",
            body = """{"banners":[{"key":"b1","tag":"本周原创","title":"标题","byline":"小伴编辑部","published_at":"2026-10-31T00:00:00Z","sample":true}],
              "news":{"date":"2026-10-31","total":3,"items":[{"key":"n1","title":"新闻","source":"来源","url":"https://x.test/a","published_at":"2026-10-31T01:00:00Z","sample":true}]},
              "featured":[{"id":7,"kind":"effect","title":"极光","version":1}],
              "repos":{"key":"1","issue_no":1,"ranks":[{"rank":1,"full_name":"a/b","star_total":12400,"star_gain":3214,"spark":[1,2,3]}],"sample":true},
              "sample":true}""",
        )
        val home = ForumApi(server.client, CONFIG) { null }.home()
        assertEquals("标题", home.banners.single().title)
        assertEquals(3, home.news.total)
        assertEquals("新闻", home.news.items.single().title)
        assertEquals(listOf(7L), home.featured.map { it.id })
        assertEquals("a/b", home.repos?.ranks?.single()?.fullName)
        assertTrue(home.sample)
    }

    @Test
    fun `missing optional parts of the home bundle fall back to empty`() = runTest {
        val server = FakeServer()
        server.on("/api/discover/home", body = """{"banners":[],"news":{"date":null,"total":0,"items":[]},"featured":[],"repos":null,"sample":false}""")
        val home = ForumApi(server.client, CONFIG) { null }.home()
        assertTrue(home.banners.isEmpty() && home.featured.isEmpty())
        assertNull(home.repos)
        assertNull(home.news.date)
    }

    @Test
    fun `model board, tracker, nav and docs map to their endpoints`() = runTest {
        val server = FakeServer()
        server.on(
            "/api/discover/models",
            body = """{"vendors":[{"key":"v","name":"厂商","brand_color":"#2F6BFF","tint":"#E4ECFF"}],
              "models":[{"key":"m","name":"模型","vendor":"v","iq":72.5,"price":3.4,"speed":90,"reasoning":true,"tier":"S"}],
              "tiers":[{"id":"S","label":"第一梯队","min":70.0},{"id":"C","label":"第四梯队","min":null}],
              "source":"Artificial Analysis","updated_at":"2026-10-31T00:00:00Z","sample":true}""",
        )
        server.on(
            "/api/discover/tracker",
            body = """{"quota":{"key":"codex","progress_pct":68,"next_reset_at":"2026-11-01T10:00:00Z","recent_intervals":[5,5.5],"published_at":"2026-10-31T09:55:00Z"},
              "launches":[{"key":"l","title":"上新","tags":["API"],"published_at":"2026-10-31T00:00:00Z"}],
              "promos":[{"key":"p","text":"免费","ends_at":"2026-11-03T00:00:00Z","pct":57,"days_left":3}],"now":"2026-10-31T10:00:00Z","sample":false}""",
        )
        server.on("/api/discover/nav", body = """{"groups":[{"name":"海外","items":[{"key":"claude","group":"海外","name":"Claude","vendor":"Anthropic","url":"https://claude.ai/"}]}]}""")
        server.on("/api/discover/docs", body = """{"groups":[{"name":"Claude","items":[{"key":"d","title":"文档","label":"index","url":"https://docs.test"}]}]}""")
        val api = ForumApi(server.client, CONFIG) { null }

        val board = api.modelBoard()
        assertEquals("厂商", board.vendor("v")?.name)
        assertEquals(72.5, board.models.single().iq)
        assertNull(board.tiers.last().min)
        assertEquals(70.0, board.tiers.first().min)

        val tracker = api.tracker()
        assertEquals(68, tracker.quota?.progressPct)
        assertEquals(listOf(5.0, 5.5), tracker.quota?.recentIntervals)
        assertEquals(3, tracker.promos.single().daysLeft)

        assertEquals("Claude", api.navLinks().groups.single().items.single().name)
        assertEquals("索引", api.docs().groups.single().items.single().labelText)
    }

    @Test
    fun `repo issue can be asked for by key and the issue list is parsed`() = runTest {
        val server = FakeServer()
        server.on("/api/discover/repos", body = """{"key":"1","issue_no":1,"week_start":"10.23","week_end":"10.29","total_gain":86420,"wow_pct":12.5,"daily_gain":[1,2,3,4,5,6,7],"ranks":[]}""")
        server.on("/api/discover/repos/issues", body = """{"items":[{"key":"2","issue_no":2,"week_start":"10.30","week_end":"11.05"},{"key":"1","issue_no":1}]}""")
        val api = ForumApi(server.client, CONFIG) { null }
        assertEquals(listOf(2, 1), api.repoIssues().items.map { it.issueNo })
        val issue = api.repoIssue("1")
        assertEquals(86420L, issue.totalGain)
        assertEquals(12.5, issue.wowPct)
        assertEquals("1", server.requests.last().url.parameters["issue"])
    }

    @Test
    fun `tools list and detail expose license text, mirror availability and artifacts`() = runTest {
        val server = FakeServer()
        server.on("/api/tools", body = """{"items":[{"key":"cc-switch","name":"CC Switch","license":"unknown","latest_version":"v3.5.0","size":12582912}]}""")
        server.on("/api/tools/cc-switch", body = """{"key":"cc-switch","name":"CC Switch","license":"open","description":"切换配置","latest_version":"v3.5.0",
              "releases":[{"version":"v3.5.0","date":"10-20","items":[{"kind":"new","text":"支持 Codex"}]}],
              "artifacts":[{"os":"windows","desc":".msi 安装包","size":12582912,"sha256":"ab","mirror_url":"https://mirror.test/a.msi","github_url":"https://github.com/o/r/releases"},
                           {"os":"macos","desc":".dmg","size":1,"github_url":"https://github.com/o/r/releases"}],
              "discussion_count":12}""")
        val api = ForumApi(server.client, CONFIG) { null }
        val list = api.tools().items.single()
        assertEquals("待确认", list.licenseText)
        assertEquals("v3.5.0", list.latestVersion)
        val tool = api.tool("cc-switch")
        assertEquals("开源", tool.licenseText)
        assertTrue(tool.hasMirror)
        assertEquals(listOf("Windows", "macOS"), tool.artifacts.map { it.osLabel })
        assertEquals(12, tool.discussionCount)
        assertFalse(tool.artifacts[1].mirrorUrl.isNotEmpty())
    }

    @Test
    fun `a tool without any mirror link has no mirror source`() {
        val tool = ToolDetail(key = "k", name = "n", artifacts = listOf(ToolArtifact(os = "linux", githubUrl = "https://github.com/o/r")))
        assertFalse(tool.hasMirror)
    }

    @Test
    fun `topics carry images, polls and bounty, and filters reach the query`() = runTest {
        val server = FakeServer()
        server.on(
            "/api/topics",
            body = """{"items":[{"id":5,"title":"投票","category":{"id":1,"slug":"casual-fun","name":"吃喝玩乐"},"author":{"uid":"u","nickname":"苏苏"},
              "images":["/api/images/a","/api/images/b"],"view_count":4100,
              "poll":{"options":[{"text":"一","votes":46},{"text":"二","votes":33},{"text":"三","votes":21}],"total":100,"my_vote":1},
              "help":{"bounty":50,"status":"open","expires_at":"2026-11-05T00:00:00Z"}}],"page":1,"page_size":30,"total":1}""",
        )
        val api = ForumApi(server.client, CONFIG) { null }
        val topic = api.topics(category = "share-movies,share-music", tag = "开源", helpStatus = "open", home = false).items.single()
        assertEquals(2, topic.images.size)
        assertEquals(CONFIG.forumBase.trimEnd('/') + "/api/images/a", api.imageUrl(topic.images[0]))
        assertEquals("https://cdn.test/x.png", api.imageUrl("https://cdn.test/x.png"))
        assertEquals(4100, topic.viewCount)
        assertEquals(46, topic.poll?.percent(0))
        assertEquals(1, topic.poll?.myVote)
        assertEquals(50, topic.help?.bounty)
        val url = server.requests.single().url
        assertEquals("share-movies,share-music", url.parameters["category"])
        assertEquals("开源", url.parameters["tag"])
        assertEquals("open", url.parameters["help_status"])
        assertNull(url.parameters["home"], "home=false 时不带 home 参数")
    }

    @Test
    fun `home defaults stay on for the old callers`() = runTest {
        val server = FakeServer()
        server.on("/api/topics", body = """{"items":[],"page":1,"page_size":30,"total":0}""")
        ForumApi(server.client, CONFIG) { null }.topics(category = "share")
        assertEquals("true", server.requests.single().url.parameters["home"])
    }

    @Test
    fun `poll percent is zero without votes and never divides by zero`() {
        val empty = Poll(options = listOf(PollOption("a"), PollOption("b")))
        assertEquals(0, empty.percent(0))
        assertEquals(0, empty.percent(5))
        val poll = Poll(options = listOf(PollOption("a", 1), PollOption("b", 2)), total = 3)
        assertEquals(33, poll.percent(0))
        assertEquals(66, poll.percent(1))
    }

    @Test
    fun `voting sends the option and returns the fresh poll`() = runTest {
        val server = FakeServer()
        server.on("/api/topics/5/poll/vote", body = """{"poll":{"options":[{"text":"一","votes":1},{"text":"二","votes":0}],"total":1,"my_vote":0}}""")
        val poll = ForumApi(server.client, CONFIG) { "tok" }.votePoll(5, 0)
        assertEquals(0, poll.myVote)
        assertEquals(1, poll.total)
        val req = server.requests.single()
        assertEquals(HttpMethod.Put, req.method)
        assertEquals("""{"option":0}""", (req.body as TextContent).text)
    }

    @Test
    fun `creating a help topic or a poll sends the extra parts`() = runTest {
        val server = FakeServer()
        server.on("/api/topics", body = """{"id":9}""")
        val api = ForumApi(server.client, CONFIG) { "tok" }
        assertEquals(9L, api.createTopic(3, "想要一个组件", "左边番茄钟，右边待办", tags = listOf("组件"), bounty = 50, bountyDays = 14))
        val help = Json.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertEquals("50", help["help"]!!.jsonObject["bounty"]!!.jsonPrimitive.content)
        assertEquals("14", help["help"]!!.jsonObject["days"]!!.jsonPrimitive.content)
        assertEquals("组件", help["tags"]!!.jsonArray.single().jsonPrimitive.content)

        api.createTopic(4, "今天几点下班", "来打个卡", pollOptions = listOf("早", "晚"), pollDays = 3)
        val poll = Json.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertEquals(2, poll["poll"]!!.jsonObject["options"]!!.jsonArray.size)
        assertEquals("3", poll["poll"]!!.jsonObject["days"]!!.jsonPrimitive.content)
        assertNull(poll["help"])

        api.createTopic(4, "普通帖子", "没有额外内容")
        val plain = Json.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertEquals(setOf("category_id", "title", "body"), plain.keys)
    }

    @Test
    fun `accepting an answer posts the post id`() = runTest {
        val server = FakeServer()
        server.on("/api/topics/5/help/accept", body = """{"status":"solved"}""")
        ForumApi(server.client, CONFIG) { "tok" }.acceptAnswer(5, 77)
        assertEquals("""{"post_id":77}""", (server.requests.single().body as TextContent).text)
    }

    @Test
    fun `help stats and discuss filters work through the controller`() = runTest {
        val server = FakeServer()
        server.on("/api/help/stats", body = """{"open":12,"solved_this_week":8,"bounty_points":1240}""")
        server.on("/api/topics", body = """{"items":[{"id":1,"title":"求助","category":{"id":1,"slug":"share-help","name":"资源求助"},"author":{"uid":"u","nickname":"木子"}}],"page":1,"page_size":30,"total":1}""")
        val api = ForumApi(server.client, CONFIG) { null }
        assertEquals(1240L, api.helpStats().bountyPoints)
        val controller = DiscussController(api, scope(), category = "share-help", home = false)
        controller.select("new")
        withContext(Dispatchers.Default) { withTimeout(5_000) { controller.state.first { !it.loading && it.topics.isNotEmpty() } } }
        val q = server.requests.last { it.url.encodedPath == "/api/topics" }.url
        assertEquals("share-help", q.parameters["category"])
        assertEquals("new", q.parameters["sort"])
        assertNull(q.parameters["home"])
    }
}
