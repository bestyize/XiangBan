package xyz.xiangban.xiaoban

import android.annotation.SuppressLint
import android.app.Activity
import android.app.AlertDialog
import android.content.pm.ActivityInfo
import android.content.Context
import android.content.Intent
import android.content.res.Configuration
import android.graphics.BitmapFactory
import android.graphics.Rect
import android.net.Uri
import android.net.http.SslError
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.ResultReceiver
import android.os.SystemClock
import android.provider.Settings
import android.util.Log
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.KeyEvent
import android.view.WindowManager
import android.webkit.ConsoleMessage
import android.webkit.JsPromptResult
import android.webkit.JsResult
import android.webkit.PermissionRequest
import android.webkit.SslErrorHandler
import android.webkit.WebChromeClient
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.FrameLayout
import android.widget.TextView
import android.widget.Toast
import android.window.OnBackInvokedCallback
import android.window.OnBackInvokedDispatcher
import androidx.activity.ComponentActivity
import androidx.compose.ui.graphics.asAndroidBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.lifecycleScope
import androidx.webkit.JavaScriptReplyProxy
import androidx.webkit.ProcessGlobalConfig
import androidx.webkit.ProxyConfig
import androidx.webkit.ProxyController
import androidx.webkit.ScriptHandler
import androidx.webkit.WebMessageCompat
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import xyz.xiangban.xiaoban.business.communitynative.AppearanceMode
import xyz.xiangban.xiaoban.business.communitynative.InMemoryPrefs
import xyz.xiangban.xiaoban.business.communitynative.ThemeState
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import xyz.xiangban.xiaoban.common.community.CommunityConfig
import java.io.ByteArrayInputStream

/** Protocol 2 pages run in a separate process with a filtering WebView proxy. */
class RuntimeV2Activity : ComponentActivity() {
    private val handler = Handler(Looper.getMainLooper())
    private val timeout = Runnable { fail("作品在 10 秒内没有准备就绪") }
    private val pauseWebView = Runnable { webView?.onPause() }
    private lateinit var pack: RuntimeV2Package
    private lateinit var chrome: RuntimeV2Chrome
    private lateinit var container: FrameLayout
    private lateinit var statusBarBackground: View
    private lateinit var script: String
    private var webView: WebView? = null
    private var injectedScript: ScriptHandler? = null
    private var document: JavaScriptReplyProxy? = null
    private var documentVisible = false
    private var resumed = false
    private var closing = false
    private var acceptingStorage = true
    private var ready = false
    private var failed = false
    private var playing = false
    // Block puzzle switches icon contrast with its game and result palettes.
    private var blockPuzzleShowingResult = false
    private val blockPuzzleFullBleed: Boolean
        get() = ::pack.isInitialized && pack.host == "official-block-puzzle.app.xiangban.xyz"
    private val pocketSprites: Boolean
        get() = ::pack.isInitialized && pack.host == "official-pocket-sprites.app.xiangban.xyz"
    private val pocketSpritesFullBleed: Boolean
        get() = pocketSprites && ready && !failed
    private val fullBleedStatusBar: Boolean
        get() = blockPuzzleFullBleed || pocketSpritesFullBleed
    private var reportedOpen = false
    private var networkDisabled = true
    private var legacySave: JSONObject? = null
    private var log: RuntimeV2Log? = null
    private var startedAt = 0L
    private var proxyReady = false
    private var info: JSONObject? = null
    private var safeInsets = Rect()
    private var lastUserGesture = 0L
    private var exportInFlight = false
    private var remoteEnabled = false
    private var quickLaunchEnabled = false
    private var remoteJob: Job? = null
    private val remoteControl by lazy { RuntimeV2RemoteControl(this) }
    private val favoriteReceiver by lazy { RuntimeV2Favorites.receiver(intent) }
    private var favorite: Boolean? = null
    private var shareJob: Job? = null
    private val shortcutWindowFocused = MutableStateFlow(false)
    private val mbtiShare = RuntimeV2MbtiShare(this)
    private val backCallback by lazy { OnBackInvokedCallback { handleBack() } }
    private val media = RuntimeV2Media(this, { chrome }, { ready && resumed && !closing && !failed && !chrome.menuOpen }, ::updateVisibility)

    override fun dispatchTouchEvent(event: MotionEvent): Boolean {
        if (event.actionMasked == MotionEvent.ACTION_UP) lastUserGesture = SystemClock.elapsedRealtime()
        return super.dispatchTouchEvent(event)
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        if (event.action == KeyEvent.ACTION_UP && event.keyCode in setOf(KeyEvent.KEYCODE_ENTER, KeyEvent.KEYCODE_DPAD_CENTER, KeyEvent.KEYCODE_SPACE)) lastUserGesture = SystemClock.elapsedRealtime()
        return super.dispatchKeyEvent(event)
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        shortcutWindowFocused.value = hasFocus
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val id = intent.getStringExtra(EXTRA_GAME_ID).orEmpty()
        val community = intent.hasExtra(AndroidRuntimeV2Community.PACK_ID)
        val workspace = !community && intent.getBooleanExtra(EXTRA_WORKSPACE, false) && MOBILE_GAME_ID.matches(id)
        val root = if (community || workspace) null else bundledGameRoot(id) ?: run { finish(); return }
        remoteEnabled = root != null && id == BuiltinMobileGames.REMOTE_CONTROL
        if (workspace) log = RuntimeV2Log(filesDir.resolve("workspace/logs/$id.txt"))
        active?.let { previous ->
            previous.closing = true
            previous.destroyWebView()
            previous.finish()
        }
        active = this
        RuntimeV2Proxy.setNetworkEnabled(false)
        if (!configureWebViewDirectory()) {
            showUnavailable("请更新 Android System WebView 或 Chrome，再打开这个作品。")
            return
        }
        if (!supportedFeatures()) {
            showUnavailable("请更新 Android System WebView 或 Chrome，再打开这个作品。")
            return
        }
        lifecycleScope.launch {
            val loaded = withContext(Dispatchers.IO) {
                runCatching {
                    pack = when {
                        workspace -> RuntimeV2Package.loadWorkspace(filesDir.resolve("workspace/games/$id"), id)
                        root == null -> AndroidRuntimeV2Community.load(this@RuntimeV2Activity, intent)
                        else -> assets.loadBundledRuntimeV2(id, root)
                    }
                    quickLaunchEnabled = pack.manifest.id == "quick-launch" && runCatching {
                        val expected = assets.open("game-packs-v2/quick-launch/index.html").use { it.readBytes() }
                        val actual = requireNotNull(pack.open(pack.manifest.entry)).use { it.readBytes() }
                        java.security.MessageDigest.isEqual(expected, actual)
                    }.getOrDefault(false)
                    script = assets.open("runtime-v2/bridge.js").bufferedReader().use { it.readText() }
                    legacySave = when {
                        workspace -> null
                        community -> LegacySaves.community(
                            filesDir, intent.getLongExtra(AndroidRuntimeV2Community.PACK_ID, 0),
                            intent.getIntArrayExtra(AndroidRuntimeV2Community.LEGACY_VERSIONS) ?: intArrayOf(),
                        )
                        id != RUNTIME_V2_FIXTURE -> LegacySaves.builtin(filesDir, id)
                        else -> null
                    }
                    networkDisabled = if (remoteEnabled) true else if (BuildConfig.DEBUG && id == RUNTIME_V2_FIXTURE) {
                        val override = filesDir.resolve("runtime-v2/debug-network-disabled")
                        override.exists() && override.readText().trim() != "false"
                    } else intent.getBooleanExtra(EXTRA_NETWORK_DISABLED, true)
                    decodeIcon()?.asImageBitmap()
                }
            }
            if (isFinishing || isDestroyed || closing) return@launch
            loaded.onSuccess { icon ->
                ThemeState.load(InMemoryPrefs().apply {
                    intent.getStringExtra(APPEARANCE)?.let { put("appearance", it) }
                    intent.getStringExtra(ACCENT)?.let { put("accent", it) }
                })
                requestedOrientation = when (pack.manifest.orientation) {
                    "landscape" -> ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
                    "any" -> ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
                    else -> ActivityInfo.SCREEN_ORIENTATION_PORTRAIT
                }
                WindowCompat.setDecorFitsSystemWindows(window, false)
                if (Build.VERSION.SDK_INT >= 28) window.attributes = window.attributes.apply {
                    layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
                }
                container = FrameLayout(this@RuntimeV2Activity)
                statusBarBackground = View(this@RuntimeV2Activity).apply {
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                }
                container.addView(statusBarBackground, FrameLayout.LayoutParams(-1, 0, Gravity.TOP))
                val kindLabel = when {
                    pack.manifest.kind == "game" -> "游戏"
                    pack.manifest.category == "quiz" -> "测评"
                    else -> "工具"
                }
                val shortcut = RuntimeWorkShortcut.forWork(id, workspace, intent.getLongExtra(AndroidRuntimeV2Community.PACK_ID, 0))
                chrome = RuntimeV2Chrome(
                    this@RuntimeV2Activity, container, pack.manifest.name, kindLabel, icon,
                    official = !community && !workspace, author = intent.getStringExtra(AndroidRuntimeV2Community.AUTHOR),
                    onRestart = { leave(reopen = true) }, onExit = ::exit,
                    onFavorite = if (favoriteReceiver != null) ({ updateFavorite(toggle = true) }) else null,
                    onShare = if (id != RUNTIME_V2_FIXTURE && !remoteEnabled) ({ shareWork(id, workspace) }) else null,
                    onPinShortcut = if (Build.VERSION.SDK_INT >= 26 && shortcut != null) ({ pinWork(shortcut, icon?.asAndroidBitmap()) }) else null,
                    onMenuChanged = { open ->
                        updateVisibility()
                        if (open) updateFavorite(toggle = false)
                    },
                    landscapeLaunch = pack.manifest.orientation == "landscape",
                    capsuleRightMarginDp = if (blockPuzzleFullBleed) 16f else 6f,
                    capsuleTopGapDp = if (blockPuzzleFullBleed) 8f else 0f,
                )
                setContentView(container)
                media.watch()
                updateSystemBars()
                ViewCompat.setOnApplyWindowInsetsListener(container) { _, insets ->
                    val safe = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
                    safeInsets = Rect(safe.left, safe.top, safe.right, safe.bottom)
                    updateStatusBarBackground()
                    updateInfo()
                    startWhenReady()
                    insets
                }
                ViewCompat.requestApplyInsets(container)
                configureProxy()
            }.onFailure { showUnavailable("无法打开作品：${it.message.orEmpty().take(100)}") }
        }
    }

    @androidx.annotation.RequiresApi(26)
    private fun pinWork(shortcut: RuntimeWorkShortcut, icon: android.graphics.Bitmap?) {
        if (chrome.shortcutBusy || closing) return
        chrome.shortcutBusy = true
        lifecycleScope.launch {
            try {
                val xiaomi = Build.MANUFACTURER.equals("Xiaomi", ignoreCase = true)
                val result = withContext(Dispatchers.IO) {
                    runCatching { shortcut.requestPin(this@RuntimeV2Activity, pack.manifest.name, icon) }
                        .getOrDefault(RuntimeWorkShortcut.PinResult.Rejected)
                }
                if (result == RuntimeWorkShortcut.PinResult.AlreadyPinned) {
                    Toast.makeText(this@RuntimeV2Activity, "快捷方式已在桌面", Toast.LENGTH_SHORT).show()
                    chrome.closeMenu()
                    return@launch
                }

                var offerSettings = xiaomi && result == RuntimeWorkShortcut.PinResult.Rejected
                if (result == RuntimeWorkShortcut.PinResult.Requested) {
                    // Wait for the launcher to finish its confirmation before reporting an unpinned request.
                    delay(1_200)
                    offerSettings = xiaomi && shortcutWindowFocused.value
                    shortcutWindowFocused.first { it }
                    delay(250)
                    val pinned = withContext(Dispatchers.IO) {
                        runCatching { shortcut.isPinned(this@RuntimeV2Activity) }.getOrDefault(false)
                    }
                    if (pinned) {
                        chrome.closeMenu()
                        return@launch
                    }
                }
                if (!closing && !isFinishing) {
                    Toast.makeText(this@RuntimeV2Activity, "未添加到桌面", Toast.LENGTH_SHORT).show()
                    if (offerSettings) {
                        AlertDialog.Builder(this@RuntimeV2Activity)
                            .setTitle("尚未添加到桌面")
                            .setMessage("请在小伴的应用权限中检查「桌面快捷方式」是否已允许，然后再试。")
                            .setNegativeButton("取消", null)
                            .setPositiveButton("去设置") { _, _ -> openShortcutSettings() }
                            .show()
                    }
                }
            } finally {
                chrome.shortcutBusy = false
            }
        }
    }

    private fun openShortcutSettings() {
        val miui = Intent("miui.intent.action.APP_PERM_EDITOR")
            .setPackage("com.miui.securitycenter").putExtra("extra_pkgname", packageName)
        val appSettings = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.parse("package:$packageName"))
        runCatching { startActivity(miui) }.recoverCatching { startActivity(appSettings) }
    }

    private fun updateFavorite(toggle: Boolean) {
        val receiver = favoriteReceiver ?: return
        if (chrome.favoriteBusy || closing) return
        chrome.favoriteBusy = true
        lifecycleScope.launch {
            try {
                favorite = if (toggle) {
                    val current = favorite ?: RuntimeV2Favorites.request(receiver, pack.manifest.name)
                    RuntimeV2Favorites.request(receiver, pack.manifest.name, saved = !current)
                } else RuntimeV2Favorites.request(receiver, pack.manifest.name)
                chrome.favorited = favorite == true
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                favorite = null
                chrome.favorited = false
                if (toggle && !closing) {
                    Toast.makeText(this@RuntimeV2Activity, e.message ?: "收藏操作失败，请稍后重试", Toast.LENGTH_SHORT).show()
                }
            } finally {
                chrome.favoriteBusy = false
            }
        }
    }

    private fun shareWork(id: String, workspace: Boolean) {
        if (chrome.shareBusy || closing) return
        chrome.shareBusy = true
        shareJob = lifecycleScope.launch {
            try {
                if (workspace) {
                    check(AndroidCreationSharing(this@RuntimeV2Activity).shareGame(id))
                } else {
                    val site = CommunityConfig().forumBase
                    val packId = intent.getLongExtra(AndroidRuntimeV2Community.PACK_ID, 0)
                    val url = if (packId > 0) "$site/packs/$packId" else "$site/create?game=$id"
                    shareText(this@RuntimeV2Activity, pack.manifest.name, url)
                }
                chrome.closeMenu()
            } catch (e: CancellationException) {
                throw e
            } catch (_: Exception) {
                if (!closing) Toast.makeText(this@RuntimeV2Activity, "分享失败，请稍后重试", Toast.LENGTH_SHORT).show()
            } finally {
                chrome.shareBusy = false
            }
        }
    }

    private fun decodeIcon(): android.graphics.Bitmap? {
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        pack.open(pack.manifest.icon)?.use { BitmapFactory.decodeStream(it, null, bounds) }
        if (bounds.outWidth <= 0 || bounds.outHeight <= 0) return null
        var sample = 1
        while (maxOf(bounds.outWidth, bounds.outHeight) / sample > 384) sample *= 2
        val options = BitmapFactory.Options().apply { inSampleSize = sample }
        return pack.open(pack.manifest.icon)?.use { BitmapFactory.decodeStream(it, null, options) }
    }

    private fun configureWebViewDirectory(): Boolean {
        if (directoryConfigured) return true
        if (!WebViewFeature.isStartupFeatureSupported(this, WebViewFeature.STARTUP_FEATURE_SET_DATA_DIRECTORY_SUFFIX)) return false
        ProcessGlobalConfig.apply(ProcessGlobalConfig().setDataDirectorySuffix(this, "runtime_v2"))
        directoryConfigured = true
        return true
    }

    private fun configureProxy() {
        try {
            if (BuildConfig.DEBUG) RuntimeV2Proxy.onRejected = { Log.d("RuntimeV2Proxy", it) }
            val config = ProxyConfig.Builder().addProxyRule("127.0.0.1:${RuntimeV2Proxy.port}").removeImplicitRules().build()
            ProxyController.getInstance().setProxyOverride(config, ContextCompat.getMainExecutor(this)) {
                if (active !== this || isFinishing || isDestroyed || closing) return@setProxyOverride
                RuntimeV2Proxy.setNetworkEnabled(!networkDisabled)
                proxyReady = true
                startWhenReady()
            }
        } catch (error: Exception) {
            Log.w("RuntimeV2", "无法建立作品网络隔离", error)
            showUnavailable("无法启用作品网络隔离，请重新打开作品。")
        }
    }

    private fun startWhenReady() {
        if (proxyReady && info != null && webView == null && !closing) restart()
    }

    private fun showUnavailable(message: String) {
        setContentView(TextView(this).apply {
            text = message
            textSize = 16f
            setPadding(24, 48, 24, 24)
        })
    }

    private val landscape get() = resources.configuration.orientation == Configuration.ORIENTATION_LANDSCAPE
    private val dark get() = when (ThemeState.mode) {
        AppearanceMode.Dark -> true
        AppearanceMode.Light -> false
        AppearanceMode.System -> resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK == Configuration.UI_MODE_NIGHT_YES
    }

    private fun updateStatusBarBackground() {
        val height = if (landscape || fullBleedStatusBar) 0 else safeInsets.top
        val layout = statusBarBackground.layoutParams
        if (layout.height != height) {
            layout.height = height
            statusBarBackground.layoutParams = layout
        }
        statusBarBackground.setBackgroundColor(when {
            fullBleedStatusBar -> android.graphics.Color.TRANSPARENT
            dark -> 0xFF17191F.toInt()
            else -> 0xFFF7F8FA.toInt()
        })
    }

    @Suppress("DEPRECATION")
    private fun updateSystemBars() {
        updateStatusBarBackground()
        if (blockPuzzleFullBleed || pocketSprites) {
            window.statusBarColor = android.graphics.Color.TRANSPARENT
        }
        if (pocketSprites) {
            window.navigationBarColor = android.graphics.Color.TRANSPARENT
            if (Build.VERSION.SDK_INT >= 29) {
                window.isStatusBarContrastEnforced = false
                window.isNavigationBarContrastEnforced = false
            }
        }
        WindowCompat.getInsetsController(window, window.decorView).apply {
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            if (landscape) {
                hide(WindowInsetsCompat.Type.systemBars())
            } else {
                show(WindowInsetsCompat.Type.systemBars())
                val useDarkIcons = when {
                    pocketSpritesFullBleed -> false
                    blockPuzzleFullBleed -> !blockPuzzleShowingResult
                    else -> !dark
                }
                isAppearanceLightStatusBars = useDarkIcons
                isAppearanceLightNavigationBars = useDarkIcons
            }
        }
    }

    private fun updateInfo() {
        if (!::chrome.isInitialized) return
        val left = safeInsets.left
        val top = safeInsets.top
        val right = safeInsets.right
        val bottom = safeInsets.bottom
        chrome.updateInsets(left, top, right, bottom, landscape)
        val capsule = chrome.capsuleBounds
        val density = resources.displayMetrics.density
        val safeArea = JSONObject().put("left", left / density).put("top", top / density)
            .put("right", right / density).put("bottom", bottom / density)
        val capsuleArea = JSONObject().put("top", capsule.top / density).put("right", chrome.capsuleRightInset / density)
            .put("width", capsule.width() / density).put("height", capsule.height() / density)
        val next = JSONObject().put("platform", "android").put("sdkVersion", 2)
            .put("orientation", if (landscape) "landscape" else "portrait")
            .put("theme", if (dark) "dark" else "light")
            .put("locale", resources.configuration.locales[0].toLanguageTag())
            .put("safeArea", safeArea).put("capsule", capsuleArea)
            .put("query", JSONObject()).put("legacySave", legacySave ?: JSONObject.NULL)
        val previous = info
        if (previous?.toString() == next.toString()) return
        info = next
        webView?.let(::inject)
        if (previous != null) {
            if (previous.getString("orientation") != next.getString("orientation")) event("orientation", next.getString("orientation"))
            if (previous.getString("theme") != next.getString("theme")) event("theme", next.getString("theme"))
            if (previous.getJSONObject("safeArea").toString() != safeArea.toString() ||
                previous.getJSONObject("capsule").toString() != capsuleArea.toString()
            ) event("safearea", JSONObject().put("safeArea", safeArea).put("capsule", capsuleArea))
        }
    }

    private fun inject(view: WebView) {
        injectedScript?.remove()
        injectedScript = WebViewCompat.addDocumentStartJavaScript(
            view, script.replace("__XB_CAPABILITIES__", JSONArray(media.capabilities(pack)).toString())
                .replace("__XB_INITIAL_INFO__", checkNotNull(info).toString()), setOf(pack.origin),
        )
    }

    private fun restart() {
        if (closing) return
        handler.removeCallbacks(timeout)
        destroyWebView()
        ready = false
        failed = false
        playing = false
        blockPuzzleShowingResult = false
        if (blockPuzzleFullBleed || pocketSprites) updateSystemBars()
        acceptingStorage = true
        chrome.loading()
        startedAt = System.currentTimeMillis()
        log?.reset()
        log?.add("host.open", "gameId" to pack.manifest.id, "version" to pack.manifest.version)
        val view = createWebView()
        webView = view
        container.addView(view, 0, FrameLayout.LayoutParams(-1, -1))
        inject(view)
        handler.postDelayed(timeout, 10_000)
        view.loadUrl(pack.entryUrl)
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun createWebView(): WebView = WebView(this).apply {
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.allowFileAccess = false
        settings.allowContentAccess = false
        settings.blockNetworkLoads = networkDisabled
        settings.mixedContentMode = WebSettings.MIXED_CONTENT_NEVER_ALLOW
        settings.setSupportMultipleWindows(false)
        settings.javaScriptCanOpenWindowsAutomatically = false
        val cookies = android.webkit.CookieManager.getInstance()
        cookies.setAcceptCookie(false)
        cookies.setAcceptThirdPartyCookies(this, false)
        WebViewCompat.addDocumentStartJavaScript(this, RESTRICT_WEB_RTC, setOf("*"))
        webViewClient = object : WebViewClient() {
            override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? {
                val url = request.url
                if (!pack.ownsOrigin(url)) {
                    return if (!networkDisabled && url.scheme == "https" && !RuntimeV2Proxy.isBlockedHost(url.host)) null
                    else response(403, "Forbidden")
                }
                val path = pack.pathFor(url) ?: return response(403, "Forbidden")
                if (request.method != "GET") return response(405, "Method Not Allowed")
                val input = pack.open(path) ?: return response(404, "Not Found")
                val mime = checkNotNull(pack.mimeType(path))
                val encoding = if (mime.startsWith("text/") || mime in setOf("application/json", "image/svg+xml", "application/xml")) "UTF-8" else null
                return WebResourceResponse(mime, encoding, 200, "OK", mapOf(
                    "Content-Security-Policy" to contentSecurityPolicy(),
                    "X-Content-Type-Options" to "nosniff",
                    "Permissions-Policy" to "camera=(self), microphone=(self), geolocation=(), fullscreen=()",
                    "Cache-Control" to "no-store",
                ), input)
            }

            override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean =
                if (request.isForMainFrame) pack.pathFor(request.url) == null
                else networkDisabled || request.url.scheme != "https" || RuntimeV2Proxy.isBlockedHost(request.url.host)

            @Deprecated("Android 7 compatibility")
            override fun shouldOverrideUrlLoading(view: WebView, url: String): Boolean =
                pack.pathFor(android.net.Uri.parse(url)) == null

            override fun onPageStarted(view: WebView, url: String, favicon: android.graphics.Bitmap?) {
                media.cancel()
                document = null
                documentVisible = false
            }

            override fun onReceivedError(view: WebView, request: WebResourceRequest, error: WebResourceError) {
                if (request.isForMainFrame) fail("无法加载作品页面")
            }

            override fun onReceivedHttpError(view: WebView, request: WebResourceRequest, response: WebResourceResponse) {
                if (request.isForMainFrame) fail("无法加载作品页面")
            }

            override fun onReceivedSslError(view: WebView, handler: SslErrorHandler, error: SslError) { handler.cancel() }
        }
        webChromeClient = object : WebChromeClient() {
            override fun onPermissionRequest(request: PermissionRequest) = media.request(request, pack)
            override fun onPermissionRequestCanceled(request: PermissionRequest) = media.cancel(request)
            override fun onConsoleMessage(message: ConsoleMessage): Boolean {
                if (message.messageLevel() == ConsoleMessage.MessageLevel.ERROR) {
                    log?.add("console.error", "message" to message.message().take(300), "source" to message.sourceId().substringAfterLast('/').take(100), "line" to message.lineNumber())
                }
                return true
            }
            override fun onJsAlert(view: WebView, url: String, message: String, result: JsResult): Boolean {
                result.cancel()
                return true
            }
            override fun onJsConfirm(view: WebView, url: String, message: String, result: JsResult): Boolean {
                result.cancel()
                return true
            }
            override fun onJsPrompt(view: WebView, url: String, message: String, defaultValue: String?, result: JsPromptResult): Boolean {
                result.cancel()
                return true
            }
        }
        WebViewCompat.addWebMessageListener(this, "xbNative", setOf(pack.origin)) { view, message, origin, mainFrame, reply ->
            if (view === webView && mainFrame && pack.ownsOrigin(origin) && message.type == WebMessageCompat.TYPE_STRING) {
                message.data?.let { handleMessage(it, reply) }
            }
        }
    }

    private fun contentSecurityPolicy(): String {
        val remote = if (networkDisabled) "" else " https:"
        val connect = if (networkDisabled) pack.origin else "https: wss:"
        val frames = if (networkDisabled) "'none'" else "https:"
        return "default-src 'self' data: blob:$remote; script-src 'self' 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' blob:$remote; " +
            "style-src 'self' 'unsafe-inline'$remote; connect-src $connect; frame-src $frames; object-src 'none'; base-uri 'self'; form-action 'none'"
    }

    private fun handleMessage(payload: String, reply: JavaScriptReplyProxy) {
        if (isDestroyed || payload.length > RuntimeV2Storage.MAX_MESSAGE_BYTES) return
        val call = runCatching { JSONObject(payload) }.getOrNull() ?: return
        val id = call.opt("id")
        if (call.optString("channel") != "xiaban.app" || call.opt("protocol") != 2 ||
            call.optString("kind") != "call" || id !is Number || !id.isNonNegativeInteger() || id.toLong() == 0L
        ) return
        val method = call.optString("method")
        val params = call.optJSONObject("params") ?: JSONObject()
        fun result(value: Any = JSONObject.NULL) = post(reply, envelope("result").put("id", id).put("ok", true).put("value", value))
        fun error(code: String) = post(reply, envelope("result").put("id", id).put("ok", false).put("error", JSONObject().put("code", code)))
        when (method) {
            "document.connect" -> {
                if (document !== reply) {
                    document = reply
                    documentVisible = false
                }
                result(JSONObject().put("info", info))
                updateVisibility()
            }
            "document.error" -> {
                log?.add("runtime.error", "filename" to params.optString("filename").take(128), "name" to params.optString("type").take(64),
                    "message" to params.optString("message").take(256), "line" to params.optInt("line"))
                result()
                if (!ready) fail("作品启动时出错")
            }
            "ready" -> {
                result()
                if (!ready && !failed && !closing) {
                    ready = true
                    log?.add("game.ready", "readyMs" to System.currentTimeMillis() - startedAt)
                    handler.removeCallbacks(timeout)
                    chrome.loaded()
                    if (pocketSprites) updateSystemBars()
                    reportOpen()
                }
            }
            "exit" -> { result(); log?.add("game.exit"); exit() }
            "quickLaunch.available", "quickLaunch.open", "quickLaunch.pin" -> {
                val name = params.optString("name")
                if (!quickLaunchEnabled || reply !== document || !QuickLaunchShortcut.targets.containsKey(name)) { error("unsupported"); return }
                if (method == "quickLaunch.available") { result(QuickLaunchShortcut.available(this, name)); return }
                if (!ready || !resumed || closing || failed || chrome.menuOpen || chrome.consentOpen ||
                    lastUserGesture == 0L || SystemClock.elapsedRealtime() - lastUserGesture !in 0..3000) { error("gesture"); return }
                lastUserGesture = 0L
                if (method == "quickLaunch.open") {
                    if (QuickLaunchShortcut.open(this, name)) result(true) else error("unavailable")
                    return
                }
                lifecycleScope.launch {
                    val requested = withContext(Dispatchers.IO) {
                        runCatching { QuickLaunchShortcut.pin(this@RuntimeV2Activity, name) }.getOrDefault(false)
                    }
                    if (!requested) { error("unavailable"); return@launch }
                    delay(1_200)
                    val focused = shortcutWindowFocused.value
                    shortcutWindowFocused.first { it }
                    delay(250)
                    val pinned = withContext(Dispatchers.IO) {
                        runCatching { QuickLaunchShortcut.isPinned(this@RuntimeV2Activity, name) }.getOrDefault(false)
                    }
                    if (pinned) result(true) else {
                        error("unavailable")
                        if (Build.MANUFACTURER.equals("Xiaomi", ignoreCase = true) && focused && !closing && !isFinishing) {
                            AlertDialog.Builder(this@RuntimeV2Activity)
                                .setTitle("尚未添加到桌面")
                                .setMessage("请在小伴的应用权限中检查「桌面快捷方式」是否已允许，然后再试。")
                                .setNegativeButton("取消", null)
                                .setPositiveButton("去设置") { _, _ -> openShortcutSettings() }
                                .show()
                        }
                    }
                }
            }
            "builtin.remote.supported" -> {
                if (remoteEnabled && reply === document) result(remoteControl.supported()) else error("unsupported")
            }
            "builtin.remote.catalog", "builtin.remote.send" -> {
                if (!remoteEnabled || reply !== document) { error("unsupported"); return }
                val send = method == "builtin.remote.send"
                if (send) {
                    if (!ready || !resumed || closing || failed || chrome.menuOpen || chrome.consentOpen ||
                        lastUserGesture == 0L || SystemClock.elapsedRealtime() - lastUserGesture !in 0..1000) {
                        error("gesture")
                        return
                    }
                }
                if (remoteJob?.isActive == true) { error("busy"); return }
                if (send) lastUserGesture = 0L
                remoteJob = lifecycleScope.launch {
                    try {
                        if (send) {
                            val outcome = withContext(Dispatchers.IO) {
                                remoteControl.send(params.optString("profile"), params.optString("key"), params.optJSONObject("state"))
                            }
                            if (outcome.error == null) result(outcome.state ?: true) else error(outcome.error)
                        } else {
                            val catalog = withContext(Dispatchers.IO) { remoteControl.catalog() }
                            result(catalog)
                        }
                    } catch (e: CancellationException) {
                        throw e
                    } catch (_: SecurityException) {
                        error("denied")
                    } catch (_: Exception) {
                        error(if (send) "send-failed" else "catalog-failed")
                    }
                }
            }
            "builtin.mbti.export" -> {
                if (pack.host != "official-mbti.app.xiangban.xyz" || pack.manifest.id != "mbti" ||
                    intent.getBooleanExtra(EXTRA_WORKSPACE, false) || intent.hasExtra(AndroidRuntimeV2Community.PACK_ID)) { error("unsupported"); return }
                if (!ready || !resumed || closing || failed || chrome.menuOpen || chrome.consentOpen ||
                    SystemClock.elapsedRealtime() - lastUserGesture !in 0..3000 || lastUserGesture == 0L) { error("gesture"); return }
                if (exportInFlight) { error("busy"); return }
                lastUserGesture = 0L
                exportInFlight = true
                lifecycleScope.launch {
                    try {
                        mbtiShare.perform(params.optString("action"), params.optString("image"), params.optString("url")) { value, code ->
                            exportInFlight = false
                            if (code == null) result(value ?: "opened") else error(code)
                        }
                    } catch (e: kotlinx.coroutines.CancellationException) { exportInFlight = false; throw e }
                    catch (_: Exception) { exportInFlight = false; error("busy") }
                }
            }
            "storage.get", "storage.set", "storage.remove", "storage.keys" -> {
                if (!acceptingStorage) { error("busy"); return }
                val file = filesDir.resolve("runtime-v2/storage/${pack.host}.json")
                RuntimeV2Storage.submit(file, method, params, payload) { value, code ->
                    if (code == null) result(value ?: JSONObject.NULL) else error(code)
                }
            }
            "game.start" -> {
                if (!ready || pack.manifest.kind != "game") error("invalid")
                else {
                    playing = true
                    if (blockPuzzleFullBleed && blockPuzzleShowingResult) {
                        blockPuzzleShowingResult = false
                        updateSystemBars()
                    }
                    log?.add("game.start")
                    result()
                }
            }
            "game.progress", "game.result" -> {
                val score = params.opt("score")
                if (!playing || score !is Number || !score.isNonNegativeInteger()) error("invalid")
                else {
                    if (method == "game.result") {
                        playing = false
                        if (blockPuzzleFullBleed) {
                            blockPuzzleShowingResult = true
                            updateSystemBars()
                        }
                        log?.add("game.result", "score" to score.toLong())
                    }
                    result()
                }
            }
            else -> error("unsupported")
        }
    }

    @Suppress("DEPRECATION")
    private fun reportOpen() {
        if (reportedOpen) return
        reportedOpen = true
        val result = if (Build.VERSION.SDK_INT >= 33) intent.getParcelableExtra(OPENED, ResultReceiver::class.java)
        else intent.getParcelableExtra<ResultReceiver>(OPENED)
        result?.send(RESULT_OK, null)
    }

    private fun Number.isNonNegativeInteger(): Boolean =
        toDouble().let { it in 0.0..9_007_199_254_740_991.0 && it == toLong().toDouble() }

    private fun envelope(kind: String) = JSONObject().put("channel", "xiaban.host").put("protocol", 2).put("kind", kind)

    private fun post(reply: JavaScriptReplyProxy, value: JSONObject) {
        if (!isDestroyed) runCatching { reply.postMessage(value.toString()) }
    }

    private fun event(name: String, data: Any = JSONObject.NULL) {
        document?.let { post(it, envelope("event").put("name", name).put("data", data)) }
    }

    private fun updateVisibility() {
        val visible = resumed && !closing && !failed && !chrome.menuOpen && !chrome.consentOpen
        if (document == null || visible == documentVisible) return
        documentVisible = visible
        event(if (visible) "show" else "hide")
    }

    private fun fail(message: String) {
        if (failed || closing || !::chrome.isInitialized) return
        failed = true
        log?.add("host.fail", "message" to message.take(200))
        handler.removeCallbacks(timeout)
        updateVisibility()
        chrome.error(message) { leave(reopen = true) }
        if (pocketSprites) updateSystemBars()
    }

    private fun handleBack() {
        if (closing) return
        if (::chrome.isInitialized && chrome.consentOpen) { media.cancel(); return }
        if (!::chrome.isInitialized || !ready || failed) { exit(); return }
        if (chrome.menuOpen) { chrome.closeMenu(); return }
        val view = webView
        if (view != null && view.canGoBack()) view.goBack()
        else exit()
    }

    private fun exit() = leave(reopen = false)

    private fun leave(reopen: Boolean) {
        if (closing) return
        closing = true
        shareJob?.cancel()
        media.cancel()
        if (webView == null) { finish(); return }
        updateVisibility()
        handler.postDelayed({
            acceptingStorage = false
            RuntimeV2Storage.afterWrites {
                if (isDestroyed) return@afterWrites
                if (reopen) {
                    closing = false
                    restart()
                } else finish()
            }
        }, if (document == null) 0 else 500)
    }

    override fun onStart() {
        super.onStart()
        if (Build.VERSION.SDK_INT >= 33) onBackInvokedDispatcher.registerOnBackInvokedCallback(OnBackInvokedDispatcher.PRIORITY_DEFAULT, backCallback)
    }

    override fun onResume() {
        super.onResume()
        resumed = true
        handler.removeCallbacks(pauseWebView)
        webView?.onResume()
        if (::chrome.isInitialized) { updateSystemBars(); updateInfo(); updateVisibility() }
    }

    override fun onPause() {
        resumed = false
        if (::chrome.isInitialized) updateVisibility()
        handler.postDelayed(pauseWebView, 500)
        super.onPause()
    }

    override fun onStop() {
        if (Build.VERSION.SDK_INT >= 33) onBackInvokedDispatcher.unregisterOnBackInvokedCallback(backCallback)
        super.onStop()
    }

    @Deprecated("System back before Android 13")
    override fun onBackPressed() = handleBack()

    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)
        if (::chrome.isInitialized) {
            updateSystemBars()
            ViewCompat.requestApplyInsets(window.decorView)
            updateInfo()
        }
    }

    override fun onDestroy() {
        handler.removeCallbacksAndMessages(null)
        destroyWebView()
        log?.close()
        media.close()
        if (active === this) {
            RuntimeV2Proxy.setNetworkEnabled(false)
            active = null
        }
        super.onDestroy()
    }

    private fun destroyWebView() {
        if (::chrome.isInitialized) media.reset()
        document = null
        documentVisible = false
        injectedScript?.remove()
        injectedScript = null
        webView?.let {
            it.stopLoading()
            (it.parent as? FrameLayout)?.removeView(it)
            it.destroy()
        }
        webView = null
    }

    private fun response(status: Int, reason: String) =
        WebResourceResponse("text/plain", "UTF-8", status, reason, emptyMap(), ByteArrayInputStream(byteArrayOf()))

    companion object {
        const val EXTRA_NETWORK_DISABLED = "runtime.v2.network.disabled"
        const val EXTRA_GAME_ID = "runtime.v2.game.id"
        const val EXTRA_WORKSPACE = "runtime.v2.workspace"
        private const val APPEARANCE = "runtime.v2.appearance"
        private const val ACCENT = "runtime.v2.accent"
        private const val OPENED = "runtime.v2.opened"
        private var directoryConfigured = false
        private var active: RuntimeV2Activity? = null

        internal fun launchIntent(activity: Activity, opened: (() -> Unit)? = null): Intent {
            val prefs = AndroidPrefs(activity)
            return Intent(activity, RuntimeV2Activity::class.java).apply {
                putExtra(APPEARANCE, prefs.get("appearance"))
                putExtra(ACCENT, prefs.get("accent"))
                if (opened != null) putExtra(OPENED, object : ResultReceiver(Handler(Looper.getMainLooper())) {
                    override fun onReceiveResult(resultCode: Int, resultData: Bundle?) {
                        if (resultCode == Activity.RESULT_OK) opened()
                    }
                })
            }
        }

        fun supported(context: Context): Boolean = WebViewFeature.isStartupFeatureSupported(context,
            WebViewFeature.STARTUP_FEATURE_SET_DATA_DIRECTORY_SUFFIX) && supportedFeatures()

        private fun supportedFeatures() = WebViewFeature.isFeatureSupported(WebViewFeature.DOCUMENT_START_SCRIPT) &&
            WebViewFeature.isFeatureSupported(WebViewFeature.WEB_MESSAGE_LISTENER) &&
            WebViewFeature.isFeatureSupported(WebViewFeature.PROXY_OVERRIDE)

        private val RESTRICT_WEB_RTC = """
            for (const key of ['RTCPeerConnection', 'webkitRTCPeerConnection']) {
              try { Object.defineProperty(window, key, {value: undefined, configurable: false, writable: false}); } catch (_) {}
            }
        """.trimIndent()
    }
}
