package xyz.xiangban.xiaoban

import android.app.Application
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.business.communitynative.defaultLinkHttpClient
import xyz.xiangban.xiaoban.common.community.CommunityConfig
import xyz.xiangban.xiaoban.common.community.ForumApi

/** Application 管理进程级任务；用户打开主界面后才启动，提醒广播等系统唤醒不会触发下载。 */
class XiaobanApplication : Application() {
    val speech by lazy { AndroidLocalSpeech(this) }
    internal val toolEnvironment by lazy { AndroidToolEnvironment(this) }
    private val downloadScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private var prefetchStarted = false
    private var ideaPrefetch: Job? = null

    @Synchronized
    fun prefetchSpeechResources() {
        if (prefetchStarted) return
        prefetchStarted = true
        downloadScope.launch {
            val client = defaultLinkHttpClient()
            // 下载器随后也被声音设置页复用，客户端保持到进程退出。
            speech.downloads(client).prefetchSilently()
        }
        val connectivity = getSystemService(ConnectivityManager::class.java)
        val unmetered = connectivity.getNetworkCapabilities(connectivity.activeNetwork)
            ?.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_METERED) == true
        if (unmetered) ideaPrefetch = downloadScope.launch {
            val client = defaultLinkHttpClient()
            try {
                val api = ForumApi(client, CommunityConfig()) { null }
                for (id in listOf("saltline", "starry-nightrider", "oil-paint", "plush-rabbit", "aquarium")) {
                    runCatching { AndroidRuntimeV2Community.prefetchIdea(this@XiaobanApplication, api.ideaRuntime(id)) }
                    currentCoroutineContext().ensureActive()
                }
            } finally {
                client.close()
            }
        }
    }

    suspend fun cancelIdeaPrefetch() {
        val job = synchronized(this) { ideaPrefetch.also { ideaPrefetch = null } }
        job?.cancelAndJoin()
    }
}
