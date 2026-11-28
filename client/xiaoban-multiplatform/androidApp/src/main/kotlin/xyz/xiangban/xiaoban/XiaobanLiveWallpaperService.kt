package xyz.xiangban.xiaoban

import android.annotation.SuppressLint
import android.app.Presentation
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.graphics.Color
import android.graphics.drawable.ColorDrawable
import android.hardware.display.DisplayManager
import android.hardware.display.VirtualDisplay
import android.os.Handler
import android.os.Looper
import android.os.PowerManager
import android.service.wallpaper.WallpaperService
import android.util.Log
import android.view.MotionEvent
import android.view.Surface
import android.view.SurfaceHolder
import android.view.ViewConfiguration
import android.view.ViewGroup
import android.view.WindowManager
import android.webkit.RenderProcessGoneDetail
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.core.content.ContextCompat
import androidx.webkit.ProcessGlobalConfig
import androidx.webkit.WebViewAssetLoader
import androidx.webkit.WebViewFeature
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import kotlin.math.roundToInt
import kotlin.math.sqrt

// Retain the original components as independently configurable slots for Android's two screens.
class AquariumWallpaperService : XiaobanLiveWallpaperService(MobileLiveWallpapers.Slot.PRIMARY)
class GalaxyWallpaperService : XiaobanLiveWallpaperService(MobileLiveWallpapers.Slot.SECONDARY)

abstract class XiaobanLiveWallpaperService internal constructor(
    private val slot: MobileLiveWallpapers.Slot,
) : WallpaperService() {
    private val power by lazy { getSystemService(PowerManager::class.java) }
    private val displays by lazy { getSystemService(DisplayManager::class.java) }

    override fun onCreateEngine(): Engine = SceneEngine()

    private inner class SceneEngine : Engine() {
        private val handler = Handler(Looper.getMainLooper())
        private val releaseHidden = Runnable { releaseRenderer() }
        private val retryRenderer = Runnable { updateRenderer() }
        private var renderer: Renderer? = null
        private var surface: Surface? = null
        private var width = 0
        private var height = 0
        private var visible = false
        private var retried = false
        private var listening = false
        private val touchSlop = ViewConfiguration.get(this@XiaobanLiveWallpaperService).scaledTouchSlop
        private var touch: Touch? = null

        private val powerChanges = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                updateRenderer()
            }
        }

        override fun onCreate(holder: SurfaceHolder) {
            super.onCreate(holder)
            setTouchEventsEnabled(true)
            setOffsetNotificationsEnabled(false)
            ContextCompat.registerReceiver(
                this@XiaobanLiveWallpaperService,
                powerChanges,
                IntentFilter().apply {
                    addAction(Intent.ACTION_SCREEN_ON)
                    addAction(Intent.ACTION_SCREEN_OFF)
                    addAction(PowerManager.ACTION_POWER_SAVE_MODE_CHANGED)
                    addAction(MobileLiveWallpapers.SCENE_CHANGED)
                },
                ContextCompat.RECEIVER_NOT_EXPORTED,
            )
            listening = true
        }

        override fun onSurfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
            super.onSurfaceChanged(holder, format, width, height)
            if (surface !== holder.surface || this.width != width || this.height != height) {
                releaseRenderer()
                retried = false
            }
            surface = holder.surface
            this.width = width
            this.height = height
            updateRenderer()
        }

        override fun onSurfaceDestroyed(holder: SurfaceHolder) {
            surface = null
            retried = false
            releaseRenderer()
            super.onSurfaceDestroyed(holder)
        }

        override fun onVisibilityChanged(visible: Boolean) {
            super.onVisibilityChanged(visible)
            if (visible && !this.visible) retried = false
            this.visible = visible
            updateRenderer()
        }

        override fun onTouchEvent(event: MotionEvent) {
            super.onTouchEvent(event)
            val action = event.actionMasked
            if (action == MotionEvent.ACTION_DOWN || action == MotionEvent.ACTION_CANCEL || action == MotionEvent.ACTION_POINTER_DOWN) {
                touch = null
            }
            val current = renderer ?: return
            if (!current.playing || !current.loaded || width <= 0 || height <= 0) return
            if (action == MotionEvent.ACTION_DOWN) {
                val x = (event.x / width).coerceIn(0f, 1f)
                val y = (event.y / height).coerceIn(0f, 1f)
                if (x.isFinite() && y.isFinite()) touch = Touch(event.getPointerId(0), x, y)
                return
            }
            if (action != MotionEvent.ACTION_MOVE && action != MotionEvent.ACTION_UP) return
            val gesture = touch ?: return
            val finished = action == MotionEvent.ACTION_UP
            if (finished) touch = null
            val index = event.findPointerIndex(gesture.pointerId)
            if (index < 0) return
            val x = (event.getX(index) / width).coerceIn(0f, 1f)
            val y = (event.getY(index) / height).coerceIn(0f, 1f)
            if (!x.isFinite() || !y.isFinite()) return
            if (!gesture.dragging) {
                val dx = (x - gesture.lastX) * width
                val dy = (y - gesture.lastY) * height
                gesture.dragging = dx * dx + dy * dy >= touchSlop * touchSlop
            }
            if (gesture.dragging) {
                // Join the last forwarded point to this one, including the tail on release.
                if ((x != gesture.lastX || y != gesture.lastY) &&
                    (finished || event.eventTime - gesture.lastWipeAt >= 30)
                ) {
                    current.view.evaluateJavascript("window.xbWallpaper?.wipe($x,$y,${gesture.lastX},${gesture.lastY})", null)
                    gesture.lastX = x
                    gesture.lastY = y
                    gesture.lastWipeAt = event.eventTime
                }
            } else if (finished && event.eventTime - event.downTime < ViewConfiguration.getLongPressTimeout()) {
                current.view.evaluateJavascript("window.xbWallpaper?.tap($x,$y)", null)
            }
        }

        private fun updateRenderer() {
            handler.removeCallbacks(releaseHidden)
            val target = surface
            if (target == null || !target.isValid || width <= 0 || height <= 0) {
                releaseRenderer()
                return
            }
            val fps = MobileLiveWallpapers.renderingFps(this@XiaobanLiveWallpaperService)
            val powerSaver = fps == 15
            if (!visible || !power.isInteractive) {
                touch = null
                if (!power.isInteractive || powerSaver) {
                    releaseRenderer()
                } else {
                    renderer?.let {
                        if (it.playing) {
                            it.playing = false
                            it.updatePlayback()
                            it.view.onPause()
                            it.display.setSurface(null)
                        }
                        handler.postDelayed(releaseHidden, 12_000)
                    }
                }
                return
            }
            val qualityScale = if (powerSaver) 0.55 else 0.75
            val maxPixels = if (powerSaver) 900_000.0 else 1_600_000.0
            val scale = minOf(qualityScale, sqrt(maxPixels / (width.toDouble() * height)))
            val profile = Profile(
                (width * scale).roundToInt().coerceAtLeast(1),
                (height * scale).roundToInt().coerceAtLeast(1),
                fps,
            )
            val preset = MobileLiveWallpapers.presetFor(this@XiaobanLiveWallpaperService, slot)
            val current = renderer
            if (current?.profile == profile) {
                if (!current.playing) {
                    current.display.setSurface(target)
                    current.view.onResume()
                    current.playing = true
                    current.updatePlayback()
                }
                current.setPreset(preset)
                return
            }
            releaseRenderer()
            startRenderer(target, profile, preset)
        }

        private fun startRenderer(target: Surface, profile: Profile, preset: String) {
            var display: VirtualDisplay? = null
            var window: Presentation? = null
            var view: WebView? = null
            try {
                if (!configureWallpaperWebView(this@XiaobanLiveWallpaperService)) return
                val virtual = displays.createVirtualDisplay(
                    "Xiaoban wallpaper",
                    // The wallpaper surface may place smaller buffers at the top left without scaling.
                    width,
                    height,
                    resources.displayMetrics.densityDpi,
                    target,
                    DisplayManager.VIRTUAL_DISPLAY_FLAG_PRESENTATION or DisplayManager.VIRTUAL_DISPLAY_FLAG_OWN_CONTENT_ONLY,
                ) ?: return
                display = virtual
                val presentation = Presentation(this@XiaobanLiveWallpaperService, virtual.display, android.R.style.Theme_Material_NoActionBar)
                window = presentation
                presentation.window?.apply {
                    addFlags(WindowManager.LayoutParams.FLAG_HARDWARE_ACCELERATED or WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE)
                    setBackgroundDrawable(ColorDrawable(Color.BLACK))
                }
                val web = WebView(presentation.context)
                view = web
                configureScene(web)
                presentation.setContentView(web)
                presentation.show()
                presentation.window?.setLayout(WindowManager.LayoutParams.MATCH_PARENT, WindowManager.LayoutParams.MATCH_PARENT)
                renderer = Renderer(profile, virtual, presentation, web, preset)
                val wallpaperUrl = "https://appassets.androidplatform.net/assets/game-packs-v2/${BuiltinMobileGames.LIVE_WALLPAPERS}/wallpaper.html"
                web.loadUrl("$wallpaperUrl?preset=$preset&fps=${profile.fps}&width=${profile.width}&height=${profile.height}&paused=1")
            } catch (error: RuntimeException) {
                Log.w(TAG, "Unable to start wallpaper renderer", error)
                renderer = null
                closeRenderer(display, window, view)
            }
        }

        @SuppressLint("SetJavaScriptEnabled")
        private fun configureScene(view: WebView) {
            val assets = WebViewAssetLoader.Builder()
                .addPathHandler("/assets/", WebViewAssetLoader.AssetsPathHandler(this@XiaobanLiveWallpaperService))
                .build()
            view.setBackgroundColor(Color.BLACK)
            view.settings.apply {
                javaScriptEnabled = true
                allowFileAccess = false
                allowContentAccess = false
                blockNetworkLoads = true
                mixedContentMode = WebSettings.MIXED_CONTENT_NEVER_ALLOW
            }
            view.webViewClient = object : WebViewClient() {
                override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? =
                    assets.shouldInterceptRequest(request.url)

                override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest) = true

                override fun onPageFinished(view: WebView, url: String) {
                    renderer?.takeIf { it.view === view }?.onLoaded()
                }

                override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean {
                    if (renderer?.view === view) {
                        releaseRenderer()
                        if (visible && !retried) {
                            retried = true
                            handler.postDelayed(retryRenderer, 1000)
                        }
                    }
                    return true
                }
            }
        }

        private fun releaseRenderer() {
            touch = null
            handler.removeCallbacks(retryRenderer)
            handler.removeCallbacks(releaseHidden)
            val current = renderer ?: return
            renderer = null
            closeRenderer(current.display, current.window, current.view)
        }

        override fun onDestroy() {
            visible = false
            surface = null
            releaseRenderer()
            if (listening) this@XiaobanLiveWallpaperService.unregisterReceiver(powerChanges)
            super.onDestroy()
        }
    }

    private class Touch(val pointerId: Int, var lastX: Float, var lastY: Float) {
        var lastWipeAt = -30L
        var dragging = false
    }

    private data class Profile(val width: Int, val height: Int, val fps: Int)

    private class Renderer(
        val profile: Profile,
        val display: VirtualDisplay,
        val window: Presentation,
        val view: WebView,
        private var preset: String,
    ) {
        var loaded = false
            private set
        var playing = true

        fun onLoaded() {
            loaded = true
            view.evaluateJavascript("window.xbWallpaper?.setPreset('$preset'); window.xbWallpaper?.setActive($playing)", null)
        }

        fun setPreset(id: String) {
            if (preset == id) return
            preset = id
            if (loaded) view.evaluateJavascript("window.xbWallpaper?.setPreset('$preset')", null)
        }

        fun updatePlayback() {
            if (loaded) view.evaluateJavascript("window.xbWallpaper?.setActive($playing)", null)
        }
    }

    private companion object {
        const val TAG = "XiaobanLiveWallpaper"
        var directoryConfigured = false

        fun closeRenderer(display: VirtualDisplay?, window: Presentation?, view: WebView?) {
            try {
                (view?.parent as? ViewGroup)?.removeView(view)
                window?.dismiss()
                view?.destroy()
            } finally {
                display?.release()
            }
        }

        fun configureWallpaperWebView(context: Context): Boolean {
            if (directoryConfigured) return true
            if (!WebViewFeature.isStartupFeatureSupported(context, WebViewFeature.STARTUP_FEATURE_SET_DATA_DIRECTORY_SUFFIX)) {
                return false
            }
            ProcessGlobalConfig.apply(ProcessGlobalConfig().setDataDirectorySuffix(context, "live_wallpaper"))
            directoryConfigured = true
            return true
        }
    }
}
