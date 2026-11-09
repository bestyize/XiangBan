package xyz.xiangban.xiaoban

import android.content.Intent
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.ResultReceiver
import androidx.core.content.IntentCompat
import androidx.core.os.BundleCompat
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.AppShortcut

/** Keep account access and bookmark state in the main process. The receiver is bound to the launched work. */
internal object RuntimeV2Favorites {
    private const val FAVORITES = "runtime.v2.favorites"
    private const val REPLY = "reply"
    private const val SAVED = "saved"
    private const val TITLE = "title"
    private const val ERROR = "error"
    private const val FAILURE = "收藏操作失败，请稍后重试"

    fun attach(intent: Intent, activity: MainActivity, source: String, id: String) {
        intent.putExtra(FAVORITES, object : ResultReceiver(Handler(Looper.getMainLooper())) {
            override fun onReceiveResult(resultCode: Int, resultData: Bundle?) {
                val request = resultData ?: return
                val reply = BundleCompat.getParcelable(request, REPLY, ResultReceiver::class.java) ?: return
                activity.lifecycleScope.launch {
                    val result = Bundle()
                    try {
                        val env = activity.communityEnv
                        val saved = if (source == "community") {
                            val packId = id.toLong()
                            if (!env.session.state.value.loggedIn) throw ApiException(401, "请登录后收藏社区作品")
                            if (request.containsKey(SAVED)) {
                                env.contentSocial.bookmark("pack", packId, request.getBoolean(SAVED)).bookmarked.also {
                                    env.contentBookmarkRevision.value++
                                }
                            } else env.contentSocial.state("pack", packId).bookmarked
                        } else if (request.containsKey(SAVED)) {
                            val item = AppShortcut(source, id, request.getString(TITLE).orEmpty())
                            require(item.valid)
                            env.setLocalAppFavorite(item, request.getBoolean(SAVED))
                        } else env.isLocalAppFavorite(source, id)
                        result.putBoolean(SAVED, saved)
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: ApiException) {
                        result.putString(ERROR, if (e.status == 401) "请登录后收藏社区作品" else FAILURE)
                    } catch (_: Exception) {
                        result.putString(ERROR, FAILURE)
                    }
                    reply.send(0, result)
                }
            }
        })
    }

    fun receiver(intent: Intent): ResultReceiver? =
        IntentCompat.getParcelableExtra(intent, FAVORITES, ResultReceiver::class.java)

    suspend fun request(receiver: ResultReceiver, title: String, saved: Boolean? = null): Boolean {
        val response = CompletableDeferred<Bundle>()
        val reply = object : ResultReceiver(Handler(Looper.getMainLooper())) {
            override fun onReceiveResult(resultCode: Int, resultData: Bundle?) {
                response.complete(resultData ?: Bundle.EMPTY)
            }
        }
        receiver.send(0, Bundle().apply {
            putParcelable(REPLY, reply)
            if (saved != null) {
                putBoolean(SAVED, saved)
                putString(TITLE, title)
            }
        })
        val result = withTimeoutOrNull(40_000) { response.await() }
        check(result != null) { "收藏请求超时，请重试" }
        check(result.containsKey(SAVED)) { result.getString(ERROR) ?: FAILURE }
        return result.getBoolean(SAVED)
    }
}
