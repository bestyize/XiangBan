package xyz.xiaban.server.admin

import kotlinx.serialization.Serializable
import xyz.xiaban.server.config.AppConfig
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.http.AppJson
import xyz.xiaban.server.http.badRequest
import xyz.xiaban.server.mail.SmtpConfig
import java.security.MessageDigest
import java.security.SecureRandom
import java.util.Base64
import javax.crypto.Cipher
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec

@Serializable
data class EmailServiceStatus(val enabled: Boolean, val configured: Boolean)

@Serializable
data class EmailSettings(
    val enabled: Boolean = true,
    val host: String = "",
    val port: Int = 465,
    val user: String = "",
    val password: String = "",
    val from: String = "",
    val fromName: String = "下班啦",
    val ssl: Boolean = true,
    val passwordConfigured: Boolean = false,
) {
    fun configured() = host.isNotBlank() && user.isNotBlank() && password.isNotBlank() && from.isNotBlank()
    fun smtp() = SmtpConfig(host, port, user, password, from, fromName, ssl)
    fun redacted() = copy(password = "", passwordConfigured = password.isNotBlank())
}

/** Persist SMTP credentials encrypted; only redacted settings leave the server. */
class EmailSettingsStore(private val db: Db, private val config: AppConfig) {
    private val key = SecretKeySpec(MessageDigest.getInstance("SHA-256").digest(
        ("xiaban-settings:" + (config.settingsEncryptionKey ?: config.jwtSecret)).toByteArray()), "AES")
    private val random = SecureRandom()

    suspend fun current(): EmailSettings {
        val saved = db.read { c -> c.queryOne("select encrypted_value from runtime_settings where key = 'email'") { it.getString(1) } }
        if (saved != null) return AppJson.decodeFromString(EmailSettings.serializer(), decrypt(saved))
        return config.smtp?.let { EmailSettings(host = it.host, port = it.port, user = it.user, password = it.password,
            from = it.from, fromName = it.fromName, ssl = it.ssl) } ?: EmailSettings()
    }

    suspend fun save(request: EmailSettings, actor: String? = null): EmailSettings {
        val previous = current()
        val next = request.copy(host = request.host.trim(), user = request.user.trim(), from = request.from.trim(),
            fromName = request.fromName.trim(), password = request.password.ifEmpty { previous.password }, passwordConfigured = false)
        if (next.port !in 1..65535) badRequest("SMTP 端口需在 1–65535 之间")
        if (next.host.isNotEmpty() && !Regex("^[A-Za-z0-9.-]{1,253}$").matches(next.host)) badRequest("SMTP 主机请填写域名或 IPv4 地址")
        if (next.from.isNotEmpty()) xyz.xiaban.server.mail.normalizeEmail(next.from)
        if (next.fromName.length > 80 || next.user.length > 254 || next.password.length > 4096) badRequest("配置内容过长")
        if (next.enabled && !next.configured()) badRequest("启用邮箱登录前，请填完整 SMTP 主机、用户名、授权码和发件邮箱")
        val value = encrypt(AppJson.encodeToString(EmailSettings.serializer(), next))
        db.tx { c ->
            c.execute("insert into runtime_settings(key, encrypted_value) values ('email', ?) on conflict(key) do update set encrypted_value = excluded.encrypted_value, updated_at = now()", value)
            if (actor != null) c.execute("insert into admin_audit_log(actor,action,summary) values (?,?,?)", actor, "email_settings", if (next.enabled) "启用邮件服务并更新 SMTP 配置" else "关闭邮件服务并更新 SMTP 配置")
        }
        return next.redacted()
    }

    private fun encrypt(value: String): String {
        val iv = ByteArray(12).also(random::nextBytes)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, key, GCMParameterSpec(128, iv))
        cipher.updateAAD("email".toByteArray())
        return Base64.getEncoder().encodeToString(iv + cipher.doFinal(value.toByteArray(Charsets.UTF_8)))
    }

    private fun decrypt(value: String): String {
        val bytes = Base64.getDecoder().decode(value)
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(128, bytes.copyOfRange(0, 12)))
        cipher.updateAAD("email".toByteArray())
        return String(cipher.doFinal(bytes.copyOfRange(12, bytes.size)), Charsets.UTF_8)
    }
}
