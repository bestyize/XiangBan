package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.*
import kotlin.test.*

class MobileWorkspaceTest {
    private fun args(vararg fields: Pair<String, JsonElement>) = JsonObject(fields.toMap())

    @Test
    fun `relative paths cannot escape or address hidden and ambiguous files`() = runTest {
        val workspace = MemoryMobileWorkspace()
        for (path in listOf("", "/etc/passwd", "../secret", "a/../b", "a//b", "C:/data", "a\\b", ".secrets", "a/./b", "a/")) {
            assertFailsWith<IllegalArgumentException>(path) { workspace.write(path, "x") }
            assertFailsWith<IllegalArgumentException>(path) { workspace.remove(path, recursive = true) }
        }
        assertFailsWith<IllegalArgumentException> { workspace.write((1..9).joinToString("/") { "folder" }, "x") }
        assertEquals(emptyList(), workspace.list())
        workspace.write("drafts/racing/index.html", "hello")
        assertEquals(listOf("drafts"), workspace.list().map { it.path })
        assertEquals(listOf("drafts/racing", "drafts/racing/index.html"), workspace.list("drafts", true).map { it.path })
        assertEquals("hello", workspace.read("drafts/racing/index.html"))
    }

    @Test
    fun `writes enforce byte limits and directories require explicit recursive removal`() = runTest {
        val workspace = MemoryMobileWorkspace()
        workspace.write("draft/data.txt", "old")
        assertFailsWith<IllegalArgumentException> { workspace.write("draft/data.txt", "中".repeat(MobileWorkspaceLimits.FILE_BYTES / 3 + 1)) }
        assertFailsWith<IllegalArgumentException> { workspace.write("draft/data.txt", "a\u0000b") }
        assertEquals("old", workspace.read("draft/data.txt"))
        assertFailsWith<IllegalArgumentException> { workspace.remove("draft") }
        workspace.remove("draft", recursive = true)
        assertEquals(emptyList(), workspace.list())
    }

    @Test
    fun `total workspace quota is checked before writing and permits replacement`() = runTest {
        var writes = 0
        val workspace = object : ValidatedMobileWorkspace() {
            override suspend fun entries(path: String, recursive: Boolean) = listOf(WorkspaceEntry("full.txt", false, MobileWorkspaceLimits.TOTAL_BYTES))
            override suspend fun readText(path: String) = ""
            override suspend fun writeText(path: String, content: String) { writes += 1 }
            override suspend fun removeEntry(path: String, recursive: Boolean) = Unit
        }
        assertFailsWith<IllegalArgumentException> { workspace.write("new.txt", "x") }
        assertEquals(0, writes)
        workspace.write("full.txt", "smaller")
        assertEquals(1, writes)
    }

    @Test
    fun `file tools support targeted edits paginated reads and bounded search`() = runTest {
        val workspace = MemoryMobileWorkspace()
        val tools = mobileWorkspaceTools(workspace).associateBy { it.name }
        workspace.write("draft/game.js", "score=0\nscore=0\nend")
        assertFalse(tools.getValue("edit_file").run(args("path" to JsonPrimitive("draft/game.js"), "old_text" to JsonPrimitive("score=0"), "new_text" to JsonPrimitive("score=1"))).ok)
        assertEquals("score=0\nscore=0\nend", workspace.read("draft/game.js"))
        assertTrue(tools.getValue("edit_file").run(args("path" to JsonPrimitive("draft/game.js"), "old_text" to JsonPrimitive("score=0"), "new_text" to JsonPrimitive("score=1"), "replace_all" to JsonPrimitive(true))).ok)
        val read = tools.getValue("read_file").run(args("path" to JsonPrimitive("draft/game.js"), "limit" to JsonPrimitive(1)))
        val page = Json.parseToJsonElement(read.output).jsonObject
        assertEquals("score=1", page.getValue("content").jsonPrimitive.content)
        assertEquals(1, page.getValue("next_offset").jsonPrimitive.int)
        val search = tools.getValue("grep").run(args("query" to JsonPrimitive("score=1")))
        assertEquals(2, Json.parseToJsonElement(search.output).jsonObject.getValue("matches").jsonArray.size)
        assertEquals("write", tools.getValue("write_file").risk)
        assertEquals("read", tools.getValue("read_file").risk)
    }

    @Test
    fun `tools preserve cancellation`() = runTest {
        val workspace = object : MobileWorkspace {
            override suspend fun list(path: String, recursive: Boolean): List<WorkspaceEntry> = throw CancellationException("stopped")
            override suspend fun read(path: String): String = throw CancellationException("stopped")
            override suspend fun write(path: String, content: String) = Unit
            override suspend fun remove(path: String, recursive: Boolean) = Unit
        }
        assertFailsWith<CancellationException> { mobileWorkspaceTools(workspace).first { it.name == "list_dir" }.run(JsonObject(emptyMap())) }
    }
}
