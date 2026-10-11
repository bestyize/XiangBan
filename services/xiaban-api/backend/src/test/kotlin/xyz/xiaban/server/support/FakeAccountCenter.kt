package xyz.xiaban.server.support

import io.ktor.http.HttpStatusCode
import java.util.UUID
import java.util.concurrent.ConcurrentHashMap
import xyz.xiaban.server.account.AccountCenter
import xyz.xiaban.server.account.AccountProfile
import xyz.xiaban.server.http.ApiException

class FakeAccountCenter : AccountCenter {
    private class Account(val uid: UUID, val profile: AccountProfile)

    private val tokens = ConcurrentHashMap<String, Account>()

    fun reset() {
        tokens.clear()
    }

    /** Pretends the account center issued a token for [uid]; reuse a uid to simulate the same person logging in again. */
    fun issue(uid: UUID = UUID.randomUUID(), email: String? = null, username: String = "user_${uid.toString().take(8)}"): String =
        "fake-${UUID.randomUUID()}".also { tokens[it] = Account(uid, AccountProfile(username, email?.trim()?.lowercase())) }

    override suspend fun verify(token: String): UUID = tokens[token]?.uid
        ?: throw ApiException(HttpStatusCode.Unauthorized, "account_token_invalid", "账号登录凭证无效，请重新登录")

    override suspend fun profile(token: String): AccountProfile = tokens[token]?.profile
        ?: throw ApiException(HttpStatusCode.Unauthorized, "account_token_invalid", "账号登录凭证无效，请重新登录")
}
