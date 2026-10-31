package xyz.xiangban.xiaoban.common.community

import kotlin.test.*

class GameLibraryTest {
    private val manifest = """{"id":"dusk-glider","entry":"index.html","runtimeProtocol":1,"name":"暮色滑翔","description":"滑翔游戏","orientation":"landscape","save":true}"""
    @Test fun `bundled catalog preserves identity orientation and save support`() {
        val game = BuiltinMobileGames.read("dusk-glider", manifest)
        assertEquals("landscape", game.orientation)
        assertTrue(game.save)
        assertFails { BuiltinMobileGames.read("ink-blade", manifest) }
        assertFails { BuiltinMobileGames.read("other-game", manifest) }
        assertFails { BuiltinMobileGames.read("dusk-glider", manifest.replace("index.html", "../index.html")) }
        assertFails { BuiltinMobileGames.read("dusk-glider", manifest.replace("\"runtimeProtocol\":1", "\"runtimeProtocol\":2")) }
        assertFails { MobileGameFormat.id("dusk-glider") } // Bundled IDs must remain protected from user replacement.
    }
    @Test fun `history is deduplicated ordered bounded and tolerant of corrupt preferences`() {
        val old = (1..15).map { "game-$it" }
        val recent = MobileGameHistory.opened(old, "game-5")
        assertEquals("game-5", recent.first())
        assertEquals(12, recent.size)
        assertEquals(1, recent.count { it == "game-5" })
        assertEquals(recent, MobileGameHistory.decode(MobileGameHistory.encode(recent)))
        assertEquals(emptyList(), MobileGameHistory.decode("broken json"))
        assertEquals(listOf("game-1"), MobileGameHistory.decode("""["../bad","game-1",null,"game-1"]"""))
    }
    @Test fun `removed games are omitted from recent display`() {
        val available = listOf(MobileGame("game-1", "一", "描述"), MobileGame("game-2", "二", "描述"))
        assertEquals(listOf("game-2", "game-1"), MobileGameHistory.available(listOf("deleted-game", "game-2", "game-1"), available).map { it.id })
    }
}
