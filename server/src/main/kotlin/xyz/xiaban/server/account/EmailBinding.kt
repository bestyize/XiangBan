package xyz.xiaban.server.account

import io.ktor.http.HttpStatusCode
import io.lettuce.core.ScriptOutputType
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.future.await
import xyz.xiaban.server.auth.Tokens
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.conflict
import xyz.xiaban.server.http.tooMany
import xyz.xiaban.server.mail.Mail
import xyz.xiaban.server.mail.verificationMail
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.users.ProfileChanges
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.SessionService
import xyz.xiaban.server.users.UserRepository
import xyz.xiaban.shared.BindResponse
import java.security.SecureRandom
import java.util.concurrent.TimeUnit

/** Passwordless registration and login; email ownership is verified before linking any account. */
class EmailBinding(
    private val redis: Redis,
    private val users: UserRepository,
    private val profiles: ProfileService,
    private val tokens: Tokens,
    private val merger: AccountMerge,
    private val available: suspend () -> Boolean,
    private val deliver: suspend (Mail) -> Unit,
    private val afterMerge: suspend (Long) -> Unit,
) {
    private val random = SecureRandom()

    suspend fun sendCode(userId: Long?, rawEmail: String, login: Boolean = false, ip: String = "unknown") {
        val email = normalize(rawEmail)
        if (userId != null && userId > 0) checkAccount(userId, email, login)
        if (!available()) throw ApiException(HttpStatusCode.ServiceUnavailable, "mail_unavailable", "邮箱服务暂未开通，请稍后再试")
        val hash = SessionService.sha256(email)
        val cooldown = "xb:code:cool:$hash"
        if (redis.cmd.set(cooldown, "1", io.lettuce.core.SetArgs.Builder.nx().ex(60)).await() != "OK") {
            tooMany("验证码刚发过，等一分钟再试", "code_cooldown")
        }
        if (userId != null && userId > 0) limit("xb:code:daily:$userId", 86400, 10)
        limit("xb:code:email:daily:$hash", 86400, 10)
        limit("xb:code:ip:hour:${SessionService.sha256(ip)}", 3600, 20)
        val code = (0 until 6).joinToString("") { random.nextInt(10).toString() }
        val key = codeKey(email, login)
        val owner = (userId ?: 0L).toString()
        redis.cmd.setex(key, 600, "${codeHash(email, code, login)}:$owner:0").await()
        try {
            // Wait for SMTP acceptance so failed deliveries are reported to the user immediately.
            deliver(verificationMail(email, code))
        } catch (e: Exception) {
            redis.cmd.del(key, cooldown).await()
            if (e is CancellationException) throw e
            throw ApiException(HttpStatusCode.ServiceUnavailable, "mail_send_failed", "验证码邮件未能发出，请稍后重试")
        }
    }

    suspend fun bind(userId: Long, rawEmail: String, code: String, username: String? = null): BindResponse {
        val email = normalize(rawEmail)
        checkAccount(userId, email, false)
        verify(userId, email, code, false)
        val target = users.claimEmail(userId, email, username) { consume(userId, email, code, false) }
        if (target != userId) {
            merger.merge(source = userId, target = target)
            afterMerge(target)
        }
        return BindResponse(tokens.issue(target, authenticated = true), target != userId, profiles.view(target))
    }

    suspend fun login(userId: Long?, rawEmail: String, code: String, deviceId: String, mergeGuest: Boolean, username: String? = null): BindResponse {
        if (!SessionService.isValidDeviceId(deviceId)) badRequest("设备标识无效", "invalid_device")
        val email = normalize(rawEmail)
        if (userId != null && userId > 0) checkAccount(userId, email, true)
        val owner = userId ?: 0L
        verify(owner, email, code, true)
        val target = users.claimEmail(owner, email, username) { consume(owner, email, code, true) }
        val merged = userId != null && userId > 0 && target != userId && mergeGuest
        if (merged) { merger.merge(source = userId, target = target); afterMerge(target) }
        users.linkDevice(SessionService.sha256(deviceId), target)
        if (target == owner || (userId == null || userId <= 0L)) users.update(target, ProfileChanges(markOnboarded = true))
        return BindResponse(tokens.issue(target, authenticated = true), merged, profiles.view(target))
    }

    private suspend fun checkAccount(userId: Long, email: String, login: Boolean) {
        val current = profiles.user(userId)
        if (current.email != null && (login || current.email != email)) conflict("请先退出当前账号，再使用其他邮箱登录", "account_switch_required")
        if (current.termsVersion.isNullOrBlank()) conflict("请先阅读并同意用户协议和隐私政策", "consent_required")
    }

    private suspend fun limit(key: String, seconds: Long, max: Long) {
        val count = redis.cmd.eval<Long>("local n=redis.call('INCR',KEYS[1]); if n==1 then redis.call('EXPIRE',KEYS[1],ARGV[1]) end; return n",
            ScriptOutputType.INTEGER, arrayOf(key), seconds.toString()).await()
        if (count > max) tooMany("验证码请求太多了，请稍后再试", "code_limit")
    }

    private suspend fun verify(userId: Long, email: String, rawCode: String, login: Boolean) {
        if (!available()) throw ApiException(HttpStatusCode.ServiceUnavailable, "mail_unavailable", "邮箱服务暂未开通，请稍后再试")
        val code = rawCode.trim()
        if (!Regex("^[0-9]{6}$").matches(code)) badRequest("请输入邮件里的 6 位验证码", "code_wrong")
        val result = verification(userId, email, code, login, false).await()
        checkVerification(result)
    }

    /** Called on the database IO thread after username constraints pass, before committing registration. */
    private fun consume(userId: Long, email: String, code: String, login: Boolean) {
        checkVerification(verification(userId, email, code.trim(), login, true).get(5, TimeUnit.SECONDS))
    }
    private fun verification(userId: Long, email: String, code: String, login: Boolean, consume: Boolean) =
        redis.cmd.eval<String>(VERIFY, ScriptOutputType.VALUE, arrayOf(codeKey(email, login)),
            codeHash(email, code, login), userId.toString(), if (consume) "1" else "0")

    private fun checkVerification(result: String) {
        when (result) {
            "ok" -> Unit
            "expired" -> badRequest("验证码过期了，重新发一个吧", "code_expired")
            else -> badRequest("验证码不对，连续错误 5 次需重新获取", "code_wrong")
        }
    }

    private fun codeKey(email: String, login: Boolean) = "xb:code:${if (login) "login" else "bind"}:${SessionService.sha256(email)}"
    private fun codeHash(email: String, code: String, login: Boolean) = SessionService.sha256("$login:$email:$code")

    companion object {
        private val pattern = Regex("""^[^\s@]+@[^\s@]+\.[^\s@]+$""")
        private val VERIFY = """
            local value=redis.call('GET',KEYS[1])
            if not value then return 'expired' end
            local expected,owner,attempts=string.match(value,'^([^:]+):([^:]+):(%d+)$')
            if not expected then return 'expired' end
            if owner~='0' and ARGV[2]~='0' and owner~=ARGV[2] then return 'expired' end
            if expected==ARGV[1] then
                if ARGV[3]=='1' then redis.call('DEL',KEYS[1]) end
                return 'ok'
            end
            local n=tonumber(attempts)+1
            if n>=5 then redis.call('DEL',KEYS[1])
            else redis.call('SET',KEYS[1],expected..':'..owner..':'..n,'KEEPTTL') end
            return 'wrong'
        """.trimIndent()

        fun normalize(raw: String): String {
            val email = raw.trim().lowercase()
            if (email.length > 254 || !pattern.matches(email)) badRequest("邮箱格式不对", "invalid_email")
            return email
        }
    }
}
