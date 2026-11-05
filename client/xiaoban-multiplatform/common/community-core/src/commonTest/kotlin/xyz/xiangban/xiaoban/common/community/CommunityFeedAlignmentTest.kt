package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlin.test.*

class CommunityFeedAlignmentTest {
    @Test fun `featured and followed discussions use the same contracts as web`() = runTest {
        val server = FakeServer()
        val empty = """{"items":[],"page":1,"page_size":30,"total":0}"""
        server.on("/api/topics", body = empty)
        server.on("/api/me/following/topics", body = empty)
        val api = ForumApi(server.client, CommunityConfig()) { "session-token" }
        api.topics(sort = "new", featured = true)
        assertEquals("true", server.requests.last().url.parameters["featured"])
        api.followingTopics(2)
        assertEquals("/api/me/following/topics", server.requests.last().url.encodedPath)
        assertEquals("2", server.requests.last().url.parameters["page"])
        assertEquals("Bearer session-token", server.requests.last().headers["Authorization"])
        api.followingTopics(3, category = "create,learn,casual", tag = "桌面小鱼缸", home = true, activeOnly = true)
        assertEquals("create,learn,casual", server.requests.last().url.parameters["category"])
        assertEquals("桌面小鱼缸", server.requests.last().url.parameters["tag"])
        assertEquals("true", server.requests.last().url.parameters["home"])
        assertEquals("true", server.requests.last().url.parameters["active_only"])
        api.topics(category = "learn-local", home = false, activeOnly = true)
        assertEquals("learn-local", server.requests.last().url.parameters["category"])
        assertNull(server.requests.last().url.parameters["home"])
        assertNull(server.requests.last().url.parameters["featured"])
        assertEquals("true", server.requests.last().url.parameters["active_only"])
    }

    @Test fun `category metadata preserves posting restrictions and legacy defaults`() {
        val modern = CommunityJson.decodeFromString<Category>(
            """{"id":7,"parent_id":2,"slug":"resource-open","name":"开源讨论","section":"resources","status":"archived","posting_enabled":false}""",
        )
        assertEquals("resources", modern.section)
        assertEquals("archived", modern.status)
        assertFalse(modern.postingEnabled)

        val legacy = CommunityJson.decodeFromString<Category>("""{"id":7,"slug":"resource-open","name":"开源讨论"}""")
        assertNull(legacy.section)
        assertEquals("active", legacy.status)
        assertTrue(legacy.postingEnabled)
    }
}
