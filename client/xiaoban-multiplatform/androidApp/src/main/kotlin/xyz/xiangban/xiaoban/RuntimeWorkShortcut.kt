package xyz.xiangban.xiaoban

import android.app.Activity
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.ShortcutInfo
import android.content.pm.ShortcutManager
import android.graphics.Bitmap
import android.graphics.drawable.Icon
import android.net.Uri
import android.widget.Toast
import androidx.annotation.RequiresApi
import java.util.UUID

/** Identifies a work without carrying runtime permissions, download URLs, or a file path. */
internal data class RuntimeWorkShortcut private constructor(val source: String, val id: String) {
    enum class PinResult { Requested, AlreadyPinned, Rejected }

    val communityId: Long? get() = if (source == "community") id.toLong() else null
    private val key: String get() = "work:$source:$id"

    fun matchesLocalSource(): Boolean =
        (bundledGameRoot(id) != null) == (source == "builtin")

    @RequiresApi(26)
    fun isPinned(activity: Activity): Boolean =
        activity.getSystemService(ShortcutManager::class.java).pinnedShortcuts.any { it.id == key && it.isEnabled }

    @RequiresApi(26)
    fun requestPin(activity: Activity, title: String, icon: Bitmap?): PinResult {
        val manager = activity.getSystemService(ShortcutManager::class.java)
        if (isPinned(activity)) return PinResult.AlreadyPinned
        if (!manager.isRequestPinShortcutSupported) return PinResult.Rejected
        val launch = Intent(activity, MainActivity::class.java)
            .setAction(ACTION_OPEN)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)
            .putExtra(EXTRA_KEY, key)
            .putExtra(EXTRA_TOKEN, UUID.randomUUID().toString())
        val shortcut = ShortcutInfo.Builder(activity, key)
            .setShortLabel(title)
            .setIcon(icon?.let {
                val scale = minOf(1f, manager.iconMaxWidth.toFloat() / it.width, manager.iconMaxHeight.toFloat() / it.height)
                val bitmap = if (scale < 1f) Bitmap.createScaledBitmap(it, (it.width * scale).toInt().coerceAtLeast(1),
                    (it.height * scale).toInt().coerceAtLeast(1), true) else it
                Icon.createWithBitmap(bitmap)
            } ?: Icon.createWithResource(activity, R.mipmap.ic_launcher))
            .setIntent(launch)
            .build()
        val result = Intent(activity, RuntimeWorkShortcutResult::class.java)
            .setData(Uri.Builder().scheme("xiaoban").authority("pinned-work").appendPath(key).build())
        val callback = PendingIntent.getBroadcast(activity, 0, result, PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_ONE_SHOT)
        if (manager.requestPinShortcut(shortcut, callback.intentSender)) return PinResult.Requested
        callback.cancel()
        return PinResult.Rejected
    }

    companion object {
        const val ACTION_OPEN = "xyz.xiangban.xiaoban.action.OPEN_PINNED_WORK"
        private const val EXTRA_KEY = "runtime.v2.shortcut.key"
        private const val EXTRA_TOKEN = "runtime.v2.shortcut.token"

        fun forWork(gameId: String, workspace: Boolean, packId: Long): RuntimeWorkShortcut? = when {
            packId > 0 -> RuntimeWorkShortcut("community", packId.toString())
            MOBILE_GAME_ID.matches(gameId) ->
                RuntimeWorkShortcut(if (workspace) "local" else "builtin", gameId)
            else -> null
        }

        @RequiresApi(26)
        fun fromPinnedIntent(activity: Activity, intent: Intent): RuntimeWorkShortcut? {
            if (intent.action != ACTION_OPEN) return null
            val key = intent.getStringExtra(EXTRA_KEY) ?: return null
            val token = intent.getStringExtra(EXTRA_TOKEN) ?: return null
            val parts = key.split(':', limit = 4)
            if (parts.size != 3 || parts[0] != "work") return null
            val (_, source, id) = parts
            val target = when (source) {
                "community" -> forWork("", false, id.toLongOrNull() ?: return null)
                "builtin", "local" -> forWork(id, source == "local", 0)
                else -> null
            } ?: return null
            // MainActivity is exported. Accept only the identity of a shortcut actually pinned for this app.
            val manager = activity.getSystemService(ShortcutManager::class.java)
            return target.takeIf {
                it.key == key && manager.pinnedShortcuts.any { shortcut ->
                    shortcut.id == key && shortcut.isEnabled && shortcut.intent?.getStringExtra(EXTRA_TOKEN) == token
                }
            }
        }
    }
}

class RuntimeWorkShortcutResult : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        Toast.makeText(context, "已添加到桌面", Toast.LENGTH_SHORT).show()
    }
}
