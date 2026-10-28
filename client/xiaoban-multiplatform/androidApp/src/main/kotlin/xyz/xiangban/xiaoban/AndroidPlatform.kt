package xyz.xiangban.xiaoban

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions
import xyz.xiangban.xiaoban.common.community.TokenStorage
import xyz.xiangban.xiaoban.common.community.Tokens

/** 令牌存在应用私有的 SharedPreferences 里（其他应用读不到）。后续可换成 Keystore 加密存储。 */
class SharedPrefsTokenStorage(context: Context) : TokenStorage {
    private val prefs = context.applicationContext.getSharedPreferences("community_session", Context.MODE_PRIVATE)

    override fun load(): Tokens? {
        val access = prefs.getString("access", null) ?: return null
        val refresh = prefs.getString("refresh", null) ?: return null
        return Tokens(access, refresh, prefs.getLong("expiresAt", 0))
    }

    override fun save(tokens: Tokens?) {
        prefs.edit().apply {
            if (tokens == null) clear() else {
                putString("access", tokens.accessToken)
                putString("refresh", tokens.refreshToken)
                putLong("expiresAt", tokens.expiresAtMillis)
            }
        }.apply()
    }
}

/** 复制、系统分享、用浏览器打开链接。 */
class AndroidPlatformActions(private val context: Context) : PlatformActions {
    override fun copy(text: String) {
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        clipboard.setPrimaryClip(ClipData.newPlainText("链接", text))
    }

    override fun share(title: String, url: String) {
        val send = Intent(Intent.ACTION_SEND).apply {
            type = "text/plain"
            putExtra(Intent.EXTRA_SUBJECT, title)
            putExtra(Intent.EXTRA_TEXT, "$title $url")
        }
        context.startActivity(Intent.createChooser(send, title).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
    }

    override fun openUrl(url: String) {
        val view = Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        runCatching { context.startActivity(view) }
    }
}
