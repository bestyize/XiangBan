package xyz.xiaban.server.auth

import com.auth0.jwt.JWT
import com.auth0.jwt.JWTVerifier
import com.auth0.jwt.algorithms.Algorithm
import io.ktor.http.HttpStatusCode
import io.ktor.server.application.Application
import io.ktor.server.application.ApplicationCall
import io.ktor.server.application.install
import io.ktor.server.auth.Authentication
import io.ktor.server.auth.jwt.jwt
import io.ktor.server.auth.principal
import io.ktor.server.response.respond
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.shared.ApiError
import java.time.Duration
import java.time.Instant

const val AUTH_USER = "user"

data class UserPrincipal(val userId: Long)

class Tokens(secret: String, private val ttl: Duration = Duration.ofDays(30)) {
    private val algorithm = Algorithm.HMAC256(secret)

    val verifier: JWTVerifier = JWT.require(algorithm).withIssuer(ISSUER).build()

    fun userIdOrNull(token: String): Long? = runCatching { verifier.verify(token).subject?.toLongOrNull() }.getOrNull()

    fun issue(userId: Long, now: Instant = Instant.now()): String = JWT.create()
        .withIssuer(ISSUER)
        .withSubject(userId.toString())
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
            validate { credential -> credential.payload.subject?.toLongOrNull()?.let(::UserPrincipal) }
            challenge { _, _ ->
                call.respond(HttpStatusCode.Unauthorized, ApiError("unauthorized", "登录状态过期了，刷新一下页面"))
            }
        }
    }
}

fun ApplicationCall.userId(): Long = principal<UserPrincipal>()?.userId
    ?: throw ApiException(HttpStatusCode.Unauthorized, "unauthorized", "登录状态过期了，刷新一下页面")
