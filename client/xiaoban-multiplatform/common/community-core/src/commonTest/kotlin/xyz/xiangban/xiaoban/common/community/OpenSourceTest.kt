package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OpenSourceTest {
    @Test
    fun catalogDecodesEditorialFieldsAndAllowsCrossCategoryDiscovery() = runTest {
        val server = FakeServer()
        server.on("/api/open-source", body = """{
          "version":1,"reviewed_at":"2026-11-02","notice":"公开资料整理",
          "categories":[{"key":"office","label":"办公效率"},{"key":"ai","label":"AI 与智能体"}],
          "projects":[{"full_name":"example/notes","name":"Notes","title":"笔记和知识","summary":"知识管理",
            "category":"office","related_categories":["ai"],"tags":["笔记"],"effort":"安装应用后使用",
            "source_url":"https://github.com/example/notes#readme","featured":true,
            "icon_url":"/open-source-media/notes-icon.png","cover_style":"illustration","cover_layout":"notes","cover_lines":["标题","内容"]}],
          "collections":[{"key":"writing","title":"写作","summary":"创作工具","projects":["example/notes"],"notes":["整理资料"]}]
        }""")
        val catalog = ForumApi(server.client, CommunityConfig()) { null }.openSourceCatalog()
        val project = catalog.project("EXAMPLE/NOTES")!!
        assertEquals("安装应用后使用", project.effort)
        assertEquals("/open-source-media/notes-icon.png", project.iconUrl)
        assertEquals("notes", project.coverLayout)
        assertEquals(listOf("标题", "内容"), project.coverLines)
        assertEquals(listOf(project), catalog.filter("office"))
        assertEquals(listOf(project), catalog.filter("ai", " 知识 ", "笔记"))
        assertEquals(listOf(project), catalog.filter("recommend"))
        assertEquals(listOf(project), catalog.filter("all", "notes"))
        assertTrue(catalog.filter("design").isEmpty())
        assertNull(catalog.project("missing/repo"))
        assertEquals("2026-11-02", catalog.reviewedAt)
        assertEquals(project.fullName, catalog.collections.single().projects.single())
        assertEquals("/api/open-source", server.requests.single().url.encodedPath)
        server.client.close()
    }
}
