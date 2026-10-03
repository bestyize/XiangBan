package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.client.request.*
import io.ktor.http.*
import xyz.xiaban.server.admin.*
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.guardian.GuardianService
import kotlinx.coroutines.future.await
import xyz.xiaban.server.support.*
import xyz.xiaban.server.users.SessionService
import java.time.Instant
import java.time.ZoneId
import kotlin.test.*

class AdminUsersFlowTest {
    @Test fun `editing validates all fields before writing and preserves verified identity`() = apiTest {
        val ids = seedUsers(2)
        val userId = ids[0]
        val ordinary = session().token
        val request = AdminUserUpdate(nickname = "运营称呼", companionName = "新小伴", personality = xyz.xiaban.shared.Personality.Gentle,
            cityCode = services.cities.all.first().code, timezone = "Asia/Shanghai", workStart = "08:30", workEnd = "17:30", payday = 15)
        assertEquals(HttpStatusCode.Unauthorized, put("/api/admin/users/$userId", ordinary, request).status)
        val cookie = adminCookie()
        val before = services.adminUsers.detail(userId)
        val result = client.put("/api/admin/users/$userId") {
            header(HttpHeaders.Cookie, cookie); header(HttpHeaders.Host, "localhost:18080"); header(HttpHeaders.Origin, "http://localhost:18080")
            contentType(ContentType.Application.Json); setBody(request)
        }
        assertEquals(HttpStatusCode.OK, result.status)
        val edited = result.body<AdminUserDetail>()
        assertEquals("运营称呼", edited.user.nickname); assertEquals("新小伴", edited.user.companionName)
        assertEquals("08:30", edited.workStart); assertEquals(15, edited.payday)
        assertEquals(before.user.email, edited.user.email); assertEquals(before.user.emailVerifiedAt, edited.user.emailVerifiedAt)
        assertEquals(HttpStatusCode.BadRequest, client.put("/api/admin/users/$userId") {
            header(HttpHeaders.Cookie, cookie); contentType(ContentType.Application.Json); setBody(AdminUserUpdate(nickname = "不应保存", workStart = "25:00"))
        }.status)
        assertEquals("运营称呼", services.adminUsers.detail(userId).user.nickname)
        assertEquals(HttpStatusCode.OK, client.put("/api/admin/users/$userId") {
            header(HttpHeaders.Cookie, cookie); contentType(ContentType.Application.Json); setBody(AdminUserUpdate(nickname = "", cityCode = "", payday = 0))
        }.status)
        val cleared = services.adminUsers.detail(userId)
        assertNull(cleared.user.nickname); assertNull(cleared.cityCode); assertNull(cleared.payday)
        val logs = services.adminUsers.auditPage(1, 20)
        assertEquals(2, logs.total); assertTrue(logs.items.all { it.action == "user_edit" && it.targetUserId == userId })
    }

    @Test fun `delete requires correct confirmation removes dependent data and rejects old tokens`() = apiTest {
        val victim = session()
        val id = services.tokens.userIdOrNull(victim.token)!!
        val survivor = session()
        services.users.setEmail(id, "delete-fixture@example.com")
        services.db.read { c ->
            c.execute("insert into checkins(user_id,work_day,checked_at,title,copy_id,library_reply) values (?,'2026-10-02',now(),'测试','test','reply')", id)
            c.execute("insert into memories(user_id,kind,content,source) values (?,'event','fixture','test')", id)
            c.execute("insert into events(user_id,name) values (?,'page_view')", id)
            c.execute("insert into guardians(user_id,contact_name,contact_email,self_name,threshold_days,base_at,consented_at,informed_at) values (?,'fixture','contact@example.com','fixture',1,now(),now(),now())", id)
        }
        services.guardian.reschedule(id)
        val cookie = adminCookie()
        assertEquals(HttpStatusCode.Unauthorized, client.delete("/api/admin/users/$id") {
            bearerAuth(victim.token); contentType(ContentType.Application.Json); setBody(AdminUserDelete(id))
        }.status)
        assertEquals(HttpStatusCode.BadRequest, client.delete("/api/admin/users/$id") {
            header(HttpHeaders.Cookie, cookie); contentType(ContentType.Application.Json); setBody(AdminUserDelete(id + 1))
        }.status)
        assertTrue(services.users.exists(id))
        assertEquals(HttpStatusCode.BadRequest, client.delete("/api/admin/users/$id") {
            header(HttpHeaders.Cookie, cookie); header(HttpHeaders.Origin, "https://foreign.example.com")
            contentType(ContentType.Application.Json); setBody(AdminUserDelete(id))
        }.status)
        assertEquals(HttpStatusCode.OK, client.delete("/api/admin/users/$id") {
            header(HttpHeaders.Cookie, cookie); header(HttpHeaders.Host, "localhost:18080"); header(HttpHeaders.Origin, "http://localhost:18080")
            contentType(ContentType.Application.Json); setBody(AdminUserDelete(id))
        }.status)
        assertFalse(services.users.exists(id))
        for (table in listOf("devices", "companions", "checkins", "memories", "events", "guardians", "admin_user_notes")) {
            assertEquals(0L, services.db.read { c -> c.queryOne("select count(*) from $table where user_id=?", id) { it.getLong(1) } }, table)
        }
        assertNull(services.redis.cmd.zscore(GuardianService.QUEUE, "notify:$id").await())
        assertEquals(HttpStatusCode.Unauthorized, get(xyz.xiaban.shared.ApiPaths.PROFILE, victim.token).status)
        assertEquals(HttpStatusCode.Unauthorized, get(xyz.xiaban.shared.ApiPaths.GUARDIAN, victim.token).status)
        assertEquals(HttpStatusCode.OK, get(xyz.xiaban.shared.ApiPaths.PROFILE, survivor.token).status)
        val logs = services.adminUsers.auditPage(1, 20)
        assertEquals("user_delete", logs.items.first().action); assertTrue(logs.items.first().summary.contains("#$id"))
        val fresh = session(victim.deviceId)
        assertNotEquals(victim.profile.inviteCode, fresh.profile.inviteCode)
    }

    @Test fun `users are paged with stable ordering literal search and validated filters`() = apiTest {
        val ids = seedUsers(23)
        val cookie = adminCookie()
        suspend fun list(query: String) = client.get("/api/admin/users?$query") { header(HttpHeaders.Cookie, cookie) }.body<AdminUserPage>()
        val first = list("page=1&pageSize=10")
        val second = list("page=2&pageSize=10")
        val last = list("page=999999&pageSize=10")
        assertEquals(23, first.total); assertEquals(3, first.totalPages)
        assertEquals(ids.reversed().take(10), first.items.map { it.id })
        assertTrue(first.items.map { it.id }.intersect(second.items.map { it.id }.toSet()).isEmpty())
        assertEquals(3, last.page); assertEquals(3, last.items.size)
        assertEquals(ids.take(10), list("sort=oldest&pageSize=10").items.map { it.id })
        assertEquals(5, list("type=email").total); assertEquals(18, list("type=guest").total)
        assertEquals(12, list("onboarding=done").total)
        assertEquals(listOf(ids[2]), list("q=person_2").items.map { it.id })
        assertEquals(listOf(ids[0]), list("q=%25").items.map { it.id })
        assertEquals(listOf(ids[4]), list("q=测试昵称").items.map { it.id })
        assertEquals(0, list("q=x%27%20OR%201%3D1%20--").total)
        val tomorrow = TestEnv.clock.instant().atZone(ZoneId.of("Asia/Shanghai")).toLocalDate().plusDays(1)
        assertEquals(0, list("from=$tomorrow").total)
        for (query in listOf("page=0", "page=x", "pageSize=101", "sort=invalid", "type=invalid", "from=not-a-date", "from=2026-10-03&to=2026-10-02")) {
            assertEquals(HttpStatusCode.BadRequest, client.get("/api/admin/users?$query") { header(HttpHeaders.Cookie, cookie) }.status, query)
        }
    }

    @Test fun `detail and notes are administrator-only with accurate activity and atomic audit logs`() = apiTest {
        val ids = seedUsers(3)
        val ordinary = session().token
        assertEquals(HttpStatusCode.Unauthorized, get("/api/admin/users", ordinary).status)
        assertEquals(HttpStatusCode.Unauthorized, get("/api/admin/users/${ids[0]}", ordinary).status)
        assertEquals(HttpStatusCode.Unauthorized, put("/api/admin/users/${ids[0]}/note", ordinary, AdminNoteUpdate("denied")).status)
        services.db.read { c ->
            c.execute("insert into checkins(user_id,work_day,checked_at,title,copy_id,library_reply) values (?,'2026-10-02',now(),'测试打卡','test','reply')", ids[0])
            c.execute("insert into checkins(user_id,work_day,checked_at,title,copy_id,library_reply,revoked_at) values (?,'2026-10-01',now(),'已撤销','test','reply',now())", ids[0])
        }
        val cookie = adminCookie()
        suspend fun detail() = client.get("/api/admin/users/${ids[0]}") { header(HttpHeaders.Cookie, cookie) }.body<AdminUserDetail>()
        assertEquals(1, detail().user.checkinCount); assertEquals(1, detail().recentCheckins.size)
        assertEquals(1, detail().devices.size); assertNotNull(detail().user.emailVerifiedAt)
        assertEquals(HttpStatusCode.OK, client.put("/api/admin/users/${ids[0]}/note") {
            header(HttpHeaders.Cookie, cookie); header(HttpHeaders.Host, "localhost:18080"); header(HttpHeaders.Origin, "http://localhost:18080")
            contentType(ContentType.Application.Json); setBody(AdminNoteUpdate("  收不到验证码，待跟进  "))
        }.status)
        val updated = detail()
        assertEquals("收不到验证码，待跟进", updated.note); assertEquals("testadmin", updated.noteUpdatedBy); assertTrue(updated.user.hasNote)
        val logs = client.get("/api/admin/audit") { header(HttpHeaders.Cookie, cookie) }.body<AdminAuditPage>()
        assertEquals(1, logs.total); assertEquals(ids[0], logs.items.single().targetUserId); assertEquals("user_note", logs.items.single().action)
        assertFalse(logs.items.single().summary.contains("收不到验证码"))
        assertEquals(HttpStatusCode.NotFound, client.get("/api/admin/users/999999") { header(HttpHeaders.Cookie, cookie) }.status)
        assertEquals(HttpStatusCode.BadRequest, client.put("/api/admin/users/${ids[0]}/note") {
            header(HttpHeaders.Cookie, cookie); contentType(ContentType.Application.Json); setBody(AdminNoteUpdate("x".repeat(2001)))
        }.status)
        val overview = client.get("/api/admin/overview") { header(HttpHeaders.Cookie, cookie) }.body<AdminOverview>()
        assertEquals(4, overview.totalUsers); assertEquals(3, overview.emailUsers); assertEquals(1, overview.guestUsers); assertEquals(1, overview.totalCheckins)
    }
}

private suspend fun ApiScope.seedUsers(count: Int): List<Long> {
    val ids = (0 until count).map { services.users.findOrCreate(SessionService.sha256("admin-users-fixture-$it"), "Asia/Shanghai") }
    val base = Instant.now().minusSeconds(86400L)
    services.db.tx { c -> ids.forEachIndexed { index, id ->
        c.execute("update users set created_at=?,email=?,email_verified_at=?,onboarded_at=? where id=?", base.plusSeconds(index.toLong()),
            if (index < 5) "person_$index@example.com" else null, if (index < 5) base else null, if (index % 2 == 0) base else null, id)
        c.execute("update companions set name=? where user_id=?", if (index == 0) "小伴_百分%100" else "小伴$index", id)
        c.execute("update user_profiles set nickname=?,nickname_key=? where user_id=?", if (index == 4) "测试昵称" else null, if (index == 4) "测试昵称" else null, id)
    } }
    return ids
}

private suspend fun ApiScope.adminCookie(): String {
    val response = client.post("/api/admin/login") {
        header(HttpHeaders.Host, "localhost:18080"); header(HttpHeaders.Origin, "http://localhost:18080")
        contentType(ContentType.Application.Json); setBody(AdminLogin("testadmin", "test-admin-password-2026"))
    }
    assertEquals(HttpStatusCode.OK, response.status)
    return response.headers[HttpHeaders.SetCookie]!!.substringBefore(';')
}
