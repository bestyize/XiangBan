package xyz.xiaban.server.admin

import kotlinx.coroutines.test.runTest
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.shared.DisplayItem
import xyz.xiaban.shared.DisplaySettings
import kotlin.test.*

class DisplaySettingsTest {
    @Test fun `default catalog matches current games and quizzes and shows the companion tab`() = runTest {
        val store = DisplaySettingsStore(read = { null }, write = { _, _ -> error("must not write defaults") })
        val defaults = store.current()
        assertTrue(defaults.chatTabVisible)
        assertEquals(listOf("falling-blocks", "merge-2048", "block-puzzle", "water-sort", "snake-arena", "ink-blade"), defaults.games.map { it.id })
        assertEquals(listOf("mbti", "mbanti", "animal", "afterhours"), defaults.quizzes.map { it.id })
        assertTrue((defaults.games + defaults.quizzes).all { it.visible })
    }

    @Test fun `saved visibility and ordering survive a new store instance with the audit actor`() = runTest {
        var raw: String? = null
        var actor: String? = null
        val store = DisplaySettingsStore(read = { raw }, write = { value, who -> raw = value; actor = who })
        val request = DisplaySettings(false,
            listOf(DisplayItem("snake-arena", true, "untrusted title"), DisplayItem("water-sort", false)),
            listOf(DisplayItem("animal", false), DisplayItem("mbanti", true)),
        )
        val saved = store.save(request, "operator")
        assertEquals("operator", actor)
        assertFalse(saved.chatTabVisible)
        assertEquals(listOf("snake-arena", "water-sort"), saved.games.take(2).map { it.id })
        assertFalse(saved.games[1].visible)
        assertEquals("小伴贪吃蛇", saved.games[0].name)
        val reloaded = DisplaySettingsStore(read = { raw }, write = { _, _ -> }).current()
        assertEquals(saved, reloaded)
    }

    @Test fun `new items append in catalog order and stale stored IDs never remove known items`() {
        val saved = DisplaySettings(true, listOf(DisplayItem("water-sort", false), DisplayItem("removed-pack", true)), emptyList())
        val normalized = DisplayCatalog.normalize(saved)
        assertEquals("water-sort", normalized.games.first().id)
        assertFalse(normalized.games.first().visible)
        assertEquals(DisplayCatalog.games.size, normalized.games.size)
        assertFalse(normalized.games.any { it.id == "removed-pack" })
        assertEquals(4, normalized.quizzes.size)
    }

    @Test fun `invalid catalog updates never overwrite stored settings`() = runTest {
        var writes = 0
        val store = DisplaySettingsStore(read = { null }, write = { _, _ -> writes++ })
        for (request in listOf(
            DisplaySettings(true, listOf(DisplayItem("unknown", true))),
            DisplaySettings(true, listOf(DisplayItem("snake-arena", true), DisplayItem("snake-arena", false))),
            DisplaySettings(true, quizzes = listOf(DisplayItem("unknown", true))),
            DisplaySettings(true, quizzes = listOf(DisplayItem("mbti", true), DisplayItem("mbti", false))),
        )) assertFailsWith<ApiException> { store.save(request, "operator") }
        assertEquals(0, writes)
    }

    @Test fun `all items can be hidden without losing their configuration positions`() = runTest {
        val request = DisplayCatalog.defaults().let { it.copy(games = it.games.map { item -> item.copy(visible = false) }, quizzes = it.quizzes.map { item -> item.copy(visible = false) }) }
        val saved = DisplaySettingsStore(read = { null }, write = { _, _ -> }).save(request, "operator")
        assertEquals(DisplayCatalog.games.size, saved.games.size)
        assertEquals(4, saved.quizzes.size)
        assertTrue((saved.games + saved.quizzes).none { it.visible })
    }

    @Test fun `invalid stored JSON falls back to the original presentation`() = runTest {
        assertEquals(DisplayCatalog.defaults(), DisplaySettingsStore(read = { "invalid JSON" }, write = { _, _ -> }).current())
    }
}
