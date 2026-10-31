package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.*
import kotlin.test.*

class MobileGamesTest {
    private val game = MobileGame("touch-runner", "触控跑酷", "点按跳跃并挑战最高分", save = true)
    private val files = listOf(MobileGameFile("index.html", "<!doctype html><script src='game.js'></script>"), MobileGameFile("game.js", "let score = 0;"), MobileGameFile("cover.svg", "<svg xmlns='http://www.w3.org/2000/svg'/>"))

    @Test
    fun `export has a desktop v1 resource manifest and excludes saves and runtime logs`() = runTest {
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        store.create(game, files)
        workspace.write("saves/${game.id}.json", "{\"level\":2}")
        workspace.write("logs/${game.id}.txt", "runtime")
        val exported = store.exportFiles(game.id)
        val manifest = Json.parseToJsonElement(exported.first().content).jsonObject
        assertEquals("manifest.json", exported.first().path)
        assertEquals(1, manifest.getValue("schemaVersion").jsonPrimitive.int)
        assertEquals("game", manifest.getValue("kind").jsonPrimitive.content)
        assertEquals(game.id, manifest.getValue("id").jsonPrimitive.content)
        assertTrue(manifest.getValue("save").jsonPrimitive.boolean)
        assertEquals("space", manifest.getValue("layer").jsonPrimitive.content)
        assertEquals(setOf("manifest.json", "index.html", "game.js", "cover.svg"), exported.map { it.path }.toSet())
        assertEquals(game, store.validate(game.id))
        assertEquals(listOf(game), store.list())
    }

    @Test
    fun `game validation matches desktop identifiers metadata paths and text limits`() {
        for (id in listOf("a", "A-game", "-game", "a_game", "falling-blocks", "pro-cube", "jade-gomoku")) {
            assertFailsWith<IllegalArgumentException>(id) { MobileGameFormat.metadata(game.copy(id = id)) }
        }
        for (path in listOf("../index.html", "/index.html", "a\\game.js", "a:b.js", "a/.hidden.js", "a/b/c/d/e.js", "assets/a.png", "a.exe", "__MACOSX/game.js", "manifest.json/game.js")) {
            assertFailsWith<IllegalArgumentException>(path) { MobileGameFormat.files(files + MobileGameFile(path, "x")) }
        }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + MobileGameFile("GAME.JS", "x")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + MobileGameFile("manifest.json", "{}")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files.filterNot { it.path == "index.html" }) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + MobileGameFile("big.js", "a".repeat(MobileGameFormat.FILE_BYTES + 1))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files((1..41).map { MobileGameFile("file$it.js", "x") }) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(listOf(MobileGameFile("index.html", "a".repeat(1024 * 1024))) + (1..4).map { MobileGameFile("file$it.js", "a".repeat(1024 * 1024)) }) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + listOf(MobileGameFile("folder.js", "x"), MobileGameFile("folder.js/child.js", "x"))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(name = "字".repeat(25))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(description = "字".repeat(81))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(name = "\u0085")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(layer = "desktop")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(capabilities = listOf("pointer"))) }
        assertEquals("😀".repeat(24), MobileGameFormat.metadata(game.copy(name = "😀".repeat(24))).name)
    }

    @Test
    fun `draft installation requires exactly one source and generates its own manifest`() = runTest {
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        files.forEach { workspace.write("drafts/runner/${it.path}", it.content) }
        val create = mobileGameTools(store).first { it.name == "game_create" }
        val base = buildJsonObject { put("id", game.id); put("name", game.name); put("description", game.description); put("source_dir", "drafts/runner") }
        assertFalse(create.run(JsonObject(base - "source_dir")).ok)
        assertFalse(create.run(JsonObject(base + ("files" to JsonArray(emptyList())))).ok)
        assertFalse(create.run(JsonObject(base + ("source_dir" to JsonPrimitive("../outside")))).ok)
        assertTrue(create.run(base).ok)
        assertEquals(files.toSet(), store.exportFiles(game.id).filterNot { it.path == "manifest.json" }.toSet())
        assertFalse(create.run(base).ok)
        assertTrue(create.run(JsonObject(base + ("overwrite" to JsonPrimitive(true)))).ok)
        workspace.write("drafts/runner/manifest.json", "{}")
        assertFalse(create.run(JsonObject(base + ("overwrite" to JsonPrimitive(true)))).ok)
    }

    @Test
    fun `invalid replacement and write failure preserve original game`() = runTest {
        val memory = MemoryMobileWorkspace()
        var failPath: String? = null
        val workspace = object : MobileWorkspace by memory {
            override suspend fun write(path: String, content: String) {
                if (path == failPath) { failPath = null; error("disk full") }
                memory.write(path, content)
            }
        }
        val store = MobileGameStore(workspace)
        store.create(game, files)
        val before = store.exportFiles(game.id)
        assertFailsWith<IllegalArgumentException> { store.create(game.copy(name = "新版"), listOf(MobileGameFile("other.js", "x")), overwrite = true) }
        assertEquals(before, store.exportFiles(game.id))
        failPath = "games/${game.id}/manifest.json"
        assertFailsWith<IllegalStateException> { store.create(game.copy(name = "新版"), files.map { it.copy(content = it.content + "updated") }, overwrite = true) }
        assertEquals(before, store.exportFiles(game.id))
    }

    @Test
    fun `tampered manifests and unsupported assets cannot be exported`() = runTest {
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        store.create(game, files)
        workspace.write("games/${game.id}/assets/hero.png", "fake binary")
        assertFailsWith<IllegalArgumentException> { store.exportFiles(game.id) }
        workspace.remove("games/${game.id}/assets", true)
        val original = workspace.read("games/${game.id}/manifest.json")
        val objectManifest = Json.parseToJsonElement(original).jsonObject
        fun withField(key: String, value: JsonElement) = JsonObject(objectManifest + (key to value)).toString()
        val wrongTypes = listOf("kind", "id", "name", "description", "layer", "orientation").map { withField(it, JsonPrimitive(false)) } +
            listOf(withField("capabilities", JsonPrimitive("pointer")), withField("capabilities", buildJsonArray { add(JsonPrimitive(1)) }), "[]")
        for (corrupt in listOf(original.replace("\"schemaVersion\":1", "\"schemaVersion\":2"), original.replace("\"schemaVersion\":1", "\"schemaVersion\":\"1\""), original.replace("\"${game.id}\"", "\"other-game\""), original.replace("\"save\":true", "\"save\":\"true\""), original.replace("\"orientation\":\"portrait\"", "\"orientation\":false")) + wrongTypes) {
            workspace.write("games/${game.id}/manifest.json", corrupt)
            assertFailsWith<IllegalArgumentException> { store.validate(game.id) }
            assertEquals(emptyList(), store.list())
            assertEquals(1, store.scan().errors.size)
            assertTrue(store.scan().errors.single().startsWith("games/${game.id}"))
        }
    }
}
