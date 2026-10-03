package xyz.xiaban.server.users

import io.ktor.client.call.body
import io.ktor.client.request.*
import io.ktor.http.*
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.future.await
import xyz.xiaban.server.account.bind
import xyz.xiaban.server.support.*
import xyz.xiaban.shared.*
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlin.test.*

class IdentityFlowTest {
    @Test fun `only a valid signed-in session restores an email identity`() = apiTest {
        val guest = session()
        assertTrue(guest.profile.userId > 0); assertEquals("guest", guest.profile.accountType)
        val signed = bind(guest.token, "identity@example.com")
        assertEquals(guest.profile.userId, signed.profile.userId); assertEquals("registered", signed.profile.accountType)
        val renewed = client.post(ApiPaths.SESSION) {
            bearerAuth(signed.token); contentType(ContentType.Application.Json); setBody(SessionRequest(guest.deviceId))
        }.body<SessionResponse>()
        assertEquals(signed.profile.userId, renewed.profile.userId)
        val unauthenticated = client.post(ApiPaths.SESSION) {
            header(HttpHeaders.Cookie, "$DEVICE_COOKIE=${guest.deviceId}"); contentType(ContentType.Application.Json); setBody(SessionRequest(guest.deviceId))
        }.body<SessionResponse>()
        assertEquals("guest", unauthenticated.profile.accountType)
        assertNotEquals(signed.profile.userId, unauthenticated.profile.userId)
        assertEquals(HttpStatusCode.Unauthorized, get(ApiPaths.PROFILE, guest.token).status)
        assertEquals(HttpStatusCode.OK, get(ApiPaths.PROFILE, signed.token).status)
    }

    @Test fun `nicknames enforce lengths normalization and a concurrent unique identity`() = apiTest {
        val a = session(); val b = session()
        for (invalid in listOf("阿", "abc", "1234", "１２３４")) {
            assertEquals(HttpStatusCode.BadRequest, put(ApiPaths.PROFILE, a.token, ProfileUpdate(nickname = invalid)).status, invalid)
        }
        for (valid in listOf("阿明", "Abcd", "a123", "User20261234")) {
            assertEquals(HttpStatusCode.OK, put(ApiPaths.PROFILE, a.token, ProfileUpdate(nickname = valid)).status, valid)
        }
        put(ApiPaths.PROFILE, a.token, ProfileUpdate(nickname = "Alice"))
        assertEquals(HttpStatusCode.Conflict, put(ApiPaths.PROFILE, b.token, ProfileUpdate(nickname = "Ａｌｉｃｅ", workEnd = "20:00")).status)
        assertEquals("18:00", get(ApiPaths.PROFILE, b.token).body<ProfileView>().workEnd)
        val statuses = coroutineScope { listOf(a, b).map { session -> async { put(ApiPaths.PROFILE, session.token, ProfileUpdate(nickname = "RaceName")).status } }.awaitAll() }
        assertEquals(setOf(HttpStatusCode.OK, HttpStatusCode.Conflict), statuses.toSet())
        assertEquals(a.profile.userId, get(ApiPaths.PROFILE, a.token).body<ProfileView>().userId)
    }

    @Test fun `avatar JPEGs enforce 64KB dimensions ownership and opaque image replacement`() = apiTest {
        val owner = session(); val other = session()
        suspend fun upload(bytes: ByteArray) = client.put("/api/account/avatar") { bearerAuth(owner.token); contentType(ContentType.Image.JPEG); setBody(bytes) }
        assertEquals(HttpStatusCode.PayloadTooLarge, upload(ByteArray(65537)).status)
        assertEquals(HttpStatusCode.BadRequest, upload("<svg onload='evil()'/>".toByteArray()).status)
        assertEquals(HttpStatusCode.BadRequest, upload(jpeg(513)).status)
        val saved = upload(jpeg(64)).body<ProfileView>()
        val firstUrl = assertNotNull(saved.avatarUrl)
        assertEquals(owner.profile.userId, saved.userId)
        assertEquals(HttpStatusCode.OK, get(firstUrl, null).status)
        assertNull(get(ApiPaths.PROFILE, other.token).body<ProfileView>().avatarUrl)
        val exact = jpeg(64).copyOf(65536)
        val replaced = upload(exact).body<ProfileView>()
        assertNotEquals(saved.avatarUrl, replaced.avatarUrl)
        assertEquals(HttpStatusCode.NotFound, get(firstUrl, null).status)
        val removed = delete("/api/account/avatar", owner.token).body<ProfileView>()
        assertNull(removed.avatarUrl)
        assertEquals(HttpStatusCode.Unauthorized, client.put("/api/account/avatar") { contentType(ContentType.Image.JPEG); setBody(jpeg(64)) }.status)
    }

    @Test fun `verification mail has clear HTML and plaintext codes with a ten-minute server expiry`() = apiTest {
        val guest = session()
        put(ApiPaths.CONSENT, guest.token, ConsentUpdate("0.2", false))
        post(ApiPaths.BIND_CODE, guest.token, EmailCodeRequest("mail-style@example.com"))
        val mail = TestEnv.awaitMail { it.to == "mail-style@example.com" }
        val code = Regex("\\d{6}").find(mail.subject)!!.value
        val html = assertNotNull(mail.html)
        assertContains(mail.text, code); assertContains(html, code); assertContains(html, "10 分钟内有效")
        val key = "xb:code:bind:${SessionService.sha256("mail-style@example.com")}"
        val ttl = services.redis.cmd.ttl(key).await()
        assertTrue(ttl in 580..600)
        services.redis.cmd.expire(key, 0).await()
        assertEquals(HttpStatusCode.BadRequest, post(ApiPaths.BIND, guest.token, BindEmailRequest("mail-style@example.com", code)).status)
    }
}

private fun jpeg(size: Int): ByteArray {
    val image = BufferedImage(size, size, BufferedImage.TYPE_INT_RGB)
    return ByteArrayOutputStream().use { out -> ImageIO.write(image, "jpeg", out); out.toByteArray() }
}
