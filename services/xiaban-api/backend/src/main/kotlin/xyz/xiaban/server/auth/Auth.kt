package xyz.xiaban.server.auth

import com.auth0.jwt.JWT
import com.auth0.jwt.JWTVerifier
import com.auth0.jwt.algorithms.Algorithm
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.server.application.Application
import io.ktor.server.application.ApplicationCall
import io.ktor.server.application.install
import io.ktor.server.auth.Authentication
import io.ktor.server.auth.jwt.jwt
import io.ktor.server.auth.principal
import io.ktor.server.response.respond
import java.time.Duration
import java.time.Instant
import java.util.UUID
import kotlinx.coroutines.future.await
import xyz.xiaban.contracts.common.ApiError
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.users.SessionService

const val AUTH_USER = "user"

data class UserPrincipal(val userId: Long)

class Tokens(secret: String, private val redis: Redis, private val accountState: suspend (Long) -> Boolean?, private val ttl: Duration = Duration.ofDays(30)) {
    private val algorithm = Algorithm.HMAC256(secret)

    val verifier: JWTVerifier = JWT.require(algorithm).withIssuer(ISSUER).build()

    suspend fun userIdOrNull(token: String): Long? {
        val jwt = runCatching { verifier.verify(token) }.getOrNull() ?: return null
        if (redis.cmd.exists(revokedKey(token)).await() == 1L) return null
        val userId = jwt.subject?.toLongOrNull() ?: return null
        val registered = accountState(userId) ?: return null
        val kind = jwt.getClaim("account_kind").asString()
        if (kind == "guest" && registered || kind == "user" && !registered) return null
        return userId
    }

    suspend fun revoke(token: String) {
        val jwt = runCatching { verifier.verify(token) }.getOrNull() ?: return
        val seconds = Duration.between(Instant.now(), jwt.expiresAt.toInstant()).seconds + 1
        if (seconds > 0) redis.cmd.setex(revokedKey(token), seconds, "1").await()
    }

    private fun revokedKey(token: String) = "xb:token:revoked:${SessionService.sha256(token)}"

    fun issue(userId: Long, now: Instant = Instant.now(), authenticated: Boolean = false): String = JWT.create()
        .withIssuer(ISSUER)
        .withJWTId(UUID.randomUUID().toString())
        .withSubject(userId.toString())
        .withClaim("account_kind", if (authenticated) "user" else "guest")
        .withIssuedAt(now)
        .withExpiresAt(now.plus(ttl))
        .sign(algorithm)

    private companion object {
        const val ISSUER = "xiaban"
    }
}

fun Application.installAuth(tokens: Tokens) {
    install(Authentication) {
        jwt(AUTH_USER) {
            verifier(tokens.verifier)
            validate {
                val token = request.headers[HttpHeaders.Authorization]?.removePrefix("Bearer ")?.trim()
                token?.let { tokens.userIdOrNull(it) }?.let(::UserPrincipal)
            }
            challenge { _, _ ->
                call.respond(HttpStatusCode.Unauthorized, ApiError("unauthorized", "登录状态过期了，刷新一下页面"))
            }
        }
    }
}

fun ApplicationCall.userId(): Long = principal<UserPrincipal>()?.userId
    ?: throw ApiException(HttpStatusCode.Unauthorized, "unauthorized", "登录状态过期了，刷新一下页面")
