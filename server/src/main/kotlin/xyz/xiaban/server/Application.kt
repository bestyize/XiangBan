package xyz.xiaban.server

import io.ktor.server.application.Application
import io.ktor.server.application.ApplicationStopped
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import io.ktor.server.routing.routing
import xyz.xiaban.server.auth.installAuth
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.http.healthRoutes
import xyz.xiaban.server.http.installHttp
import xyz.xiaban.server.users.userRoutes

fun main() {
    val config = AppConfig.fromEnv()
    val services = Services.create(config)
    embeddedServer(Netty, port = config.port, host = "0.0.0.0") {
        xiaban(services)
        monitor.subscribe(ApplicationStopped) { services.close() }
    }.start(wait = true)
}

fun Application.xiaban(services: Services) {
    installHttp(services.config)
    installAuth(services.tokens)
    routing {
        healthRoutes(services)
        userRoutes(services)
    }
}
