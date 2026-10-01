package xyz.xiaban.server.support

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.delete
import io.ktor.client.request.get
import io.ktor.client.request.post
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.serialization.kotlinx.json.json
import io.ktor.server.testing.testApplication
import xyz.xiaban.server.Services
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.xiaban
import xyz.xiaban.shared.ApiPaths
import xyz.xiaban.shared.SessionRequest
import xyz.xiaban.shared.SessionResponse
import java.util.UUID
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation as ClientContentNegotiation

fun apiTest(block: suspend ApiScope.() -> Unit) {
    val services = TestEnv.services()
    testApplication {
        application { xiaban(services) }
        val client = createClient { install(ClientContentNegotiation) { json(AppJson) } }
        ApiScope(client, services).block()
    }
}

class ApiScope(val client: HttpClient, val services: Services) {
    fun newDeviceId(): String = UUID.randomUUID().toString().replace("-", "")

    suspend fun session(deviceId: String = newDeviceId(), timezone: String = "Asia/Shanghai"): SessionResponse =
        client.post(ApiPaths.SESSION) { json(SessionRequest(deviceId, timezone)) }.body()

    suspend fun get(path: String, token: String?): HttpResponse = client.get(path) { auth(token) }

    suspend fun post(path: String, token: String?, body: Any? = null): HttpResponse =
        client.post(path) { auth(token); body?.let { json(it) } }

    suspend fun put(path: String, token: String?, body: Any): HttpResponse = client.put(path) { auth(token); json(body) }

    suspend fun delete(path: String, token: String?): HttpResponse = client.delete(path) { auth(token) }

    private fun HttpRequestBuilder.auth(token: String?) {
        token?.let { bearerAuth(it) }
    }

    private fun HttpRequestBuilder.json(body: Any) {
        contentType(ContentType.Application.Json)
        setBody(body)
    }
}
