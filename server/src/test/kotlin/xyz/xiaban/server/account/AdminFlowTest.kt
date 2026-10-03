package xyz.xiaban.server.account

import io.ktor.client.call.body
import io.ktor.client.request.*
import io.ktor.http.*
import xyz.xiaban.server.admin.AdminLogin
import xyz.xiaban.server.admin.AdminPassword
import xyz.xiaban.server.admin.EmailSettings
import xyz.xiaban.server.admin.EmailSettingsStore
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.support.*
import xyz.xiaban.shared.OkResponse
import kotlin.test.*

class AdminFlowTest {
    @Test fun `SMTP settings require admin and persist encrypted without returning the credential`() = apiTest {
        assertEquals(HttpStatusCode.Unauthorized, client.get("/api/admin/email").status)
        val login = client.post("/api/admin/login") {
            contentType(ContentType.Application.Json); setBody(AdminLogin("testadmin", "test-admin-password-2026"))
        }
        assertEquals(HttpStatusCode.OK, login.status)
        val header = login.headers[HttpHeaders.SetCookie]!!
        assertTrue(header.contains("HttpOnly", true)); assertTrue(header.contains("SameSite=Strict", true))
        val cookie = header.substringBefore(';')
        val settings = EmailSettings(host = "smtp.example.com", user = "sender@example.com", from = "sender@example.com", password = "smtp-test-secret")
        val saved = client.put("/api/admin/email") {
            header(HttpHeaders.Cookie, cookie); contentType(ContentType.Application.Json); setBody(settings)
        }.body<EmailSettings>()
        assertEquals("", saved.password); assertTrue(saved.passwordConfigured)
        val encrypted = services.db.read { c -> c.queryOne("select encrypted_value from runtime_settings where key='email'") { it.getString(1) } }!!
        assertFalse(encrypted.contains("smtp-test-secret"))
        assertEquals("smtp-test-secret", EmailSettingsStore(services.db, services.config).current().password)
        client.put("/api/admin/email") {
            header(HttpHeaders.Cookie, cookie); contentType(ContentType.Application.Json); setBody(saved.copy(fromName = "新名字"))
        }.body<EmailSettings>()
        assertEquals("smtp-test-secret", services.emailSettings.current().password)
        val badOrigin = client.put("/api/admin/email") {
            header(HttpHeaders.Cookie, cookie); header(HttpHeaders.Origin, "https://foreign.example.com")
            contentType(ContentType.Application.Json); setBody(settings)
        }
        assertEquals(HttpStatusCode.BadRequest, badOrigin.status)
        client.post("/api/admin/logout") { header(HttpHeaders.Cookie, cookie) }.body<OkResponse>()
        assertEquals(HttpStatusCode.Unauthorized, client.get("/api/admin/email") { header(HttpHeaders.Cookie, cookie) }.status)
    }

    @Test fun `changing administrator password revokes all existing admin sessions`() = apiTest {
        val response = client.post("/api/admin/login") {
            contentType(ContentType.Application.Json); setBody(AdminLogin("testadmin", "test-admin-password-2026"))
        }
        val cookie = response.headers[HttpHeaders.SetCookie]!!.substringBefore(';')
        assertEquals(HttpStatusCode.OK, client.put("/api/admin/password") {
            header(HttpHeaders.Cookie, cookie); contentType(ContentType.Application.Json)
            setBody(AdminPassword("test-admin-password-2026", "replacement-admin-password"))
        }.status)
        assertEquals(HttpStatusCode.Unauthorized, client.get("/api/admin/session") { header(HttpHeaders.Cookie, cookie) }.status)
        assertEquals(HttpStatusCode.Unauthorized, client.post("/api/admin/login") {
            contentType(ContentType.Application.Json); setBody(AdminLogin("testadmin", "test-admin-password-2026"))
        }.status)
        assertEquals(HttpStatusCode.OK, client.post("/api/admin/login") {
            contentType(ContentType.Application.Json); setBody(AdminLogin("testadmin", "replacement-admin-password"))
        }.status)
    }
}
