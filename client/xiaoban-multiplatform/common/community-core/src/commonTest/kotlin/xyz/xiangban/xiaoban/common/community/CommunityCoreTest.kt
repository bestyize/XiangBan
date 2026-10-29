package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.headersOf
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertIs
import kotlin.test.assertNull
import kotlin.test.assertTrue

private const val PACK = """{"id":7,"kind":"effect","slug":"aurora","title":"极光光晕","summary":"流动的极光","tags":["官方"],"author":null,"trust_level":0,"status":"published","version":2,"cover_url":null,"apply_count":3,"topic_id":null,"one_click":true,"pure_data":true,"updated_at":"2026-10-28T00:00:00Z","unknown_future_field":1}"""

private fun page(items: String, page: Int = 1, size: Int = 2, total: Int = 3) = """{"items":[$items],"page":$page,"page_size":$size,"total":$total}"""

class ForumApiTest {
    private val config = CommunityConfig()

    @Test
    fun `packs are parsed, unknown fields ignored, and the query is built from the filters`() = runTest {
        val server = FakeServer()
        server.on("/api/packs", body = page(PACK))
        val api = ForumApi(server.client, config) { "tok" }
        val result = api.packs(kind = "effect", sort = "hot", q = "极光 x", page = 2)
        assertEquals(1, result.items.size)
        assertEquals("极光光晕", result.items[0].title)
        assertEquals("官方", result.items[0].trustLabel)
        assertTrue(result.items[0].oneClick)
        val url = server.requests.single().url
        assertEquals("/api/packs", url.encodedPath)
        assertEquals("effect", url.parameters["kind"])
        assertEquals("hot", url.parameters["sort"])
        assertEquals("极光 x", url.parameters["q"])
        assertEquals("2", url.parameters["page"])
        assertEquals("Bearer tok", server.requests.single().headers[HttpHeaders.Authorization])
    }

    @Test
    fun `public calls work without a token and send no authorization header`() = runTest {
        val server = FakeServer()
        server.on("/api/packs/7", body = PACK)
        val pack = ForumApi(server.client, config) { null }.pack(7)
        assertEquals(7, pack.id)
        assertNull(server.requests.single().headers[HttpHeaders.Authorization])
    }

    @Test
    fun `error responses become ApiException with the server message and code`() = runTest {
        val server = FakeServer()
        server.on("/api/packs/9", HttpStatusCode.NotFound, """{"error":true,"code":"not_found","message":"内容包不存在"}""")
        server.on("/api/topics/1", HttpStatusCode.TooManyRequests, """{"message":"太快了","retry_after":30}""")
        server.on("/api/topics/2", HttpStatusCode.InternalServerError, "<html>oops</html>")
        val api = ForumApi(server.client, config) { null }
        val notFound = assertFailsWith<ApiException> { api.pack(9) }
        assertEquals(404, notFound.status)
        assertEquals("内容包不存在", notFound.message)
        assertEquals("not_found", notFound.code)
        val limited = assertFailsWith<ApiException> { api.topic(1) }
        assertEquals(30, limited.retryAfter)
        val broken = assertFailsWith<ApiException> { api.topic(2) }
        assertEquals("请求失败（500）", broken.message)
    }

    @Test
    fun `a dropped connection is reported as status 0`() = runTest {
        val server = FakeServer()
        server.onThrow("/api/")
        val e = assertFailsWith<ApiException> { ForumApi(server.client, config) { null }.projects() }
        assertEquals(0, e.status)
    }

    @Test
    fun `weekly entries are typed and unknown kinds are skipped`() = runTest {
        val server = FakeServer()
        server.on(
            "/api/weekly/2026-W44",
            body = """{"week":"2026-W44","title":"第44期","summary":"x","items":[
              {"type":"pack","id":7,"kind":"effect","title":"极光","summary":"s","cover_url":null,"note":"推荐"},
              {"type":"topic","id":5,"title":"帖子","summary":"","note":""},
              {"type":"text","title":"小提示","note":"先看权限"},
              {"type":"video","id":1}]}""",
        )
        val entries = ForumApi(server.client, config) { null }.weekly("2026-W44").entries()
        assertEquals(3, entries.size)
        assertEquals(WeeklyEntry.PackEntry(7, "effect", "极光", "s", null, "推荐"), entries[0])
        assertIs<WeeklyEntry.TopicEntry>(entries[1])
        assertEquals(WeeklyEntry.TextEntry("小提示", "先看权限"), entries[2])
    }

    @Test
    fun `share urls point at the web pages`() {
        val api = ForumApi(FakeServer().client, CommunityConfig(forumBase = "https://chat.example/")) { null }
        assertEquals("https://chat.example/packs/12", api.packUrl(12))
        assertEquals("https://chat.example/t/5", api.topicUrl(5))
        assertEquals("https://chat.example/weekly/2026-W44", api.weeklyUrl("2026-W44"))
    }

    @Test
    fun `writing to the forum sends the token and the right body`() = runTest {
        val server = FakeServer()
        // 后登记的优先匹配，所以通用的写前面。
        server.on("/api/topics", body = """{"id":42}""")
        server.on("/api/topics/9/bookmark", body = """{"bookmarked":true}""")
        server.on("/api/topics/9/like", body = """{"liked":true,"like_count":3}""")
        server.on("/api/topics/9/posts", body = """{"id":1}""")
        server.on("/api/categories", body = """[{"id":3,"parent_id":1,"slug":"share-software","name":"精品软件","description":"d","min_post_level":1,"admin_only":false,"kind":"normal","sort_order":1}]""")
        val api = ForumApi(server.client, config) { "T1" }
        assertEquals(42L, api.createTopic(3, "标题", "这是一段足够长的正文内容"))
        api.reply(9, "回复内容", replyTo = 5)
        api.setTopicLiked(9, true)
        api.setTopicLiked(9, false)
        api.setTopicBookmarked(9, true)
        assertEquals("精品软件", api.categories().single().name)
        val create = server.requests[0]
        assertEquals("POST", create.method.value)
        assertEquals("Bearer T1", create.headers["Authorization"])
        val sent = Json.parseToJsonElement((create.body as TextContent).text).jsonObject
        assertEquals("3", sent["category_id"].toString())
        assertEquals("标题", sent["title"].toString().trim('"'))
        val reply = Json.parseToJsonElement((server.requests[1].body as TextContent).text).jsonObject
        assertEquals("回复内容", reply["body"].toString().trim('"'))
        assertEquals("5", reply["reply_to"].toString())
        assertEquals(listOf("PUT", "DELETE", "PUT"), server.requests.subList(2, 5).map { it.method.value })
    }

    @Test
    fun `pages know whether there is more`() {
        assertTrue(Page<Int>(page = 1, pageSize = 20, total = 21).hasMore)
        assertFalse(Page<Int>(page = 2, pageSize = 20, total = 40).hasMore)
    }
}

class AccountAndSessionTest {
    private val config = CommunityConfig()
    private val login = """{"access_token":"A1","refresh_token":"R1","expires_in":900,"is_new_user":false,"user":{"uid":"u1","username":"小明","identities":[]}}"""

    @Test
    fun `code login either signs in or asks to register`() = runTest {
        val server = FakeServer()
        server.on("/api/auth/code/login", body = login)
        val account = AccountApi(server.client, config)
        val done = assertIs<CodeLogin.Done>(account.loginWithCode("a@b.c", "123456"))
        assertEquals("A1", done.login.accessToken)
        val sent = Json.parseToJsonElement((server.requests.single().body as TextContent).text).jsonObject
        assertEquals("a@b.c", sent["identifier"].toString().trim('"'))
        assertEquals("xiaoban-android", sent["client_id"].toString().trim('"'))
        assertEquals("小伴手机应用", sent["device_name"].toString().trim('"'))

        val registering = FakeServer()
        registering.on("/api/auth/code/login", body = """{"registration_required":true,"registration_token":"REG","expires_in":600}""")
        assertEquals(CodeLogin.NeedsRegistration("REG"), AccountApi(registering.client, config).loginWithCode("new@b.c", "000000"))
    }

    @Test
    fun `account errors carry the server code`() = runTest {
        val server = FakeServer()
        server.on("/api/auth/password/login", HttpStatusCode.Unauthorized, """{"message":"用户名或密码错误","code":"bad_credentials"}""")
        val e = assertFailsWith<ApiException> { AccountApi(server.client, config).loginWithPassword("x", "y") }
        assertEquals(401, e.status)
        assertEquals("bad_credentials", e.code)
    }

    private fun session(server: FakeServer, storage: TokenStorage, clock: () -> Long) = SessionManager(AccountApi(server.client, config), storage, clock)

    @Test
    fun `signing in stores tokens and a restart restores the login`() = runTest {
        val server = FakeServer()
        server.on("/api/auth/password/login", body = login)
        val storage = InMemoryTokenStorage()
        val manager = session(server, storage) { 1_000 }
        assertFalse(manager.state.value.loggedIn)
        manager.loginWithPassword("小明", "pw")
        assertTrue(manager.state.value.loggedIn)
        assertEquals("小明", manager.state.value.user?.username)
        assertEquals(Tokens("A1", "R1", 1_000 + 900_000, "u1"), storage.load())
        assertTrue(session(server, storage) { 2_000 }.state.value.loggedIn, "重启后仍是登录态")
        assertEquals("A1", session(server, storage) { 2_000 }.accessToken())
    }

    @Test
    fun `a token close to expiry is refreshed once even when many requests ask at the same time`() = runTest {
        val server = FakeServer()
        server.on("/api/auth/refresh", body = """{"access_token":"A2","refresh_token":"R2","expires_in":900}""")
        val storage = InMemoryTokenStorage(Tokens("A1", "R1", expiresAtMillis = 10_000))
        val manager = session(server, storage) { 5_000 }
        val tokens = listOf(1, 2, 3).map { async { manager.accessToken() } }.map { it.await() }
        assertEquals(listOf("A2", "A2", "A2"), tokens)
        assertEquals(1, server.requests.count { it.url.encodedPath == "/api/auth/refresh" })
        assertEquals("R2", storage.load()?.refreshToken)
    }

    @Test
    fun `a refused refresh logs out, a network failure keeps the login`() = runTest {
        val refused = FakeServer()
        refused.on("/api/auth/refresh", HttpStatusCode.Unauthorized, """{"message":"登录已过期"}""")
        val storage = InMemoryTokenStorage(Tokens("A1", "R1", 0))
        val manager = session(refused, storage) { 5_000 }
        assertNull(manager.accessToken())
        assertFalse(manager.state.value.loggedIn)
        assertNull(storage.load())

        val offline = FakeServer()
        offline.onThrow("/api/auth/refresh")
        val kept = InMemoryTokenStorage(Tokens("A1", "R1", 0))
        val m2 = session(offline, kept) { 5_000 }
        assertNull(m2.accessToken())
        assertTrue(m2.state.value.loggedIn, "没连上网不应该被登出")
        assertEquals("R1", kept.load()?.refreshToken)
    }

    @Test
    fun `logout clears the tokens even if the server cannot be reached`() = runTest {
        val server = FakeServer()
        server.onThrow("/api/auth/logout")
        val storage = InMemoryTokenStorage(Tokens("A1", "R1", 9_999_999))
        val manager = session(server, storage) { 1 }
        manager.logout()
        assertNull(storage.load())
        assertFalse(manager.state.value.loggedIn)
        assertNull(manager.accessToken())
    }
}

/** 控制器在真实线程上跑网络（假服务也是），所以用真实时间等状态到位，而不是虚拟时间。 */
private suspend fun <T> StateFlow<T>.await(predicate: (T) -> Boolean): T =
    withContext(Dispatchers.Default) { withTimeout(5_000) { first(predicate) } }

class ControllersTest {
    private val config = CommunityConfig()
    private fun scope() = CoroutineScope(Dispatchers.Unconfined)
    private fun pack(id: Int) = PACK.replace("\"id\":7", "\"id\":$id")

    @Test
    fun `feed loads a tab, pages on demand, and a failed page can be retried`() = runTest {
        val server = FakeServer()
        server.on({ it.url.parameters["page"] == "1" && it.url.encodedPath == "/api/packs" }, body = page("${pack(1)},${pack(2)}"))
        server.on({ it.url.parameters["page"] == "2" && it.url.encodedPath == "/api/packs" }, HttpStatusCode.InternalServerError, "{}")
        val feed = FeedController(ForumApi(server.client, config) { null }, scope())
        feed.select("")
        feed.state.await { !it.loading && it.packs.isNotEmpty() }
        assertEquals(listOf(1L, 2L), feed.state.value.packs.map { it.id })
        assertTrue(feed.state.value.hasMore)

        feed.loadMore()
        feed.state.await { it.error != null }
        assertEquals("请求失败（500）", feed.state.value.error)
        assertEquals(2, feed.state.value.packs.size, "已加载的内容保留")

        server.on({ it.url.parameters["page"] == "2" && it.url.encodedPath == "/api/packs" }, body = page(pack(3), page = 2))
        feed.loadMore()
        feed.state.await { it.packs.size == 3 }
        assertEquals(listOf(1L, 2L, 3L), feed.state.value.packs.map { it.id })
        assertFalse(feed.state.value.hasMore)
        assertNull(feed.state.value.error)
    }

    @Test
    fun `feed tabs map to the right endpoints and switching drops the old results`() = runTest {
        val server = FakeServer()
        server.on({ it.url.encodedPath == "/api/weekly" }, body = page("""{"week":"2026-W44","title":"第44期"}"""))
        server.on({ it.url.encodedPath == "/api/projects" }, body = """{"items":[{"slug":"xiaoban","name":"小伴"}]}""")
        server.on({ it.url.encodedPath == "/api/packs" }, body = page(pack(1)))
        val feed = FeedController(ForumApi(server.client, config) { null }, scope())
        feed.select("weekly")
        feed.state.await { it.weekly.isNotEmpty() }
        assertEquals(listOf("2026-W44"), feed.state.value.weekly.map { it.week })
        assertTrue(feed.state.value.packs.isEmpty())
        feed.select("projects")
        feed.state.await { it.projects.isNotEmpty() }
        assertEquals(listOf("xiaoban"), feed.state.value.projects.map { it.slug })
        assertTrue(feed.state.value.weekly.isEmpty())
        feed.select("game")
        feed.state.await { it.packs.isNotEmpty() }
        assertEquals("game", server.requests.last().url.parameters["kind"])
    }

    @Test
    fun `a slow answer for an old tab does not overwrite the new tab`() = runTest {
        val slow = HttpClient(MockEngine { request ->
            val kind = request.url.parameters["kind"]
            if (kind == "effect") delay(300)
            respond(page(pack(if (kind == "effect") 1 else 2)), HttpStatusCode.OK, headersOf(HttpHeaders.ContentType, "application/json"))
        })
        val feed = FeedController(ForumApi(slow, config) { null }, scope())
        feed.select("effect")
        feed.select("game") // 第一个请求还没回来就切换
        feed.state.await { it.packs.isNotEmpty() }
        withContext(Dispatchers.Default) { delay(500) } // 等慢的那个也回来
        assertEquals(listOf(2L), feed.state.value.packs.map { it.id })
        assertEquals("game", feed.state.value.tab)
    }

    @Test
    fun `discussion list pages and keeps the sort`() = runTest {
        val server = FakeServer()
        val topic = """{"id":5,"title":"番茄钟怎么做","excerpt":"x","category":{"id":1,"slug":"share-help","name":"资源求助"},"author":{"uid":"u","nickname":"小明"},"reply_count":2}"""
        server.on({ it.url.encodedPath == "/api/topics" }, body = page(topic, total = 1))
        val discuss = DiscussController(ForumApi(server.client, config) { null }, scope())
        discuss.select("hot")
        discuss.state.await { it.topics.isNotEmpty() }
        assertEquals("hot", server.requests.last().url.parameters["sort"])
        assertEquals("资源求助", discuss.state.value.topics[0].category.name)
        assertFalse(discuss.state.value.hasMore)
    }

    @Test
    fun `loader reports ready, not found and network failures`() = runTest {
        val server = FakeServer()
        server.on("/api/packs/1", body = PACK)
        server.on("/api/packs/2", HttpStatusCode.NotFound, """{"message":"内容包不存在"}""")
        server.onThrow("/api/packs/3")
        val api = ForumApi(server.client, config) { null }
        val ok = Loader(scope()) { api.pack(1) }
        ok.reload()
        assertEquals(7L, assertIs<Load.Ready<Pack>>(ok.state.await { it !is Load.Loading }).value.id)
        val missing = Loader(scope()) { api.pack(2) }
        missing.reload()
        val failed = assertIs<Load.Failed>(missing.state.await { it !is Load.Loading })
        assertTrue(failed.notFound)
        assertEquals("内容包不存在", failed.message)
        val offline = Loader(scope()) { api.pack(3) }
        offline.reload()
        assertEquals("网络连接失败，请稍后再试", assertIs<Load.Failed>(offline.state.await { it !is Load.Loading }).message)
    }
}

class TextAndTokensTest {
    @Test
    fun `html bodies become readable plain text`() {
        assertEquals("第一段\n\n第二段\n\n· 要点一\n\n· 要点二", htmlToPlain("<p>第一段</p><p>第二段</p><ul><li>要点一</li><li>要点二</li></ul>"))
        assertEquals("a & b <c>\n第二行", htmlToPlain("a &amp; b &lt;c&gt;<br>第二行"))
        assertEquals("纯文字", htmlToPlain("  <strong>纯</strong><em>文字</em>  "))
    }

    @Test
    fun `kinds and sizes read naturally`() {
        assertEquals("特效", kindLabel("effect"))
        assertEquals("future", kindLabel("future"))
        assertEquals("512 B", formatSize(512))
        assertEquals("1.5 KB", formatSize(1536))
        assertEquals("2.0 MB", formatSize(2 * 1024 * 1024))
    }

    @Test
    fun `cover gradients rotate by id and never fail on negative ids`() {
        assertEquals(DesignTokens.coverGradient(0), DesignTokens.coverGradient(5))
        assertTrue(DesignTokens.coverGradient(1) != DesignTokens.coverGradient(2))
        DesignTokens.coverGradient(-3)
    }

    @Test
    fun `accent palettes start with the brand red and fall back to it`() {
        assertEquals(6, DesignTokens.Accents.size)
        assertEquals(DesignTokens.Brand, DesignTokens.Accents.first().light.btn)
        assertEquals(DesignTokens.Accents.first(), DesignTokens.accentById(null))
        assertEquals(DesignTokens.Accents.first(), DesignTokens.accentById("no-such-accent"))
        assertEquals("海湾蓝", DesignTokens.accentById("blue").name)
        assertEquals(DesignTokens.Accents.size, DesignTokens.Accents.map { it.id }.toSet().size)
    }

    @Test
    fun `brand colour matches the web front end`() {
        assertEquals(0xFFE3203CL, DesignTokens.Brand)
        assertEquals(8f, DesignTokens.RadiusCard)
    }
}

class PressToTalkTest {
    @Test
    fun `finger position decides what releasing does`() {
        assertEquals(PttAction.Send, Ptt.action(x = 100f, y = 10f, width = 300f, startY = 20f, upThreshold = 60f))
        assertEquals(PttAction.Send, Ptt.action(x = 290f, y = -30f, width = 300f, startY = 20f, upThreshold = 60f))
        assertEquals(PttAction.Cancel, Ptt.action(x = 100f, y = -60f, width = 300f, startY = 20f, upThreshold = 60f))
        assertEquals(PttAction.Text, Ptt.action(x = 250f, y = -60f, width = 300f, startY = 20f, upThreshold = 60f))
    }

    @Test
    fun `limits and countdown follow the spec`() {
        assertTrue(Ptt.tooShort(499))
        assertTrue(!Ptt.tooShort(500))
        assertEquals(null, Ptt.secondsLeft(9_999))
        assertEquals(10, Ptt.secondsLeft(10_000))
        assertEquals(9, Ptt.secondsLeft(11_200))
        assertEquals(0, Ptt.secondsLeft(20_000))
        assertTrue(!Ptt.expired(19_999))
        assertTrue(Ptt.expired(20_000))
        assertEquals("0:06", Ptt.clock(6_400))
        assertEquals("0:20", Ptt.clock(20_000))
    }
}

class AiGatewayAndCloudChatTest {
    private val config = CommunityConfig()
    private fun scope() = CoroutineScope(Dispatchers.Unconfined)

    private val tiersJson = """{"tiers":[
        {"tier":"standrad","display_name":"日常","description":"响应更快、额度更省","is_default":true,"requires_paid":false,"available":true,"reasoning_efforts":[]},
        {"tier":"premium","display_name":"深入","description":"适合复杂任务","is_default":false,"requires_paid":true,"available":false,"reasoning_efforts":["low","high"]}
    ],"media":{}}"""

    private val balanceJson = """{
        "product":{"product_id":"xiaoban-android","name":"小伴手机"},
        "quota_per_unit":500000,
        "total":250000,
        "paid":0,
        "buckets":[{"id":1,"kind":"trial","total":250000,"remaining":250000,"expires_at":"2026-11-28T00:00:00Z","created_at":"2026-10-28T00:00:00Z"}],
        "trial":{"available":false,"quota":250000,"days":30}
    }"""

    @Test
    fun `money formatting and tier selection match desktop rules`() {
        assertEquals("$0.50", formatAiMoney(250_000, 500_000))
        assertEquals("$0.125", formatAiMoney(62_500, 500_000))
        assertEquals("$12.50", formatAiMoney(6_250_000, 500_000))
        assertEquals("$120", formatAiMoney(60_000_000, 500_000))
        assertEquals("试用额度", bucketKindLabel("trial"))
        assertEquals("月卡", bucketKindLabel("monthly"))
        assertEquals("加油包", bucketKindLabel("topup"))

        val tiers = listOf(
            AiTier(tier = "standrad", displayName = "日常", isDefault = true, available = true),
            AiTier(tier = "premium", displayName = "深入", isDefault = false, requiresPaid = true, available = false),
        )
        assertEquals("standrad", pickAiTier(tiers, ""))
        assertEquals("standrad", pickAiTier(tiers, "premium"), "不可用的档位回退到默认可用档位")
    }

    @Test
    fun `tiers, balance, redeem and streaming SSE work end to end`() = runTest {
        val server = FakeServer()
        server.on("/api/ai/tiers", body = tiersJson)
        server.on("/api/ai/balance", body = balanceJson)
        server.on("/api/ai/redeem", body = """{"ok":true,"kind":"topup","quota":500000,"expires_at":null,"balance":750000}""")
        server.onSse(
            "/v1/chat/completions",
            body = """
                : keep-alive
                data: {"id":"1","model":"standrad","choices":[{"delta":{"content":"你好，"}}]}

                data: {"id":"1","model":"standrad","choices":[{"delta":{"content":"我是云端小伴"}}]}

                data: [DONE]
            """.trimIndent() + "\n",
        )
        val api = AiGatewayApi(server.client, config) { "tok-ai" }

        val tiers = api.tiers()
        assertEquals(2, tiers.size)
        assertEquals("日常", tiers[0].displayName)
        assertFalse(tiers[1].available)

        val balance = api.balance()
        assertEquals(250_000L, balance.total)
        assertEquals(1, balance.buckets.size)

        val redeemed = api.redeem("ABCD-EFGH-JKLM-NPQR")
        assertEquals(500_000L, redeemed.quota)
        assertEquals(750_000L, redeemed.balance)

        val chunks = mutableListOf<String>()
        val full = api.streamChat(listOf(ChatMessage("user", "打个招呼")), tier = "standrad") { chunks += it }
        assertEquals(listOf("你好，", "我是云端小伴"), chunks)
        assertEquals("你好，我是云端小伴", full)
        val chatReq = server.requests.last()
        assertEquals("Bearer tok-ai", chatReq.headers[HttpHeaders.Authorization])
        val bodyObj = Json.parseToJsonElement((chatReq.body as TextContent).text).jsonObject
        assertEquals("standrad", bodyObj["model"].toString().trim('"'))
        assertEquals("true", bodyObj["stream"].toString())
    }

    @Test
    fun `openai formatted quota and paid errors are parsed with code and message`() = runTest {
        val server = FakeServer()
        server.on(
            "/v1/chat/completions",
            HttpStatusCode.PaymentRequired,
            """{"error":{"message":"额度已用完，请兑换额度后继续使用","type":"invalid_request_error","code":"insufficient_quota"}}""",
        )
        val api = AiGatewayApi(server.client, config) { "tok" }
        val err = assertFailsWith<ApiException> {
            api.streamChat(listOf(ChatMessage("user", "hi")), tier = "standrad")
        }
        assertEquals(402, err.status)
        assertEquals("insufficient_quota", err.code)
        assertEquals("额度已用完，请兑换额度后继续使用", err.message)
    }

    @Test
    fun `cloud chat controller loads meta, streams replies, retries failures, and redeems codes`() = runTest {
        val server = FakeServer()
        server.on("/api/ai/tiers", body = tiersJson)
        server.on("/api/ai/balance", body = balanceJson)
        val api = AiGatewayApi(server.client, config) { "tok" }
        val controller = CloudChatController(api, scope())

        controller.loadMeta()
        controller.state.await { !it.loadingMeta && it.tiers.isNotEmpty() }
        assertEquals("standrad", controller.state.value.selectedTier)
        assertEquals(250_000L, controller.state.value.balance?.total)

        // 不可用的付费档不能被切过去
        controller.selectTier("premium")
        assertEquals("standrad", controller.state.value.selectedTier)

        // 第一次对话失败，随后 retryLast 重试成功
        server.on(
            "/v1/chat/completions",
            HttpStatusCode.BadGateway,
            """{"error":{"message":"AI 服务暂时不可用，请稍后再试","type":"api_error","code":"bad_gateway"}}""",
        )
        controller.send("今天天气怎么样")
        controller.state.await { !it.sending && it.messages.size == 2 }
        assertEquals("AI 服务暂时不可用，请稍后再试", controller.state.value.messages.last().error)

        server.onSse(
            "/v1/chat/completions",
            body = "data: {\"choices\":[{\"delta\":{\"content\":\"晴朗温和\"}}]}\n\ndata: [DONE]\n",
        )
        controller.retryLast()
        controller.state.await { !it.sending && it.messages.lastOrNull()?.error == null && it.messages.lastOrNull()?.text == "晴朗温和" }
        assertEquals(2, controller.state.value.messages.size)
        assertEquals("今天天气怎么样", controller.state.value.messages[0].text)
        assertEquals("晴朗温和", controller.state.value.messages[1].text)

        // 兑换码成功后刷新提示与余额
        server.on("/api/ai/redeem", body = """{"ok":true,"kind":"topup","quota":500000,"expires_at":null,"balance":750000}""")
        controller.redeem("ABCD-EFGH-JKLM-NPQR")
        controller.state.await { !it.redeeming && it.redeemMessage != null }
        assertEquals("兑换成功，到账 $1.00", controller.state.value.redeemMessage)
    }
}

