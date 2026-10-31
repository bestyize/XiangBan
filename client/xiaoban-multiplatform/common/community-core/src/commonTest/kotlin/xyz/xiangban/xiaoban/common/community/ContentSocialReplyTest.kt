package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.*
import kotlin.test.*

private const val REPLY_ASSET = "11111111-2222-3333-4444-555555555555"
class ContentSocialReplyTest {
    @Test fun `content bookmarks and reports use exact private routes and current credentials`() = runTest {
        val s = FakeServer(); var token = "old"; val api = ContentSocialApi(s.client, CommunityConfig()) { token }
        s.on("/api/content-social/pack/7", body = """{"bookmarked":true}""")
        s.on("/api/content-social/pack/7/bookmark", body = """{"bookmarked":false}""")
        s.on("/api/content-social/scheme/8/report", body = """{"ok":true}""")
        assertTrue(api.state("pack", 7).bookmarked); token = "renewed"
        assertFalse(api.bookmark("pack", 7, false).bookmarked)
        api.report("scheme", 8, "copyright", "  公开内容侵权  ")
        assertEquals(listOf("GET", "DELETE", "POST"), s.requests.map { it.method.value })
        assertEquals(listOf("Bearer old", "Bearer renewed", "Bearer renewed"), s.requests.map { it.headers[HttpHeaders.Authorization] })
        assertEquals(setOf("reason", "detail"), CommunityJson.parseToJsonElement((s.requests.last().body as TextContent).text).jsonObject.keys)
        assertTrue(s.requests.none { "admin" in it.url.encodedPath }); s.client.close()
    }
    @Test fun `content bookmark pages decode real pack and safe scheme items`() = runTest {
        val s = FakeServer(); val api = ContentSocialApi(s.client, CommunityConfig()) { "access" }
        s.on("/api/me/content-bookmarks", body = """{"items":[{"kind":"scheme","id":8,"created_at":"2026-10-31T00:00:00Z","content":{"id":8,"title":"公开方案","items":[{"pack_id":99,"version":1,"available":false}]}}],"page":2,"page_size":20,"total":21}""")
        val result = api.bookmarks("scheme", 2)
        assertEquals("2", s.requests.last().url.parameters["page"]); assertEquals("scheme", s.requests.last().url.parameters["kind"])
        assertFalse(result.items.single().content.items.single().available)
        assertNull(result.items.single().content.items.single().title)
        s.on("/api/me/content-bookmarks", body = """{"items":[{"kind":"pack","id":7,"created_at":"now","content":{"id":7,"kind":"effect","title":"作品","version":3}}],"total":1}""")
        assertEquals(3, api.bookmarks("pack").items.single().content.version); s.client.close()
    }
    @Test fun `anonymous invalid and unavailable content never appears bookmarked or reported`() = runTest {
        val s = FakeServer(); val anonymous = ContentSocialApi(s.client, CommunityConfig()) { null }
        assertEquals(401, assertFailsWith<ApiException> { anonymous.state("pack", 7) }.status)
        assertEquals(401, assertFailsWith<ApiException> { anonymous.bookmark("scheme", 8, true) }.status)
        assertTrue(s.requests.isEmpty())
        val api = ContentSocialApi(s.client, CommunityConfig()) { "access" }
        assertEquals(400, assertFailsWith<ApiException> { api.bookmark("post", 7, true) }.status)
        assertEquals(400, assertFailsWith<ApiException> { api.report("pack", 7, "private", "") }.status)
        s.on("/api/content-social/pack/7/bookmark", HttpStatusCode.NotFound, """{"message":"内容已不可用"}""")
        assertEquals(404, assertFailsWith<ApiException> { api.bookmark("pack", 7, true) }.status)
        s.client.close()
    }
    @Test fun `reply sends typed references and free attachment association without local paths`() = runTest {
        val s = FakeServer(); val api = ForumApi(s.client, CommunityConfig()) { "access" }
        s.on("/api/topics/9/posts", body = """{"id":12,"floor":2}""")
        api.reply(9, "我的作品", workRefs = listOf(WorkReference("pack", 7, 3), WorkReference("scheme", 8)), attachments = listOf(ForumAttachment(REPLY_ASSET, "说明.zip", 10)))
        val json = CommunityJson.parseToJsonElement((s.requests.single().body as TextContent).text).jsonObject
        assertEquals(setOf("body", "work_refs", "attachment_ids", "attachment_prices"), json.keys)
        assertEquals(3, json["work_refs"]!!.jsonArray[0].jsonObject["version"]!!.jsonPrimitive.int)
        assertFalse("version" in json["work_refs"]!!.jsonArray[1].jsonObject)
        assertEquals(0, json["attachment_prices"]!!.jsonArray.single().jsonObject["price"]!!.jsonPrimitive.int)
        assertEquals(REPLY_ASSET, json["attachment_ids"]!!.jsonArray.single().jsonPrimitive.content)
        assertEquals("Bearer access", s.requests.single().headers[HttpHeaders.Authorization]); s.client.close()
    }
    @Test fun `edit omits unchanged fields and explicit empty lists clear associations`() = runTest {
        val s = FakeServer(); val api = ForumApi(s.client, CommunityConfig()) { "access" }
        s.on("/api/posts/12", body = """{"ok":true}""")
        api.editReply(12, "只修改正文")
        assertEquals(setOf("body"), CommunityJson.parseToJsonElement((s.requests.last().body as TextContent).text).jsonObject.keys)
        api.editReply(12, "清除引用附件", workRefs = emptyList(), attachments = emptyList())
        val cleared = CommunityJson.parseToJsonElement((s.requests.last().body as TextContent).text).jsonObject
        assertEquals(emptyList(), cleared["work_refs"]!!.jsonArray.toList()); assertEquals(emptyList(), cleared["attachment_ids"]!!.jsonArray.toList())
        assertTrue(s.requests.all { it.method.value == "PATCH" }); assertTrue(s.requests.none { it.method.value == "DELETE" }); s.client.close()
    }
    @Test fun `hidden or mismatched references never surface content metadata`() {
        val card = CommunityContentCard(7, "公开作品", kind = "effect", version = 3)
        val ref = ResolvedWorkReference("pack", 7, 3, true, card)
        assertNotNull(safeReferencedContent(ref))
        assertNull(safeReferencedContent(ref.copy(available = false)))
        assertNull(safeReferencedContent(ref.copy(id = 8)))
        assertNull(safeReferencedContent(ref.copy(version = 2)))
        assertNull(safeReferencedContent(ref.copy(kind = "chat")))
        assertNull(safeReferencedContent(ref.copy(kind = "scheme")))
        assertEquals("社区内容已下架", notificationHeadline(CommunityNotification(1, "content_removed", "不能显示私人下架理由")))
    }
    @Test fun `invalid duplicate oversized references and attachments are rejected before send`() = runTest {
        val s = FakeServer(); val api = ForumApi(s.client, CommunityConfig()) { "access" }
        for (refs in listOf(listOf(WorkReference("pack", 0)), listOf(WorkReference("scheme", 8, 1)), listOf(WorkReference("pack", 7, 0)), listOf(WorkReference("chat", 7)), List(4) { WorkReference("pack", it + 1L) }, listOf(WorkReference("pack", 7, 1), WorkReference("pack", 7, 2))))
            assertEquals(400, assertFailsWith<ApiException> { api.reply(9, "不应发送", workRefs = refs) }.status)
        assertEquals(400, assertFailsWith<ApiException> { api.reply(9, "附件无效", attachments = listOf(ForumAttachment("file:///private", "x", 1))) }.status)
        assertTrue(s.requests.isEmpty()); s.client.close()
    }
    @Test fun `own source and moderation failures remain authoritative and do not delete attachments`() = runTest {
        val s = FakeServer(); val api = ForumApi(s.client, CommunityConfig()) { "access" }
        s.on("/api/posts/12/source", body = """{"body":"原文","work_refs":[{"kind":"pack","id":7,"version":3}],"attachments":[{"id":"$REPLY_ASSET","name":"说明","size":10,"price":0}]}""")
        assertEquals(REPLY_ASSET, api.replySource(12).attachments.single().id)
        for (status in listOf(HttpStatusCode.Forbidden, HttpStatusCode.Conflict, HttpStatusCode.NotFound)) {
            s.on("/api/posts/12", status, """{"message":"请按当前权限重试"}""")
            assertEquals(status.value, assertFailsWith<ApiException> { api.editReply(12, "未成功保存", workRefs = emptyList()) }.status)
        }
        assertTrue(s.requests.none { it.method.value == "DELETE" }); s.client.close()
    }
    @Test fun `public URL picker resolves only anonymous pack or scheme and leaves version snapshot to server`() = runTest {
        val s = FakeServer(); val api = ReplyReferencesApi(s.client, CommunityConfig())
        s.on("/api/share/pack/7", body = """{"kind":"pack","title":"公开作品","summary":"公开摘要","url":"https://chat.xiangban.xyz/packs/7","pack_kind":"effect"}""")
        val selected = api.fromUrl("https://chat.xiangban.xyz/packs/7")
        assertEquals(WorkReference("pack", 7), selected.reference)
        assertNull(selected.reference.version)
        assertEquals("公开作品", selected.card.title)
        assertNull(s.requests.single().headers[HttpHeaders.Authorization])
        for (url in listOf("{kind:pack,id:7}", "https://evil.example/packs/7", "https://chat.xiangban.xyz/t/7", "https://chat.xiangban.xyz/packs/7?token=SECRET"))
            assertEquals(400, assertFailsWith<ApiException> { api.fromUrl(url) }.status)
        assertEquals(1, s.requests.size)
        s.on("/api/share/scheme/8", HttpStatusCode.NotFound, """{"message":"方案不存在"}""")
        assertEquals(404, assertFailsWith<ApiException> { api.fromUrl("https://chat.xiangban.xyz/schemes/8") }.status)
        s.client.close()
    }
    @Test fun `reply unauthorized and rate limited attempts do not auto replay or consume attachments`() = runTest {
        val s = FakeServer(); var token = "old"; val api = ForumApi(s.client, CommunityConfig()) { token }
        s.on("/api/topics/9/posts", HttpStatusCode.Unauthorized, """{"message":"请重新登录"}""")
        val files = listOf(ForumAttachment(REPLY_ASSET, "说明", 10))
        assertEquals(401, assertFailsWith<ApiException> { api.reply(9, "保留草稿", attachments = files) }.status)
        assertEquals(1, s.requests.size)
        token = "renewed"
        s.on("/api/topics/9/posts", HttpStatusCode.TooManyRequests, """{"message":"请稍后再发"}""")
        assertEquals(429, assertFailsWith<ApiException> { api.reply(9, "明确重试", attachments = files) }.status)
        assertEquals("Bearer renewed", s.requests.last().headers[HttpHeaders.Authorization])
        assertEquals(2, s.requests.size); assertTrue(s.requests.none { it.method.value == "DELETE" })
        s.client.close()
    }

}
