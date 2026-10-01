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
import xyz.xiaban.server.Services
import xyz.xiaban.server.auth.AUTH_USER
import xyz.xiaban.server.auth.userId
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.ProfileUpdate
import xyz.xiaban.shared.SessionRequest

const val DEVICE_COOKIE = "xb_did"

fun Route.userRoutes(services: Services) {
    // Not behind authenticate(): a stale or foreign token must fall back to the device, never 401.
    post(ApiPaths.SESSION) {
        val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
        val response = services.sessions.open(
            tokenUserId = bearer?.let(services.tokens::userIdOrNull),
            cookieDeviceId = call.request.cookies[DEVICE_COOKIE],
            req = call.receive<SessionRequest>(),
        )
        call.response.cookies.append(
            Cookie(
                name = DEVICE_COOKIE,
                value = response.deviceId,
                maxAge = 400 * 24 * 3600,
                path = "/api",
                secure = services.config.isProd,
                httpOnly = true,
                extensions = mapOf("SameSite" to "Lax"),
            ),
        )
        call.respond(response)
    }

    get(ApiPaths.META) {
        call.response.header(HttpHeaders.CacheControl, "public, max-age=3600")
        call.respond(services.profiles.meta())
    }

    authenticate(AUTH_USER) {
        get(ApiPaths.PROFILE) { call.respond(services.profiles.view(call.userId())) }
        put(ApiPaths.PROFILE) { call.respond(services.profiles.update(call.userId(), call.receive<ProfileUpdate>())) }
    }
}
