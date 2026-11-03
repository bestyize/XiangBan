package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class OpenSourceTest {
    @Test
    fun projectReadmeLoadsThePublicBilingualSnapshotWithoutAuth() = runTest {
        val server = FakeServer()
        val file = "projects/${"a".repeat(64)}.json"
        server.on("/open-source-readmes/index.json", body = """{"projects":{"example/notes":"$file","bad/path":"../private"}}""")
        server.on("/open-source-readmes/$file", body = """{
            "schema":1,"full_name":"example/notes","card":{"name":"Notes","stars":42,"license":null},
            "source_url":"https://github.com/example/notes","commit":"abc123","fetched_at":"2026-11-03",
            "zh":"中文正文","en":"English README","licenses":[{"name":"MIT","text":"License text"}]
        }""")
        val api = ForumApi(server.client, CommunityConfig()) { error("public README should not request an access token") }
        try {
            val readme = api.projectReadme("EXAMPLE/Notes")!!
            assertEquals("中文正文", readme.zh)
            assertEquals("English README", readme.en)
            assertEquals(42L, readme.card.stars)
            assertEquals("MIT", readme.licenses.single().name)
            assertNull(api.projectReadme("missing/repo"))
            assertFailsWith<ApiException> { api.projectReadme("bad/path") }
            assertEquals(listOf("/open-source-readmes/index.json", "/open-source-readmes/$file",
                "/open-source-readmes/index.json", "/open-source-readmes/index.json"),
                server.requests.map { it.url.encodedPath })
        } finally {
            server.client.close()
        }
    }

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
