package xyz.xiaban.server.admin

import io.ktor.http.HttpStatusCode
import io.ktor.server.application.ApplicationCall
import io.ktor.server.request.receive
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import io.ktor.server.routing.delete
import io.ktor.server.routing.get
import io.ktor.server.routing.put
import io.ktor.server.routing.route
import xyz.xiaban.server.Services
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.shared.FlagsUpdate
import xyz.xiaban.shared.OkResponse
import java.security.MessageDigest

/** Operator endpoints, enabled only when ADMIN_TOKEN is set; called with the X-Admin-Token header. */
fun Route.adminRoutes(services: Services) {
    val token = services.config.adminToken ?: return
    route("/api/admin") {
        get("/flags") {
            call.requireAdmin(token)
            call.respond(services.flags.view())
        }
        put("/flags") {
            call.requireAdmin(token)
            call.respond(services.flags.update(call.receive<FlagsUpdate>()))
        }
        get("/reports") {
            call.requireAdmin(token)
            val limit = call.request.queryParameters["limit"]?.toIntOrNull()?.coerceIn(1, 200) ?: 50
            call.respond(services.moderation.reports(limit))
        }
        delete("/copies/{id}/disabled") {
            call.requireAdmin(token)
            services.moderation.restore(call.parameters["id"]!!)
            call.respond(OkResponse())
        }
    }
}

private fun ApplicationCall.requireAdmin(token: String) {
    val given = request.headers["X-Admin-Token"]?.toByteArray() ?: ByteArray(0)
    if (!MessageDigest.isEqual(given, token.toByteArray())) {
        throw ApiException(HttpStatusCode.NotFound, "not_found", "这里什么都没有")
    }
}
