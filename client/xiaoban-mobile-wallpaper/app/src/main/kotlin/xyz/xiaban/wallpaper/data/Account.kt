package xyz.xiaban.wallpaper.data

import org.json.JSONException
import org.json.JSONObject
import java.io.File
import java.io.IOException
import java.net.HttpURLConnection
import java.net.SocketTimeoutException
import java.net.URL
import java.net.UnknownHostException

data class AccountUser(val uid: String, val nickname: String, val email: String, val avatarUrl: String?)

/** [expiresAt] is when the access token stops working, in epoch seconds. */
data class AccountSession(val accessToken: String, val refreshToken: String, val expiresAt: Long, val user: AccountUser)

data class LoginOutcome(val session: AccountSession, val isNewUser: Boolean)

/** [sessionExpired] is set only when the account centre said the saved login is no longer valid, never for a network failure. */
class AccountException(message: String, val sessionExpired: Boolean = false) : Exception(message)

object AccountRules {
    const val CLIENT_ID = "xiaoban-mobile-wallpaper"
    const val CODE_LENGTH = 6
    const val DEFAULT_COOLDOWN_SECONDS = 60
    const val FALLBACK_NICKNAME = "小伴伙伴"

    /** Refresh a little before the access token runs out, so a request never leaves with a stale one. */
    const val REFRESH_MARGIN_SECONDS = 120L

    private val EMAIL = Regex("^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$")

    fun isEmail(value: String) = EMAIL.matches(value.trim())

    fun needsRefresh(session: AccountSession, nowSeconds: Long) = session.expiresAt - nowSeconds < REFRESH_MARGIN_SECONDS
}

/** Reads the account centre's JSON answers. Pure, so it can be tested without a network. */
object AccountParser {
    fun cooldownSeconds(body: String): Int =
        obj(body).optInt("cooldown_seconds", AccountRules.DEFAULT_COOLDOWN_SECONDS).takeIf { it > 0 } ?: AccountRules.DEFAULT_COOLDOWN_SECONDS

    fun login(body: String, email: String, nowSeconds: Long): LoginOutcome {
        val o = obj(body)
        val user = o.optJSONObject("user") ?: throw AccountException("账号服务返回的数据不完整")
        val uid = user.optString("uid").takeIf { it.isNotBlank() } ?: throw AccountException("账号服务返回的数据不完整")
        val session = AccountSession(
            accessToken = token(o, "access_token"),
            refreshToken = token(o, "refresh_token"),
            expiresAt = nowSeconds + o.optLong("expires_in", DEFAULT_TTL_SECONDS),
            user = AccountUser(
                uid = uid,
                nickname = user.optString("nickname").takeIf { it.isNotBlank() } ?: AccountRules.FALLBACK_NICKNAME,
                email = email.trim(),
                avatarUrl = user.optString("avatar_url").takeIf { it.isNotBlank() },
            ),
        )
        return LoginOutcome(session, o.optBoolean("is_new_user", false))
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
            400 -> "请求有误，请检查邮箱格式"
            401 -> "验证码错误或已过期"
            429 -> "请求过于频繁，请稍后再试"
            in 500..599 -> "账号服务暂时不可用，请稍后重试"
            else -> "账号服务返回异常（$status）"
        }
    }

    private fun token(o: JSONObject, key: String) = o.optString(key).takeIf { it.isNotBlank() } ?: throw AccountException("账号服务返回的数据不完整")

    private fun obj(body: String) = try {
        JSONObject(body)
    } catch (_: JSONException) {
        throw AccountException("账号服务返回了无法识别的内容")
    }

    private const val DEFAULT_TTL_SECONDS = 1800L
}

/** Talks to the unified account centre, the same one the wallpaper website and the desktop app sign in with. */
class AccountClient(private val origin: String = DEFAULT_ORIGIN) {
    /** Returns how many seconds the user must wait before asking for another code. */
    fun sendCode(email: String): Int {
        val body = post("/api/auth/code/send", JSONObject().put("identifier", email.trim()).put("purpose", "login"))
        return AccountParser.cooldownSeconds(body)
    }

    fun loginWithCode(email: String, code: String, nowSeconds: Long = nowSeconds()): LoginOutcome {
        val body = post(
            "/api/auth/code/login",
            JSONObject().put("identifier", email.trim()).put("code", code.trim())
                .put("client_id", AccountRules.CLIENT_ID).put("device_name", DEVICE_NAME),
        )
        return AccountParser.login(body, email, nowSeconds)
    }

    /** The refresh token is single use: the answer carries a new one that replaces it. */
    fun refresh(session: AccountSession, nowSeconds: Long = nowSeconds()): AccountSession {
        val body = post(
            "/api/auth/refresh",
            JSONObject().put("refresh_token", session.refreshToken).put("client_id", AccountRules.CLIENT_ID),
            rejectionMeansExpired = true,
        )
        return AccountParser.refreshed(body, session, nowSeconds)
    }

    fun logout(refreshToken: String) {
        post("/api/auth/logout", JSONObject().put("refresh_token", refreshToken))
    }

    private fun post(path: String, payload: JSONObject, rejectionMeansExpired: Boolean = false): String {
        try {
            val conn = URL(origin + path).openConnection() as HttpURLConnection
            try {
                conn.requestMethod = "POST"
                conn.instanceFollowRedirects = false
                conn.connectTimeout = CONNECT_MS
                conn.readTimeout = READ_MS
                conn.doOutput = true
                conn.setRequestProperty("Content-Type", "application/json")
                conn.setRequestProperty("Accept", "application/json")
                conn.setRequestProperty("User-Agent", USER_AGENT)
                conn.outputStream.use { it.write(payload.toString().toByteArray()) }
                val status = conn.responseCode
                val stream = if (status in 200..299) conn.inputStream else conn.errorStream
                val text = stream?.bufferedReader()?.use { it.readText() }.orEmpty()
                if (status in 200..299) return text
                val expired = rejectionMeansExpired && (status == 400 || status == 401 || status == 403)
                throw AccountException(if (expired) SESSION_EXPIRED else AccountParser.failure(status, text), sessionExpired = expired)
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
            user = AccountUser(user.getString("uid"), user.getString("nick"), user.getString("email"), user.optString("avatar").takeIf { it.isNotBlank() }),
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
                JSONObject().put("uid", session.user.uid).put("nick", session.user.nickname).put("email", session.user.email)
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
