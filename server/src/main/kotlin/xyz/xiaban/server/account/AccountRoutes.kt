package xyz.xiaban.server.account

import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.server.auth.authenticate
import io.ktor.server.request.receive
import io.ktor.server.response.header
import io.ktor.server.response.respond
import io.ktor.server.response.respondText
import kotlinx.serialization.json.JsonObject
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.shared.ConsentUpdate
import xyz.xiaban.shared.EventBatch
import io.ktor.server.routing.Route
import io.ktor.server.routing.delete
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import io.ktor.server.routing.put
import xyz.xiaban.server.Services
import xyz.xiaban.server.auth.AUTH_USER
import xyz.xiaban.server.auth.userId
import xyz.xiaban.server.http.conflict
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.BindEmailRequest
import xyz.xiaban.shared.EmailCodeRequest
import xyz.xiaban.shared.GuardianPause
import xyz.xiaban.shared.GuardianUpdate
import xyz.xiaban.shared.OkResponse
import xyz.xiaban.shared.SubscriptionUpdate

fun Route.accountRoutes(services: Services) {
    authenticate(AUTH_USER) {
        post(ApiPaths.BIND_CODE) {
            services.emailBinding.sendCode(call.userId(), call.receive<EmailCodeRequest>().email)
            call.respond(OkResponse())
        }
        post(ApiPaths.BIND) {
            val req = call.receive<BindEmailRequest>()
            call.respond(services.emailBinding.bind(call.userId(), req.email, req.code))
        }
        put(ApiPaths.SUBSCRIPTION) {
            val userId = call.userId()
            val enabled = call.receive<SubscriptionUpdate>().enabled
            if (enabled && services.profiles.user(userId).email == null) conflict("先绑定邮箱才能收提醒邮件", "email_required")
            services.users.setRecallOptIn(userId, enabled)
            call.respond(services.profiles.view(userId))
        }

        get(ApiPaths.GUARDIAN) {
            val view = services.guardian.view(call.userId())
            if (view == null) call.respond(HttpStatusCode.NoContent) else call.respond(view)
        }
        put(ApiPaths.GUARDIAN) {
            val userId = call.userId()
            val view = services.guardian.set(userId, call.receive<GuardianUpdate>())
            services.unlocks.grant(userId, setOf("guardian"))
            services.events.server(userId, "guardian_set", mapOf("days" to view.thresholdDays.toString()))
            call.respond(view)
        }
        put(ApiPaths.GUARDIAN_PAUSE) { call.respond(services.guardian.pause(call.userId(), call.receive<GuardianPause>().paused)) }
        delete(ApiPaths.GUARDIAN) {
            services.guardian.remove(call.userId())
            call.respond(OkResponse())
        }
    }

    authenticate(AUTH_USER) {
        get(ApiPaths.ACCOUNT_EXPORT) {
            call.response.header(HttpHeaders.ContentDisposition, "attachment; filename=\"xiaban-export.json\"")
            call.respondText(AppJson.encodeToString(JsonObject.serializer(), services.account.export(call.userId())), ContentType.Application.Json)
        }
        delete(ApiPaths.ACCOUNT) {
            services.account.delete(call.userId())
            call.respond(OkResponse())
        }
        put(ApiPaths.CONSENT) {
            val req = call.receive<ConsentUpdate>()
            services.account.consent(call.userId(), req.termsVersion, req.overseas)
            call.respond(services.profiles.view(call.userId()))
        }
        post(ApiPaths.EVENTS) {
            val batch = call.receive<EventBatch>()
            services.events.record(call.userId(), batch.events, call.request.headers[HttpHeaders.UserAgent])
            call.respond(OkResponse())
        }
    }

    get(ApiPaths.UNSUBSCRIBE) {
        val userId = call.request.queryParameters["u"]?.toLongOrNull()
        val token = call.request.queryParameters["t"].orEmpty()
        val ok = userId != null && services.unsubscribeLinks.verify(userId, token)
        if (ok) services.users.setRecallOptIn(userId!!, false)
        val message = if (ok) "已经退订，小伴不会再发这类邮件打扰你了。" else "这个退订链接不对，可以在「我的」里关掉提醒邮件。"
        call.respondText(
            "<!doctype html><meta charset=utf-8><meta name=viewport content=\"width=device-width\"><title>下班了么</title>" +
                "<body style=\"font-family:sans-serif;background:#1f2758;color:#fff1c9;padding:48px 24px;text-align:center\"><p>$message</p></body>",
            ContentType.Text.Html,
            if (ok) HttpStatusCode.OK else HttpStatusCode.BadRequest,
        )
    }
}
