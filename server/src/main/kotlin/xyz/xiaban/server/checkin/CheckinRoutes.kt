package xyz.xiaban.server.checkin

import io.ktor.server.auth.authenticate
import io.ktor.server.plugins.ratelimit.rateLimit
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import io.ktor.server.routing.delete
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import xyz.xiaban.server.Services
import xyz.xiaban.server.auth.AUTH_USER
import xyz.xiaban.server.auth.userId
import xyz.xiaban.server.http.RateLimits
import xyz.xiaban.shared.ApiPaths

fun Route.checkinRoutes(services: Services) {
    authenticate(AUTH_USER) {
        rateLimit(RateLimits.CHECKIN) {
            post(ApiPaths.CHECKIN) { call.respond(services.checkins.checkIn(call.userId())) }
            delete(ApiPaths.CHECKIN_TODAY) { call.respond(services.checkins.undo(call.userId())) }
        }
        get(ApiPaths.CHECKIN_TODAY) { call.respond(services.checkins.today(call.userId())) }
    }
}
