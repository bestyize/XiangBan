package xyz.xiangban.xiaoban

import android.os.Bundle
import androidx.activity.ComponentActivity
import kotlin.coroutines.resume
import android.Manifest
import android.app.AlertDialog
import android.content.Intent
import android.os.Build
import android.text.format.Formatter
import android.widget.Toast
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.Lifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.business.communitynative.CommunityEnv
import xyz.xiangban.xiaoban.business.communitynative.CommunityNativeApp
import xyz.xiangban.xiaoban.business.communitynative.openCommunityWork
import xyz.xiangban.xiaoban.common.community.AndroidMobileWorkspace
import xyz.xiangban.xiaoban.common.community.nativeAgentBridgeOrNull
import xyz.xiangban.xiaoban.common.link.SharedPrefsLinkStorage
import xyz.xiangban.xiaoban.common.link.SqliteRemoteCache

class MainActivity : ComponentActivity() {
    internal lateinit var communityEnv: CommunityEnv
        private set
    private var sharingPlatform: CommunitySharingPlatform? = null
    private var shortcutJob: Job? = null
    private lateinit var reminders: AndroidReminders
    private var voicePlatform: AndroidPlatformActions? = null

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        sharingPlatform?.accept(intent)
        openWorkShortcut(intent)
    }
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        val storage = SharedPrefsTokenStorage(this)
        // 第一次录音时才弹麦克风权限；用户选完以后接着往下走。
        var micWaiting: CancellableContinuation<Boolean>? = null
        val micLauncher = registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
            val waiting = micWaiting
            micWaiting = null
            waiting?.takeIf { it.isActive }?.resume(granted)
        }
        val app = application as XiaobanApplication
        val speech = app.speech
        val platform = AndroidPlatformActions(
            this, gate = speech::newGate, attachments = AndroidChatAttachments(this),
            toolEnvironment = app.toolEnvironment,
        ) {
            withContext(Dispatchers.Main.immediate) {
                suspendCancellableCoroutine { continuation ->
                    if (micWaiting != null || !lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)) {
                        continuation.resume(false)
                        return@suspendCancellableCoroutine
                    }
                    micWaiting = continuation
                    try { micLauncher.launch(Manifest.permission.RECORD_AUDIO) }
                    catch (_: IllegalStateException) {
                        micWaiting = null
                        continuation.resume(false)
                    }
                }
            }
        }
        voicePlatform = platform
        val forumPlatform = CommunitySharingPlatform(ForumPickerPlatform(platform, AndroidForumPicker(this)), this)
        sharingPlatform = forumPlatform
        forumPlatform.accept(intent)
        val linkStorage = SharedPrefsLinkStorage(this)
        val remoteCache = SqliteRemoteCache(this)
        var askingNotifications = false
        val notifications = registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
            askingNotifications = false
            lifecycleScope.launch {
                val (scheduled, failed) = withContext(Dispatchers.IO) { reminders.permissionResult(granted) }
                val message = when {
                    scheduled > 0 && failed > 0 -> "已安排 $scheduled 条提醒，另有 $failed 条未能设置"
                    scheduled > 0 -> "已安排 $scheduled 条提醒，系统可能延后送达"
                    failed > 0 -> "提醒未设置，请检查小伴通知权限和提醒渠道"
                    else -> null
                }
                message?.let { Toast.makeText(this@MainActivity, it, Toast.LENGTH_SHORT).show() }
            }
        }
        reminders = AndroidReminders(this) {
            if (Build.VERSION.SDK_INT >= 33) runOnUiThread {
                if (!askingNotifications && lifecycle.currentState.isAtLeast(Lifecycle.State.STARTED)) {
                    askingNotifications = true
                    try { notifications.launch(Manifest.permission.POST_NOTIFICATIONS) }
                    catch (_: IllegalStateException) { askingNotifications = false }
                }
            }
        }
        val agentBridge = nativeAgentBridgeOrNull()
        val agentDir = filesDir.resolve("agent").absolutePath
        val prefs = AndroidPrefs(this)
        val chatFiles = AndroidChatHistoryFiles(this)
        val workspace = AndroidMobileWorkspace(filesDir.resolve("workspace"))
        val utcOffset = java.util.TimeZone.getDefault().getOffset(System.currentTimeMillis()) / 60_000
        // Debug only: adb reverse can preview an undeployed forum API on this computer.
        // This is process-local, accepts loopback only, and never changes release configuration.
        val previewBase = if (BuildConfig.DEBUG) intent.getStringExtra("forum_preview_base")?.takeIf {
            it.matches(Regex("http://127\\.0\\.0\\.1:[0-9]{2,5}")) && it.substringAfterLast(':').toInt() in 1024..65535
        } else null
        val communityConfig = xyz.xiangban.xiaoban.common.community.CommunityConfig(runtimeV2 = RuntimeV2Activity.supported(this)).let { config ->
            if (previewBase == null) config else config.copy(forumBase = previewBase)
        }
        communityEnv = CommunityEnv(
            config = communityConfig, storage = storage, platform = forumPlatform,
            linkStorage = linkStorage, remoteCache = remoteCache,
            scope = CoroutineScope(lifecycleScope.coroutineContext + Dispatchers.Default),
            agentBridge = agentBridge, agentDir = agentDir, reminders = reminders,
            speech = speech, prefs = prefs, chatFiles = chatFiles, workspace = workspace, utcOffsetMinutes = utcOffset,
        )
        setContent { CommunityNativeApp(communityEnv) }
        if (savedInstanceState == null) openWorkShortcut(intent)
        hideNavigationBar()
        (application as XiaobanApplication).prefetchSpeechResources()
    }

    override fun onStart() {
        super.onStart()
        voicePlatform?.setVoiceForeground(true)
        if (::reminders.isInitialized) lifecycleScope.launch(Dispatchers.IO) { reminders.restore() }
    }

    override fun onStop() {
        voicePlatform?.setVoiceForeground(false)
        if (::communityEnv.isInitialized) communityEnv.reader.stop()
        super.onStop()
    }

    override fun onDestroy() {
        voicePlatform?.releaseVoice()
        voicePlatform = null
        super.onDestroy()
    }

    private fun openWorkShortcut(intent: Intent) {
        if (Build.VERSION.SDK_INT < 26 || intent.action != RuntimeWorkShortcut.ACTION_OPEN || shortcutJob?.isActive == true) return
        shortcutJob = lifecycleScope.launch {
            try {
                val shortcut = withContext(Dispatchers.IO) { RuntimeWorkShortcut.fromPinnedIntent(this@MainActivity, intent) }
                    ?: error("桌面快捷方式已失效")
                val packId = shortcut.communityId
                if (packId != null) {
                    openCommunityWork(communityEnv, packId, confirmMobileDownload = ::confirmShortcutDownload)
                } else {
                    check(shortcut.matchesLocalSource() && AndroidGameLauncher.open(this@MainActivity, shortcut.id)) { "无法打开这个作品" }
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Toast.makeText(this@MainActivity, e.message ?: "无法打开作品，请重试", Toast.LENGTH_SHORT).show()
            }
        }
    }

    private suspend fun confirmShortcutDownload(size: Long): Boolean = suspendCancellableCoroutine { continuation ->
        val dialog = AlertDialog.Builder(this)
            .setTitle("使用移动网络下载？")
            .setMessage("作品包大小为 ${Formatter.formatFileSize(this, size)}。")
            .setNegativeButton("取消", null)
            .setPositiveButton("下载") { _, _ -> if (continuation.isActive) continuation.resume(true) }
            .setOnDismissListener { if (continuation.isActive) continuation.resume(false) }
            .create()
        continuation.invokeOnCancellation { runOnUiThread { dialog.dismiss() } }
        dialog.show()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) hideNavigationBar()
    }
}

@Preview
@Composable
fun AppAndroidPreview() {
    App()
}
