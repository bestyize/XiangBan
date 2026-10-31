package xyz.xiangban.xiaoban.common.community

import java.io.File
import java.nio.file.Files
import java.util.Comparator
import kotlinx.coroutines.test.runTest
import kotlin.test.*

class MobileWorkspaceFileTest {
    private fun temp() = Files.createTempDirectory("xiaoban-workspace-test-").toFile()
    private fun clean(root: File) {
        Files.walk(root.toPath()).use { paths -> paths.sorted(Comparator.reverseOrder()).forEach { Files.deleteIfExists(it) } }
    }

    @Test
    fun `persistent files can be reopened and safely removed`() = runTest {
        val root = temp()
        try {
            val workspace = JvmMobileWorkspace(root)
            workspace.write("drafts/runner/index.html", "<h1>触控游戏</h1>")
            assertEquals("<h1>触控游戏</h1>", JvmMobileWorkspace(root).read("drafts/runner/index.html"))
            workspace.write("drafts/runner/index.html", "updated")
            assertEquals("updated", workspace.read("drafts/runner/index.html"))
            assertEquals(3, workspace.list("", true).size)
            assertFailsWith<IllegalArgumentException> { workspace.remove("drafts") }
            workspace.remove("drafts", true)
            assertEquals(emptyList(), workspace.list())
        } finally { clean(root) }
    }

    @Test
    fun `file and directory symlinks including dangling links cannot escape workspace`() = runTest {
        val parent = temp()
        try {
            val root = File(parent, "workspace")
            val outside = File(parent, "outside").apply { mkdirs() }
            val secret = File(outside, "secret.txt").apply { writeText("private") }
            val workspace = JvmMobileWorkspace(root)
            Files.createSymbolicLink(File(root, "link.txt").toPath(), secret.toPath())
            assertFailsWith<IllegalArgumentException> { workspace.read("link.txt") }
            assertFailsWith<IllegalArgumentException> { workspace.write("link.txt", "overwrite") }
            assertFailsWith<IllegalArgumentException> { workspace.remove("link.txt") }
            assertFailsWith<IllegalArgumentException> { workspace.list() }
            Files.delete(File(root, "link.txt").toPath())
            Files.createSymbolicLink(File(root, "folder").toPath(), outside.toPath())
            assertFailsWith<IllegalArgumentException> { workspace.read("folder/secret.txt") }
            assertFailsWith<IllegalArgumentException> { workspace.write("folder/new.txt", "new") }
            assertFailsWith<IllegalArgumentException> { workspace.remove("folder", true) }
            assertEquals("private", secret.readText())
            assertFalse(File(outside, "new.txt").exists())
            Files.delete(File(root, "folder").toPath())
            val missing = File(outside, "missing.txt")
            Files.createSymbolicLink(File(root, "dangling.txt").toPath(), missing.toPath())
            assertFailsWith<IllegalArgumentException> { workspace.write("dangling.txt", "new") }
            assertFalse(missing.exists())
            Files.createSymbolicLink(File(parent, "root-link").toPath(), root.toPath())
            assertFailsWith<IllegalArgumentException> { JvmMobileWorkspace(File(parent, "root-link")) }
        } finally { clean(parent) }
    }

    @Test
    fun `unsafe entries invalid UTF8 and externally oversized files are rejected`() = runTest {
        val root = temp()
        try {
            val workspace = JvmMobileWorkspace(root)
            File(root, "invalid.txt").writeBytes(byteArrayOf(0xff.toByte(), 0xfe.toByte()))
            assertFails { workspace.read("invalid.txt") }
            File(root, "huge.txt").outputStream().use { it.write(ByteArray(MobileWorkspaceLimits.FILE_BYTES + 1)) }
            assertFailsWith<IllegalArgumentException> { workspace.read("huge.txt") }
            File(root, ".hidden").writeText("private")
            assertFailsWith<IllegalArgumentException> { workspace.list() }
        } finally { clean(root) }
    }
}
