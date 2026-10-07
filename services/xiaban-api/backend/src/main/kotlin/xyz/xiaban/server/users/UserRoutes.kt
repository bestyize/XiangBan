package xyz.xiaban.server.users

import io.ktor.http.Cookie
import io.ktor.http.HttpHeaders
import io.ktor.server.auth.authenticate
import io.ktor.server.request.receive
import io.ktor.server.response.header
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import io.ktor.server.routing.put
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.ProfileUpdate
import xyz.xiaban.contracts.account.session.SessionApiPaths
import xyz.xiaban.contracts.account.session.SessionRequest
import xyz.xiaban.contracts.afterwork.unlocks.UnlockApiPaths
import xyz.xiaban.server.Services
import xyz.xiaban.server.auth.AUTH_USER
import xyz.xiaban.server.auth.userId

const val DEVICE_COOKIE = "xb_did"

fun Route.userRoutes(services: Services) {
    // Not behind authenticate(): a stale or foreign token must fall back to the device, never 401.
    post(SessionApiPaths.SESSION) {
        val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
        val response = services.sessions.open(
            tokenUserId = bearer?.let { services.tokens.userIdOrNull(it) },
            cookieDeviceId = call.request.cookies[DEVICE_COOKIE],
            req = call.receive<SessionRequest>(),
        )
        call.response.cookies.append(
            Cookie(
                name = DEVICE_COOKIE,
                value = response.deviceId,
                maxAge = 400 * 24 * 3600,
                path = "/api",
                secure = services.config.publicBaseUrl.startsWith("https://"),
                httpOnly = true,
                extensions = mapOf("SameSite" to "Lax"),
            ),
        )
        call.respond(response)
    }

    get(ProfileApiPaths.META) {
        call.response.header(HttpHeaders.CacheControl, "public, max-age=3600")
        call.respond(services.profiles.meta())
    }

    authenticate(AUTH_USER) {
        get(ProfileApiPaths.PROFILE) { call.respond(services.profiles.view(call.userId())) }
        put(ProfileApiPaths.PROFILE) {
            val userId = call.userId()
            val req = call.receive<ProfileUpdate>()
            val profile = services.profiles.update(userId, req)
            if (req.onboarded == true) services.unlocks.grant(userId, setOf("named"))
            call.respond(profile)
        }
        get(UnlockApiPaths.UNLOCKS) { call.respond(services.unlocks.list(call.userId())) }
    }
}
