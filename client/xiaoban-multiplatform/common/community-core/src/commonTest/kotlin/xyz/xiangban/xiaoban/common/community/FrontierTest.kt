package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpMethod
import io.ktor.http.content.TextContent
import kotlinx.coroutines.test.runTest
import kotlin.test.*

class FrontierTest {
    @Test fun `frontier list article and interactions use the frontier channel`() = runTest {
        val server = FakeServer()
        server.on("/api/discover/frontier", body = """{"items":[{"key":"story","title":"前沿文章","cover_url":"https://example.com/cover.png"}],"page":1,"page_size":20,"total":1}""")
        server.on("/api/discover/frontier/story", body = """{"key":"story","title":"前沿文章","body_html":"<p>正文</p>"}""")
        server.on("/api/discover/frontier/story/comments", body = """{"items":[],"page":1,"page_size":20,"total":0}""")
        server.on("/api/discover/frontier/story/comments/1/like", body = "{}")
        server.on("/api/discover/frontier/story/comments/1", body = "{}")
        val api = ForumApi(server.client, CommunityConfig()) { "token" }
        assertEquals("前沿文章", api.frontier().items.single().title)
        assertEquals("<p>正文</p>", api.news("story", frontier = true).bodyHtml)
        api.newsComments("story", frontier = true)
        api.commentNews("story", "留言内容", 2, frontier = true)
        assertTrue((server.requests.last().body as TextContent).text.contains("\"reply_to\":2"))
        api.likeNewsComment("story", 1, true, frontier = true)
        assertEquals(HttpMethod.Put, server.requests.last().method)
        api.likeNewsComment("story", 1, false, frontier = true)
        assertEquals(HttpMethod.Delete, server.requests.last().method)
        api.deleteNewsComment("story", 1, frontier = true)
        assertTrue(server.requests.all { it.url.encodedPath.startsWith("/api/discover/frontier") })
    }
}
