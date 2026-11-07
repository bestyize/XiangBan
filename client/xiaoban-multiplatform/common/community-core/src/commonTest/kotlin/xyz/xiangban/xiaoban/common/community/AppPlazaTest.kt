package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlin.test.*

class AppPlazaTest {
    @Test fun `favorites survive restart and isolate local and public identities`() {
        val local = AppShortcut("local", "my-game", "我的作品")
        val public = AppShortcut("community", "42", "社区作品")
        val items = AppFavorites.toggle(AppFavorites.toggle(emptyList(), local), public)
        assertEquals(listOf(public, local), AppFavorites.decode(AppFavorites.encode(items)))
        assertEquals(listOf(local), AppFavorites.toggle(items, public.copy(title = "新名字")))
        assertEquals(emptyList(), AppFavorites.decode("broken"))
        assertEquals(emptyList(), AppFavorites.decode("""[{"source":"local","id":"../bad","title":"bad"}]"""))
        assertFails { AppFavorites.toggle(items, AppShortcut("community", "-1", "bad")) }
    }

    @Test fun `recent shortcuts retain host order and omit deleted local games`() {
        val local = AppShortcut("local", "my-game", "我的游戏")
        val official = AppShortcut("builtin", "ink-blade", "水墨剑客")
        assertEquals(listOf(official, local, AppShortcut("community", "12", "社区游戏")), recentAppShortcuts(
            listOf("deleted", "ink-blade", "my-game", "ink-blade"), listOf(local, official),
            listOf(RecentCommunityGame(12, 2, "社区游戏"), RecentCommunityGame(12, 1, "旧名字")),
        ))
        assertEquals(12, recentAppShortcuts(emptyList(), emptyList(), (1L..20).map { RecentCommunityGame(it, 1, "游戏 $it") }).size)
    }

    @Test fun `waterfall balances mixed heights with stable order and keeps every item`() {
        val items = listOf(3f, 1f, 1.5f, 2f, 1f)
        assertEquals(listOf(listOf(3f, 1f), listOf(1f, 1.5f, 2f)), appWaterfallColumns(items) { it })
        assertEquals(listOf(emptyList(), emptyList()), appWaterfallColumns(emptyList<Int>()) { 1f })
        assertEquals(listOf(listOf(0, 2), listOf(1, 3)), appWaterfallColumns((0..3).toList()) { Float.NaN })
    }

    @Test fun `orientation reads only the currently published version and remix preserves source`() {
        val pack = Pack(1, "game", title = "原作", version = 2, versions = listOf(PackVersion(1,
            manifest = Json.parseToJsonElement("""{"orientation":"landscape"}""").jsonObject)))
        assertNull(appOrientation(pack))
        assertEquals("landscape", appOrientation(pack.copy(version = 1)))
        val prompt = appRemixPrompt("原作", "原简介", "换个主题")
        listOf("原作", "原简介", "换个主题", "方案让我确认").forEach { assertTrue(it in prompt) }
    }
}
