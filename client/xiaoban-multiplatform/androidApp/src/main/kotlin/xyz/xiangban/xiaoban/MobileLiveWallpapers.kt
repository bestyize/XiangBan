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
import org.json.JSONObject

internal object MobileLiveWallpapers {
    private val scenes = mapOf(
        "aquarium" to AquariumWallpaperService::class.java,
        "galaxy" to GalaxyWallpaperService::class.java,
    )

    data class Selection(val home: String?, val lock: String?) {
        fun hasNewWallpaper(previous: Selection) =
            (home != null && home != previous.home) || (lock != null && lock != previous.lock)

        fun toJson() = JSONObject()
            .put("home", home ?: JSONObject.NULL)
            .put("lock", lock ?: JSONObject.NULL)
    }

    fun supported(context: Context): Boolean = runCatching {
        val manager = WallpaperManager.getInstance(context)
        context.packageManager.hasSystemFeature(PackageManager.FEATURE_LIVE_WALLPAPER) &&
            manager.isWallpaperSupported && manager.isSetWallpaperAllowed
    }.getOrDefault(false)

    fun selection(context: Context): Selection {
        val manager = WallpaperManager.getInstance(context)
        fun scene(info: WallpaperInfo?): String? {
            if (info == null || info.packageName != context.packageName) return null
            return scenes.entries.firstOrNull { it.value.name == info.serviceName }?.key
        }

        val home = runCatching { scene(manager.wallpaperInfo) }.getOrNull()
        // Older versions may limit wallpaper info and do not expose a separate lock component.
        val lock = if (Build.VERSION.SDK_INT >= 34) runCatching {
            val info = manager.getWallpaperInfo(WallpaperManager.FLAG_LOCK)
            if (info == null && manager.getWallpaperId(WallpaperManager.FLAG_LOCK) < 0) {
                // No separate lock wallpaper is configured, so Android shares the home wallpaper.
                home
            } else scene(info)
        }.getOrNull() else null
        return Selection(home, lock)
    }

    fun openPicker(context: Context, scene: String, launch: (Intent) -> Unit): Boolean {
        val service = scenes[scene] ?: return false
        if (!supported(context)) return false
        val preview = Intent(WallpaperManager.ACTION_CHANGE_LIVE_WALLPAPER)
            .putExtra(WallpaperManager.EXTRA_LIVE_WALLPAPER_COMPONENT, ComponentName(context, service))
        if (launchPicker(preview, launch)) return true
        if (!launchPicker(Intent(WallpaperManager.ACTION_LIVE_WALLPAPER_CHOOSER), launch)) return false
        val title = context.getString(if (scene == "aquarium") R.string.live_wallpaper_aquarium else R.string.live_wallpaper_galaxy)
        Toast.makeText(context, "请在壁纸列表中选择「$title」", Toast.LENGTH_LONG).show()
        return true
    }

    private fun launchPicker(intent: Intent, launch: (Intent) -> Unit): Boolean =
        runCatching { launch(intent) }
            .onFailure { Log.w("XiaobanLiveWallpaper", "Unable to launch ${intent.action}", it) }
            .isSuccess
}
