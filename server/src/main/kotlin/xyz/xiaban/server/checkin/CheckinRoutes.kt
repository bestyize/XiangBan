package xyz.xiaban.server.checkin

import io.ktor.server.auth.authenticate
import io.ktor.server.plugins.ratelimit.rateLimit
import io.ktor.server.request.receive
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import io.ktor.server.routing.delete
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import xyz.xiaban.server.Services
import xyz.xiaban.server.auth.userId
import xyz.xiaban.server.auth.AUTH_USER
import xyz.xiaban.server.http.RateLimits
import xyz.xiaban.server.http.notFound
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.OkResponse
import xyz.xiaban.shared.ReportRequest
import java.time.LocalDateTime

fun Route.checkinRoutes(services: Services) {
    authenticate(AUTH_USER) {
        rateLimit(RateLimits.CHECKIN) {
            post(ApiPaths.CHECKIN) { call.respond(services.checkins.checkIn(call.userId())) }
            delete(ApiPaths.CHECKIN_TODAY) { call.respond(services.checkins.undo(call.userId())) }
        }
        get(ApiPaths.CHECKIN_TODAY) {
            val wait = call.request.queryParameters["wait"]?.toLongOrNull() ?: 0
            call.respond(services.checkins.today(call.userId(), wait))
        }
        get(ApiPaths.STATS) {
            val user = services.profiles.user(call.userId())
            call.respond(services.stats.snapshot(user, services.clock.instant()).toView())
        }
        post(ApiPaths.REPORTS) {
            val userId = call.userId()
            val req = call.receive<ReportRequest>()
            val row = req.checkinId.toLongOrNull()?.let { services.checkinRepository.find(it) }
                ?.takeIf { it.userId == userId }
                ?: notFound("找不到这条打卡")
            val user = services.profiles.user(userId)
            val today = WorkDay.of(LocalDateTime.ofInstant(services.clock.instant(), user.timezone))
            services.moderation.report(userId, row, req.reason, today)
            call.respond(OkResponse())
        }
    }
}
