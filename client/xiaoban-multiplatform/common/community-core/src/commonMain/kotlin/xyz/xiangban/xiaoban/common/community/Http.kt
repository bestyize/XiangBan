package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.header
import io.ktor.client.request.request
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.contentType
import kotlinx.serialization.DeserializationStrategy
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonPrimitive

val CommunityJson: Json = Json { ignoreUnknownKeys = true; explicitNulls = false; coerceInputValues = true; encodeDefaults = true }

/** 服务端返回的错误：[status] 为 0 表示没连上。[code] 是服务端的机读错误码，[retryAfter] 是限流时建议的等待秒数。 */
class ApiException(val status: Int, override val message: String, val code: String? = null, val retryAfter: Int? = null) : Exception(message) {
    val unauthorized: Boolean get() = status == 401
}

/** 论坛和账号中心共用的 JSON 请求封装：统一拼地址、带令牌、把错误响应转成 [ApiException]。 */
class JsonHttp(private val client: HttpClient, private val base: String) {
    suspend fun <T> send(
        method: HttpMethod,
        path: String,
        strategy: DeserializationStrategy<T>,
        body: String? = null,
        token: String? = null,
    ): T {
        val text = raw(method, path, body, token)
        return try {
            CommunityJson.decodeFromString(strategy, text)
        } catch (e: Exception) {
            throw ApiException(200, "服务返回了无法识别的内容")
        }
    }

    suspend fun raw(method: HttpMethod, path: String, body: String? = null, token: String? = null): String {
        val response = try {
            client.request("${base.trimEnd('/')}$path") {
                this.method = method
                applyAuth(token)
                if (body != null) {
                    contentType(ContentType.Application.Json)
                    setBody(body)
                }
            }
        } catch (e: Exception) {
            throw ApiException(0, "网络连接失败，请稍后再试")
        }
        val text = response.bodyAsText()
        if (response.status.value !in 200..299) throw errorFrom(response.status.value, text)
        return text
    }

    private fun HttpRequestBuilder.applyAuth(token: String?) {
        if (token != null) header(HttpHeaders.Authorization, "Bearer $token")
    }

    private fun errorFrom(status: Int, text: String): ApiException = parseApiError(status, text)
}

/** 解析服务端错误体：兼容平台统一的 `{"message","code"}` 与 OpenAI 兼容接口的 `{"error":{"message","code"}}`。 */
fun parseApiError(status: Int, text: String): ApiException {
    val obj = runCatching { CommunityJson.parseToJsonElement(text) as? JsonObject }.getOrNull()
    val nested = obj?.get("error") as? JsonObject
    val message = nested?.get("message")?.jsonPrimitive?.contentOrNull
        ?: obj?.get("message")?.jsonPrimitive?.contentOrNull
        ?: "请求失败（$status）"
    val code = nested?.get("code")?.jsonPrimitive?.contentOrNull
        ?: obj?.get("code")?.jsonPrimitive?.contentOrNull
    val retry = obj?.get("retry_after")?.let { runCatching { it.jsonPrimitive.int }.getOrNull() }
    return ApiException(status, message, code, retry)
}
