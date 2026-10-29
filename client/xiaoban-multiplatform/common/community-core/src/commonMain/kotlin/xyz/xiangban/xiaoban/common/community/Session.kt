package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

data class Tokens(val accessToken: String, val refreshToken: String, val expiresAtMillis: Long, val accountId: String? = null)

/** 令牌存哪里由平台决定：Android 用应用私有存储，测试用内存。 */
interface TokenStorage {
    fun load(): Tokens?
    fun save(tokens: Tokens?)
}

class InMemoryTokenStorage(private var value: Tokens? = null) : TokenStorage {
    override fun load() = value
    override fun save(tokens: Tokens?) { value = tokens }
}

data class SessionState(val loggedIn: Boolean = false, val user: AccountUser? = null, val accountId: String? = null)

/**
 * 登录态：保存令牌、临近过期时自动刷新（并发请求共用一次刷新）、登录与退出。
 * 访问令牌到期前 [refreshMarginMillis] 就换新，避免请求途中过期。
 */
class SessionManager(
    private val account: AccountApi,
    private val storage: TokenStorage,
    private val now: () -> Long,
    private val refreshMarginMillis: Long = 60_000,
    private val onAccountChanged: (String?, String?) -> Unit = { _, _ -> },
) {
    private val lock = Mutex()
    private var tokens: Tokens? = storage.load()
    private val _state = MutableStateFlow(SessionState(loggedIn = tokens != null, accountId = tokens?.accountId))
    val state: StateFlow<SessionState> = _state.asStateFlow()

    private fun accept(pair: TokenPair, accountId: String? = tokens?.accountId): Tokens {
        val next = Tokens(pair.accessToken, pair.refreshToken, now() + pair.expiresIn * 1000, accountId)
        tokens = next
        storage.save(next)
        return next
    }

    suspend fun loginWithCode(identifier: String, code: String): CodeLogin {
        val result = account.loginWithCode(identifier, code)
        if (result is CodeLogin.Done) signedIn(result.login)
        return result
    }

    suspend fun loginWithPassword(identifier: String, password: String) = signedIn(account.loginWithPassword(identifier, password))
    suspend fun register(registrationToken: String, username: String, password: String) = signedIn(account.register(registrationToken, username, password))

    private suspend fun signedIn(login: LoginResponse) {
        val oldId = lock.withLock {
            val old = tokens?.accountId
            accept(TokenPair(login.accessToken, login.refreshToken, login.expiresIn), login.user.uid)
            old
        }
        _state.value = SessionState(loggedIn = true, user = login.user, accountId = login.user.uid)
        if (oldId != login.user.uid) onAccountChanged(oldId, login.user.uid)
    }

    /** 当前可用的访问令牌；没登录返回 null；刷新被服务端拒绝就当作已退出。 */
    suspend fun accessToken(): String? = lock.withLock {
        val current = tokens ?: return@withLock null
        if (current.expiresAtMillis - refreshMarginMillis > now()) return@withLock current.accessToken
        try {
            accept(account.refresh(current.refreshToken)).accessToken
        } catch (e: ApiException) {
            // 没连上（status 0）保留登录态，下次再试；服务端明确拒绝才退出。
            if (e.status != 0) clear()
            null
        }
    }

    suspend fun logout() {
        val refresh = lock.withLock { tokens?.refreshToken }
        if (refresh != null) account.logout(refresh)
        lock.withLock { clear() }
    }

    private fun clear() {
        val oldId = tokens?.accountId
        tokens = null
        storage.save(null)
        _state.value = SessionState()
        if (oldId != null) onAccountChanged(oldId, null)
    }

    /** 论坛认出令牌后给的昵称等资料，用来补全“我的”页。 */
    fun rememberUser(user: AccountUser) { _state.value = _state.value.copy(user = user) }
}
