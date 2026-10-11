package xyz.xiaban.server.users

import io.ktor.client.call.body
import io.ktor.client.request.*
import io.ktor.http.*
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlin.test.*
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.account.session.SessionApiPaths
import xyz.xiaban.contracts.account.session.SessionRequest
import xyz.xiaban.contracts.account.session.SessionResponse
import xyz.xiaban.server.account.bind
import xyz.xiaban.server.support.*

class IdentityFlowTest {
    @Test fun `only a valid signed-in session restores an email identity`() = apiTest {
        val guest = session()
        assertTrue(guest.profile.userId > 0); assertEquals("guest", guest.profile.accountType)
        val signed = bind(guest.token, "identity@example.com")
        assertEquals(guest.profile.userId, signed.profile.userId); assertEquals("registered", signed.profile.accountType)
        val renewed = client.post(SessionApiPaths.SESSION) {
            bearerAuth(signed.token); contentType(ContentType.Application.Json); setBody(SessionRequest(guest.deviceId))
        }.body<SessionResponse>()
        assertEquals(signed.profile.userId, renewed.profile.userId)
        val unauthenticated = client.post(SessionApiPaths.SESSION) {
            header(HttpHeaders.Cookie, "$DEVICE_COOKIE=${guest.deviceId}"); contentType(ContentType.Application.Json); setBody(SessionRequest(guest.deviceId))
        }.body<SessionResponse>()
        assertEquals("guest", unauthenticated.profile.accountType)
        assertNotEquals(signed.profile.userId, unauthenticated.profile.userId)
        assertEquals(HttpStatusCode.Unauthorized, get(ProfileApiPaths.PROFILE, guest.token).status)
        assertEquals(HttpStatusCode.OK, get(ProfileApiPaths.PROFILE, signed.token).status)
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
        assertNull(get(ProfileApiPaths.PROFILE, other.token).body<ProfileView>().avatarUrl)
        val exact = jpeg(64).copyOf(65536)
        val replaced = upload(exact).body<ProfileView>()
        assertNotEquals(saved.avatarUrl, replaced.avatarUrl)
        assertEquals(HttpStatusCode.NotFound, get(firstUrl, null).status)
        val removed = delete("/api/account/avatar", owner.token).body<ProfileView>()
        assertNull(removed.avatarUrl)
        assertEquals(HttpStatusCode.Unauthorized, client.put("/api/account/avatar") { contentType(ContentType.Image.JPEG); setBody(jpeg(64)) }.status)
    }
}

private fun jpeg(size: Int): ByteArray {
    val image = BufferedImage(size, size, BufferedImage.TYPE_INT_RGB)
    return ByteArrayOutputStream().use { out -> ImageIO.write(image, "jpeg", out); out.toByteArray() }
}
