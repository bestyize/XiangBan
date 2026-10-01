package xyz.xiaban.server.http

import io.ktor.http.HttpStatusCode
import io.ktor.serialization.kotlinx.json.json
import io.ktor.server.application.Application
import io.ktor.server.application.install
import io.ktor.server.application.log
import io.ktor.server.plugins.BadRequestException
import io.ktor.server.plugins.calllogging.CallLogging
import io.ktor.server.plugins.contentnegotiation.ContentNegotiation
import io.ktor.server.plugins.defaultheaders.DefaultHeaders
import io.ktor.server.plugins.forwardedheaders.XForwardedHeaders
import io.ktor.server.plugins.statuspages.StatusPages
import io.ktor.server.request.httpMethod
import io.ktor.server.request.path
import io.ktor.server.response.respond
import kotlinx.serialization.json.Json
import org.slf4j.event.Level
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.shared.ApiError

val AppJson = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

fun Application.installHttp(config: AppConfig) {
    install(ContentNegotiation) { json(AppJson) }
    if (config.trustProxy) install(XForwardedHeaders)
    install(DefaultHeaders) {
        header("X-Content-Type-Options", "nosniff")
        header("Cache-Control", "no-store")
    }
    install(CallLogging) {
        level = Level.INFO
        filter { it.request.path().startsWith("/api") }
        format { call -> "${call.response.status()?.value} ${call.request.httpMethod.value} ${call.request.path()}" }
    }
    install(StatusPages) {
        exception<ApiException> { call, e -> call.respond(e.status, ApiError(e.code, e.message)) }
        exception<BadRequestException> { call, _ ->
            call.respond(HttpStatusCode.BadRequest, ApiError("bad_request", "请求格式不对"))
        }
        exception<Throwable> { call, e ->
            call.application.log.error("Unhandled error on ${call.request.path()}", e)
            call.respond(HttpStatusCode.InternalServerError, ApiError("internal", "小伴走神了，稍后再试"))
        }
        status(HttpStatusCode.NotFound) { call, _ ->
            call.respond(HttpStatusCode.NotFound, ApiError("not_found", "这里什么都没有"))
        }
        status(HttpStatusCode.TooManyRequests) { call, _ ->
            call.respond(HttpStatusCode.TooManyRequests, ApiError("busy", "小伴有点忙，稍后再来"))
        }
    }
}
