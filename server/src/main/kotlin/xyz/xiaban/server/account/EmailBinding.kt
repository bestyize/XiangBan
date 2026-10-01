package xyz.xiaban.server.account

import kotlinx.coroutines.future.await
import xyz.xiaban.server.auth.Tokens
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.tooMany
import xyz.xiaban.server.mail.MailJob
import xyz.xiaban.server.mail.MailQueue
import xyz.xiaban.server.redis.Redis
import xyz.xiaban.server.users.ProfileService
import xyz.xiaban.server.users.SessionService
import xyz.xiaban.server.users.UserRepository
import xyz.xiaban.shared.BindResponse
import java.security.SecureRandom

/** Binds an email to the anonymous account with a 6-digit code (PRD 9.4); phone binding needs SMS credentials and is left out. */
class EmailBinding(
    private val redis: Redis,
    private val users: UserRepository,
    private val profiles: ProfileService,
    private val tokens: Tokens,
    private val merger: AccountMerge,
    private val mail: MailQueue,
    private val afterMerge: suspend (targetUserId: Long) -> Unit,
) {
    private val random = SecureRandom()

    suspend fun sendCode(userId: Long, rawEmail: String) {
        val email = normalize(rawEmail)
        val hash = SessionService.sha256(email)
        if (redis.cmd.set("xb:code:cool:$hash", "1", io.lettuce.core.SetArgs.Builder.nx().ex(60)).await() != "OK") {
            tooMany("验证码刚发过，等一分钟再试", "code_cooldown")
        }
        val daily = redis.cmd.incr("xb:code:daily:$userId").await()
        if (daily == 1L) redis.cmd.expire("xb:code:daily:$userId", 24 * 3600L).await()
        if (daily > DAILY_CODES) tooMany("今天发的验证码太多了，明天再来吧", "code_limit")

        val code = (0 until 6).joinToString("") { random.nextInt(10).toString() }
        redis.cmd.setex(codeKey(hash), CODE_TTL_SECONDS, "$code:$userId:0").await()
        mail.send(
            MailJob(
                kind = "bind_code",
                to = email,
                subject = "下班了么验证码：$code",
                text = "你的验证码是 $code，10 分钟内有效。\n\n如果不是你本人在绑定下班了么，忽略这封邮件就好。\n\n— 下班了么 xiaban.xyz",
            ),
        )
    }

    suspend fun bind(userId: Long, rawEmail: String, code: String): BindResponse {
        val email = normalize(rawEmail)
        val key = codeKey(SessionService.sha256(email))
        val (expected, owner, attempts) = redis.cmd.get(key).await()?.split(":")?.takeIf { it.size == 3 }
            ?: badRequest("验证码过期了，重新发一个吧", "code_expired")
        if (owner != userId.toString()) badRequest("验证码过期了，重新发一个吧", "code_expired")
        if (code.trim() != expected) {
            if (attempts.toInt() + 1 >= MAX_ATTEMPTS) redis.cmd.del(key).await()
            else redis.cmd.set(key, "$expected:$owner:${attempts.toInt() + 1}", io.lettuce.core.SetArgs.Builder.keepttl()).await()
            badRequest("验证码不对，再看一眼邮件", "code_wrong")
        }
        redis.cmd.del(key).await()

        val existing = users.findIdByEmail(email)
        if (existing == null || existing == userId) {
            users.setEmail(userId, email)
            return BindResponse(tokens.issue(userId), merged = false, profile = profiles.view(userId))
        }
        merger.merge(source = userId, target = existing)
        afterMerge(existing)
        return BindResponse(tokens.issue(existing), merged = true, profile = profiles.view(existing))
    }

    private fun codeKey(hash: String) = "xb:code:$hash"

    companion object {
        private const val CODE_TTL_SECONDS = 600L
        private const val MAX_ATTEMPTS = 5
        private const val DAILY_CODES = 10
        private val pattern = Regex("""^[^\s@]+@[^\s@]+\.[^\s@]+$""")

        fun normalize(raw: String): String {
            val email = raw.trim().lowercase()
            if (email.length > 254 || !pattern.matches(email)) badRequest("邮箱格式不对", "invalid_email")
            return email
        }
    }
}
