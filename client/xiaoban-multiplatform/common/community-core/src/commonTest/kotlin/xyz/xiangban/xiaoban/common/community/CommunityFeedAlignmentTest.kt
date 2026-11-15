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
        api.followingTopics(3, category = "learn,casual-games", tag = "桌面小鱼缸", home = true, activeOnly = true)
        assertEquals("learn,casual-games", server.requests.last().url.parameters["category"])
        assertEquals("桌面小鱼缸", server.requests.last().url.parameters["tag"])
        assertEquals("true", server.requests.last().url.parameters["home"])
        assertEquals("true", server.requests.last().url.parameters["active_only"])
        api.topics(category = "learn", home = false, activeOnly = true)
        assertEquals("learn", server.requests.last().url.parameters["category"])
        assertNull(server.requests.last().url.parameters["home"])
        assertNull(server.requests.last().url.parameters["featured"])
        assertEquals("true", server.requests.last().url.parameters["active_only"])
    }

    @Test fun `category metadata preserves posting restrictions and legacy defaults`() {
        val modern = CommunityJson.decodeFromString<Category>(
            """{"id":7,"parent_id":2,"slug":"resource-movies","name":"影视","section":"resources","status":"archived","posting_enabled":false}""",
        )
        assertEquals("resources", modern.section)
        assertEquals("archived", modern.status)
        assertFalse(modern.postingEnabled)

        val legacy = CommunityJson.decodeFromString<Category>("""{"id":7,"slug":"resource-movies","name":"影视"}""")
        assertNull(legacy.section)
        assertEquals("active", legacy.status)
        assertTrue(legacy.postingEnabled)
    }

    @Test fun `browsing and posting use the same live boards and permissions`() {
        val game = Category(2, slug = "create-game", name = "游戏", section = "ideas")
        val custom = Category(3, slug = "project-journal", name = "项目手记", section = "ideas", minPostLevel = 1)
        val closed = Category(4, slug = "resources", name = "资源", section = "resources", postingEnabled = false)
        val movies = Category(5, 4, "resource-movies", "影视")
        val archived = Category(6, slug = "site", name = "站务", section = "site", status = "archived")
        val feedback = Category(8, slug = "feedback", name = "建议", section = "site", adminOnly = true)
        val catalog = ForumCategories(listOf(game, custom, movies, closed, archived, feedback))

        assertEquals(listOf(game, custom), catalog.boardsIn(ForumSection.Ideas))
        assertEquals(listOf(movies), catalog.boardsIn(ForumSection.Resources))
        assertEquals(listOf(feedback), catalog.boardsIn(ForumSection.Site))
        assertNull(catalog.parent(game))
        assertTrue(catalog.canPost(game, 0))
        assertFalse(catalog.canPost(custom, 0))
        assertTrue(catalog.canPost(custom, 1))
        assertFalse(catalog.canPost(movies, 3))
        assertFalse(catalog.canPost(archived, 3))
        assertFalse(catalog.canPost(feedback, 2))
        assertTrue(catalog.canPost(feedback, 3))
    }

    @Test fun `discuss browsing and posting use the same flat board order and resolve old selections`() {
        val learn = Category(1, slug = "learn", name = "学习和教程", section = "discuss")
        val work = Category(4, slug = "casual-work", name = "职场", section = "discuss")
        val travel = Category(5, slug = "casual-travel", name = "旅行", section = "discuss")
        val game = Category(6, slug = "casual-games", name = "游戏", section = "discuss", allowShortTitle = true)
        val site = Category(7, slug = "site", name = "站务", section = "site")
        val archived = Category(9, slug = "casual-old", name = "旧板块", section = "discuss", status = "archived")
        val catalog = ForumCategories(listOf(work, travel, game, archived, learn, site))

        assertEquals(listOf("设计", "游戏", "3D", "学习和教程", "旅行", "职场", "站务"), catalog.discussionTabs.map { it.label })
        assertEquals(listOf(game, learn, travel, work, site), catalog.discussionBoards)
        assertEquals(learn, catalog.discussionTab("learn-codex")?.category)
        assertEquals(site, catalog.discussionTab("site-feedback")?.category)
        assertTrue(catalog.canPost(learn, 3))
        assertTrue(catalog.allowsShortTitle(game))
        assertNull(catalog.defaultBoard(listOf(game, learn, travel), 3))
    }

    @Test fun `only the actual uncategorized board can be the default`() {
        val normal = Category(1, slug = "resource-movies", name = "影视", section = "resources")
        val uncategorized = CommunityJson.decodeFromString<Category>(
            """{"id":2,"slug":"resources-uncategorized","name":"默认","section":"resources","kind":"uncategorized","min_post_level":1}""",
        )
        val archived = uncategorized.copy(status = "archived")
        val catalog = ForumCategories(listOf(normal, uncategorized))

        assertEquals(uncategorized, catalog.defaultBoard(catalog.boardsIn(ForumSection.Resources), 1))
        assertNull(catalog.defaultBoard(catalog.boardsIn(ForumSection.Resources), 0))
        assertNull(catalog.defaultBoard(listOf(normal), 1))
        assertNull(ForumCategories(listOf(normal, archived)).defaultBoard(listOf(normal, archived), 1))
    }
}
