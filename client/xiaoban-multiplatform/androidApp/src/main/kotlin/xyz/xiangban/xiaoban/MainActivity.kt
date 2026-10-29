package xyz.xiangban.xiaoban

import android.os.Bundle
import androidx.activity.ComponentActivity
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
        val platform = AndroidPlatformActions(this)
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
            App(storage, platform, linkStorage, remoteCache, agentBridge, agentDir, reminders)
        }
    }
}

@Preview
@Composable
fun AppAndroidPreview() {
    App()
}
