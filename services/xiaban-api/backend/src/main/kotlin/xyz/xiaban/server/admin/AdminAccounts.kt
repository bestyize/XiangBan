package xyz.xiaban.server.admin

import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.future.await
import kotlinx.coroutines.withContext
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.http.tooMany
import xyz.xiaban.server.redis.Redis
import java.security.MessageDigest
import java.security.SecureRandom
import java.util.Base64
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.PBEKeySpec

class AdminAccounts(private val db: Db, private val redis: Redis, private val config: AppConfig) {
    private val random = SecureRandom()

    suspend fun login(username: String, password: String, ip: String): String {
        val rateKey = "xb:admin:login:${xyz.xiaban.server.users.SessionService.sha256(ip)}"
        val attempts = redis.cmd.eval<Long>("local n=redis.call('INCR',KEYS[1]); if n==1 then redis.call('EXPIRE',KEYS[1],900) end; return n",
            io.lettuce.core.ScriptOutputType.INTEGER, arrayOf(rateKey)).await()
        if (attempts > 10) tooMany("登录尝试过多，请 15 分钟后再试")
        bootstrap()
        val row = db.read { c -> c.queryOne("select password_hash, session_version from administrators where username = ?", username) { it.getString(1) to it.getInt(2) } }
        if (password.length > 256 || row == null || !verify(password, row.first)) {
            throw ApiException(HttpStatusCode.Unauthorized, "admin_login_failed", "管理员账号或密码不正确")
        }
        val token = Base64.getUrlEncoder().withoutPadding().encodeToString(ByteArray(32).also(random::nextBytes))
        redis.cmd.setex(sessionKey(token), 8 * 3600L, "$username:${row.second}").await()
        redis.cmd.del(rateKey).await()
        return token
    }

    suspend fun username(token: String?): String? {
        if (token == null || !Regex("^[A-Za-z0-9_-]{43}$").matches(token)) return null
        val value = redis.cmd.get(sessionKey(token)).await() ?: return null
        val parts = value.split(':')
        if (parts.size != 2) return null
        val version = db.read { c -> c.queryOne("select session_version from administrators where username = ?", parts[0]) { it.getInt(1) } }
        return parts[0].takeIf { version?.toString() == parts[1] }
    }

    suspend fun logout(token: String?) { token?.let { redis.cmd.del(sessionKey(it)).await() } }

    suspend fun changePassword(username: String, old: String, next: String) {
        if (next.length !in 6..256) badRequest("新密码需为 6–256 个字符")
        val previous = db.read { c -> c.queryOne("select password_hash from administrators where username = ?", username) { it.getString(1) } }
        if (previous == null || old.length > 256 || !verify(old, previous)) badRequest("当前密码不正确")
        val replacement = hash(next)
        db.tx { c ->
            val changed = c.execute("update administrators set password_hash = ?, session_version = session_version + 1, updated_at = now() where username = ? and password_hash = ?", replacement, username, previous)
            if (changed == 0) badRequest("密码已发生变化，请重新登录")
            c.execute("insert into admin_audit_log(actor,action,summary) values (?,?,?)", username, "admin_password", "修改管理员密码并撤销旧会话")
        }
    }

    private suspend fun bootstrap() {
        val password = config.adminBootstrapPassword ?: return
        if (password.length < 6 || !Regex("^[A-Za-z0-9_-]{1,40}$").matches(config.adminUsername)) return
        if (db.read { c -> c.queryOne("select username from administrators limit 1") { it.getString(1) } } != null) return
        val encoded = hash(password)
        db.read { c -> c.execute("insert into administrators(username,password_hash) values (?,?) on conflict do nothing", config.adminUsername, encoded) }
    }

    private fun sessionKey(token: String) = "xb:admin:session:${xyz.xiaban.server.users.SessionService.sha256(token)}"
    private suspend fun hash(password: String): String {
        val salt = ByteArray(16).also(random::nextBytes)
        return Base64.getEncoder().encodeToString(salt) + ":" + Base64.getEncoder().encodeToString(derive(password, salt))
    }
    private suspend fun verify(password: String, value: String): Boolean {
        val (salt, expected) = value.split(':')
        return MessageDigest.isEqual(derive(password, Base64.getDecoder().decode(salt)), Base64.getDecoder().decode(expected))
    }
    private suspend fun derive(password: String, salt: ByteArray): ByteArray = withContext(Dispatchers.Default) {
        val spec = PBEKeySpec(password.toCharArray(), salt, 210_000, 256)
        try { SecretKeyFactory.getInstance("PBKDF2WithHmacSHA256").generateSecret(spec).encoded } finally { spec.clearPassword() }
    }
}
