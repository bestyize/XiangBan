package xyz.xiangban.xiaoban

import android.annotation.SuppressLint
import android.app.Activity
import android.content.Intent
import android.content.pm.ActivityInfo
import android.graphics.Color
import android.net.Uri
import android.net.http.SslError
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.Message
import android.system.ErrnoException
import android.system.Os
import android.system.OsConstants
import android.view.Gravity
import android.view.ViewGroup
import android.webkit.ConsoleMessage
import android.webkit.GeolocationPermissions
import android.webkit.JsPromptResult
import android.webkit.JsResult
import android.webkit.PermissionRequest
import android.webkit.SslErrorHandler
import android.webkit.ValueCallback
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Toast
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import xyz.xiangban.xiaoban.common.community.AndroidMobileWorkspace
import xyz.xiangban.xiaoban.common.community.MobileGameStore
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.File
import java.util.UUID
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

/** Offline HTML-game runner. The opaque-origin iframe has scripts, and no native bridge. */
class MobileGameActivity : Activity() {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
    private val nonce = UUID.randomUUID().toString()
    // This token is deliberately different from hostNonce, which is visible to the game.
    private val consoleToken = "xb-private-host:${UUID.randomUUID()}:"
    private var webView: WebView? = null
    private var snapshot: GameSnapshot? = null
    private var exportBytes: ByteArray? = null
    private var gameId = ""
    private var ready = false
    private var playing = false
    private var lastLogAt = 0L
    private var logsThisSecond = 0
    private var lastSaveAt = 0L
    private var activityResumed = false
    private val saveHandler = Handler(Looper.getMainLooper())
    private var pendingSave: ByteArray? = null
    private var saveScheduled = false
    private var saveErrorShown = false
    private val saveRunnable = Runnable {
        saveScheduled = false
        flushPendingSave()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        gameId = intent.getStringExtra(EXTRA_GAME_ID).orEmpty()
        if (!MOBILE_GAME_ID.matches(gameId)) { finish(); return }
        showMessage("正在检查游戏…")
        scope.launch {
            try {
                val game = withContext(Dispatchers.IO) { loadSnapshot(gameId) }
                if (isFinishing || isDestroyed) return@launch
                snapshot = game
                if (intent.getBooleanExtra(EXTRA_EXPORT, false)) {
                    exportBytes = withContext(Dispatchers.IO) { game.zip() }
                    chooseExportLocation()
                } else {
                    withContext(Dispatchers.IO) { resetRuntimeLog() }
                    showGame(game)
                }
            } catch (error: Exception) {
                showMessage("无法打开游戏：${error.message.orEmpty().take(160)}", closeButton = true)
            }
        }
    }

    private suspend fun loadSnapshot(id: String): GameSnapshot {
        privateFile("games/$id").also { require(it.isDirectory) { "游戏不存在" } }
        val store = MobileGameStore(AndroidMobileWorkspace(privateFile("")))
        store.validate(id)
        val files = store.exportFiles(id)
        require(files.size in 2..41) { "游戏文件数量不符合规范" }
        val contents = linkedMapOf<String, ByteArray>()
        var total = 0
        for (file in files) {
            require(safeGamePath(file.path)) { "游戏路径不符合规范" }
            privateFile("games/$id/${file.path}")
            val bytes = file.content.toByteArray(Charsets.UTF_8)
            require(bytes.size <= 1024 * 1024) { "游戏文件过大" }
            total += bytes.size
            require(total <= 4 * 1024 * 1024 + 16 * 1024) { "游戏文件总大小超限" }
            require(contents.put(file.path, bytes) == null) { "重复游戏路径" }
        }
        val manifest = JSONObject(String(requireNotNull(contents["manifest.json"]), Charsets.UTF_8))
        require(manifest.optString("id") == id && manifest.optString("kind") == "game") { "游戏清单不匹配" }
        require(manifest.optInt("schemaVersion") == 1) { "手机版暂只支持第一版文本游戏包" }
        require(manifest.optString("layer", "space") == "space") { "桌面层游戏需要在电脑上运行" }
        require(contents.containsKey("index.html")) { "游戏缺少 index.html" }
        val saveEnabled = manifest.optBoolean("save", false)
        val save = if (saveEnabled) readSave() else null
        return GameSnapshot(id, manifest.optString("name", id), manifest.optString("orientation"), saveEnabled, save, contents)
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun showGame(game: GameSnapshot) {
        requestedOrientation = if (game.orientation == "landscape") ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE
            else ActivityInfo.SCREEN_ORIENTATION_SENSOR_PORTRAIT
        val layout = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; setBackgroundColor(Color.BLACK) }
        val bar = LinearLayout(this).apply { gravity = Gravity.CENTER_VERTICAL }
        bar.addView(Button(this).apply { text = "退出"; setOnClickListener { finish() } })
        bar.addView(TextView(this).apply { text = game.name; setTextColor(Color.WHITE); textSize = 16f },
            LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f))
        layout.addView(bar)
        val view = WebView(this)
        webView = view
        view.setBackgroundColor(Color.BLACK)
        view.settings.apply {
            javaScriptEnabled = true
            domStorageEnabled = false
            databaseEnabled = false
            allowFileAccess = false
            allowContentAccess = false
            blockNetworkLoads = true
            @Suppress("DEPRECATION")
            allowFileAccessFromFileURLs = false
            @Suppress("DEPRECATION")
            allowUniversalAccessFromFileURLs = false
            mixedContentMode = WebSettings.MIXED_CONTENT_NEVER_ALLOW
            cacheMode = WebSettings.LOAD_NO_CACHE
            javaScriptCanOpenWindowsAutomatically = false
            setSupportMultipleWindows(true)
            setGeolocationEnabled(false)
            mediaPlaybackRequiresUserGesture = true
            @Suppress("DEPRECATION")
            saveFormData = false
        }
        // CookieManager is process-wide. Avoid changing the account/community WebView's settings;
        // instead this host uses an otherwise unused origin and an opaque iframe with no cookie API.
        view.setDownloadListener { _, _, _, _, _ -> }
        view.webViewClient = object : WebViewClient() {
            override fun shouldInterceptRequest(view: WebView?, request: WebResourceRequest): WebResourceResponse =
                resourceResponse(request, game)

            override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest): Boolean =
                request.isForMainFrame || request.url.toString() != gameUrl(game.id)

            @Deprecated("Deprecated in Java")
            override fun shouldOverrideUrlLoading(view: WebView?, url: String?): Boolean = true

            override fun onReceivedSslError(view: WebView?, handler: SslErrorHandler, error: SslError?) { handler.cancel() }
        }
        view.webChromeClient = object : WebChromeClient() {
            override fun onPermissionRequest(request: PermissionRequest) { request.deny() }
            override fun onGeolocationPermissionsShowPrompt(origin: String?, callback: GeolocationPermissions.Callback) {
                callback.invoke(origin, false, false)
            }
            override fun onCreateWindow(view: WebView?, isDialog: Boolean, isUserGesture: Boolean, resultMsg: Message?): Boolean = false
            override fun onShowFileChooser(webView: WebView?, filePathCallback: ValueCallback<Array<Uri>>?, fileChooserParams: FileChooserParams?): Boolean {
                filePathCallback?.onReceiveValue(null)
                return true
            }
            override fun onJsAlert(view: WebView?, url: String?, message: String?, result: JsResult): Boolean { result.cancel(); return true }
            override fun onJsConfirm(view: WebView?, url: String?, message: String?, result: JsResult): Boolean { result.cancel(); return true }
            override fun onJsPrompt(view: WebView?, url: String?, message: String?, defaultValue: String?, result: JsPromptResult): Boolean { result.cancel(); return true }
            override fun onConsoleMessage(message: ConsoleMessage): Boolean {
                val text = message.message()
                if (message.sourceId() == HOST_URL && text.startsWith(consoleToken)) {
                    receiveHostEvent(text.removePrefix(consoleToken))
                }
                return true
            }
        }
        layout.addView(view, LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, 0, 1f))
        setContentView(layout)
        view.loadUrl(HOST_URL)
        // Validation may finish after the user backgrounds this activity.
        if (!activityResumed) {
            view.onPause()
            view.pauseTimers()
        }
    }

    private fun resourceResponse(request: WebResourceRequest, game: GameSnapshot): WebResourceResponse {
        val uri = request.url
        if (request.method != "GET" || uri.scheme != "https" || uri.host != HOST_NAME || uri.port != -1 || uri.userInfo != null) return blocked()
        if (uri.path == HOST_PATH && request.isForMainFrame && uri.query == null) {
            return response("text/html", hostHtml(game).toByteArray(), HOST_CSP.replace("%NONCE%", nonce))
        }
        if (request.isForMainFrame) return blocked()
        val prefix = "/games/${game.id}/"
        val path = uri.path?.takeIf { it.startsWith(prefix) }?.removePrefix(prefix) ?: return blocked()
        if (!safeGamePath(path)) return blocked()
        val bytes = game.files[path] ?: return blocked()
        val served = if (path.endsWith(".html", ignoreCase = true)) instrumentHtml(String(bytes, Charsets.UTF_8)).toByteArray() else bytes
        return response(mime(path), served, GAME_CSP)
    }

    private fun response(mime: String, bytes: ByteArray, csp: String) = WebResourceResponse(
        mime, "UTF-8", 200, "OK", mapOf(
            "Content-Security-Policy" to csp,
            "Cache-Control" to "no-store", "X-Content-Type-Options" to "nosniff",
            "Referrer-Policy" to "no-referrer", "Permissions-Policy" to "camera=(), microphone=(), geolocation=(), payment=(), usb=(), clipboard-read=(), clipboard-write=(), fullscreen=()",
        ), ByteArrayInputStream(bytes),
    )

    private fun blocked() = WebResourceResponse("text/plain", "UTF-8", 403, "Forbidden", mapOf("Cache-Control" to "no-store"), ByteArrayInputStream(ByteArray(0)))

    private fun hostHtml(game: GameSnapshot): String {
        val init = JSONObject().put("channel", "xiaban.game-host").put("protocol", 1).put("type", "host.init")
            .put("nonce", nonce).put("save", game.save ?: JSONObject.NULL)
            .put("safeArea", JSONObject().put("top", 0).put("right", 0).put("bottom", 0).put("left", 0))
        return """<!doctype html><html><head><meta name="viewport" content="width=device-width,initial-scale=1,maximum-scale=1,user-scalable=no">
            <style nonce="$nonce">html,body,iframe{margin:0;border:0;width:100%;height:100%;overflow:hidden;background:#111}iframe{display:block}</style></head>
            <body><iframe id="game" sandbox="allow-scripts" allow="camera 'none'; microphone 'none'; geolocation 'none'; clipboard-read 'none'; clipboard-write 'none'; fullscreen 'none'" referrerpolicy="no-referrer"></iframe>
            <script nonce="$nonce">(()=>{'use strict';
            const frame=document.getElementById('game'),nonce=${JSONObject.quote(nonce)},token=${JSONObject.quote(consoleToken)};
            const init=${scriptJson(init.toString())};let ready=false,playing=false,lastError=0,errorCount=0;
            const report=e=>console.log(token+JSON.stringify(e));
            window.addEventListener('message',e=>{const m=e.data;
              if(e.source!==frame.contentWindow||!m||m.channel!=='xiaban.game-pack'||m.protocol!==1||m.nonce!==nonce)return;
              if(m.type==='runtime.error'){
                const t=Date.now();if(t-lastError>1000){lastError=t;errorCount=0;}if(++errorCount>5)return;
                report({type:'runtime.error',message:String(m.message||'').slice(0,1600),source:String(m.source||'').slice(0,160),line:Number(m.line)||0});return;
              }
              if(m.type==='game.ready'){if(!ready){ready=true;report({type:'game.ready'});}return;}
              // Closing this frame grants no capability and also works for games without a handshake.
              if(m.type==='game.exit'){report({type:'game.exit'});return;}
              if(!ready)return;
              if(m.type==='game.save'&&${game.saveEnabled}){
                try{if(!m.data||Object.prototype.toString.call(m.data)!=='[object Object]'||Array.isArray(m.data))return;
                  const json=JSON.stringify(m.data);if(new TextEncoder().encode(json).length>32768)return;report({type:'game.save',data:JSON.parse(json)});
                }catch(_){}return;
              }
              if(m.type==='game.start'){playing=true;report({type:'game.start'});return;}
              if((m.type==='game.progress'||m.type==='game.result')&&playing&&Number.isSafeInteger(m.score)&&m.score>=0){
                if(m.type==='game.result')playing=false;report({type:m.type,score:m.score});
              }
            });
            frame.addEventListener('load',()=>{ready=false;playing=false;frame.contentWindow.postMessage(init,'*');report({type:'host.loaded'});});
            frame.src=${JSONObject.quote(gameUrl(game.id))};
            window.addEventListener('keydown',e=>{if(e.key==='Escape')report({type:'game.exit'});});
            })();</script></body></html>""".trimIndent()
    }

    private fun instrumentHtml(html: String): String {
        val bootstrap = """<script>(()=>{'use strict';const nonce=new URLSearchParams(location.search).get('hostNonce');
            const report=(message,source,line)=>parent.postMessage({channel:'xiaban.game-pack',protocol:1,nonce,type:'runtime.error',message:String(message).slice(0,1600),source:String(source||'').slice(0,160),line:Number(line)||0},'*');
            window.addEventListener('error',e=>report(e.message||'资源加载失败',e.filename||e.target&&e.target.src,e.lineno),true);
            window.addEventListener('unhandledrejection',e=>report(e.reason&&e.reason.message||e.reason||'Promise rejected','',0));
            window.addEventListener('keydown',e=>{if(e.key==='Escape')parent.postMessage({channel:'xiaban.game-pack',protocol:1,nonce,type:'game.exit'},'*');});
            })();</script>""".trimIndent()
        val doctype = Regex("(?is)^\\s*<!doctype[^>]*>").find(html)
        return if (doctype != null) html.substring(0, doctype.range.last + 1) + bootstrap + html.substring(doctype.range.last + 1) else bootstrap + html
    }

    private fun receiveHostEvent(json: String) {
        if (json.toByteArray().size > 40 * 1024) return
        val event = runCatching { JSONObject(json) }.getOrNull() ?: return
        val type = event.optString("type")
        when (type) {
            "host.loaded" -> { ready = false; playing = false }
            "game.ready" -> ready = true
            "game.exit" -> { finish(); return }
            "game.save" -> {
                if (!ready || snapshot?.saveEnabled != true) return
                val data = event.optJSONObject("data") ?: return
                val bytes = data.toString().toByteArray(Charsets.UTF_8)
                if (bytes.size > MAX_SAVE_BYTES) return
                // Keep the newest value while coalescing rapid updates. Never discard the last save.
                pendingSave = bytes
                if (!saveScheduled) {
                    saveScheduled = true
                    saveHandler.postDelayed(saveRunnable, (200 - (System.currentTimeMillis() - lastSaveAt)).coerceAtLeast(0))
                }
                return
            }
            "game.start" -> { if (!ready) return; playing = true }
            "game.progress", "game.result" -> {
                if (!ready || !playing) return
                val score = event.optDouble("score", -1.0)
                if (!score.isFinite() || score < 0 || score > 9007199254740991.0 || score % 1.0 != 0.0) return
                if (type == "game.result") playing = false
            }
            "runtime.error" -> {}
            else -> return
        }
        val now = System.currentTimeMillis()
        if (now - lastLogAt >= 1000) { lastLogAt = now; logsThisSecond = 0 }
        if (++logsThisSecond > 10) return
        event.put("time", now).put("gameId", gameId)
        scope.launch(Dispatchers.IO) { runCatching { appendRuntimeLog(event.toString()) } }
    }

    private fun readSave(): JSONObject? = runCatching {
        val file = privateFile("saves/$gameId.json")
        if (!file.isFile || file.length() > MAX_SAVE_BYTES) return@runCatching null
        JSONObject(file.readText(Charsets.UTF_8))
    }.getOrNull()

    /** A bounded 32 KiB atomic write runs serially on Main so finish/onPause can flush it reliably. */
    private fun flushPendingSave() {
        saveHandler.removeCallbacks(saveRunnable)
        saveScheduled = false
        val bytes = pendingSave ?: return
        try {
            writePrivate("saves/$gameId.json", bytes)
            pendingSave = null
            lastSaveAt = System.currentTimeMillis()
            saveErrorShown = false
        } catch (error: Exception) {
            if (!saveErrorShown) {
                saveErrorShown = true
                Toast.makeText(this, "游戏存档保存失败，请检查手机可用空间", Toast.LENGTH_LONG).show()
                runCatching {
                    appendRuntimeLog(JSONObject().put("type", "host.save.error").put("gameId", gameId)
                        .put("time", System.currentTimeMillis()).put("message", error.message.orEmpty().take(300)).toString())
                }
            }
        }
    }

    @Synchronized
    private fun resetRuntimeLog() {
        writePrivate("logs/$gameId.txt", "".toByteArray())
        appendRuntimeLog(JSONObject().put("type", "host.open").put("gameId", gameId).put("time", System.currentTimeMillis()).toString())
    }

    @Synchronized
    private fun appendRuntimeLog(line: String) {
        val file = privateFile("logs/$gameId.txt")
        val old = if (file.isFile && file.length() <= 64 * 1024) file.readLines(Charsets.UTF_8) else emptyList()
        // Keep complete JSON records; slicing serialized JSON can corrupt escaped error messages.
        require(line.toByteArray(Charsets.UTF_8).size <= 40 * 1024)
        val lines = (old + line).takeLast(100).toMutableList()
        while (lines.sumOf { it.toByteArray(Charsets.UTF_8).size + 1 } > 64 * 1024) lines.removeAt(0)
        writePrivate("logs/$gameId.txt", (lines.joinToString("\n") + "\n").toByteArray(Charsets.UTF_8))
    }

    /** Reject symlinks component by component, including games, saves and logs. */
    private fun privateFile(relative: String): File {
        var current = File(filesDir.canonicalFile, "workspace")
        require(!isSymbolic(current) && current.canonicalFile == current.absoluteFile) { "工作区路径不能是符号链接" }
        if (relative.isNotEmpty()) for (part in relative.split('/')) {
            require(part.isNotEmpty() && part != "." && part != ".." && !part.contains('\\')) { "无效路径" }
            current = File(current, part)
            require(!isSymbolic(current) && current.canonicalFile == current.absoluteFile) { "工作区路径不能是符号链接" }
        }
        return current
    }

    private fun isSymbolic(file: File): Boolean = try {
        OsConstants.S_ISLNK(Os.lstat(file.path).st_mode)
    } catch (error: ErrnoException) {
        if (error.errno == OsConstants.ENOENT) false else throw error
    }

    @Synchronized
    private fun writePrivate(relative: String, bytes: ByteArray) {
        val file = privateFile(relative)
        require(file.parentFile?.mkdirs() == true || file.parentFile?.isDirectory == true)
        val temporary = File.createTempFile("xb-game-", ".tmp", file.parentFile)
        try {
            temporary.writeBytes(bytes)
            require(temporary.renameTo(privateFile(relative))) { "无法保存游戏数据" }
        } finally { temporary.delete() }
    }

    @Suppress("DEPRECATION")
    private fun chooseExportLocation() {
        val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "application/zip"
            putExtra(Intent.EXTRA_TITLE, "$gameId.zip")
        }
        runCatching { startActivityForResult(intent, EXPORT_REQUEST) }.onFailure {
            showMessage("当前设备没有可用的文件保存程序", closeButton = true)
        }
    }

    @Deprecated("Deprecated in Java")
    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode != EXPORT_REQUEST) return
        val uri = data?.data
        val bytes = exportBytes
        if (resultCode != RESULT_OK || uri == null || bytes == null) { finish(); return }
        scope.launch {
            val saved = withContext(Dispatchers.IO) {
                runCatching { requireNotNull(contentResolver.openOutputStream(uri, "w")).use { it.write(bytes) } }.isSuccess
            }
            Toast.makeText(this@MobileGameActivity, if (saved) "已导出桌面兼容游戏包" else "游戏包保存失败", Toast.LENGTH_LONG).show()
            finish()
        }
    }

    private fun showMessage(text: String, closeButton: Boolean = false) {
        val layout = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL; gravity = Gravity.CENTER; setPadding(40, 40, 40, 40) }
        layout.addView(TextView(this).apply { this.text = text; textSize = 16f })
        if (closeButton) layout.addView(Button(this).apply { this.text = "返回"; setOnClickListener { finish() } })
        setContentView(layout)
    }

    override fun onPause() {
        activityResumed = false
        flushPendingSave()
        webView?.onPause()
        webView?.pauseTimers()
        super.onPause()
    }

    override fun onResume() {
        super.onResume()
        activityResumed = true
        webView?.resumeTimers()
        webView?.onResume()
    }

    override fun onDestroy() {
        flushPendingSave()
        saveHandler.removeCallbacks(saveRunnable)
        scope.cancel()
        webView?.apply {
            resumeTimers()
            stopLoading()
            (parent as? ViewGroup)?.removeView(this)
            destroy()
        }
        webView = null
        super.onDestroy()
    }

    override fun finish() {
        flushPendingSave()
        super.finish()
    }

    private fun gameUrl(id: String) = "$ORIGIN/games/$id/index.html?hostNonce=$nonce"

    private data class GameSnapshot(
        val id: String, val name: String, val orientation: String, val saveEnabled: Boolean,
        val save: JSONObject?, val files: Map<String, ByteArray>,
    ) {
        fun zip(): ByteArray = ByteArrayOutputStream().use { buffer ->
            ZipOutputStream(buffer).use { zip ->
                files.forEach { (path, bytes) ->
                    zip.putNextEntry(ZipEntry(path))
                    zip.write(bytes)
                    zip.closeEntry()
                }
            }
            buffer.toByteArray()
        }
    }

    companion object {
        const val EXTRA_GAME_ID = "mobile.game.id"
        const val EXTRA_EXPORT = "mobile.game.export"
        private const val EXPORT_REQUEST = 4201
        private const val MAX_SAVE_BYTES = 32 * 1024
        private const val HOST_NAME = "xiaoban-game.invalid"
        private const val ORIGIN = "https://$HOST_NAME"
        private const val HOST_PATH = "/host.html"
        private const val HOST_URL = "$ORIGIN$HOST_PATH"
        // Top-level code is ours. The iframe has an opaque origin and cannot read the private token.
        private const val HOST_CSP = "default-src 'none'; script-src 'nonce-%NONCE%'; style-src 'nonce-%NONCE%'; frame-src $ORIGIN; base-uri 'none'; form-action 'none'"
        private const val GAME_CSP = "default-src 'none'; script-src 'unsafe-inline' $ORIGIN; style-src 'unsafe-inline' $ORIGIN; img-src $ORIGIN data:; media-src $ORIGIN; font-src $ORIGIN; connect-src 'none'; frame-src 'none'; worker-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'"
        private val PATH_PART = Regex("[A-Za-z0-9_-][A-Za-z0-9._-]*")
        private val TEXT_EXTENSIONS = setOf("html", "js", "css", "json", "svg", "md", "txt")

        private fun safeGamePath(path: String): Boolean {
            val parts = path.split('/')
            return parts.size in 1..4 && parts.all { PATH_PART.matches(it) } && path.substringAfterLast('.', "").lowercase() in TEXT_EXTENSIONS
        }

        private fun scriptJson(json: String) = json.replace("<", "\\u003c").replace("\u2028", "\\u2028").replace("\u2029", "\\u2029")
        private fun mime(path: String) = when (path.substringAfterLast('.').lowercase()) {
            "html" -> "text/html"
            "js" -> "application/javascript"
            "css" -> "text/css"
            "json" -> "application/json"
            "svg" -> "image/svg+xml"
            else -> "text/plain"
        }
    }
}
