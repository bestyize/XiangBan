package xyz.xiaban.server.account

import io.ktor.client.HttpClient
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.get
import io.ktor.client.statement.bodyAsText
import io.ktor.http.HttpStatusCode
import io.ktor.http.isSuccess
import java.security.KeyFactory
import java.security.PublicKey
import java.security.Signature
import java.security.spec.X509EncodedKeySpec
import java.time.Clock
import java.util.Base64
import java.util.UUID
import java.util.concurrent.atomic.AtomicReference
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.AppJson

/** What this service keeps from an account center account: the unique, unchangeable username and the verified email, if any. */
data class AccountProfile(val username: String, val email: String?)

/** The unified account center (services/server-account): who a token belongs to, and its username and verified email. */
interface AccountCenter {
    /** Verifies an access token's signature, issuer, audience and expiry; returns its `sub`. */
    suspend fun verify(token: String): UUID

    /** The token's account profile; the email is null when the account only has a phone number. */
    suspend fun profile(token: String): AccountProfile
}

class RemoteAccountCenter(
    private val http: HttpClient,
    private val issuer: String,
    private val baseUrl: String,
    private val audiences: Set<String>,
    private val clock: Clock = Clock.systemUTC(),
) : AccountCenter {
    private class KeySet(val keys: Map<String, PublicKey>, val fetchedAtMs: Long)

    private val cache = AtomicReference(KeySet(emptyMap(), 0))

    override suspend fun verify(token: String): UUID {
        val parts = token.split('.')
        if (parts.size != 3) invalid()
        val header = decode(parts[0])
        if (header["alg"]?.jsonPrimitive?.contentOrNull != "EdDSA") invalid()
        val key = key(header["kid"]?.jsonPrimitive?.contentOrNull ?: invalid())
        val signed = Signature.getInstance("Ed25519").run {
            initVerify(key)
            update("${parts[0]}.${parts[1]}".toByteArray())
            runCatching { verify(Base64.getUrlDecoder().decode(parts[2])) }.getOrDefault(false)
        }
        if (!signed) invalid()

        val claims = decode(parts[1])
        val exp = claims["exp"]?.jsonPrimitive?.longOrNull ?: invalid()
        if (claims["iss"]?.jsonPrimitive?.contentOrNull != issuer) invalid()
        if (claims["aud"]?.jsonPrimitive?.contentOrNull !in audiences) invalid()
        if (exp + LEEWAY_SECONDS < clock.instant().epochSecond) {
            throw ApiException(HttpStatusCode.Unauthorized, "account_token_expired", "账号登录已过期，请重新登录")
        }
        return runCatching { UUID.fromString(claims["sub"]?.jsonPrimitive?.contentOrNull) }.getOrNull() ?: invalid()
    }

    override suspend fun profile(token: String): AccountProfile {
        val response = try {
            http.get("$baseUrl/api/me") { bearerAuth(token) }
        } catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            unavailable()
        }
        if (response.status == HttpStatusCode.Unauthorized) invalid()
        if (!response.status.isSuccess()) unavailable()
        val body = runCatching { AppJson.parseToJsonElement(response.bodyAsText()).jsonObject }.getOrElse { unavailable() }
        val username = body["username"]?.jsonPrimitive?.contentOrNull?.trim()?.takeIf { it.isNotEmpty() } ?: unavailable()
        val email = body["email"]?.jsonPrimitive?.contentOrNull?.trim()?.lowercase()?.takeIf { it.isNotEmpty() }
        return AccountProfile(username, email)
    }

    private suspend fun key(kid: String): PublicKey {
        cache.get().keys[kid]?.let { return it }
        // Unknown kid: refresh the key set, but not more than once every 30 seconds so bad tokens cannot hammer the account center.
        val current = cache.get()
        if (clock.millis() - current.fetchedAtMs >= REFRESH_MIN_INTERVAL_MS) {
            val fetched = KeySet(fetchKeys(), clock.millis())
            cache.set(fetched)
            fetched.keys[kid]?.let { return it }
        }
        invalid()
    }

    private suspend fun fetchKeys(): Map<String, PublicKey> {
        val body = try {
            val response = http.get("$baseUrl/.well-known/jwks.json")
            if (!response.status.isSuccess()) unavailable()
            response.bodyAsText()
        } catch (e: ApiException) {
            throw e
        } catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            unavailable()
        }
        val keys = runCatching { AppJson.parseToJsonElement(body).jsonObject["keys"] as kotlinx.serialization.json.JsonArray }
            .getOrElse { unavailable() }
        val factory = KeyFactory.getInstance("Ed25519")
        return keys.mapNotNull { element ->
            val jwk = element.jsonObject
            if (jwk["kty"]?.jsonPrimitive?.contentOrNull != "OKP" || jwk["crv"]?.jsonPrimitive?.contentOrNull != "Ed25519") return@mapNotNull null
            val kid = jwk["kid"]?.jsonPrimitive?.contentOrNull ?: return@mapNotNull null
            val raw = runCatching { Base64.getUrlDecoder().decode(jwk["x"]?.jsonPrimitive?.contentOrNull) }.getOrNull()
                ?.takeIf { it.size == 32 } ?: return@mapNotNull null
            kid to factory.generatePublic(X509EncodedKeySpec(ED25519_SPKI_PREFIX + raw))
        }.toMap()
    }

    private fun decode(part: String): JsonObject =
        runCatching { AppJson.parseToJsonElement(String(Base64.getUrlDecoder().decode(part))).jsonObject }.getOrNull() ?: invalid()

    private fun invalid(): Nothing =
        throw ApiException(HttpStatusCode.Unauthorized, "account_token_invalid", "账号登录凭证无效，请重新登录")

    private fun unavailable(): Nothing =
        throw ApiException(HttpStatusCode.ServiceUnavailable, "account_center_unavailable", "账号中心暂时不可用，请稍后再试")

    private companion object {
        const val LEEWAY_SECONDS = 10L
        const val REFRESH_MIN_INTERVAL_MS = 30_000L
        val ED25519_SPKI_PREFIX = byteArrayOf(0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00)
    }
}
