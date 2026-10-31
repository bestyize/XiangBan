package xyz.xiangban.xiaoban.common.community

import java.io.ByteArrayOutputStream
import java.security.MessageDigest
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream
import kotlinx.coroutines.test.runTest
import kotlin.test.*

class CommunityGameTrialTest {
    private val manifest = """{"schemaVersion":1,"kind":"game","id":"test-game","name":"测试游戏","description":"离线游戏","save":true,"layer":"space"}"""
    private fun zip(files: List<Pair<String, ByteArray>>): ByteArray = ByteArrayOutputStream().use { output ->
        ZipOutputStream(output).use { zip -> files.forEach { (path, bytes) -> zip.putNextEntry(ZipEntry(path)); zip.write(bytes); zip.closeEntry() } }
        output.toByteArray()
    }
    private fun bytes(extra: List<Pair<String, ByteArray>> = emptyList(), manifestText: String = manifest) = zip(listOf("manifest.json" to manifestText.toByteArray(), "index.html" to "<h1>Play</h1>".toByteArray()) + extra)
    private fun descriptor(bytes: ByteArray) = CommunityGameDownload(55, "game", 1,
        MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }, bytes.size.toLong(), "https://resource.example/game.zip")
    private fun read(bytes: ByteArray) = CommunityGameArchive.read(bytes, descriptor(bytes))

    @Test fun `valid archive remains in memory with save support`() {
        val game = read(bytes(listOf("assets/style.css" to "body{}".toByteArray())))
        assertEquals("test-game", game.game.id); assertTrue(game.game.save)
        assertEquals(setOf("manifest.json", "index.html", "assets/style.css"), game.files.keys)
    }
    @Test fun `integrity and exact published version are required`() {
        val zip = bytes(); val info = descriptor(zip)
        assertFails { CommunityGameArchive.read(zip, info.copy(sha256 = "0".repeat(64))) }
        assertFails { CommunityGameArchive.read(zip, info.copy(size = zip.size + 1L)) }
        val pack = Pack(55, "game", title = "Game", versions = listOf(PackVersion(1, sha256 = info.sha256, size = info.size)))
        info.validate(pack, 1)
        assertFails { info.validate(pack.copy(status = "removed"), 1) }
        assertFails { info.validate(pack, 2) }
        assertFails { info.validate(pack.copy(id = 56), 1) }
    }
    @Test fun `paths duplicate casing and binary data are rejected`() {
        listOf("../escape.js", "/escape.js", "a/../../escape.js", "a\\escape.js", "a.png", "INDEX.html", "index.html/child.js").forEach { path ->
            assertFails(path) { read(bytes(listOf(path to "bad".toByteArray()))) }
        }
        assertFails { read(bytes(listOf("bad.js" to byteArrayOf(0xC3.toByte(), 0x28)))) }
        assertFails { read(bytes(listOf("bad.js" to byteArrayOf(0)))) }
    }
    @Test fun `expanded limits and unsupported formats fail before execution`() {
        assertFails { read(bytes(listOf("large.js" to ByteArray(MobileGameFormat.FILE_BYTES + 1) { 65 }))) }
        assertFails { read(bytes((1..41).map { "f$it.js" to byteArrayOf(65) })) }
        assertFails { read(bytes(manifestText = manifest.replace("\"schemaVersion\":1", "\"schemaVersion\":2"))) }
        assertFails { read(bytes(manifestText = manifest.replace("\"space\"", "\"desktop\""))) }
        assertFails { read(bytes(manifestText = manifest.dropLast(1) + ",\"capabilities\":[\"network\"]}")) }
        assertFails { read(zip(listOf("index.html" to "ok".toByteArray()))) }
    }
    @Test fun `history replaces revisions preserves order and is bounded`() {
        val old = (1L..15).map { RecentCommunityGame(it,1,"game$it") }
        val next = CommunityGameHistory.opened(old, RecentCommunityGame(5,2,"updated"))
        assertEquals(12,next.size); assertEquals(5L,next.first().packId); assertEquals(2,next.first().version)
        assertEquals(1,next.count { it.packId == 5L })
        assertEquals(next,CommunityGameHistory.decode(CommunityGameHistory.encode(next)))
        assertTrue(CommunityGameHistory.decode("invalid").isEmpty())
    }
    @Test fun `only canonical public pack links become cards`() {
        val html = """<a href="/packs/55">game</a><a href='https://chat.xiangban.xyz/packs/55'>duplicate</a><a href='https://evil.example/packs/3'>external</a><a href='/packs/2?token=secret'>query</a><a href='/packs/6'>next</a>"""
        assertEquals(listOf(55L,6L), linkedCommunityPacks(html,CommunityConfig()))
    }
    @Test fun `game collection keeps pack identity and real paging`() = runTest {
        val server = FakeServer(); val api = ContentSocialApi(server.client,CommunityConfig()) { "access" }
        server.on("/api/me/content-bookmarks?kind=game&page=2", body = """{"items":[{"kind":"pack","id":55,"created_at":"now","content":{"id":55,"title":"Game","kind":"game"}}],"page":2,"page_size":20,"total":45}""")
        val page = api.bookmarks("game",2); assertTrue(page.hasMore); assertEquals("pack",page.items.single().kind)
        server.client.close()
    }
}
