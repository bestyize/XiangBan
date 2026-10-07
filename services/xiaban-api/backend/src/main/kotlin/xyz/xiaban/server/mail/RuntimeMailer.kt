package xyz.xiaban.server.mail

import io.ktor.http.HttpStatusCode
import xyz.xiaban.server.admin.EmailSettingsStore
import xyz.xiaban.server.http.ApiException

/** Read the saved configuration for each delivery so changes also reach background workers. */
class RuntimeMailer(private val settings: EmailSettingsStore) : Mailer {
    suspend fun available(): Boolean = settings.current().let { it.enabled && it.configured() }
    override suspend fun send(mail: Mail) {
        val current = settings.current()
        if (!current.enabled || !current.configured()) throw ApiException(HttpStatusCode.ServiceUnavailable,
            "mail_unavailable", "邮箱服务暂未开通，请稍后再试")
        SmtpMailer(current.smtp()).send(mail)
    }
}
