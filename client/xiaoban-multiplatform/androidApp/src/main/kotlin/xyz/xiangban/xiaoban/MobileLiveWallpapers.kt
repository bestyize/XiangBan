package xyz.xiangban.xiaoban

import android.app.WallpaperInfo
import android.app.WallpaperManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import android.widget.Toast
import java.io.File
import java.io.IOException
import org.json.JSONObject

internal object MobileLiveWallpapers {
    const val SCENE_CHANGED = "xyz.xiangban.xiaoban.action.WALLPAPER_SCENE_CHANGED"

    // Keep the existing Android component names so an upgrade does not clear the user's wallpaper.
    enum class Slot(val defaultScene: String, val service: Class<*>, val label: Int) {
        PRIMARY("aquarium", AquariumWallpaperService::class.java, R.string.live_wallpaper_aquarium),
        SECONDARY("galaxy", GalaxyWallpaperService::class.java, R.string.live_wallpaper_galaxy),
    }

    fun isScene(scene: String) = Slot.entries.any { it.defaultScene == scene }

    data class Selection(private val homeSlot: Slot?, private val lockSlot: Slot?, val home: String?, val lock: String?) {
        fun hasNewWallpaper(previous: Selection) =
            (home != null && home != previous.home) || (lock != null && lock != previous.lock)

        private fun switchTarget(mode: String): String? = when {
            homeSlot != null && homeSlot == lockSlot -> if (mode == "wallpaper") "both" else null
            mode == "home" -> if (homeSlot != null) "home" else null
            mode == "lock" -> if (lockSlot != null) "lock" else null
            homeSlot != null -> "home"
            lockSlot != null -> "lock"
            else -> null
        }

        fun switchSlot(mode: String): Slot? = when (switchTarget(mode)) {
            "home", "both" -> homeSlot
            "lock" -> lockSlot
            else -> null
        }

        fun unusedSlot(scene: String): Slot? {
            val available = Slot.entries.filter { it != homeSlot && it != lockSlot }
            return available.firstOrNull { it.defaultScene == scene } ?: available.firstOrNull()
        }

        fun toJson() = JSONObject()
            .put("home", home ?: JSONObject.NULL)
            .put("lock", lock ?: JSONObject.NULL)
            .put("switchTargets", JSONObject().apply {
                for (mode in listOf("wallpaper", "home", "lock")) put(mode, switchTarget(mode) ?: JSONObject.NULL)
            })
    }

    fun supported(context: Context): Boolean = runCatching {
        val manager = WallpaperManager.getInstance(context)
        context.packageManager.hasSystemFeature(PackageManager.FEATURE_LIVE_WALLPAPER) &&
            manager.isWallpaperSupported && manager.isSetWallpaperAllowed
    }.getOrDefault(false)

    fun selection(context: Context): Selection {
        val manager = WallpaperManager.getInstance(context)
        fun slot(info: WallpaperInfo?): Slot? {
            if (info == null || info.packageName != context.packageName) return null
            return Slot.entries.firstOrNull { it.service.name == info.serviceName }
        }

        val home = runCatching { slot(manager.wallpaperInfo) }.getOrNull()
        val lock = runCatching {
            val info = if (Build.VERSION.SDK_INT >= 34) manager.getWallpaperInfo(WallpaperManager.FLAG_LOCK) else null
            if (info == null && manager.getWallpaperId(WallpaperManager.FLAG_LOCK) < 0) home else slot(info)
        }.getOrNull()
        val homeScene = home?.let { sceneFor(context, it) }
        val lockScene = if (lock == home) homeScene else lock?.let { sceneFor(context, it) }
        return Selection(home, lock, homeScene, lockScene)
    }

    private fun sceneFile(context: Context, slot: Slot) = File(context.filesDir, "wallpaper-${slot.defaultScene}")

    fun sceneFor(context: Context, slot: Slot): String = try {
        sceneFile(context, slot).readText().takeIf(::isScene) ?: slot.defaultScene
    } catch (_: IOException) {
        slot.defaultScene
    }

    fun setScene(context: Context, slot: Slot, scene: String): Boolean {
        if (!isScene(scene)) return false
        if (sceneFor(context, slot) == scene) return true
        // The WebView service runs in another process. Rename in place so readers see a complete value.
        val file = sceneFile(context, slot)
        try {
            val pending = File.createTempFile(file.name, ".tmp", context.filesDir)
            try {
                pending.writeText(scene)
                if (!pending.renameTo(file)) return false
            } finally {
                pending.delete()
            }
        } catch (error: IOException) {
            Log.w("XiaobanLiveWallpaper", "Unable to save wallpaper scene", error)
            return false
        }
        context.sendBroadcast(Intent(SCENE_CHANGED).setPackage(context.packageName))
        return true
    }

    fun openPicker(context: Context, slot: Slot, launch: (Intent) -> Unit): Boolean {
        val preview = Intent(WallpaperManager.ACTION_CHANGE_LIVE_WALLPAPER)
            .putExtra(WallpaperManager.EXTRA_LIVE_WALLPAPER_COMPONENT, ComponentName(context, slot.service))
        if (launchPicker(preview, launch)) return true
        return openChooser(context, slot, launch)
    }

    fun openChooser(context: Context, slot: Slot, launch: (Intent) -> Unit): Boolean {
        if (!launchPicker(Intent(WallpaperManager.ACTION_LIVE_WALLPAPER_CHOOSER), launch)) return false
        val title = context.getString(slot.label)
        Toast.makeText(context, "请在壁纸列表中选择「$title」", Toast.LENGTH_LONG).show()
        return true
    }

    private fun launchPicker(intent: Intent, launch: (Intent) -> Unit): Boolean =
        runCatching { launch(intent) }
            .onFailure { Log.w("XiaobanLiveWallpaper", "Unable to launch ${intent.action}", it) }
            .isSuccess
}
