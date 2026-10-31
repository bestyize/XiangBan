package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlin.test.*

private const val BACKUP_ID = "11111111-2222-3333-4444-555555555555"
private const val BACKUP_JSON = """{"id":"$BACKUP_ID","schema_version":1,"title":"我的私有项目","summary":"选择的公开引用","items":[{"pack_id":7,"version":3,"available":true,"kind":"effect","title":"公开作品"},{"pack_id":8,"version":1,"available":false}],"revision":3,"status":"active","updated_at":"2026-10-31T00:00:00Z"}"""
class SchemeBackupsTest {
    @Test fun `default closed guard performs no request or token refresh`() = runTest {
        val s = FakeServer(); var tokens = 0
        val api = SchemeBackupsApi(s.client, CommunityConfig(), { tokens++; "access" })
        assertEquals(403, assertFailsWith<ApiException> { api.list() }.status)
        assertEquals(403, assertFailsWith<ApiException> { api.detail(BACKUP_ID) }.status)
        assertEquals(0, tokens); assertTrue(s.requests.isEmpty()); s.client.close()
    }
    @Test fun `explicit opt in reads only own paginated active or removed references`() = runTest {
        val s = FakeServer(); var enabled = true
        val api = SchemeBackupsApi(s.client, CommunityConfig(), { "owner" }, { enabled })
        s.on("/api/me/scheme-backups", body = """{"items":[$BACKUP_JSON],"page":2,"page_size":20,"total":21}""")
        val page = api.list(2)
        assertEquals("2", s.requests.last().url.parameters["page"]); assertEquals("active", s.requests.last().url.parameters["status"])
        assertFalse(page.items.single().items.last().available); assertNull(page.items.single().items.last().title)
        api.list(1, removed = true)
        assertEquals("removed", s.requests.last().url.parameters["status"])
        assertTrue(s.requests.all { it.method.value == "GET" && it.headers[HttpHeaders.Authorization] == "Bearer owner" })
        enabled = false
        assertEquals(403, assertFailsWith<ApiException> { api.list() }.status)
        assertEquals(2, s.requests.size); s.client.close()
    }
    @Test fun `detail is owner authenticated and never leaks arbitrary id path or other owners`() = runTest {
        val s = FakeServer(); val api = SchemeBackupsApi(s.client, CommunityConfig(), { "owner" }, { true })
        assertEquals(400, assertFailsWith<ApiException> { api.detail("../secret?token=x") }.status)
        assertTrue(s.requests.isEmpty())
        s.on("/api/me/scheme-backups/$BACKUP_ID", body = BACKUP_JSON)
        assertEquals(3L, api.detail(BACKUP_ID).revision)
        s.on("/api/me/scheme-backups/$BACKUP_ID", HttpStatusCode.NotFound, """{"message":"方案备份不存在"}""")
        assertEquals(404, assertFailsWith<ApiException> { api.detail(BACKUP_ID) }.status)
        assertTrue(s.requests.all { it.method.value == "GET" }); s.client.close()
    }
    @Test fun `anonymous revoked consent and unsupported schema never load a backup`() = runTest {
        val s = FakeServer(); val anonymous = SchemeBackupsApi(s.client, CommunityConfig(), { null }, { true })
        assertEquals(401, assertFailsWith<ApiException> { anonymous.list() }.status)
        var enabled = true
        val revoked = SchemeBackupsApi(s.client, CommunityConfig(), { enabled = false; "owner" }, { enabled })
        assertEquals(403, assertFailsWith<ApiException> { revoked.list() }.status)
        assertTrue(s.requests.isEmpty())
        val api = SchemeBackupsApi(s.client, CommunityConfig(), { "owner" }, { true })
        s.on("/api/me/scheme-backups/$BACKUP_ID", body = BACKUP_JSON.replace("\"schema_version\":1", "\"schema_version\":2"))
        assertEquals(200, assertFailsWith<ApiException> { api.detail(BACKUP_ID) }.status)
        s.client.close()
    }
}
