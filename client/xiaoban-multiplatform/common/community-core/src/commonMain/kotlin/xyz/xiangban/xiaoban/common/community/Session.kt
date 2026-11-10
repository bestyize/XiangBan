package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
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

/** accountId is the internal UID used for account isolation; user.publicAccountId is the display ID. */
data class SessionState(
    val loggedIn: Boolean = false,
    val user: AccountUser? = null,
    val accountId: String? = null,
    val generation: Long = 0,
)

data class SessionIdentity(val accountId: String?, val generation: Long)

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
    private val refreshLock = Mutex()
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

    private suspend fun signedIn(login: LoginResponse) = lock.withLock {
        val old = tokens?.accountId
        accept(TokenPair(login.accessToken, login.refreshToken, login.expiresIn), login.user.uid)
        _state.value = SessionState(true, login.user, login.user.uid, _state.value.generation + 1)
        if (old != login.user.uid) onAccountChanged(old, login.user.uid)
    }

    fun identity(): SessionIdentity? = _state.value.let {
        if (it.loggedIn) SessionIdentity(it.accountId, it.generation) else null
    }

    /** Bound callers get the original failure, so a temporary outage doesn't look like a logout. */
    suspend fun accessToken(expected: SessionIdentity? = null): String? = refreshLock.withLock {
        val captured = lock.withLock capture@{
            checkIdentity(expected)
            val current = tokens ?: return@capture null
            current to identity()
        } ?: return@withLock null
        val (current, owner) = captured
        if (current.expiresAtMillis - refreshMarginMillis > now()) return@withLock current.accessToken
        val refreshed = try {
            account.refresh(current.refreshToken)
        } catch (e: ApiException) {
            return@withLock lock.withLock failed@{
                checkIdentity(expected)
                if (identity() != owner || tokens != current) return@failed null
                val invalid = e.status == 401 || e.status == 403 ||
                    e.code in setOf("invalid_grant", "invalid_refresh_token", "refresh_token_revoked", "refresh_token_expired")
                if (invalid) clear()
                else if (expected != null) throw e
                null
            }
        }
        lock.withLock {
            checkIdentity(expected)
            if (identity() != owner || tokens != current) null else accept(refreshed).accessToken
        }
    }

    private fun checkIdentity(expected: SessionIdentity?) {
        if (expected != null && identity() != expected) {
            throw ApiException(401, "账号已切换，请重新发起请求", "account_changed")
        }
    }

    suspend fun logout() {
        val refresh = lock.withLock {
            val token = tokens?.refreshToken
            clear()
            token
        }
        if (refresh != null) account.logout(refresh)
    }

    private fun clear() {
        val oldId = tokens?.accountId
        tokens = null
        storage.save(null)
        _state.value = SessionState(generation = _state.value.generation + 1)
        if (oldId != null) onAccountChanged(oldId, null)
    }

    /** Restore the account-center profile for sessions loaded from token storage. */
    suspend fun loadProfile() {
        val token = accessToken() ?: return
        val profile = account.me(token)
        lock.withLock {
            if (tokens?.accessToken != token) return@withLock
            val uid = tokens?.accountId
            if (uid != null && uid != profile.uid) return@withLock
            if (uid == null) {
                tokens = tokens!!.copy(accountId = profile.uid)
                storage.save(tokens)
            }
            _state.value = _state.value.copy(loggedIn = true, user = profile, accountId = profile.uid)
        }
    }

    /** 论坛认出令牌后给的昵称等资料，用来补全“我的”页。 */
    fun rememberUser(user: AccountUser) {
        _state.update { if (it.loggedIn && (it.accountId == null || it.accountId == user.uid)) it.copy(user = user) else it }
    }
}
