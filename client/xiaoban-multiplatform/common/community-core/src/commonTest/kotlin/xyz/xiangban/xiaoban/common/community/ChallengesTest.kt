package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.*

private const val CHALLENGE_JSON = """{"key":"creative-day","title":"官方挑战","description":"规则与说明","starts_at":"2026-10-01T00:00:00Z","ends_at":"2026-11-01T00:00:00Z","submission_tag":"creation","topic_id":9,"status":"open","entry_count":1}"""
private const val ENTRY_JSON = """{"pack_id":7,"pack":{"id":7,"kind":"effect","title":"参赛作品","topic_id":10,"author":{"uid":"u","nickname":"作者"}},"featured":true,"vote_method":"topic_like","vote_topic_id":10,"likes":0}"""
private fun challenge(status: String) = CommunityChallenge("key", "标题", "规则", "2099-01-01T00:00:00Z", "2099-02-01T00:00:00Z", "tag", status = status)

class ChallengesTest {
    @Test
    fun `published challenge list and entry pages use public contract without admin or bearer`() = runTest {
        val server = FakeServer()
        server.on("/api/challenges", body = """{"items":[$CHALLENGE_JSON],"page":2,"page_size":25,"total":26}""")
        server.on("/api/challenges/creative-day", body = """{"challenge":$CHALLENGE_JSON,"entries":{"items":[$ENTRY_JSON],"page":2,"page_size":50,"total":51}}""")
        val api = ChallengesApi(server.client, CommunityConfig()) { "PRIVATE-ACCESS" }
        assertEquals("官方挑战", api.list(2).items.single().title)
        assertEquals("2", server.requests.first().url.parameters["page"])
        val detail = api.detail("creative-day", 2)
        assertEquals("2", server.requests.last().url.parameters["page"])
        assertTrue(detail.entries.items.single().featured)
        assertEquals(0L, detail.entries.items.single().likes)
        assertEquals(10L, challengeVoteTopic(detail.entries.items.single()))
        assertTrue(server.requests.all { it.headers[HttpHeaders.Authorization] == null })
        assertTrue(server.requests.none { it.url.encodedPath.contains("admin") })
        server.client.close()
    }

    @Test
    fun `submission sends only owned pack reference and duplicate submission is idempotent response`() = runTest {
        val server = FakeServer()
        server.on("/api/challenges/creative-day/entries", body = """{"status":"submitted","pack_id":7}""")
        val api = ChallengesApi(server.client, CommunityConfig()) { "ACCESS" }
        repeat(2) { assertEquals(7L, api.submit("creative-day", 7).packId) }
        server.requests.forEach {
            val payload = CommunityJson.parseToJsonElement((it.body as TextContent).text).jsonObject
            assertEquals(setOf("pack_id"), payload.keys)
            assertEquals("7", payload["pack_id"]!!.jsonPrimitive.content)
            assertEquals("POST", it.method.value)
            assertEquals("Bearer ACCESS", it.headers[HttpHeaders.Authorization])
        }
        server.client.close()
    }

    @Test
    fun `anonymous invalid pack and mismatched success responses never become submitted`() = runTest {
        val server = FakeServer(); val anonymous = ChallengesApi(server.client, CommunityConfig()) { null }
        assertEquals(401, assertFailsWith<ApiException> { anonymous.submit("challenge", 7) }.status)
        assertEquals(400, assertFailsWith<ApiException> { anonymous.submit("challenge", 0) }.status)
        assertEquals(0, server.requests.size)
        server.on("/api/challenges/challenge/entries", body = """{"status":"submitted","pack_id":8}""")
        val api = ChallengesApi(server.client, CommunityConfig()) { "ACCESS" }
        assertFailsWith<ApiException> { api.submit("challenge", 7) }
        server.client.close()
    }

    @Test
    fun `deadline races removed work and owner permission errors preserve service reason`() = runTest {
        val server = FakeServer(); val api = ChallengesApi(server.client, CommunityConfig()) { "ACCESS" }
        for ((status, message) in listOf(HttpStatusCode.Conflict to "挑战已经截止或已关闭", HttpStatusCode.BadRequest to "只能提交已上架且公开可读的作品", HttpStatusCode.Forbidden to "只能提交自己的作品", HttpStatusCode.NotFound to "主题挑战不存在")) {
            server.on("/api/challenges/challenge/entries", status, """{"message":"$message"}""")
            val error = assertFailsWith<ApiException> { api.submit("challenge", 7) }
            assertEquals(status.value, error.status); assertEquals(message, error.message)
        }
        server.client.close()
    }

    @Test
    fun `server phase controls submission regardless of mobile local timestamps`() {
        assertTrue(challengeCanSubmit(challenge("open")), "即使示例日期在未来，也只认服务器返回的状态")
        assertEquals("征集中", challengePhase(challenge("open")))
        for (status in listOf("upcoming", "ended", "closed", "draft", "unknown")) assertFalse(challengeCanSubmit(challenge(status)))
        assertEquals("尚未开始", challengePhase(challenge("upcoming")))
        assertEquals("已截止", challengePhase(challenge("ended")))
    }

    @Test
    fun `only actual owned published packs appear in submission chooser`() {
        val mine = Pack(7, "effect", title = "自己的作品", author = PackAuthor("u"))
        val packs = listOf(mine, mine.copy(id = 8, status = "removed"), mine.copy(id = 9, status = "rejected"),
            mine.copy(id = 10, author = PackAuthor("other")), mine.copy(id = 11, author = null))
        assertEquals(listOf(7L), ownedChallengePacks(packs, "u").map { it.id })
        assertTrue(ownedChallengePacks(packs, null).isEmpty())
    }

    @Test
    fun `missing mismatched or unknown vote metadata never fabricates a voting target`() {
        val pack = Pack(7, "effect", title = "作品", topicId = 10)
        val entry = CommunityChallengeEntry(7, pack, voteMethod = "topic_like", voteTopicId = 10)
        assertEquals(10L, challengeVoteTopic(entry))
        assertNull(entry.likes)
        assertNull(challengeVoteTopic(entry.copy(voteTopicId = null)))
        assertNull(challengeVoteTopic(entry.copy(voteTopicId = 99)))
        assertNull(challengeVoteTopic(entry.copy(voteMethod = "invented")))
        assertNull(challengeVoteTopic(entry.copy(packId = 99)))
    }

    @Test
    fun `entry likes call real topic API and reload authoritative count without local arithmetic`() = runTest {
        val server = FakeServer()
        server.on({ it.url.encodedPath == "/api/topics/10/like" }, body = """{"liked":true,"like_count":99}""")
        server.on({ it.url.encodedPath == "/api/topics/10" }, body = """{"id":10,"title":"真实讨论","category":{"id":1,"slug":"creation","name":"创作"},"author":{"uid":"u","nickname":"作者"},"like_count":42,"viewer":{"liked":true}}""")
        val forum = ForumApi(server.client, CommunityConfig()) { "ACCESS" }
        val pack = Pack(7, "effect", title = "作品", topicId = 10)
        val entry = CommunityChallengeEntry(7, pack, voteMethod = "topic_like", voteTopicId = 10, likes = 100)
        val updated = forum.setChallengeEntryLiked(entry, true)
        assertEquals(42, updated.likeCount)
        assertTrue(updated.viewer?.liked == true)
        assertEquals(listOf("PUT", "GET"), server.requests.map { it.method.value })
        assertEquals(listOf("/api/topics/10/like", "/api/topics/10"), server.requests.map { it.url.encodedPath })
        assertFailsWith<ApiException> { forum.setChallengeEntryLiked(entry.copy(voteTopicId = 99), true) }
        assertEquals(2, server.requests.size)
        server.client.close()
    }

    @Test
    fun `draft or withdrawn challenge 404 and offline list error remain recoverable`() = runTest {
        val server = FakeServer()
        server.on("/api/challenges/withdrawn", HttpStatusCode.NotFound, """{"message":"主题挑战不存在"}""")
        server.onThrow("/api/challenges")
        val api = ChallengesApi(server.client, CommunityConfig()) { null }
        assertEquals(0, assertFailsWith<ApiException> { api.list() }.status)
        // Replace the broad network-failure route with the specific withdrawn response for this request.
        server.on("/api/challenges/withdrawn", HttpStatusCode.NotFound, """{"message":"主题挑战不存在"}""")
        assertEquals(404, assertFailsWith<ApiException> { api.detail("withdrawn") }.status)
        server.on("/api/challenges", body = """{"items":[],"page":1,"page_size":25,"total":0}""")
        assertTrue(api.list().items.isEmpty())
        server.client.close()
    }
}
