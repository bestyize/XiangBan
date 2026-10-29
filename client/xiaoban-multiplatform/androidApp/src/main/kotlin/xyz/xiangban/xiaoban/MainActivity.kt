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
import xyz.xiangban.xiaoban.common.community.nativeAgentBridgeOrNull
import xyz.xiangban.xiaoban.common.link.SharedPrefsLinkStorage
import xyz.xiangban.xiaoban.common.link.SqliteRemoteCache

class MainActivity : ComponentActivity() {
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
        val speech = AndroidLocalSpeech(this)
        val platform = AndroidPlatformActions(this, gate = speech::newGate) {
            suspendCoroutine { continuation ->
                micWaiting = continuation
                micLauncher.launch(Manifest.permission.RECORD_AUDIO)
            }
        }
        val linkStorage = SharedPrefsLinkStorage(this)
        val remoteCache = SqliteRemoteCache(this)
        // 小伴要给你设提醒时才会弹出通知权限的请求；没有内核库（本地没编 Rust 库的调试包）就只有纯云端聊天。
        val notifications = registerForActivityResult(ActivityResultContracts.RequestPermission()) {}
        val reminders = AndroidReminders(this) {
            if (Build.VERSION.SDK_INT >= 33) runOnUiThread { notifications.launch(Manifest.permission.POST_NOTIFICATIONS) }
        }
        val agentBridge = nativeAgentBridgeOrNull()
        val agentDir = filesDir.resolve("agent").absolutePath
        setContent {
            App(storage, platform, linkStorage, remoteCache, agentBridge, agentDir, reminders, speech)
        }
    }
}

@Preview
@Composable
fun AppAndroidPreview() {
    App()
}
