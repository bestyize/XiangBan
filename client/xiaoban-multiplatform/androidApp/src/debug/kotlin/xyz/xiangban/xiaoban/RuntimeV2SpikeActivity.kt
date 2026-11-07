package xyz.xiangban.xiaoban

import android.Manifest
import android.app.Activity
import android.app.AlertDialog
import android.content.pm.PackageManager
import android.graphics.Color
import android.net.Uri
import android.os.Bundle
import android.view.View
import android.view.WindowManager
import android.webkit.PermissionRequest
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.TextView
import android.widget.Toast
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.webkit.WebMessageCompat
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import org.json.JSONObject
import java.io.ByteArrayInputStream
import java.util.concurrent.Executors

class RuntimeV2SpikeActivity : Activity() {
    private val origin = "https://spike1.app.xiangban.xyz"
    private val rules = setOf("https://*.app.xiangban.xyz")
    private lateinit var webView: WebView
    private val storageExecutor = Executors.newSingleThreadExecutor()
    private var pendingPermission: PermissionRequest? = null
    private var lastBack = 0L
    private var loaded = false
    private val csp = "default-src 'self' https: wss: data: blob:; script-src 'self' https: 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval'; style-src 'self' https: 'unsafe-inline'; connect-src 'self' https: wss:; worker-src 'self' blob:; object-src 'none'; base-uri 'self'"

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        WindowCompat.setDecorFitsSystemWindows(window, false)
        if (android.os.Build.VERSION.SDK_INT >= 28) window.attributes = window.attributes.apply {
            layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES
        }
        val missing = listOf(WebViewFeature.DOCUMENT_START_SCRIPT, WebViewFeature.WEB_MESSAGE_LISTENER).filterNot { WebViewFeature.isFeatureSupported(it) }
        if (missing.isNotEmpty()) {
            setContentView(TextView(this).apply { text = "WebView 版本不支持运行时协议 2 小样：缺少 ${missing.joinToString()}。请更新 Android System WebView 或 Chrome 后重试。"; textSize = 18f; setPadding(32, 80, 32, 32) })
            return
        }
        webView = WebView(this)
        webView.setBackgroundColor(Color.WHITE)
        webView.settings.javaScriptEnabled = true
        webView.settings.domStorageEnabled = true
        webView.settings.allowFileAccess = false
        webView.settings.allowContentAccess = false
        webView.webViewClient = object : WebViewClient() {
            override fun shouldInterceptRequest(view: WebView, request: WebResourceRequest): WebResourceResponse? {
                val uri = request.url
                if (RuntimeV2Requests.isLocal(uri.host) || (uri.scheme != "https" && uri.scheme != "wss")) return blocked()
                if (!RuntimeV2Requests.isWorkHost(uri.host)) return null
                val path = RuntimeV2Requests.assetPath(uri.toString()) ?: return blocked()
                return try {
                    WebResourceResponse(RuntimeV2Requests.contentType(path), null, 200, "OK", mapOf(
                        "Access-Control-Allow-Origin" to "*", "Content-Security-Policy" to csp,
                        "X-Content-Type-Options" to "nosniff"
                    ), assets.open(path))
                } catch (_: Exception) { WebResourceResponse("text/plain", "UTF-8", 404, "Not Found", emptyMap(), ByteArrayInputStream(byteArrayOf())) }
            }
            override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean =
                request.isForMainFrame && request.url.toString().let { !it.startsWith("$origin/") && it != origin }
        }
        webView.webChromeClient = object : WebChromeClient() {
            override fun onPermissionRequest(request: PermissionRequest) {
                if (request.origin.toString() != origin || pendingPermission != null || request.resources.any { it != PermissionRequest.RESOURCE_VIDEO_CAPTURE && it != PermissionRequest.RESOURCE_AUDIO_CAPTURE }) { request.deny(); return }
                pendingPermission = request
                AlertDialog.Builder(this@RuntimeV2SpikeActivity).setTitle("允许测试摄像头或麦克风？")
                    .setMessage("测试页请求：${request.resources.joinToString { if (it == PermissionRequest.RESOURCE_VIDEO_CAPTURE) "摄像头" else "麦克风" }}。")
                    .setPositiveButton("同意") { _, _ -> requestSystemPermissions(request) }
                    .setNegativeButton("拒绝") { _, _ -> denyPending() }.setOnCancelListener { denyPending() }.show()
            }
            override fun onPermissionRequestCanceled(request: PermissionRequest) { if (pendingPermission == request) pendingPermission = null }
        }
        WebViewCompat.addWebMessageListener(webView, "xbNative", rules) { _, message, sourceOrigin, isMainFrame, reply ->
            if (!isMainFrame || sourceOrigin.toString() != origin || message.type != WebMessageCompat.TYPE_STRING) return@addWebMessageListener
            val payload = message.data ?: return@addWebMessageListener
            storageExecutor.execute {
                val result = try {
                    val call = JSONObject(payload)
                    require(call.optString("channel") == "xiaban.app" && call.optInt("protocol") == 2 && call.optString("kind") == "call")
                    val response = JSONObject().put("channel", "xiaban.host").put("protocol", 2).put("kind", "result").put("id", call.get("id"))
                    val file = filesDir.resolve("runtime-v2-spike.json")
                    val data = if (file.exists()) JSONObject(file.readText()) else JSONObject()
                    val params = call.optJSONObject("params") ?: JSONObject()
                    val value: Any = when (call.getString("method")) {
                        "ready" -> JSONObject.NULL
                        "storage.get" -> data.opt(params.getString("key")) ?: JSONObject.NULL
                        "storage.set" -> {
                            val key = params.getString("key")
                            require(key.length <= 128)
                            data.put(key, params.get("value"))
                            val encoded = data.toString()
                            require(encoded.toByteArray(Charsets.UTF_8).size <= 10 * 1024 * 1024)
                            val atomic = android.util.AtomicFile(file)
                            val stream = atomic.startWrite()
                            try { stream.write(encoded.toByteArray(Charsets.UTF_8)); atomic.finishWrite(stream) } catch (e: Exception) { atomic.failWrite(stream); throw e }
                            JSONObject.NULL
                        }
                        else -> error("unsupported method")
                    }
                    response.put("ok", true).put("value", value)
                } catch (e: Exception) {
                    val id = try { JSONObject(payload).opt("id") } catch (_: Exception) { null }
                    JSONObject().put("channel", "xiaban.host").put("protocol", 2).put("kind", "result").put("id", id).put("ok", false).put("error", JSONObject().put("code", "invalid").put("message", e.message ?: "error"))
                }
                runOnUiThread { if (!isFinishing && !isDestroyed) reply.postMessage(result.toString()) }
            }
        }
        setContentView(webView)
        ViewCompat.setOnApplyWindowInsetsListener(webView) { _, insets ->
            val safe = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout())
            val density = resources.displayMetrics.density
            val values = JSONObject().put("top", safe.top / density).put("right", safe.right / density).put("bottom", safe.bottom / density).put("left", safe.left / density)
            if (!loaded) {
                val script = assets.open("runtime-v2-spike/bridge.js").bufferedReader().use { it.readText() }.replace("__SAFE_AREA__", values.toString())
                WebViewCompat.addDocumentStartJavaScript(webView, script, rules)
                loaded = true
                webView.loadUrl("$origin/v1/index.html")
            } else webView.evaluateJavascript("window.__xbSetSafeArea && window.__xbSetSafeArea($values)", null)
            insets
        }
        ViewCompat.requestApplyInsets(webView)
    }
    private fun blocked() = WebResourceResponse("text/plain", "UTF-8", 403, "Forbidden", emptyMap(), ByteArrayInputStream(byteArrayOf()))
    private fun denyPending() { pendingPermission?.deny(); pendingPermission = null }
    private fun requestSystemPermissions(request: PermissionRequest) {
        if (pendingPermission != request) return
        val permissions = request.resources.map { if (it == PermissionRequest.RESOURCE_VIDEO_CAPTURE) Manifest.permission.CAMERA else Manifest.permission.RECORD_AUDIO }.distinct()
        val missing = permissions.filter { checkSelfPermission(it) != PackageManager.PERMISSION_GRANTED }
        if (missing.isEmpty()) { request.grant(request.resources); pendingPermission = null } else requestPermissions(missing.toTypedArray(), 42)
    }
    @Deprecated("Android permission callback")
    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == 42) {
            val request = pendingPermission ?: return
            if (grantResults.isNotEmpty() && grantResults.all { it == PackageManager.PERMISSION_GRANTED }) request.grant(request.resources) else request.deny()
            pendingPermission = null
        }
    }
    private fun event(name: String) { if (::webView.isInitialized && loaded) webView.evaluateJavascript("window.__xbEvent && window.__xbEvent('$name')", null) }
    override fun onResume() { super.onResume(); if (::webView.isInitialized) { webView.onResume(); event("show") } }
    override fun onPause() { event("hide"); super.onPause() }
    // targetSdk 36+ on Android 16 never calls onBackPressed; back must go through OnBackInvokedDispatcher.
    private fun handleBack() {
        if (::webView.isInitialized && webView.canGoBack()) { webView.goBack(); return }
        val now = android.os.SystemClock.elapsedRealtime()
        if (now - lastBack < 2000) finish() else { lastBack = now; Toast.makeText(this, "再按一次退出", Toast.LENGTH_SHORT).show() }
    }
    private val backCallback by lazy { android.window.OnBackInvokedCallback { handleBack() } }
    override fun onStart() {
        super.onStart()
        if (android.os.Build.VERSION.SDK_INT >= 33) onBackInvokedDispatcher.registerOnBackInvokedCallback(android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT, backCallback)
    }
    override fun onStop() {
        if (android.os.Build.VERSION.SDK_INT >= 33) onBackInvokedDispatcher.unregisterOnBackInvokedCallback(backCallback)
        super.onStop()
    }
    @Deprecated("System back before Android 13") override fun onBackPressed() = handleBack()
    override fun onDestroy() { denyPending(); storageExecutor.shutdown(); if (::webView.isInitialized) webView.destroy(); super.onDestroy() }
}
