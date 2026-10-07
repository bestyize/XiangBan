package xyz.xiaban.server.account

import io.ktor.http.ContentType
import io.ktor.http.Cookie
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.server.auth.authenticate
import io.ktor.server.plugins.origin
import io.ktor.server.plugins.ratelimit.rateLimit
import io.ktor.server.request.receive
import io.ktor.server.response.header
import io.ktor.server.response.respond
import io.ktor.server.response.respondText
import io.ktor.server.routing.Route
import io.ktor.server.routing.delete
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import io.ktor.server.routing.put
import kotlinx.serialization.json.JsonObject
import xyz.xiaban.contracts.account.auth.AccountApiPaths
import xyz.xiaban.contracts.account.auth.BindEmailRequest
import xyz.xiaban.contracts.account.auth.EmailCodeRequest
import xyz.xiaban.contracts.account.auth.EmailLoginRequest
import xyz.xiaban.contracts.account.auth.LogoutRequest
import xyz.xiaban.contracts.account.consent.ConsentApiPaths
import xyz.xiaban.contracts.account.consent.ConsentUpdate
import xyz.xiaban.contracts.account.guardian.GuardianApiPaths
import xyz.xiaban.contracts.account.guardian.GuardianPause
import xyz.xiaban.contracts.account.guardian.GuardianUpdate
import xyz.xiaban.contracts.account.profile.ProfileApiPaths
import xyz.xiaban.contracts.account.profile.SubscriptionUpdate
import xyz.xiaban.contracts.common.OkResponse
import xyz.xiaban.contracts.telemetry.EventBatch
import xyz.xiaban.contracts.telemetry.TelemetryApiPaths
import xyz.xiaban.server.Services
import xyz.xiaban.server.auth.AUTH_USER
import xyz.xiaban.server.auth.userId
import xyz.xiaban.server.events.Actor
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.RateLimits
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.conflict
import xyz.xiaban.server.users.DEVICE_COOKIE
import xyz.xiaban.server.users.SessionService

fun Route.accountRoutes(services: Services) {
    get("/api/account/email/status") {
        val settings = services.emailSettings.current()
        call.respond(xyz.xiaban.server.admin.EmailServiceStatus(settings.enabled, services.emailAvailable()))
    }
    post(AccountApiPaths.EMAIL_LOGIN_CODE) {
        val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
        val userId = bearer?.let { services.tokens.userIdOrNull(it) }
        services.emailBinding.sendCode(userId, call.receive<EmailCodeRequest>().email, login = true, ip = call.request.origin.remoteHost)
        call.respond(OkResponse())
    }
    post(AccountApiPaths.EMAIL_LOGIN) {
        val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
        val userId = bearer?.let { services.tokens.userIdOrNull(it) }
        val req = call.receive<EmailLoginRequest>()
        val termsVersion = req.termsVersion?.trim()?.also {
            if (it.isEmpty() || it.length > 20) badRequest("协议版本无效", "invalid_terms_version")
        }
        val result = services.emailBinding.login(userId, req.email, req.code, req.deviceId, req.mergeGuest, req.username)
        val profile = if (termsVersion != null) {
            services.account.consent(result.profile.userId, termsVersion, false)
            services.profiles.view(result.profile.userId)
        } else result.profile
        bearer?.let { services.tokens.revoke(it) }
        call.respond(result.copy(profile = profile))
    }
    authenticate(AUTH_USER) {
        post(AccountApiPaths.LOGOUT) {
            val req = call.receive<LogoutRequest>()
            val ids = listOfNotNull(req.deviceId, call.request.cookies[DEVICE_COOKIE]).filter(SessionService::isValidDeviceId).distinct()
            call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()?.let { services.tokens.revoke(it) }
            for (id in ids) services.users.unlinkDevice(call.userId(), SessionService.sha256(id))
            call.response.cookies.append(Cookie(DEVICE_COOKIE, "", maxAge = 0, path = "/api", httpOnly = true,
                secure = services.config.publicBaseUrl.startsWith("https://"), extensions = mapOf("SameSite" to "Lax")))
            call.respond(OkResponse())
        }
        post(AccountApiPaths.BIND_CODE) {
            services.emailBinding.sendCode(call.userId(), call.receive<EmailCodeRequest>().email, ip = call.request.origin.remoteHost)
            call.respond(OkResponse())
        }
        post(AccountApiPaths.BIND) {
            val req = call.receive<BindEmailRequest>()
            val result = services.emailBinding.bind(call.userId(), req.email, req.code, req.username)
            call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()?.let { services.tokens.revoke(it) }
            call.respond(result)
        }
        put(ProfileApiPaths.SUBSCRIPTION) {
            val userId = call.userId()
            val enabled = call.receive<SubscriptionUpdate>().enabled
            if (enabled && services.profiles.user(userId).email == null) conflict("先绑定邮箱才能收提醒邮件", "email_required")
            services.users.setRecallOptIn(userId, enabled)
            call.respond(services.profiles.view(userId))
        }

        get(GuardianApiPaths.GUARDIAN) {
            val view = services.guardian.view(call.userId())
            if (view == null) call.respond(HttpStatusCode.NoContent) else call.respond(view)
        }
        put(GuardianApiPaths.GUARDIAN) {
            val userId = call.userId()
            val view = services.guardian.set(userId, call.receive<GuardianUpdate>())
            services.unlocks.grant(userId, setOf("guardian"))
            services.events.server(userId, "guardian_set", mapOf("days" to view.thresholdDays.toString()))
            call.respond(view)
        }
        put(GuardianApiPaths.GUARDIAN_PAUSE) { call.respond(services.guardian.pause(call.userId(), call.receive<GuardianPause>().paused)) }
        delete(GuardianApiPaths.GUARDIAN) {
            services.guardian.remove(call.userId())
            call.respond(OkResponse())
        }
    }

    authenticate(AUTH_USER) {
        get(AccountApiPaths.ACCOUNT_EXPORT) {
            call.response.header(HttpHeaders.ContentDisposition, "attachment; filename=\"xiaban-export.json\"")
            call.respondText(AppJson.encodeToString(JsonObject.serializer(), services.account.export(call.userId())), ContentType.Application.Json)
        }
        delete(AccountApiPaths.ACCOUNT) {
            services.account.delete(call.userId())
            call.respond(OkResponse())
        }
        put(ConsentApiPaths.CONSENT) {
            val req = call.receive<ConsentUpdate>()
            services.account.consent(call.userId(), req.termsVersion, req.overseas)
            call.respond(services.profiles.view(call.userId()))
        }
    }

    // Open to guests too: they are counted by a hash of their anonymous device id (PRD 10, admin metrics).
    rateLimit(RateLimits.EVENTS) {
        post(TelemetryApiPaths.EVENTS) {
            val batch = call.receive<EventBatch>()
            val bearer = call.request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
            val userId = bearer?.let { services.tokens.userIdOrNull(it) }
            val actor = userId?.let { Actor.Registered(it) } ?: batch.deviceId?.let(Actor::guest)
            if (actor != null) services.events.record(actor, batch.events, call.request.headers[HttpHeaders.UserAgent])
            call.respond(OkResponse())
        }
    }

    get(ProfileApiPaths.UNSUBSCRIBE) {
        val userId = call.request.queryParameters["u"]?.toLongOrNull()
        val token = call.request.queryParameters["t"].orEmpty()
        val ok = userId != null && services.unsubscribeLinks.verify(userId, token)
        if (ok) services.users.setRecallOptIn(userId!!, false)
        val message = if (ok) "已经退订，小伴不会再发这类邮件打扰你了。" else "这个退订链接不对，可以在「我的」里关掉提醒邮件。"
        call.respondText(
            "<!doctype html><meta charset=utf-8><meta name=viewport content=\"width=device-width\"><title>下班啦</title>" +
                "<body style=\"font-family:sans-serif;background:#1f2758;color:#fff1c9;padding:48px 24px;text-align:center\"><p>$message</p></body>",
            ContentType.Text.Html,
            if (ok) HttpStatusCode.OK else HttpStatusCode.BadRequest,
        )
    }
}
