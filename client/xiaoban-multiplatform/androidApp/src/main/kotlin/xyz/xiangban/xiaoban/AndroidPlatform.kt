package xyz.xiangban.xiaoban

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.speech.tts.TextToSpeech
import android.speech.tts.UtteranceProgressListener
import java.util.Locale
import android.util.Base64
import org.json.JSONObject
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions
import xyz.xiangban.xiaoban.common.community.TokenStorage
import xyz.xiangban.xiaoban.common.community.Tokens

/** 令牌存在应用私有的 SharedPreferences 里（其他应用读不到）。后续可换成 Keystore 加密存储。 */
class SharedPrefsTokenStorage(context: Context) : TokenStorage {
    private val prefs = context.applicationContext.getSharedPreferences("community_session", Context.MODE_PRIVATE)

    override fun load(): Tokens? {
        val access = prefs.getString("access", null) ?: return null
        val refresh = prefs.getString("refresh", null) ?: return null
        // 旧版没有保存 uid；从已存储的账号令牌提取 sub，只用于隔离本地缓存，不用于鉴权。
        val accountId = prefs.getString("accountId", null) ?: runCatching {
            val payload = access.split('.').getOrNull(1) ?: return@runCatching null
            JSONObject(String(Base64.decode(payload, Base64.URL_SAFE or Base64.NO_WRAP), Charsets.UTF_8)).optString("sub").takeIf { it.isNotBlank() }
        }.getOrNull()
        return Tokens(access, refresh, prefs.getLong("expiresAt", 0), accountId)
    }

    override fun save(tokens: Tokens?) {
        prefs.edit().apply {
            if (tokens == null) clear() else {
                putString("access", tokens.accessToken)
                putString("refresh", tokens.refreshToken)
                putLong("expiresAt", tokens.expiresAtMillis)
                putString("accountId", tokens.accountId)
            }
        }.apply()
    }
}

/** 复制、系统分享、用浏览器打开链接。 */
class AndroidPlatformActions(private val context: Context, requestMic: suspend () -> Boolean = { false }) : PlatformActions {
    private val voice = AndroidVoice(context, requestMic)

    override suspend fun startRecording(): Boolean = voice.start()
    override fun stopRecording(): ByteArray? = voice.stop()
    override fun playAudio(bytes: ByteArray, mime: String, onDone: () -> Unit): Boolean = voice.play(bytes, onDone)
    override fun stopAudio() = voice.stopPlayback()
    override suspend fun streamRecording(onFrame: (ByteArray) -> Unit): (() -> Unit)? = voice.stream(onFrame)
    override fun listenOnce(onPartial: (String) -> Unit, onResult: (String?, String?) -> Unit): (() -> Unit)? = voice.listenOnce(onPartial, onResult)

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

    private var tts: TextToSpeech? = null
    private var ttsReady = false
    private var ttsFailed = false
    private var ttsWaiting: Pair<String, String>? = null
    private val ttsDone = java.util.concurrent.ConcurrentHashMap<String, (Boolean) -> Unit>()

    private fun speakWith(text: String, utteranceId: String): Boolean {
        if (ttsFailed) return false
        val engine = tts ?: TextToSpeech(context.applicationContext) { status ->
            ttsReady = status == TextToSpeech.SUCCESS
            ttsFailed = !ttsReady
            val waiting = ttsWaiting
            ttsWaiting = null
            if (ttsReady) {
                tts?.language = Locale.CHINA
                tts?.setOnUtteranceProgressListener(object : UtteranceProgressListener() {
                    override fun onStart(utteranceId: String?) {}
                    override fun onDone(utteranceId: String?) { utteranceId?.let { ttsDone.remove(it)?.invoke(true) } }
                    @Deprecated("Deprecated in Java")
                    override fun onError(utteranceId: String?) { utteranceId?.let { ttsDone.remove(it)?.invoke(false) } }
                })
                waiting?.let { (line, id) -> tts?.speak(line, TextToSpeech.QUEUE_FLUSH, null, id) }
            } else {
                waiting?.let { (_, id) -> ttsDone.remove(id)?.invoke(false) }
            }
        }.also { tts = it }
        if (!ttsReady) { ttsWaiting = text to utteranceId; return true }
        return engine.speak(text, TextToSpeech.QUEUE_FLUSH, null, utteranceId) == TextToSpeech.SUCCESS
    }

    override fun speak(text: String): Boolean = speakWith(text.take(3900), "xiaoban")

    override fun speakText(text: String, onDone: (Boolean) -> Unit): Boolean {
        val id = "call-${System.nanoTime()}"
        ttsDone[id] = onDone
        val started = speakWith(text.take(3900), id)
        if (!started) ttsDone.remove(id)
        return started
    }

    override fun stopSpeaking() {
        ttsWaiting = null
        ttsDone.clear()
        tts?.stop()
    }

    override fun openUrl(url: String) {
        val view = Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        runCatching { context.startActivity(view) }
    }
}
