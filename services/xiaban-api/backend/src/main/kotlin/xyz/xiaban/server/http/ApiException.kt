package xyz.xiaban.server.http

import io.ktor.http.HttpStatusCode

class ApiException(val status: HttpStatusCode, val code: String, override val message: String) : RuntimeException(message)

fun badRequest(message: String, code: String = "bad_request"): Nothing =
    throw ApiException(HttpStatusCode.BadRequest, code, message)

fun notFound(message: String, code: String = "not_found"): Nothing =
    throw ApiException(HttpStatusCode.NotFound, code, message)

fun conflict(message: String, code: String = "conflict"): Nothing =
    throw ApiException(HttpStatusCode.Conflict, code, message)

fun tooMany(message: String = "小伴有点忙，稍后再来", code: String = "busy"): Nothing =
    throw ApiException(HttpStatusCode.TooManyRequests, code, message)
