package xyz.xiaban.server.mail

import jakarta.mail.Authenticator
import jakarta.mail.Message
import jakarta.mail.PasswordAuthentication
import jakarta.mail.Session
import jakarta.mail.Transport
import jakarta.mail.internet.MimeBodyPart
import jakarta.mail.internet.MimeMultipart
import jakarta.mail.internet.InternetAddress
import jakarta.mail.internet.MimeMessage
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.slf4j.LoggerFactory
import java.util.Properties

data class Mail(val to: String, val subject: String, val text: String, val html: String? = null)

interface Mailer {
    suspend fun send(mail: Mail)
}

data class SmtpConfig(
    val host: String,
    val port: Int,
    val user: String,
    val password: String,
    val from: String,
    val fromName: String,
    /** Implicit TLS (usually port 465); otherwise STARTTLS. */
    val ssl: Boolean,
)

class SmtpMailer(private val config: SmtpConfig) : Mailer {
    private val session: Session = Session.getInstance(
        Properties().apply {
            put("mail.smtp.host", config.host)
            put("mail.smtp.port", config.port.toString())
            put("mail.smtp.auth", "true")
            put(if (config.ssl) "mail.smtp.ssl.enable" else "mail.smtp.starttls.enable", "true")
            if (!config.ssl) put("mail.smtp.starttls.required", "true")
            put("mail.smtp.ssl.checkserveridentity", "true")
            put("mail.smtp.connectiontimeout", "5000")
            put("mail.smtp.timeout", "10000")
            put("mail.smtp.writetimeout", "10000")
        },
        object : Authenticator() {
            override fun getPasswordAuthentication() = PasswordAuthentication(config.user, config.password)
        },
    )

    override suspend fun send(mail: Mail) = withContext(Dispatchers.IO) {
        val message = MimeMessage(session).apply {
            setFrom(InternetAddress(config.from, config.fromName, "UTF-8"))
            setRecipients(Message.RecipientType.TO, InternetAddress.parse(mail.to))
            setSubject(mail.subject, "UTF-8")
            if (mail.html == null) setText(mail.text, "UTF-8") else {
                val alternative = MimeMultipart("alternative")
                alternative.addBodyPart(MimeBodyPart().apply { setText(mail.text, "UTF-8") })
                alternative.addBodyPart(MimeBodyPart().apply { setText(mail.html, "UTF-8", "html") })
                setContent(alternative)
            }
        }
        Transport.send(message)
    }
}

/** Development fallback when SMTP is not configured: the mail only goes to the log. */
class LogMailer : Mailer {
    val sent = mutableListOf<Mail>()

    override suspend fun send(mail: Mail) {
        synchronized(sent) { sent += mail }
        log.warn("SMTP not configured, mail to {} logged instead:\n{}\n{}", maskEmail(mail.to), mail.subject, mail.text)
    }

    private companion object {
        val log = LoggerFactory.getLogger(LogMailer::class.java)
    }
}

fun maskEmail(email: String): String {
    val at = email.indexOf('@')
    if (at <= 0) return "***"
    val name = email.substring(0, at)
    val shown = if (name.length <= 2) name.take(1) else name.take(2)
    return "$shown***${email.substring(at)}"
}
