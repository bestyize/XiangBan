package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlin.test.*

private val sharingConfig = CommunityConfig()
class CommunitySharingTest {
    @Test fun `public paths round trip to canonical browser fallbacks`() {
        for ((kind, id, path) in listOf(Triple("pack", "17", "packs/17"), Triple("topic", "9", "t/9"), Triple("scheme", "8", "schemes/8"), Triple("project", "open_project-1", "projects/open_project-1"), Triple("weekly", "2026-W44", "weekly/2026-W44"))) {
            val target = assertNotNull(communityShareTarget("https://chat.xiangban.xyz/$path", sharingConfig))
            assertEquals(kind, target.kind); assertEquals(id, target.id)
            assertFalse(target.requestsApply)
            assertEquals("https://chat.xiangban.xyz/$path", target.webUrl(sharingConfig))
            assertEquals(target, communityShareTarget("xiaoban://community/open/$kind/$id", sharingConfig))
        }
    }
    @Test fun `all content apply links become previews and retain no version command`() {
        for (kind in listOf("effect", "reminder", "routine", "task", "goal", "widget", "game", "wallpaper", "skill", "expert", "workflow")) {
            val target = assertNotNull(communityShareTarget("xiaoban://community/apply/$kind/17?version=3", sharingConfig))
            assertTrue(target.requestsApply); assertEquals("pack", target.kind)
            assertEquals("https://chat.xiangban.xyz/packs/17", target.webUrl(sharingConfig))
        }
    }
    @Test fun `private routes credentials encoded separators and arbitrary authorities cannot open`() {
        for (raw in listOf(
            "https://evil.example/t/1", "https://chat.xiangban.xyz.evil.example/t/1", "https://evil@chat.xiangban.xyz/t/1",
            "https://chat.xiangban.xyz:8443/t/1", "http://chat.xiangban.xyz/t/1", "file:///t/1", "content://community/t/1", "javascript:alert(1)",
            "https://chat.xiangban.xyz/api/topics/1", "https://chat.xiangban.xyz/t/1?token=SECRET", "https://chat.xiangban.xyz/t/1#SECRET",
            "https://chat.xiangban.xyz/t/%31", "https://chat.xiangban.xyz/t/../1", "https://chat.xiangban.xyz/t/1/", "https://chat.xiangban.xyz/t/0",
            "https://chat.xiangban.xyz/t/01", "https://chat.xiangban.xyz/t/-1", "https://chat.xiangban.xyz/t/9223372036854775808",
            "xiaoban://community/open/chat/1", "xiaoban://community/open/topic/1?token=SECRET", "xiaoban://community:99/open/topic/1",
            "xiaoban://evil/open/topic/1", "xiaoban://community/apply/scheme/1", "xiaoban://community/apply/game/1?version=-1",
            "xiaoban://community/apply/game/1?version=1&version=2", "xiaoban://community/apply/game/1?command=exec", "xiaoban://community/apply/game/1?version=99999999999999",
            "xiaoban://community/open/topic/1\n", "https://chat.xiangban.xyz/projects/" + "a".repeat(101)
        )) assertNull(communityShareTarget(raw, sharingConfig), raw)
    }
    @Test fun `anonymous metadata contract drops executable links and cover addresses`() = runTest {
        val server = FakeServer()
        server.on("/api/share/pack/17", body = """{"kind":"pack","title":"公开标题","summary":"公开摘要","url":"https://chat.xiangban.xyz/packs/17","cover_url":"https://resource.example/cover?token=PRIVATE","app_link":"xiaoban://community/apply/game/17?version=3","one_click":true,"pack_kind":"game"}""")
        val info = CommunitySharingApi(server.client, sharingConfig).publicInfo(CommunityShareTarget("pack", "17"))
        assertEquals("公开标题", info.title); assertEquals("game", info.packKind); assertTrue(info.oneClick)
        assertNull(info.appLink); assertNull(info.coverUrl)
        assertEquals("https://chat.xiangban.xyz/packs/17", info.url)
        assertNull(server.requests.single().headers[HttpHeaders.Authorization])
        server.client.close()
    }
    @Test fun `private deleted metadata and network failure stay recoverable errors`() = runTest {
        val server = FakeServer(); val api = CommunitySharingApi(server.client, sharingConfig)
        server.on("/api/share/topic/1", HttpStatusCode.NotFound, """{"message":"帖子不存在"}""")
        assertEquals(404, assertFailsWith<ApiException> { api.publicInfo(CommunityShareTarget("topic", "1")) }.status)
        server.onThrow("/api/share/topic/1")
        assertEquals(0, assertFailsWith<ApiException> { api.publicInfo(CommunityShareTarget("topic", "1")) }.status)
        server.on("/api/share/topic/1", body = """{"kind":"topic","title":"已恢复","url":"https://chat.xiangban.xyz/t/1"}""")
        assertEquals("已恢复", api.publicInfo(CommunityShareTarget("topic", "1")).title)
        server.client.close()
    }
    @Test fun `metadata cannot redirect sharing to an external private or mismatched target`() = runTest {
        val server = FakeServer(); val api = CommunitySharingApi(server.client, sharingConfig)
        for ((kind, url, title) in listOf(Triple("topic", "https://evil.example/t/1", "标题"), Triple("topic", "https://chat.xiangban.xyz/t/2", "标题"), Triple("scheme", "https://chat.xiangban.xyz/t/1", "标题"), Triple("topic", "https://chat.xiangban.xyz/t/1?token=SECRET", "标题"), Triple("topic", "https://chat.xiangban.xyz/t/1", " "))) {
            server.on("/api/share/topic/1", body = """{"kind":"$kind","title":"$title","url":"$url"}""")
            assertFailsWith<ApiException> { api.publicInfo(CommunityShareTarget("topic", "1")) }
        }
        server.client.close()
    }
    @Test fun `bad ids are rejected without a network request`() = runTest {
        val server = FakeServer(); val api = CommunitySharingApi(server.client, sharingConfig)
        for (target in listOf(CommunityShareTarget("topic", "1?token=x"), CommunityShareTarget("chat", "1"), CommunityShareTarget("scheme", "0"), CommunityShareTarget("weekly", "../secret")))
            assertEquals(400, assertFailsWith<ApiException> { api.publicInfo(target) }.status)
        assertTrue(server.requests.isEmpty()); server.client.close()
    }
}
