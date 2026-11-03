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
        api.followingTopics(3, category = "create,learn,casual", tag = "桌面小鱼缸", home = true)
        assertEquals("create,learn,casual", server.requests.last().url.parameters["category"])
        assertEquals("桌面小鱼缸", server.requests.last().url.parameters["tag"])
        assertEquals("true", server.requests.last().url.parameters["home"])
        api.topics(category = "learn-local", home = false)
        assertEquals("learn-local", server.requests.last().url.parameters["category"])
        assertNull(server.requests.last().url.parameters["home"])
        assertNull(server.requests.last().url.parameters["featured"])
    }
}
