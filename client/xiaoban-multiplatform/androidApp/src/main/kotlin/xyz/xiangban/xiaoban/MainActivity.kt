package xyz.xiangban.xiaoban

import android.os.Bundle
import androidx.activity.ComponentActivity
import kotlin.coroutines.Continuation
import kotlin.coroutines.resume
import kotlin.coroutines.suspendCoroutine
import android.Manifest
import android.os.Build
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import xyz.xiangban.xiaoban.business.communitynative.CommunityEnv
import xyz.xiangban.xiaoban.business.communitynative.CommunityNativeApp
import xyz.xiangban.xiaoban.common.community.AndroidMobileWorkspace
import xyz.xiangban.xiaoban.common.community.nativeAgentBridgeOrNull
import xyz.xiangban.xiaoban.common.link.SharedPrefsLinkStorage
import xyz.xiangban.xiaoban.common.link.SqliteRemoteCache

class MainActivity : ComponentActivity() {
    internal lateinit var communityEnv: CommunityEnv
        private set
    private var sharingPlatform: CommunitySharingPlatform? = null
    override fun onNewIntent(intent: android.content.Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        sharingPlatform?.accept(intent)
    }
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        val storage = SharedPrefsTokenStorage(this)
        // 第一次录音时才弹麦克风权限；用户选完以后接着往下走。
        var micWaiting: Continuation<Boolean>? = null
        val micLauncher = registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
            micWaiting?.resume(granted)
            micWaiting = null
        }
        val app = application as XiaobanApplication
        val speech = app.speech
        val platform = AndroidPlatformActions(
            this, gate = speech::newGate, attachments = AndroidChatAttachments(this),
            toolEnvironment = app.toolEnvironment,
        ) {
            suspendCoroutine { continuation ->
                micWaiting = continuation
                micLauncher.launch(Manifest.permission.RECORD_AUDIO)
            }
        }
        val forumPlatform = CommunitySharingPlatform(ForumPickerPlatform(platform, AndroidForumPicker(this)), this)
        sharingPlatform = forumPlatform
        forumPlatform.accept(intent)
        val linkStorage = SharedPrefsLinkStorage(this)
        val remoteCache = SqliteRemoteCache(this)
        // 小伴要给你设提醒时才会弹出通知权限的请求；没有内核库（本地没编 Rust 库的调试包）就只有纯云端聊天。
        val notifications = registerForActivityResult(ActivityResultContracts.RequestPermission()) {}
        val reminders = AndroidReminders(this) {
            if (Build.VERSION.SDK_INT >= 33) runOnUiThread { notifications.launch(Manifest.permission.POST_NOTIFICATIONS) }
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
        hideNavigationBar()
        (application as XiaobanApplication).prefetchSpeechResources()
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
