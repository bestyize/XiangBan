package xyz.xiaban.server.users

import io.ktor.client.call.body
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue
import xyz.xiaban.contracts.account.profile.MetaResponse
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileUpdate
import xyz.xiaban.contracts.account.profile.ProfileView
import xyz.xiaban.contracts.account.session.SessionApiPaths
import xyz.xiaban.contracts.account.session.SessionRequest
import xyz.xiaban.contracts.account.session.SessionResponse
import xyz.xiaban.contracts.companion.personality.Personality
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.support.apiTest

class SessionFlowTest {

    @Test
    fun `new device gets an anonymous account with defaults`() = apiTest {
        val session = session()
        with(session.profile) {
            assertEquals("小伴", companionName)
            assertEquals(Personality.Toxic, personality)
            assertNull(nickname)
            assertEquals("09:00", workStart)
            assertEquals("18:00", workEnd)
            assertFalse(onboarded)
        }
    }

    @Test
    fun `same device restores the same account`() = apiTest {
        val deviceId = newDeviceId()
        val first = session(deviceId)
        put(ProfileApiPaths.PROFILE, first.token, ProfileUpdate(companionName = "团子"))
        val second = session(deviceId)
        assertEquals("团子", second.profile.companionName)
        assertEquals(first.profile.createdAt, second.profile.createdAt)
    }

    @Test
    fun `device cookie wins over a fresh device id in the body`() = apiTest {
        val original = newDeviceId()
        val first = session(original)
        put(ProfileApiPaths.PROFILE, first.token, ProfileUpdate(companionName = "阿明"))
        val restored = client.post(SessionApiPaths.SESSION) {
            header(HttpHeaders.Cookie, "$DEVICE_COOKIE=$original")
            contentType(ContentType.Application.Json)
            setBody(SessionRequest(newDeviceId(), materialize = true))
        }.body<SessionResponse>()
        assertEquals(original, restored.deviceId)
        assertEquals("阿明", restored.profile.companionName)
    }

    @Test
    fun `a stale token falls back to the device instead of failing`() = apiTest {
        val deviceId = newDeviceId()
        val first = session(deviceId)
        val restored = client.post(SessionApiPaths.SESSION) {
            header(HttpHeaders.Authorization, "Bearer not-a-valid-token")
            contentType(ContentType.Application.Json)
            setBody(SessionRequest(deviceId, materialize = true))
        }
        assertEquals(HttpStatusCode.OK, restored.status)
        assertEquals(first.profile.createdAt, restored.body<SessionResponse>().profile.createdAt)
    }

    @Test
    fun `onboarding answers are saved and validated`() = apiTest {
        val token = session().token
        val saved = put(
            ProfileApiPaths.PROFILE, token,
            ProfileUpdate(
                companionName = "煤球", personality = Personality.Tsundere,
                cityCode = "hangzhou", workStart = "10:00", workEnd = "19:30", onboarded = true,
            ),
        ).body<ProfileView>()
        assertEquals("杭州", saved.cityName)
        assertEquals("19:30", saved.workEnd)
        assertTrue(saved.onboarded)

        assertEquals(HttpStatusCode.BadRequest, put(ProfileApiPaths.PROFILE, token, ProfileUpdate(companionName = "猫")).status)
        assertEquals(HttpStatusCode.BadRequest, put(ProfileApiPaths.PROFILE, token, ProfileUpdate(cityCode = "atlantis")).status)
        assertEquals(HttpStatusCode.BadRequest, put(ProfileApiPaths.PROFILE, token, ProfileUpdate(workEnd = "25:00")).status)
    }

    @Test
    fun `a share-card visit records who invited the new account`() = apiTest {
        val inviter = session().profile
        val invited = client.post(SessionApiPaths.SESSION) {
            contentType(ContentType.Application.Json)
            setBody(SessionRequest(newDeviceId(), "Asia/Shanghai", ref = inviter.inviteCode, from = "card", materialize = true))
        }.body<SessionResponse>()
        assertTrue(invited.profile.inviteCode != inviter.inviteCode)
        val (referredBy, source) = services.db.read { c ->
            c.queryOne("select referred_by, ref_source from users where invite_code = ?", invited.profile.inviteCode) {
                it.getLong(1) to it.getString(2)
            }!!
        }
        assertEquals("card", source)
        assertTrue(referredBy > 0)
        assertFalse(invited.profile.showCityOnCard, "city stays off the card by default")
    }

    @Test
    fun `profile requires a token and meta does not`() = apiTest {
        assertEquals(HttpStatusCode.Unauthorized, get(ProfileApiPaths.PROFILE, null).status)
        val meta = get(ProfileApiPaths.META, null).body<MetaResponse>()
        assertTrue(meta.cities.any { it.code == "beijing" })
        assertEquals(4, meta.personalities.size)
    }
}
