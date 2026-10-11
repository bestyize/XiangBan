package xyz.xiaban.wallpaper.data

import org.json.JSONException
import org.json.JSONObject
import java.io.File
import java.io.IOException
import java.net.HttpURLConnection
import java.net.SocketTimeoutException
import java.net.URL
import java.net.URLEncoder
import java.net.UnknownHostException

/** [username] is unique and unchangeable; it is also the name shown everywhere. */
data class AccountUser(val uid: String, val username: String, val avatarUrl: String?)

/** [expiresAt] is when the access token stops working, in epoch seconds. */
data class AccountSession(val accessToken: String, val refreshToken: String, val expiresAt: Long, val user: AccountUser)

data class LoginOutcome(val session: AccountSession, val isNewUser: Boolean)

/** A code login either signs an existing account in, or says the account still has to be created with a username and password. */
sealed interface CodeLoginResult {
    data class SignedIn(val outcome: LoginOutcome) : CodeLoginResult
    data class RegistrationRequired(val token: String) : CodeLoginResult
}

data class UsernameCheck(val available: Boolean, val message: String?)

/**
 * [sessionExpired] is set only when the account centre said the saved login is no longer valid, never for a network failure.
 * [code] is the account centre's machine readable error (username_taken, registration_expired, identity_exists...).
 */
class AccountException(message: String, val sessionExpired: Boolean = false, val code: String? = null) : Exception(message)

object AccountRules {
    const val CLIENT_ID = "xiaoban-mobile-wallpaper"
    const val CODE_LENGTH = 6
    const val DEFAULT_COOLDOWN_SECONDS = 60
    const val USERNAME_MIN = 4
    const val USERNAME_MAX = 20
    const val PASSWORD_MIN = 8
    const val PASSWORD_MAX = 128

    /** Refresh a little before the access token runs out, so a request never leaves with a stale one. */
    const val REFRESH_MARGIN_SECONDS = 120L

    private val EMAIL = Regex("^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$")

    fun isEmail(value: String) = EMAIL.matches(value.trim())

    /** The same rules as the account centre, only to save requests while typing; the server has the last word. */
    fun usernameError(raw: String): String? {
        val value = raw.trim()
        if (value.isEmpty()) return null
        if (!value[0].let { it in 'A'..'Z' || it in 'a'..'z' }) return "用户名需以字母开头"
        if (!value.all { it in 'A'..'Z' || it in 'a'..'z' || it in '0'..'9' || it == '_' }) return "只能包含字母、数字和下划线"
        if (value.length !in USERNAME_MIN..USERNAME_MAX) return "需为 $USERNAME_MIN-$USERNAME_MAX 个字符"
        return null
    }

    fun passwordError(raw: String, username: String = ""): String? = when {
        raw.isEmpty() -> null
        raw.length < PASSWORD_MIN -> "密码至少 $PASSWORD_MIN 位"
        raw.length > PASSWORD_MAX -> "密码最多 $PASSWORD_MAX 位"
        username.isNotBlank() && raw.equals(username.trim(), ignoreCase = true) -> "密码不能和用户名相同"
        else -> null
    }

    fun needsRefresh(session: AccountSession, nowSeconds: Long) = session.expiresAt - nowSeconds < REFRESH_MARGIN_SECONDS
}

/** Reads the account centre's JSON answers. Pure, so it can be tested without a network. */
object AccountParser {
    fun cooldownSeconds(body: String): Int =
        obj(body).optInt("cooldown_seconds", AccountRules.DEFAULT_COOLDOWN_SECONDS).takeIf { it > 0 } ?: AccountRules.DEFAULT_COOLDOWN_SECONDS

    fun login(body: String, nowSeconds: Long): LoginOutcome = login(obj(body), nowSeconds)

    private fun login(o: JSONObject, nowSeconds: Long): LoginOutcome {
        val user = o.optJSONObject("user") ?: throw incomplete()
        val session = AccountSession(
            accessToken = token(o, "access_token"),
            refreshToken = token(o, "refresh_token"),
            expiresAt = nowSeconds + o.optLong("expires_in", DEFAULT_TTL_SECONDS),
            user = AccountUser(
                uid = user.optString("uid").takeIf { it.isNotBlank() } ?: throw incomplete(),
                username = user.optString("username").takeIf { it.isNotBlank() } ?: throw incomplete(),
                avatarUrl = user.optString("avatar_url").takeIf { it.isNotBlank() },
            ),
        )
        return LoginOutcome(session, o.optBoolean("is_new_user", false))
    }

    fun codeLogin(body: String, nowSeconds: Long): CodeLoginResult {
        val o = obj(body)
        if (o.optBoolean("registration_required", false)) {
            val token = o.optString("registration_token").takeIf { it.isNotBlank() } ?: throw incomplete()
            return CodeLoginResult.RegistrationRequired(token)
        }
        return CodeLoginResult.SignedIn(login(o, nowSeconds))
    }

    fun usernameCheck(body: String): UsernameCheck {
        val o = obj(body)
        return UsernameCheck(o.optBoolean("available", false), o.optString("message").takeIf { it.isNotBlank() && !o.isNull("message") })
    }

    /** The account centre's machine readable error code, or null. */
    fun errorCode(body: String): String? = try {
        JSONObject(body).optString("code").takeIf { it.isNotBlank() }
    } catch (_: JSONException) {
        null
    }

    /** A refresh answers with new tokens only, so the profile is carried over from [previous]. */
    fun refreshed(body: String, previous: AccountSession, nowSeconds: Long): AccountSession {
        val o = obj(body)
        return previous.copy(
            accessToken = token(o, "access_token"),
            refreshToken = token(o, "refresh_token"),
            expiresAt = nowSeconds + o.optLong("expires_in", DEFAULT_TTL_SECONDS),
        )
    }

    /** The account centre's own message when it sent one, otherwise a plain sentence for the status. */
    fun failure(status: Int, body: String): String {
        val message = try {
            JSONObject(body).optString("message")
        } catch (_: JSONException) {
            ""
        }
        if (message.isNotBlank()) return message
        return when (status) {
            400 -> "请求有误，请检查输入"
            401 -> "账号或验证码不正确，或已过期"
            429 -> "请求过于频繁，请稍后再试"
            in 500..599 -> "账号服务暂时不可用，请稍后重试"
            else -> "账号服务返回异常（$status）"
        }
    }

    private fun token(o: JSONObject, key: String) = o.optString(key).takeIf { it.isNotBlank() } ?: throw incomplete()

    private fun incomplete() = AccountException("账号服务返回的数据不完整")

    private fun obj(body: String) = try {
        JSONObject(body)
    } catch (_: JSONException) {
        throw AccountException("账号服务返回了无法识别的内容")
    }

    private const val DEFAULT_TTL_SECONDS = 1800L
}

/** Talks to the unified account centre, the same one the wallpaper website and the desktop app sign in with. */
class AccountClient(private val origin: String = DEFAULT_ORIGIN) {
    /** Returns how many seconds the user must wait before asking for another code. [resetPassword] asks for a password-reset code instead. */
    fun sendCode(identifier: String, resetPassword: Boolean = false): Int {
        val purpose = if (resetPassword) "reset_password" else "login"
        val body = request("POST", "/api/auth/code/send", JSONObject().put("identifier", identifier.trim()).put("purpose", purpose))
        return AccountParser.cooldownSeconds(body)
    }

    fun loginWithCode(identifier: String, code: String, nowSeconds: Long = nowSeconds()): CodeLoginResult {
        val body = request(
            "POST",
            "/api/auth/code/login",
            JSONObject().put("identifier", identifier.trim()).put("code", code.trim())
                .put("client_id", AccountRules.CLIENT_ID).put("device_name", DEVICE_NAME),
        )
        return AccountParser.codeLogin(body, nowSeconds)
    }

    fun loginWithPassword(identifier: String, password: String, nowSeconds: Long = nowSeconds()): LoginOutcome {
        val body = request(
            "POST",
            "/api/auth/password/login",
            JSONObject().put("identifier", identifier.trim()).put("password", password)
                .put("client_id", AccountRules.CLIENT_ID).put("device_name", DEVICE_NAME),
        )
        return AccountParser.login(body, nowSeconds)
    }

    fun register(registrationToken: String, username: String, password: String, nowSeconds: Long = nowSeconds()): LoginOutcome {
        val body = request(
            "POST",
            "/api/auth/register",
            JSONObject().put("registration_token", registrationToken).put("username", username.trim()).put("password", password)
                .put("client_id", AccountRules.CLIENT_ID).put("device_name", DEVICE_NAME),
        )
        return AccountParser.login(body, nowSeconds)
    }

    fun resetPassword(identifier: String, code: String, newPassword: String) {
        request(
            "POST",
            "/api/auth/password/reset",
            JSONObject().put("identifier", identifier.trim()).put("code", code.trim()).put("new_password", newPassword),
        )
    }

    fun checkUsername(username: String): UsernameCheck {
        val query = URLEncoder.encode(username.trim(), "UTF-8")
        return AccountParser.usernameCheck(request("GET", "/api/auth/username/available?username=$query", null))
    }

    /** The refresh token is single use: the answer carries a new one that replaces it. */
    fun refresh(session: AccountSession, nowSeconds: Long = nowSeconds()): AccountSession {
        val body = request(
            "POST",
            "/api/auth/refresh",
            JSONObject().put("refresh_token", session.refreshToken).put("client_id", AccountRules.CLIENT_ID),
            rejectionMeansExpired = true,
        )
        return AccountParser.refreshed(body, session, nowSeconds)
    }

    fun logout(refreshToken: String) {
        request("POST", "/api/auth/logout", JSONObject().put("refresh_token", refreshToken))
    }

    private fun request(method: String, path: String, payload: JSONObject?, rejectionMeansExpired: Boolean = false): String {
        try {
            val conn = URL(origin + path).openConnection() as HttpURLConnection
            try {
                conn.requestMethod = method
                conn.instanceFollowRedirects = false
                conn.connectTimeout = CONNECT_MS
                conn.readTimeout = READ_MS
                conn.setRequestProperty("Accept", "application/json")
                conn.setRequestProperty("User-Agent", USER_AGENT)
                if (payload != null) {
                    conn.doOutput = true
                    conn.setRequestProperty("Content-Type", "application/json")
                    conn.outputStream.use { it.write(payload.toString().toByteArray()) }
                }
                val status = conn.responseCode
                val stream = if (status in 200..299) conn.inputStream else conn.errorStream
                val text = stream?.bufferedReader()?.use { it.readText() }.orEmpty()
                if (status in 200..299) return text
                val expired = rejectionMeansExpired && (status == 400 || status == 401 || status == 403)
                throw AccountException(
                    if (expired) SESSION_EXPIRED else AccountParser.failure(status, text),
                    sessionExpired = expired,
                    code = AccountParser.errorCode(text),
                )
            } finally {
                conn.disconnect()
            }
        } catch (e: AccountException) {
            throw e
        } catch (e: SocketTimeoutException) {
            throw AccountException("账号服务响应超时，请稍后重试")
        } catch (e: UnknownHostException) {
            throw AccountException("无法连接账号服务，请检查网络")
        } catch (e: IOException) {
            throw AccountException("网络出错：${e.message ?: "连接中断"}")
        }
    }

    companion object {
        const val DEFAULT_ORIGIN = "https://account.xiangban.xyz"
        const val SESSION_EXPIRED = "登录已失效，请重新登录"
        private const val DEVICE_NAME = "小伴壁纸 Android"
        private const val CONNECT_MS = 15_000
        private const val READ_MS = 20_000
        private const val USER_AGENT = "XiaoBanMobile/0.1"

        fun nowSeconds() = System.currentTimeMillis() / 1000
    }
}

/** The saved login, one small file replaced atomically. It lives in this app's private storage and is excluded from backups. */
class AccountStore(private val file: File) {
    @Synchronized
    fun load(): AccountSession? = try {
        val o = JSONObject(file.readText())
        val user = o.getJSONObject("user")
        AccountSession(
            accessToken = o.getString("access"),
            refreshToken = o.getString("refresh"),
            expiresAt = o.getLong("exp"),
            user = AccountUser(user.getString("uid"), user.getString("name"), user.optString("avatar").takeIf { it.isNotBlank() }),
        )
    } catch (_: Exception) {
        null
    }

    @Synchronized
    fun save(session: AccountSession) {
        val json = JSONObject()
            .put("access", session.accessToken).put("refresh", session.refreshToken).put("exp", session.expiresAt)
            .put(
                "user",
                JSONObject().put("uid", session.user.uid).put("name", session.user.username)
                    .put("avatar", session.user.avatarUrl ?: ""),
            )
        file.parentFile?.mkdirs()
        val staging = File(file.parentFile, file.name + ".new")
        staging.writeText(json.toString())
        if (!staging.renameTo(file)) {
            file.delete()
            staging.renameTo(file)
        }
    }

    @Synchronized
    fun clear() {
        file.delete()
        File(file.parentFile, file.name + ".new").delete()
    }
}
