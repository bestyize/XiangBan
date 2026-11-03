package xyz.xiangban.xiaoban.common.community

import kotlin.test.*

class NavigationDirectoryTest {
    @Test fun `legacy app entries move to Harness without overwriting editorial changes`() {
        val item = NavLink("claude", group = "海外", name = "我的 Claude 入口", vendor = "自定义厂商", url = "https://example.com/custom")
        val entries = NavGroups(listOf(NavGroup("海外", listOf(item)))).navigationDirectory()
        val claude = entries.first { it.key == "claude" }
        assertEquals("harness", claude.category)
        assertEquals(item.name, claude.name)
        assertEquals(item.url, claude.url)
        assertEquals("", claude.logo)
        assertEquals(66, entries.size)
        assertFalse(entries.filter { it.category == "models" }.any { it.name == "ChatGPT" })
    }

    @Test fun `v2 is authoritative for ordering edits and empty published categories`() {
        val first = NavLink("second", name = "自定义入口", url = "https://example.com", category = "api", subtype = "original")
        val second = first.copy(key = "first", name = "另一个入口")
        assertEquals(listOf(first, second), NavGroups(listOf(NavGroup("API", listOf(first, second))), 2).navigationDirectory())
        assertTrue(NavGroups(catalogVersion = 2).navigationDirectory().isEmpty())
    }

    @Test fun `v1 deletions remain removed while newly introduced entries are added`() {
        val entries = NavGroups(catalogVersion = 1).navigationDirectory()
        assertFalse(entries.any { it.key == "grok" })
        assertTrue(entries.any { it.key == "hermes-agent" })
        assertEquals(47, entries.size)
    }

    @Test fun `unknown keys with existing URLs do not duplicate bundled app entries`() {
        val item = NavLink("custom", name = "Copilot", url = "https://copilot.microsoft.com/")
        val entries = NavGroups(listOf(NavGroup("海外", listOf(item)))).navigationDirectory()
        assertEquals(1, entries.count { it.url.trimEnd('/') == item.url.trimEnd('/') })
        assertEquals("harness", entries.first { it.key == "custom" }.category)
    }

    @Test fun `filters distinguish manufacturers assistants skills routers and hosted inference`() {
        val entries = NavGroups().navigationDirectory()
        assertEquals(listOf("models", "harness", "skills", "api"), navigationCategories.map { it.key })
        assertEquals(18, entries.count { it.matchesNavigation("models") })
        assertEquals(25, entries.count { it.matchesNavigation("harness") })
        assertEquals(5, entries.count { it.matchesNavigation("skills") })
        assertEquals(10, entries.count { it.matchesNavigation("api", "original") })
        assertEquals(5, entries.count { it.matchesNavigation("api", "thirdparty") })
        assertEquals(3, entries.count { it.matchesNavigation("api", "cloud") })
        assertEquals("聚合路由", entries.first { it.key == "openrouter-api" }.navigationType)
        assertEquals("推理服务", entries.first { it.key == "groq-api" }.navigationType)
    }

    @Test fun `unsupported protocols and credential-bearing links are excluded`() {
        assertNull(navigationUrl("javascript:alert(1)"))
        assertNull(navigationUrl("file:///tmp/test"))
        assertNull(navigationUrl("https://user:pass@example.com"))
        assertEquals("example.com", navigationDomain("https://www.example.com/path"))
        val item = NavLink("unsafe", name = "unsafe", url = "javascript:alert(1)", category = "api")
        assertTrue(NavGroups(listOf(NavGroup("API", listOf(item))), 2).navigationDirectory().isEmpty())
    }
}
