package xyz.xiangban.xiaoban.common.community

import io.ktor.client.HttpClient
import io.ktor.http.HttpMethod
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlinx.serialization.serializer

/** 相伴统一账号中心。和网页端 `api/account.ts` 是同一组接口。 */
class AccountApi(client: HttpClient, private val config: CommunityConfig) {
    private val http = JsonHttp(client, config.accountBase)

    suspend fun sendCode(identifier: String, purpose: String = "login") {
        http.raw(HttpMethod.Post, "/api/auth/code/send", obj { put("identifier", identifier); put("purpose", purpose) })
    }

    suspend fun loginWithCode(identifier: String, code: String): CodeLogin {
        val text = http.raw(
            HttpMethod.Post, "/api/auth/code/login",
            obj { put("identifier", identifier); put("code", code); put("client_id", config.clientId); put("device_name", config.deviceName) },
        )
        val parsed = CommunityJson.parseToJsonElement(text) as? JsonObject ?: throw ApiException(200, "服务返回了无法识别的内容")
        if (parsed["registration_required"]?.jsonPrimitive?.booleanOrNull == true) {
            val token = parsed["registration_token"]?.jsonPrimitive?.contentOrNull ?: throw ApiException(200, "服务返回了无法识别的内容")
            return CodeLogin.NeedsRegistration(token)
        }
        return CodeLogin.Done(CommunityJson.decodeFromJsonElement(serializer<LoginResponse>(), parsed))
    }

    suspend fun loginWithPassword(identifier: String, password: String): LoginResponse =
        http.send(
            HttpMethod.Post, "/api/auth/password/login", serializer(),
            obj { put("identifier", identifier); put("password", password); put("client_id", config.clientId); put("device_name", config.deviceName) },
        )

    suspend fun register(registrationToken: String, username: String, password: String): LoginResponse =
        http.send(
            HttpMethod.Post, "/api/auth/register", serializer(),
            obj { put("registration_token", registrationToken); put("username", username); put("password", password); put("client_id", config.clientId); put("device_name", config.deviceName) },
        )

    suspend fun refresh(refreshToken: String): TokenPair =
        http.send(HttpMethod.Post, "/api/auth/refresh", serializer(), obj { put("refresh_token", refreshToken); put("client_id", config.clientId) })

    suspend fun logout(refreshToken: String) {
        runCatching { http.raw(HttpMethod.Post, "/api/auth/logout", obj { put("refresh_token", refreshToken) }) }
    }

    private fun obj(build: kotlinx.serialization.json.JsonObjectBuilder.() -> Unit): String = buildJsonObject(build).toString()
}
