package xyz.xiangban.xiaoban

import android.app.ActivityManager
import android.app.WallpaperInfo
import android.app.WallpaperManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.PowerManager
import android.util.Log
import android.widget.Toast
import java.io.File
import java.io.IOException
import org.json.JSONObject

internal object MobileLiveWallpapers {
    const val SCENE_CHANGED = "xyz.xiangban.xiaoban.action.WALLPAPER_SCENE_CHANGED"
    private val presets = setOf("dusk", "morning", "moss", "neon", "snow")

    // Both the component names and the on-disk filenames predate Rain Window.
    enum class Slot(val fileName: String, val defaultPreset: String, val service: Class<*>, val label: Int) {
        PRIMARY("wallpaper-aquarium", "dusk", AquariumWallpaperService::class.java, R.string.live_wallpaper_aquarium),
        SECONDARY("wallpaper-galaxy", "neon", GalaxyWallpaperService::class.java, R.string.live_wallpaper_galaxy);

        fun presetFromStored(value: String?) = value?.takeIf(::isPreset) ?: defaultPreset
    }

    fun isPreset(id: String) = id in presets

    fun renderingFps(context: Context) = if (context.getSystemService(PowerManager::class.java).isPowerSaveMode ||
        context.getSystemService(ActivityManager::class.java).isLowRamDevice
    ) 15 else 30

    data class Selection(private val homeSlot: Slot?, private val lockSlot: Slot?, val home: String?, val lock: String?) {
        fun hasNewWallpaper(previous: Selection) =
            (homeSlot != null && (homeSlot != previous.homeSlot || home != previous.home)) ||
                (lockSlot != null && (lockSlot != previous.lockSlot || lock != previous.lock))

        fun switchTarget(mode: String): String? = when {
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

        fun unusedSlot(preset: String): Slot? {
            val available = Slot.entries.filter { it != homeSlot && it != lockSlot }
            return available.firstOrNull { it.defaultPreset == preset } ?: available.firstOrNull()
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
        val homePreset = home?.let { presetFor(context, it) }
        val lockPreset = if (lock == home) homePreset else lock?.let { presetFor(context, it) }
        return Selection(home, lock, homePreset, lockPreset)
    }

    private fun readPreset(file: File): String? = try {
        file.readText()
    } catch (_: IOException) {
        null
    }

    fun presetFor(context: Context, slot: Slot): String =
        slot.presetFromStored(readPreset(File(context.filesDir, slot.fileName)))

    fun setPreset(context: Context, slot: Slot, preset: String): Boolean {
        if (!isPreset(preset)) return false
        val file = File(context.filesDir, slot.fileName)
        if (readPreset(file) == preset) return true
        // The WebView service runs in another process. Rename in place so readers see a complete value.
        try {
            val pending = File.createTempFile(file.name, ".tmp", context.filesDir)
            try {
                pending.writeText(preset)
                if (!pending.renameTo(file)) return false
            } finally {
                pending.delete()
            }
        } catch (error: IOException) {
            Log.w("XiaobanLiveWallpaper", "Unable to save wallpaper preset", error)
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
