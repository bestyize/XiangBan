package xyz.xiangban.xiaoban.business.communitynative

import io.ktor.client.HttpClient
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import xyz.xiangban.xiaoban.common.community.AccountApi
import xyz.xiangban.xiaoban.common.community.CommunityConfig
import xyz.xiangban.xiaoban.common.community.ForumApi
import xyz.xiangban.xiaoban.common.community.InMemoryTokenStorage
import xyz.xiangban.xiaoban.common.community.SessionManager
import xyz.xiangban.xiaoban.common.community.TokenStorage
import xyz.xiangban.xiaoban.common.community.defaultHttpClient

/** 各平台才能做的事：复制到剪贴板、系统分享面板、用浏览器打开链接。不提供时对应按钮不可用。 */
interface PlatformActions {
    fun copy(text: String)
    fun share(title: String, url: String)
    fun openUrl(url: String)
}

/** 把网络层、登录态和平台能力装在一起，界面只拿这一个对象。 */
class CommunityEnv(
    val config: CommunityConfig = CommunityConfig(),
    storage: TokenStorage = InMemoryTokenStorage(),
    val platform: PlatformActions? = null,
    client: HttpClient = defaultHttpClient(),
) {
    val account = AccountApi(client, config)

    @OptIn(ExperimentalTime::class)
    val session = SessionManager(account, storage, now = { Clock.System.now().toEpochMilliseconds() })
    val forum = ForumApi(client, config) { session.accessToken() }
}
