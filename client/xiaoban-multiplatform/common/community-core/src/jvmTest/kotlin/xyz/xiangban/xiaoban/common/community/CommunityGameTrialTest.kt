package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlin.test.*

class CommunityGameTrialTest {
    private fun descriptor(kind: String = "app", size: Long = 1000, sha: String = "a".repeat(64)) = CommunityGameDownload(55, kind, 1, sha, size, "https://resource.example/game.zip")
    private fun pack(manifest: kotlinx.serialization.json.JsonObject, info: CommunityGameDownload, kind: String = info.kind) =
        Pack(info.packId, kind, title = "Work", versions = listOf(PackVersion(1, sha256 = info.sha256, size = info.size, manifest = manifest)))
    private val protocol2 = buildJsonObject { put("runtimeProtocol", 2) }

    @Test fun `only protocol 2 packs of the exact published version can run`() {
        val info = descriptor()
        assertEquals(2, info.validate(pack(protocol2, info), 1))
        assertEquals(2, descriptor(kind = "game").validate(pack(protocol2, descriptor(kind = "game")), 1))
        assertFails { info.validate(pack(protocol2, info).copy(status = "removed"), 1) }
        assertFails { info.validate(pack(protocol2, info), 2) }
        assertFails { info.validate(pack(protocol2, info).copy(id = 56), 1) }
        assertFails { info.validate(pack(protocol2, info, kind = "game"), 1) }
        assertFails { info.validate(pack(protocol2, info.copy(sha256 = "b".repeat(64))), 1) }
        assertFails { info.validate(pack(protocol2, info.copy(size = 1001)), 1) }
        assertFails { info.validate(pack(buildJsonObject {}, info), 1) }
        assertFails { info.validate(pack(buildJsonObject { put("runtimeProtocol", 1) }, info), 1) }
        assertFails { info.validate(pack(buildJsonObject { put("runtimeProtocol", "2") }, info), 1) }
    }
    @Test fun `the signed archive limit is one hundred megabytes`() {
        val limit = CommunityGameDownload.MAX_RUNTIME_V2_BYTES.toLong()
        val ok = descriptor(size = limit)
        assertEquals(2, ok.validate(pack(protocol2, ok), 1))
        val big = descriptor(size = limit + 1)
        assertFails { big.validate(pack(protocol2, big), 1) }
        val bad = descriptor(sha = "xyz")
        assertFails { bad.validate(pack(protocol2, bad), 1) }
    }

    @Test fun `runtime negotiation and network policy come from pack requests`() = runTest {
        val server = FakeServer()
        server.on("/api/packs/55", body = """{"id":55,"kind":"app","title":"Tool","network_disabled":true}""")
        val api = ForumApi(server.client, CommunityConfig(runtimeV2 = true)) { null }
        assertEquals(true, api.pack(55).networkDisabled)
        assertEquals("1,2", server.requests.single().headers["X-Xiaoban-Runtime"])
        server.client.close()
    }

    @Test fun `offline approval expires after one day and rejects a backward clock`() {
        val checkedAt = 1_800_000_000_000L
        val approval = CommunityToolApproval(checkedAt, CommunityConfig().forumBase, "tool", "Tool")
        val restored = CommunityJson.decodeFromString<CommunityToolApproval>(CommunityJson.encodeToString(approval))
        assertTrue(restored.isFresh(checkedAt + 86_400_000))
        assertFalse(restored.isFresh(checkedAt + 86_400_001))
        assertFalse(restored.isFresh(checkedAt - 1))
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

    @Test fun `app collection accepts app packs and rejects games`() = runTest {
        val server = FakeServer()
        val api = ContentSocialApi(server.client, CommunityConfig()) { "access" }
        val app = """{"items":[{"kind":"pack","id":55,"created_at":"now","content":{"id":55,"title":"Tool","kind":"app"}}]}"""
        server.on("/api/me/content-bookmarks?kind=app&page=1", body = app)
        server.on("/api/me/content-bookmarks?kind=app&page=2", body = app.replace("\"app\"", "\"game\""))
        assertEquals(55L, api.bookmarks("app").items.single().id)
        assertFailsWith<ApiException> { api.bookmarks("app", 2) }
        server.client.close()
    }
}
