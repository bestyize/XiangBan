package xyz.xiaban.server

import io.ktor.server.application.Application
import io.ktor.server.application.ApplicationStopped
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import io.ktor.server.routing.routing
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.http.healthRoutes
import xyz.xiaban.server.http.installHttp

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
    routing {
        healthRoutes(services)
    }
}
