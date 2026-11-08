package xyz.xiangban.xiaoban

import android.annotation.SuppressLint
import android.app.Activity
import android.content.pm.ActivityInfo
import android.content.Context
import android.content.Intent
import android.content.res.Configuration
import android.graphics.BitmapFactory
import android.graphics.Rect
import android.net.http.SslError
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.ResultReceiver
import android.util.Log
import android.view.WindowManager
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
import android.window.OnBackInvokedCallback
import android.window.OnBackInvokedDispatcher
import androidx.activity.ComponentActivity
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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import xyz.xiangban.xiaoban.business.communitynative.AppearanceMode
import xyz.xiangban.xiaoban.business.communitynative.InMemoryPrefs
import xyz.xiangban.xiaoban.business.communitynative.ThemeState
import java.io.ByteArrayInputStream

/** Protocol 2 pages run in a separate process with a filtering WebView proxy. */
class RuntimeV2Activity : ComponentActivity() {
    private val handler = Handler(Looper.getMainLooper())
    private val timeout = Runnable { fail("作品在 10 秒内没有准备就绪") }
    private val pauseWebView = Runnable { webView?.onPause() }
    private lateinit var pack: RuntimeV2Package
    private lateinit var chrome: RuntimeV2Chrome
    private lateinit var container: FrameLayout
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
    private var reportedOpen = false
    private var networkDisabled = true
    private var legacySave: JSONObject? = null
    private var proxyReady = false
    private var info: JSONObject? = null
    private var safeInsets = Rect()
    private val backCallback by lazy { OnBackInvokedCallback { handleBack() } }
    private val media = RuntimeV2Media(this, { chrome }, { ready && resumed && !closing && !failed && !chrome.menuOpen }, ::updateVisibility)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val id = intent.getStringExtra(MobileGameActivity.EXTRA_GAME_ID).orEmpty()
        val community = intent.hasExtra(AndroidRuntimeV2Community.PACK_ID)
        val root = if (community) null else bundledGameRoot(id) ?: run { finish(); return }
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
                    pack = if (root == null) AndroidRuntimeV2Community.load(this@RuntimeV2Activity, intent)
                    else assets.loadBundledRuntimeV2(id, root)
                    script = assets.open("runtime-v2/bridge.js").bufferedReader().use { it.readText() }
                    if (root != null && id != RUNTIME_V2_FIXTURE) legacySave = MobileGameActivity.readBuiltinLegacySave(filesDir, id)
                    networkDisabled = if (BuildConfig.DEBUG && id == RUNTIME_V2_FIXTURE) {
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
                val kindLabel = when {
                    pack.manifest.kind == "game" -> "游戏"
                    pack.manifest.category == "quiz" -> "测评"
                    else -> "工具"
                }
                chrome = RuntimeV2Chrome(
                    this@RuntimeV2Activity, container, pack.manifest.name, kindLabel, icon,
                    official = !community, author = intent.getStringExtra(AndroidRuntimeV2Community.AUTHOR),
                    onRestart = { leave(reopen = true) }, onExit = ::exit, onMenuChanged = { updateVisibility() },
                    landscapeLaunch = pack.manifest.orientation == "landscape",
                )
                setContentView(container)
                media.watch()
                updateSystemBars()
                ViewCompat.setOnApplyWindowInsetsListener(container) { _, insets ->
                    val safe = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
                    safeInsets = Rect(safe.left, safe.top, safe.right, safe.bottom)
                    updateInfo()
                    startWhenReady()
                    insets
                }
                ViewCompat.requestApplyInsets(container)
                configureProxy()
            }.onFailure { showUnavailable("无法打开作品：${it.message.orEmpty().take(100)}") }
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

    private fun updateSystemBars() {
        WindowCompat.getInsetsController(window, window.decorView).apply {
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            if (landscape) hide(WindowInsetsCompat.Type.systemBars())
            else {
                show(WindowInsetsCompat.Type.systemBars())
                isAppearanceLightStatusBars = !dark
                isAppearanceLightNavigationBars = !dark
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
        acceptingStorage = true
        chrome.loading()
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
                result()
                if (!ready) fail("作品启动时出错")
            }
            "ready" -> {
                result()
                if (!ready && !failed && !closing) {
                    ready = true
                    handler.removeCallbacks(timeout)
                    chrome.loaded()
                    reportOpen()
                }
            }
            "exit" -> { result(); exit() }
            "storage.get", "storage.set", "storage.remove", "storage.keys" -> {
                if (!acceptingStorage) { error("busy"); return }
                val file = filesDir.resolve("runtime-v2/storage/${pack.host}.json")
                RuntimeV2Storage.submit(file, method, params, payload) { value, code ->
                    if (code == null) result(value ?: JSONObject.NULL) else error(code)
                }
            }
            "game.start" -> {
                if (!ready || pack.manifest.kind != "game") error("invalid")
                else { playing = true; result() }
            }
            "game.progress", "game.result" -> {
                val score = params.opt("score")
                if (!playing || score !is Number || !score.isNonNegativeInteger()) error("invalid")
                else {
                    if (method == "game.result") playing = false
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
        handler.removeCallbacks(timeout)
        updateVisibility()
        chrome.error(message) { leave(reopen = true) }
    }

    private fun handleBack() {
        if (closing) return
        if (::chrome.isInitialized && chrome.consentOpen) { media.cancel(); return }
        if (!::chrome.isInitialized || !ready || failed || chrome.menuOpen) { exit(); return }
        val view = webView
        if (view != null && view.canGoBack()) view.goBack()
        else chrome.openMenu(armed = true)
    }

    private fun exit() = leave(reopen = false)

    private fun leave(reopen: Boolean) {
        if (closing) return
        closing = true
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
