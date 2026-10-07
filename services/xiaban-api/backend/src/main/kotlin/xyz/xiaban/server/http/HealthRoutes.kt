package xyz.xiaban.server.http

import io.ktor.http.HttpStatusCode
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import io.ktor.server.routing.get
import kotlinx.coroutines.future.await
import xyz.xiaban.contracts.common.HealthResponse
import xyz.xiaban.contracts.common.SystemApiPaths
import xyz.xiaban.server.Services
import xyz.xiaban.server.db.queryOne

fun Route.healthRoutes(services: Services) {
    get(SystemApiPaths.HEALTH) {
        val dbOk = runCatching { services.db.read { it.queryOne("select 1") { rs -> rs.getInt(1) } } == 1 }
            .getOrDefault(false)
        val redisOk = runCatching { services.redis.cmd.ping().await() == "PONG" }.getOrDefault(false)
        val ok = dbOk && redisOk
        call.respond(
            if (ok) HttpStatusCode.OK else HttpStatusCode.ServiceUnavailable,
            HealthResponse(if (ok) "ok" else "degraded", dbOk, redisOk),
        )
    }
}
