package xyz.xiaban.server.support

import io.ktor.http.HttpStatusCode
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicInteger
import xyz.xiaban.server.account.AccountCenter
import xyz.xiaban.server.http.ApiException

class FakeAccountCenter : AccountCenter {
    private class Account(val uid: UUID, val email: String?)

    private val tokens = ConcurrentHashMap<String, Account>()
    val emailLookups = AtomicInteger()

    fun reset() {
        tokens.clear()
        emailLookups.set(0)
    }

    /** Pretends the account center issued a token for [uid]; reuse a uid to simulate the same person logging in again. */
    fun issue(uid: UUID = UUID.randomUUID(), email: String? = null): String =
        "fake-${UUID.randomUUID()}".also { tokens[it] = Account(uid, email) }

    override suspend fun verify(token: String): UUID = tokens[token]?.uid
        ?: throw ApiException(HttpStatusCode.Unauthorized, "account_token_invalid", "账号登录凭证无效，请重新登录")

    override suspend fun email(token: String): String? {
        emailLookups.incrementAndGet()
        return tokens[token]?.email?.trim()?.lowercase()
    }
}
