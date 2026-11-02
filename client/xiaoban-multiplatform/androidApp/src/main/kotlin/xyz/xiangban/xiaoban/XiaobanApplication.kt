package xyz.xiangban.xiaoban

import android.app.Application
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.business.communitynative.defaultLinkHttpClient

/** Application 管理进程级任务；用户打开主界面后才启动，提醒广播等系统唤醒不会触发下载。 */
class XiaobanApplication : Application() {
    val speech by lazy { AndroidLocalSpeech(this) }
    private val downloadScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var prefetchStarted = false

    @Synchronized
    fun prefetchSpeechResources() {
        if (prefetchStarted) return
        prefetchStarted = true
        downloadScope.launch {
            val client = defaultLinkHttpClient()
            // 下载器随后也被声音设置页复用，客户端保持到进程退出。
            speech.downloads(client).prefetchSilently()
        }
    }
}
