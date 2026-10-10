package xyz.xiaban.server.account

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import java.security.KeyPair
import java.security.KeyPairGenerator
import java.security.Signature
import java.time.Clock
import java.time.Instant
import java.time.ZoneOffset
import java.util.Base64
import java.util.UUID
import java.util.concurrent.atomic.AtomicInteger
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlinx.coroutines.runBlocking
import xyz.xiaban.server.http.ApiException

class RemoteAccountCenterTest {
    private val issuer = "https://account.example.test"
    private val now = Instant.parse("2026-10-11T08:00:00Z")
    private val clock = Clock.fixed(now, ZoneOffset.UTC)
    private val keyPair: KeyPair = KeyPairGenerator.getInstance("Ed25519").generateKeyPair()
    private val jwksFetches = AtomicInteger()
    private var meBody = """{"uid":"x","email":"Person@Example.com"}"""
    private var meStatus = HttpStatusCode.OK

    private fun center(kid: String = "k1"): RemoteAccountCenter {
        val x = Base64.getUrlEncoder().withoutPadding().encodeToString(keyPair.public.encoded.takeLast(32).toByteArray())
        val engine = MockEngine { request ->
            when (request.url.encodedPath) {
                "/.well-known/jwks.json" -> {
                    jwksFetches.incrementAndGet()
                    respond("""{"keys":[{"kty":"OKP","crv":"Ed25519","alg":"EdDSA","kid":"$kid","x":"$x"}]}""", HttpStatusCode.OK,
                        headersOf(HttpHeaders.ContentType, "application/json"))
                }
                "/api/me" -> respond(meBody, meStatus, headersOf(HttpHeaders.ContentType, "application/json"))
                else -> respond("", HttpStatusCode.NotFound)
            }
        }
        return RemoteAccountCenter(HttpClient(engine), issuer, issuer, setOf("xiaban-web", "xiaban-api"), clock)
    }

    private fun token(
        uid: UUID = UUID.randomUUID(),
        kid: String = "k1",
        iss: String = issuer,
        aud: String = "xiaban-web",
        exp: Long = now.epochSecond + 600,
        alg: String = "EdDSA",
        signer: KeyPair = keyPair,
    ): String {
        fun enc(text: String) = Base64.getUrlEncoder().withoutPadding().encodeToString(text.toByteArray())
        val signingInput = enc("""{"alg":"$alg","kid":"$kid","typ":"JWT"}""") + "." +
            enc("""{"iss":"$iss","sub":"$uid","aud":"$aud","exp":$exp,"iat":${now.epochSecond}}""")
        val signature = Signature.getInstance("Ed25519").run {
            initSign(signer.private)
            update(signingInput.toByteArray())
            sign()
        }
        return signingInput + "." + Base64.getUrlEncoder().withoutPadding().encodeToString(signature)
    }

    @Test fun `accepts a valid token and returns its uid`() = runBlocking {
        val uid = UUID.randomUUID()
        assertEquals(uid, center().verify(token(uid)))
    }

    @Test fun `rejects wrong issuer, audience, signature, algorithm and unknown key`() = runBlocking {
        val center = center()
        val other = KeyPairGenerator.getInstance("Ed25519").generateKeyPair()
        for (bad in listOf(
            token(iss = "https://evil.example"),
            token(aud = "xiangban-wallpaper-web"),
            token(signer = other),
            token(alg = "none"),
            token(kid = "unknown"),
        )) {
            assertEquals("account_token_invalid", assertFailsWith<ApiException> { center.verify(bad) }.code)
        }
        assertEquals("account_token_invalid", assertFailsWith<ApiException> { center.verify("a.b") }.code)
    }

    @Test fun `rejects an expired token beyond the leeway`() = runBlocking {
        val center = center()
        center.verify(token(exp = now.epochSecond - 5))
        assertEquals("account_token_expired", assertFailsWith<ApiException> { center.verify(token(exp = now.epochSecond - 60)) }.code)
    }

    @Test fun `caches keys and does not refetch for every unknown key id`() = runBlocking {
        val center = center()
        center.verify(token())
        center.verify(token())
        assertEquals(1, jwksFetches.get())
        repeat(3) { assertFailsWith<ApiException> { center.verify(token(kid = "other-$it")) } }
        assertEquals(1, jwksFetches.get())
    }

    @Test fun `reads the email normalised and treats a missing email as null`() = runBlocking {
        val center = center()
        assertEquals("person@example.com", center.email("t"))
        meBody = """{"uid":"x"}"""
        assertNull(center.email("t"))
    }

    @Test fun `maps account center failures to clear errors`() = runBlocking {
        val center = center()
        meStatus = HttpStatusCode.Unauthorized
        assertEquals("account_token_invalid", assertFailsWith<ApiException> { center.email("t") }.code)
        meStatus = HttpStatusCode.InternalServerError
        assertEquals("account_center_unavailable", assertFailsWith<ApiException> { center.email("t") }.code)
    }
}
