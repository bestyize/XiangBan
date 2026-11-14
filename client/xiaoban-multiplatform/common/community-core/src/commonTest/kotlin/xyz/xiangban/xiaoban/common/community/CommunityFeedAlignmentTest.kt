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
        val ideas = Category(1, slug = "create", name = "创意", section = "ideas", minPostLevel = 3)
        val game = Category(2, 1, "create-game", "游戏")
        val custom = Category(3, 1, "project-journal", "项目手记", minPostLevel = 1)
        val closed = Category(4, slug = "resources", name = "资源", section = "resources", postingEnabled = false)
        val movies = Category(5, 4, "resource-movies", "影视")
        val archived = Category(6, slug = "site", name = "站务", status = "archived")
        val notice = Category(7, 6, "site-notice", "公告")
        val feedback = Category(8, slug = "feedback", name = "建议", section = "site", adminOnly = true)
        val catalog = ForumCategories(listOf(game, custom, ideas, movies, closed, notice, archived, feedback))

        assertEquals(listOf(game, custom), catalog.boardsIn(ForumSection.Ideas))
        assertEquals(listOf(movies), catalog.boardsIn(ForumSection.Resources))
        assertEquals(listOf(feedback), catalog.boardsIn(ForumSection.Site))
        assertEquals(ideas, catalog.parent(game))
        assertFalse(catalog.canPost(ideas, 3))
        assertTrue(catalog.canPost(game, 0))
        assertFalse(catalog.canPost(custom, 0))
        assertTrue(catalog.canPost(custom, 1))
        assertFalse(catalog.canPost(movies, 3))
        assertFalse(catalog.canPost(notice, 3))
        assertFalse(catalog.canPost(feedback, 2))
        assertTrue(catalog.canPost(feedback, 3))
    }

    @Test fun `discussion shows promoted boards in public order with only tutorial and site children nested`() {
        val learn = Category(1, slug = "learn", name = "学习和教程", section = "discuss")
        val codex = Category(2, 1, "learn-codex", "Codex")
        val casual = Category(3, slug = "casual", name = "休闲", section = "discuss")
        val work = Category(4, 3, "casual-work", "职场")
        val travel = Category(5, 3, "casual-travel", "旅行")
        val game = Category(6, 3, "casual-games", "游戏")
        val site = Category(7, slug = "site", name = "站务", section = "site")
        val feedback = Category(8, 7, "site-feedback", "建议反馈")
        val archived = Category(9, 3, "casual-old", "旧板块", status = "archived")
        val catalog = ForumCategories(listOf(codex, work, travel, game, archived, feedback, learn, casual, site))

        assertEquals(listOf("设计", "游戏", "3D", "学习和教程", "旅行", "职场", "站务"), catalog.discussionTabs.map { it.label })
        assertEquals(listOf(game, learn, travel, work, site), catalog.discussionTabs.mapNotNull { it.category })
        assertEquals(listOf(codex), catalog.children(learn))
        assertEquals(listOf(feedback), catalog.children(site))
        assertFalse(catalog.canPost(learn, 3))
        assertTrue(catalog.canPost(codex, 3))
        assertEquals(game, catalog.defaultBoard(listOf(game, learn, travel), 3, null))
        assertEquals(codex, catalog.defaultBoard(listOf(game, learn, travel), 3, codex.id))
        assertEquals(codex, catalog.defaultBoard(listOf(learn), 3, null))
        val junior = ForumCategories(listOf(learn, codex.copy(minPostLevel = 1), game.copy(parentId = null)))
        assertEquals(game.copy(parentId = null), junior.defaultBoard(listOf(learn, game.copy(parentId = null)), 0, codex.id))
        assertNull(junior.defaultBoard(listOf(learn), 0, null))
    }
}
