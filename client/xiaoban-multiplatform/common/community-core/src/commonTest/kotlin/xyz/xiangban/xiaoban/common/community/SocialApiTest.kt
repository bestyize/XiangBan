package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlin.test.*

class SocialApiTest {
    @Test
    fun `notifications preserve actor null targets read state and exact page filter`() = runTest {
        val server = FakeServer()
        server.on("/api/me/notifications", body = """{"items":[{"id":5,"kind":"pack_rejected","title":"作品","excerpt":"原因","actor":null,"pack_id":7,"read":false,"created_at":"2026-10-31T02:00:00Z"}],"page":2,"page_size":30,"total":31}""")
        val api = SocialApi(server.client, CommunityConfig()) { "access" }
        val page = api.notifications(2, true)
        assertEquals(7L, page.items.single().packId)
        assertNull(page.items.single().actor)
        assertNull(page.items.single().topicId)
        assertEquals("2", server.requests.single().url.parameters["page"])
        assertEquals("true", server.requests.single().url.parameters["unread"])
        assertEquals("Bearer access", server.requests.single().headers[HttpHeaders.Authorization])
        assertEquals("内容包「作品」未通过审核", notificationHeadline(page.items.single()))
        server.client.close()
    }

    @Test
    fun `selected empty and all notification read requests have distinct payloads and invalidate badge`() = runTest {
        val server = FakeServer()
        server.on("/api/me/notifications/read", body = """{"marked":2}""")
        val api = SocialApi(server.client, CommunityConfig()) { "access" }
        assertEquals(0L, api.notificationRevision.value)
        api.markRead(listOf(5, 5, 7))
        val selected = CommunityJson.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertEquals(listOf("5", "7"), selected["ids"]!!.jsonArray.map { it.jsonPrimitive.content })
        api.markRead(emptyList())
        val empty = CommunityJson.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertEquals(0, empty["ids"]!!.jsonArray.size)
        api.markRead()
        val all = CommunityJson.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertFalse(all.containsKey("ids"))
        assertEquals(3L, api.notificationRevision.value)
        server.client.close()
    }

    @Test
    fun `failed mark read does not claim success or trigger badge update`() = runTest {
        val server = FakeServer()
        server.on("/api/me/notifications/read", HttpStatusCode.ServiceUnavailable, """{"message":"稍后重试"}""")
        val api = SocialApi(server.client, CommunityConfig()) { "access" }
        assertEquals(503, assertFailsWith<ApiException> { api.markRead(listOf(5)) }.status)
        assertEquals(0L, api.notificationRevision.value)
        assertFailsWith<ApiException> { api.markRead(listOf(-1)) }
        assertEquals(1, server.requests.size)
        server.client.close()
    }

    @Test
    fun `private social endpoints refuse anonymous requests before any network call`() = runTest {
        val server = FakeServer(); val api = SocialApi(server.client, CommunityConfig()) { null }
        assertEquals(401, assertFailsWith<ApiException> { api.notifications() }.status)
        assertEquals(401, assertFailsWith<ApiException> { api.bookmarks() }.status)
        assertEquals(401, assertFailsWith<ApiException> { api.blocks() }.status)
        assertEquals(401, assertFailsWith<ApiException> { api.markRead() }.status)
        assertEquals(401, assertFailsWith<ApiException> { api.follow("uid", true) }.status)
        assertEquals(401, assertFailsWith<ApiException> { api.block("uid", true) }.status)
        assertEquals(401, assertFailsWith<ApiException> { api.report("topic", 7, "spam", "") }.status)
        assertEquals(0, server.requests.size)
        server.client.close()
    }

    @Test
    fun `follow and block use idempotent put delete and authoritative response state`() = runTest {
        val server = FakeServer()
        server.on("/api/users/user/follow", body = """{"following":true,"following_count":2,"follower_count":3}""")
        server.on("/api/users/user/block", body = """{"blocked":true}""")
        val api = SocialApi(server.client, CommunityConfig()) { "access" }
        assertTrue(api.follow("user", true).following)
        assertTrue(api.block("user", true).blocked)
        server.on("/api/users/user/follow", body = """{"following":false,"following_count":2,"follower_count":2}""")
        server.on("/api/users/user/block", body = """{"blocked":false}""")
        assertFalse(api.follow("user", false).following)
        assertFalse(api.block("user", false).blocked)
        assertEquals(listOf("PUT", "PUT", "DELETE", "DELETE"), server.requests.map { it.method.value })
        server.client.close()
    }

    @Test
    fun `profile connections and topics use existing public APIs and pages`() = runTest {
        val server = FakeServer()
        server.on("/api/users/user", body = """{"uid":"user","nickname":"作者","following":true,"blocked":false,"following_count":5,"follower_count":8}""")
        server.on("/api/users/user/following", body = """{"items":[{"uid":"other","nickname":"用户"}],"page":2,"total":21}""")
        server.on("/api/users/user/followers", body = """{"items":[]}""")
        server.on("/api/users/user/topics", body = """{"items":[]}""")
        val api = SocialApi(server.client, CommunityConfig()) { null }
        assertEquals(8L, api.profile("user").followerCount)
        assertEquals("other", api.connections("user", false, 2).items.single().uid)
        assertEquals("2", server.requests.last().url.parameters["page"])
        api.connections("user", true); api.userTopics("user", 3)
        assertEquals(listOf("/api/users/user", "/api/users/user/following", "/api/users/user/followers", "/api/users/user/topics"), server.requests.map { it.url.encodedPath })
        assertTrue(server.requests.all { it.headers[HttpHeaders.Authorization] == null })
        server.client.close()
    }

    @Test
    fun `bookmarks are real topics and blocks decode independently`() = runTest {
        val server = FakeServer()
        server.on("/api/me/bookmarks", body = """{"items":[{"id":7,"title":"帖子","category":{"id":1,"name":"分类","slug":"category"},"author":{"uid":"u","nickname":"作者"}}],"page":1,"total":1}""")
        server.on("/api/me/blocks", body = """{"items":[{"uid":"u","nickname":"被屏蔽用户","blocked_at":"2026-10-31T00:00:00Z"}]}""")
        val api = SocialApi(server.client, CommunityConfig()) { "access" }
        assertEquals(7L, api.bookmarks().items.single().id)
        assertEquals("被屏蔽用户", api.blocks().items.single().nickname)
        assertEquals(2, server.requests.size)
        server.client.close()
    }

    @Test
    fun `reports validate supported target and reason while preserving permission and limit errors`() = runTest {
        val server = FakeServer(); val api = SocialApi(server.client, CommunityConfig()) { "access" }
        assertFailsWith<ApiException> { api.report("pack", 7, "spam", "") }
        assertFailsWith<ApiException> { api.report("topic", 7, "invented", "") }
        assertFailsWith<ApiException> { api.report("post", 7, "other", "x".repeat(501)) }
        assertEquals(0, server.requests.size)
        server.on("/api/reports", body = """{"ok":true}""")
        api.report("post", 7, "copyright", "  补充说明  ")
        val payload = CommunityJson.parseToJsonElement((server.requests.last().body as TextContent).text).jsonObject
        assertEquals("post", payload["target_type"]!!.jsonPrimitive.content)
        assertEquals("7", payload["target_id"]!!.jsonPrimitive.content)
        assertEquals("补充说明", payload["detail"]!!.jsonPrimitive.content)
        server.on("/api/reports", HttpStatusCode.Forbidden, """{"message":"无权阅读这个帖子"}""")
        assertEquals(403, assertFailsWith<ApiException> { api.report("topic", 7, "spam", "") }.status)
        server.on("/api/reports", HttpStatusCode.TooManyRequests, """{"message":"举报次数太多","retry_after":3600}""")
        assertEquals(429, assertFailsWith<ApiException> { api.report("topic", 7, "spam", "") }.status)
        server.client.close()
    }

    @Test
    fun `notification unknown future kinds remain readable without actor guesses`() {
        assertEquals("未来消息", notificationHeadline(CommunityNotification(5, "future_kind", "未来消息")))
        assertEquals("社区消息", notificationHeadline(CommunityNotification(5, "future_kind", "")))
        assertEquals("有人 回复了你的帖子「标题」", notificationHeadline(CommunityNotification(5, "reply", "标题")))
    }
}
