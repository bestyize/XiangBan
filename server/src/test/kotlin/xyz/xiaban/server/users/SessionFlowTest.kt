package xyz.xiaban.server.users

import io.ktor.client.call.body
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import xyz.xiaban.server.support.apiTest
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.MetaResponse
import xyz.xiaban.shared.Personality
import xyz.xiaban.shared.ProfileUpdate
import xyz.xiaban.shared.ProfileView
import xyz.xiaban.shared.SessionRequest
import xyz.xiaban.shared.SessionResponse
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

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
        put(ApiPaths.PROFILE, first.token, ProfileUpdate(companionName = "团子"))
        val second = session(deviceId)
        assertEquals("团子", second.profile.companionName)
        assertEquals(first.profile.createdAt, second.profile.createdAt)
    }

    @Test
    fun `device cookie wins over a fresh device id in the body`() = apiTest {
        val original = newDeviceId()
        val first = session(original)
        put(ApiPaths.PROFILE, first.token, ProfileUpdate(nickname = "阿明"))
        val restored = client.post(ApiPaths.SESSION) {
            header(HttpHeaders.Cookie, "$DEVICE_COOKIE=$original")
            contentType(ContentType.Application.Json)
            setBody(SessionRequest(newDeviceId()))
        }.body<SessionResponse>()
        assertEquals(original, restored.deviceId)
        assertEquals("阿明", restored.profile.nickname)
    }

    @Test
    fun `a stale token falls back to the device instead of failing`() = apiTest {
        val deviceId = newDeviceId()
        val first = session(deviceId)
        val restored = client.post(ApiPaths.SESSION) {
            header(HttpHeaders.Authorization, "Bearer not-a-valid-token")
            contentType(ContentType.Application.Json)
            setBody(SessionRequest(deviceId))
        }
        assertEquals(HttpStatusCode.OK, restored.status)
        assertEquals(first.profile.createdAt, restored.body<SessionResponse>().profile.createdAt)
    }

    @Test
    fun `onboarding answers are saved and validated`() = apiTest {
        val token = session().token
        val saved = put(
            ApiPaths.PROFILE, token,
            ProfileUpdate(
                companionName = "煤球", personality = Personality.Tsundere, nickname = "打工人",
                cityCode = "hangzhou", workStart = "10:00", workEnd = "19:30", onboarded = true,
            ),
        ).body<ProfileView>()
        assertEquals("杭州", saved.cityName)
        assertEquals("19:30", saved.workEnd)
        assertTrue(saved.onboarded)

        assertEquals(HttpStatusCode.BadRequest, put(ApiPaths.PROFILE, token, ProfileUpdate(companionName = "猫")).status)
        assertEquals(HttpStatusCode.BadRequest, put(ApiPaths.PROFILE, token, ProfileUpdate(nickname = "加我vx123456")).status)
        assertEquals(HttpStatusCode.BadRequest, put(ApiPaths.PROFILE, token, ProfileUpdate(cityCode = "atlantis")).status)
        assertEquals(HttpStatusCode.BadRequest, put(ApiPaths.PROFILE, token, ProfileUpdate(workEnd = "25:00")).status)

        val cleared = put(ApiPaths.PROFILE, token, ProfileUpdate(nickname = "")).body<ProfileView>()
        assertNull(cleared.nickname)
    }

    @Test
    fun `profile requires a token and meta does not`() = apiTest {
        assertEquals(HttpStatusCode.Unauthorized, get(ApiPaths.PROFILE, null).status)
        val meta = get(ApiPaths.META, null).body<MetaResponse>()
        assertTrue(meta.cities.any { it.code == "beijing" })
        assertEquals(4, meta.personalities.size)
    }
}
