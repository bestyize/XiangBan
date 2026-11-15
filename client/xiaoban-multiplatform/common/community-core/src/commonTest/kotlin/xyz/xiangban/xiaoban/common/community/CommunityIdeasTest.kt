package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class CommunityIdeasTest {
    @Test
    fun featuredCatalogIsPublicAndUsesItsDeclaredOrder() = runTest {
        val server = FakeServer()
        server.on("/ideas-catalog.json", body = """
            {"items":[
              {"id":"gallery","kind":"gallery","title":"展厅","description":"预览","cover":"/gallery.webp","author":"A","date":"2026-11-15","href":"/ideas?tab=gallery"},
              {"id":"game","kind":"work","title":"游戏","description":"简介","cover":"/game.webp","images":["/game-detail.webp"],"author":"B","date":"2026-11-16","href":"/ideas/works/game","pack_slug":"test-game","site":"https://example.com"}
            ]}
        """.trimIndent())
        val api = ForumApi(server.client, CommunityConfig()) { error("公开精选目录无需读取令牌") }

        val featured = api.featuredIdeas()

        assertEquals("/ideas-catalog.json", server.requests.single().url.encodedPath)
        assertNull(server.requests.single().headers["Authorization"])
        assertEquals(listOf("gallery", "game"), featured.map { it.id })
        assertEquals("test-game", featured.last().packSlug)
        assertEquals(listOf("/game-detail.webp"), featured.last().images)
        assertTrue(featured.first().images.isEmpty())
        assertEquals("/ideas?tab=gallery", featured.first().href)
    }

    @Test
    fun ideaRuntimeUsesResourceOriginAndContentAddressedArchive() = runTest {
        val sha256 = "c".repeat(64)
        val server = FakeServer()
        server.on("/idea-players/v1/packages/oil-paint.json", body = """
            {"kind":"game","version":"1.1.3","sha256":"$sha256","size":3408,"url":"https://res.xiangban.xyz/files/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.zip"}
        """.trimIndent())
        val config = CommunityConfig()
        val api = ForumApi(server.client, config) { error("公开作品包无需读取令牌") }

        assertFailsWith<IllegalArgumentException> { api.ideaRuntime("../settings") }
        val download = api.ideaRuntime("oil-paint")

        assertEquals("${config.resourceBase}/files/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.zip", download.url)
        assertEquals("res.xiangban.xyz", server.requests.single().url.host)
        assertEquals("1.1.3", download.version)
        assertEquals("game", download.kind)
        assertEquals(3408L, download.size)
        assertNull(server.requests.single().headers["Authorization"])
    }

    @Test
    fun mixedPageKeepsEditorialOrderDeduplicatesAndUsesAllSourcesForPagination() {
        val gallery = FeaturedIdea("gallery", "gallery", "展厅", "", "/a.webp", author = "A", date = "2026-11-15", href = "/ideas")
        val curated = gallery.copy(id = "curated", kind = "work", date = "2026-11-17", packSlug = "curated")
        val category = CategoryRef(1, "ideas-uncategorized", "默认")
        val author = PublicUser("uid", "作者")
        val post = Topic(7, "帖子", category = category, author = author, createdAt = "2026-11-15T00:00:00Z")
        val pack = Pack(7, "game", title = "作品包", topicId = 6, updatedAt = "2026-11-15T10:00:00+08:00")
        val scheme = Scheme(7, PackAuthor(author.uid, author.nickname), "方案", items = emptyList(), createdAt = "2026-11-15T03:00:00Z")
        val postPage = Page(items = listOf(post.copy(id = 6), post.copy(id = 5), post))
        val packPage = Page(items = listOf(pack, pack.copy(id = 8, slug = "curated", topicId = 5)))
        val schemePage = Page(items = listOf(scheme), pageSize = 1, total = 2)

        val feed = mergeIdeaFeedPage(listOf(gallery, curated), packPage, schemePage, postPage)

        assertEquals(listOf("featured:gallery", "featured:curated", "scheme:7", "pack:7", "topic:7"), feed.entries.map { it.key })
        assertTrue(feed.hasMore)
        assertEquals(setOf("pack:7", "pack:8", "topic:7"), mergeIdeaFeedPage(emptyList(), packPage, null, postPage).entries.map { it.key }.toSet())
    }
}
