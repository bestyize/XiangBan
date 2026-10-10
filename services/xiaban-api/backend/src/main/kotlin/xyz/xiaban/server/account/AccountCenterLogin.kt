package xyz.xiaban.server.account

import xyz.xiaban.contracts.account.auth.AccountCenterLoginRequest
import xyz.xiaban.contracts.account.auth.BindResponse
import xyz.xiaban.server.auth.Tokens
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.conflict
import xyz.xiaban.server.users.ProfileChanges
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.SessionService
import xyz.xiaban.server.users.UserRepository

/** Signs people in with an account-center token and maps its uid to a local account (see [UserRepository.claimAccount]). */
class AccountCenterLogin(
    private val center: AccountCenter,
    private val users: UserRepository,
    private val profiles: ProfileService,
    private val tokens: Tokens,
    private val merger: AccountMerge,
    private val afterMerge: suspend (Long) -> Unit,
) {
    suspend fun login(userId: Long?, req: AccountCenterLoginRequest): BindResponse {
        if (!SessionService.isValidDeviceId(req.deviceId)) badRequest("设备标识无效", "invalid_device")
        val uid = center.verify(req.accountToken)
        val owner = userId?.takeIf { it > 0 } ?: 0L
        val linked = users.findIdByAccountUid(uid)
        // The email only matters for accounts not yet linked locally, so skip the extra round trip for returning users.
        val email = if (linked == null) center.email(req.accountToken) else null
        if (owner > 0) {
            val current = profiles.user(owner)
            if (current.termsVersion.isNullOrBlank()) conflict("请先阅读并同意用户协议和隐私政策", "consent_required")
            val switching = when {
                current.accountUid != null -> current.accountUid != uid
                current.email != null -> linked != null || email != current.email
                else -> false
            }
            if (switching) conflict("请先退出当前账号，再使用其他账号登录", "account_switch_required")
        }
        val target = users.claimAccount(owner, uid, email, req.username)

        val merged = owner > 0 && target != owner && req.mergeGuest
        if (merged) { merger.merge(source = owner, target = target); afterMerge(target) }
        users.linkDevice(SessionService.sha256(req.deviceId), target)
        if (target == owner || owner == 0L) users.update(target, ProfileChanges(markOnboarded = true))
        return BindResponse(tokens.issue(target, authenticated = true), merged, profiles.view(target))
    }
}
