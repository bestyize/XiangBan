package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.*
import kotlin.test.*

class MobileGamesTest {
    private val game = MobileGame("touch-runner", "触控跑酷", "点按跳跃并挑战最高分", category = "action")
    private val svg = "<svg xmlns='http://www.w3.org/2000/svg'/>"
    private val files = listOf(MobileGameFile("index.html", "<!doctype html><script src='game.js'></script>"), MobileGameFile("game.js", "let score = 0;"),
        MobileGameFile("icon.svg", svg), MobileGameFile("cover.svg", svg))

    @Test
    fun `delete removes only selected work progress and logs`() = runTest {
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        store.create(game, files)
        val other = game.copy(id = "other-game")
        store.create(other, files)
        for (id in listOf(game.id, other.id)) {
            workspace.write("saves/$id.json", "{\"level\":2}")
            workspace.write("logs/$id.txt", "runtime")
        }
        workspace.write("drafts/original/index.html", "draft")
        store.delete(game.id)
        assertEquals(listOf(other), store.list())
        assertEquals(listOf("saves/${other.id}.json"), workspace.list("saves").map { it.path })
        assertEquals("runtime", workspace.read("logs/${other.id}.txt"))
        assertEquals("draft", workspace.read("drafts/original/index.html"))
        assertEquals(listOf("logs/${other.id}.txt"), workspace.list("logs").map { it.path })
        for (id in listOf("../other-game", "dusk-glider", "", "games")) {
            assertFails { store.delete(id) }
        }
        assertEquals(other, store.validate(other.id))
        store.delete(other.id)
        assertTrue(store.list().isEmpty())
    }

    @Test
    fun `delete without progress succeeds and failure restores the complete work`() = runTest {
        val memory = MemoryMobileWorkspace()
        var failPath: String? = null
        val workspace = object : MobileWorkspace by memory {
            override suspend fun remove(path: String, recursive: Boolean) {
                if (path == failPath) { failPath = null; error("storage unavailable") }
                memory.remove(path, recursive)
            }
        }
        val store = MobileGameStore(workspace)
        store.create(game, files)
        store.delete(game.id)
        assertTrue(store.list().isEmpty())
        store.create(game, files)
        workspace.write("saves/${game.id}.json", "{\"level\":3}")
        workspace.write("logs/${game.id}.txt", "keep log")
        val before = store.exportFiles(game.id)
        failPath = "logs/${game.id}.txt"
        assertFailsWith<IllegalStateException> { store.delete(game.id) }
        assertEquals(before, store.exportFiles(game.id))
        assertEquals("{\"level\":3}", workspace.read("saves/${game.id}.json"))
        assertEquals("keep log", workspace.read("logs/${game.id}.txt"))
    }

    @Test
    fun `export has a protocol 2 manifest and excludes saves and runtime logs`() = runTest {
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        store.create(game, files)
        workspace.write("saves/${game.id}.json", "{\"level\":2}")
        workspace.write("logs/${game.id}.txt", "runtime")
        val exported = store.exportFiles(game.id)
        val manifest = Json.parseToJsonElement(exported.first().content).jsonObject
        assertEquals("manifest.json", exported.first().path)
        assertEquals(3, manifest.getValue("schemaVersion").jsonPrimitive.int)
        assertEquals(2, manifest.getValue("runtimeProtocol").jsonPrimitive.int)
        assertEquals("game", manifest.getValue("kind").jsonPrimitive.content)
        assertEquals(game.id, manifest.getValue("id").jsonPrimitive.content)
        assertEquals("action", manifest.getValue("category").jsonPrimitive.content)
        assertEquals("icon.svg", manifest.getValue("icon").jsonPrimitive.content)
        assertEquals(listOf("touch", "mouse", "keyboard"), manifest.getValue("input").jsonArray.map { it.jsonPrimitive.content })
        assertEquals(setOf("manifest.json", "index.html", "game.js", "icon.svg", "cover.svg"), exported.map { it.path }.toSet())
        assertEquals(game, store.validate(game.id))
        assertEquals(listOf(game), store.list())
    }

    @Test
    fun `validation covers identifiers metadata paths and text limits`() {
        for (id in listOf("a", "A-game", "-game", "game-", "a_game", "falling-blocks", "pro-cube", "jade-gomoku", "mbti")) {
            assertFailsWith<IllegalArgumentException>(id) { MobileGameFormat.metadata(game.copy(id = id)) }
        }
        for (path in listOf("../index.html", "/index.html", "a\\game.js", "a:b.js", "a/.hidden.js", "a/b/c/d/e/f/g.js", "assets/a.png", "a.exe", "__MACOSX/game.js", "manifest.json/game.js")) {
            assertFailsWith<IllegalArgumentException>(path) { MobileGameFormat.files(files + MobileGameFile(path, "x")) }
        }
        MobileGameFormat.files(files + MobileGameFile("lib/util.mjs", "export {}"))
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + MobileGameFile("GAME.JS", "x")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + MobileGameFile("manifest.json", "{}")) }
        for (name in MobileGameFormat.required) {
            assertFailsWith<IllegalArgumentException>(name) { MobileGameFormat.files(files.filterNot { it.path == name }) }
        }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + MobileGameFile("big.js", "a".repeat(MobileGameFormat.FILE_BYTES + 1))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + (1..MobileGameFormat.FILES).map { MobileGameFile("file$it.js", "x") }) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + (1..9).map { MobileGameFile("file$it.js", "a".repeat(1024 * 1024)) }) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.files(files + listOf(MobileGameFile("folder.js", "x"), MobileGameFile("folder.js/child.js", "x"))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(name = "字".repeat(25))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(description = "字".repeat(81))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(name = "\u0085")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(orientation = "diagonal")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(category = "tool")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(kind = "app", category = "action")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(kind = "effect")) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(input = listOf("mouse"))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(input = listOf("touch", "gamepad"))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(capabilities = listOf("pointer"))) }
        assertFailsWith<IllegalArgumentException> { MobileGameFormat.metadata(game.copy(version = "1.0 beta")) }
        assertEquals("tool", MobileGameFormat.metadata(game.copy(kind = "app", category = "tool", capabilities = listOf("camera"))).category)
        assertEquals("😀".repeat(24), MobileGameFormat.metadata(game.copy(name = "😀".repeat(24))).name)
    }

    @Test
    fun `draft installation requires exactly one source and generates its own manifest`() = runTest {
        val workspace = MemoryMobileWorkspace()
        val store = MobileGameStore(workspace)
        files.forEach { workspace.write("drafts/runner/${it.path}", it.content) }
        val create = mobileGameTools(store).first { it.name == "app_create" }
        val base = buildJsonObject { put("id", game.id); put("name", game.name); put("description", game.description); put("source_dir", "drafts/runner"); put("category", "action") }
        assertFalse(create.run(JsonObject(base - "source_dir")).ok)
        assertFalse(create.run(JsonObject(base + ("files" to JsonArray(emptyList())))).ok)
        assertFalse(create.run(JsonObject(base + ("source_dir" to JsonPrimitive("../outside")))).ok)
        assertTrue(create.run(base).ok)
        assertEquals(files.toSet(), store.exportFiles(game.id).filterNot { it.path == "manifest.json" }.toSet())
        assertFalse(create.run(base).ok)
        assertTrue(create.run(JsonObject(base + ("overwrite" to JsonPrimitive(true)))).ok)
        val tool = JsonObject(base + mapOf("overwrite" to JsonPrimitive(true), "kind" to JsonPrimitive("app"), "category" to JsonPrimitive("tool")))
        assertTrue(create.run(tool).ok)
        assertEquals("app", store.validate(game.id).kind)
        assertFalse(create.run(JsonObject(tool + ("category" to JsonPrimitive("puzzle")))).ok)
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
        val wrongTypes = listOf("kind", "id", "name", "description", "category", "orientation").map { withField(it, JsonPrimitive(false)) } +
            listOf(withField("input", JsonPrimitive("touch")), withField("capabilities", JsonPrimitive("pointer")), withField("capabilities", buildJsonArray { add(JsonPrimitive(1)) }), "[]")
        for (corrupt in listOf(original.replace("\"schemaVersion\":3", "\"schemaVersion\":2"), original.replace("\"runtimeProtocol\":2", "\"runtimeProtocol\":1"), original.replace("\"schemaVersion\":3", "\"schemaVersion\":\"3\""), original.replace("\"${game.id}\"", "\"other-game\""), original.replace("\"icon\":\"icon.svg\"", "\"icon\":\"../icon.svg\""), original.replace("\"orientation\":\"portrait\"", "\"orientation\":false")) + wrongTypes) {
            workspace.write("games/${game.id}/manifest.json", corrupt)
            assertFailsWith<IllegalArgumentException> { store.validate(game.id) }
            assertEquals(emptyList(), store.list())
            assertEquals(1, store.scan().errors.size)
            assertTrue(store.scan().errors.single().startsWith("games/${game.id}"))
        }
    }
}
