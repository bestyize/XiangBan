package xyz.xiaban.server.http

import io.ktor.server.application.Application
import io.ktor.server.application.install
import io.ktor.server.auth.principal
import io.ktor.server.plugins.origin
import io.ktor.server.plugins.ratelimit.RateLimit
import io.ktor.server.plugins.ratelimit.RateLimitName
import xyz.xiaban.server.auth.UserPrincipal
import kotlin.time.Duration.Companion.minutes

/** Per-instance limits keyed by user (falling back to IP); Nginx adds the per-IP limit in front. */
object RateLimits {
    val CHECKIN = RateLimitName("checkin")
    val EVENTS = RateLimitName("events")
}

fun Application.installRateLimits() {
    install(RateLimit) {
        register(RateLimits.CHECKIN) {
            rateLimiter(limit = 20, refillPeriod = 1.minutes)
            requestKey { call -> call.principal<UserPrincipal>()?.userId ?: call.request.origin.remoteHost }
        }
        // The events endpoint is public, so it is limited per IP; a normal page sends a batch every few seconds.
        register(RateLimits.EVENTS) {
            rateLimiter(limit = 60, refillPeriod = 1.minutes)
            requestKey { call -> call.request.origin.remoteHost }
        }
    }
}
