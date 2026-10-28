package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.MockRequestHandleScope
import io.ktor.client.engine.mock.respond
import io.ktor.client.request.HttpRequestData
import io.ktor.client.request.HttpResponseData
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf

/** 记录所有请求并按路径返回预设 JSON 的假服务。 */
class FakeServer {
    val requests = mutableListOf<HttpRequestData>()
    private val routes = mutableListOf<Pair<(HttpRequestData) -> Boolean, suspend MockRequestHandleScope.(HttpRequestData) -> HttpResponseData>>()

    fun on(match: (HttpRequestData) -> Boolean, status: HttpStatusCode = HttpStatusCode.OK, body: String) {
        routes += match to { respond(body, status, headersOf(HttpHeaders.ContentType, "application/json")) }
    }

    fun on(pathStartsWith: String, status: HttpStatusCode = HttpStatusCode.OK, body: String) =
        on({ it.url.encodedPath.startsWith(pathStartsWith) || (it.url.encodedPath + (it.url.encodedQuery.takeIf { q -> q.isNotEmpty() }?.let { q -> "?$q" } ?: "")).startsWith(pathStartsWith) }, status, body)

    fun onThrow(pathStartsWith: String) {
        routes += ({ r: HttpRequestData -> r.url.encodedPath.startsWith(pathStartsWith) }) to { throw Exception("offline") }
    }

    val client: HttpClient = HttpClient(MockEngine { request ->
        requests += request
        val route = routes.lastOrNull { it.first(request) } ?: return@MockEngine respond("""{"message":"没有这个接口"}""", HttpStatusCode.NotFound, headersOf(HttpHeaders.ContentType, "application/json"))
        route.second(this, request)
    })
}
