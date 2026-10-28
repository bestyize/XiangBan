package xyz.xiangban.xiaoban.business.communitynative

import io.ktor.client.HttpClient
import io.ktor.client.plugins.HttpTimeout
import io.ktor.client.plugins.websocket.WebSockets
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.AccountApi
import xyz.xiangban.xiaoban.common.community.AiGatewayApi
import xyz.xiangban.xiaoban.common.community.CommunityConfig
import xyz.xiangban.xiaoban.common.community.ForumApi
import xyz.xiangban.xiaoban.common.community.InMemoryTokenStorage
import xyz.xiangban.xiaoban.common.community.SessionManager
import xyz.xiangban.xiaoban.common.community.TokenStorage
import xyz.xiangban.xiaoban.common.link.InMemoryLinkStorage
import xyz.xiangban.xiaoban.common.link.LinkStorage
import xyz.xiangban.xiaoban.common.link.PhoneLink
import xyz.xiangban.xiaoban.common.link.RemoteChatController

/** 各平台才能做的事：复制到剪贴板、系统分享面板、用浏览器打开链接。不提供时对应按钮不可用。 */
interface PlatformActions {
    fun copy(text: String)
    fun share(title: String, url: String)
    fun openUrl(url: String)
}

fun defaultLinkHttpClient(): HttpClient = HttpClient {
    install(HttpTimeout) {
        connectTimeoutMillis = 10_000
        requestTimeoutMillis = 30_000
    }
    install(WebSockets)
    expectSuccess = false
}

/** 把网络层、登录态、互联客户端和平台能力装在一起，界面只拿这一个对象。 */
class CommunityEnv(
    val config: CommunityConfig = CommunityConfig(),
    storage: TokenStorage = InMemoryTokenStorage(),
    val platform: PlatformActions? = null,
    val client: HttpClient = defaultLinkHttpClient(),
    val linkStorage: LinkStorage = InMemoryLinkStorage(),
    val scope: CoroutineScope = CoroutineScope(Dispatchers.Default + SupervisorJob()),
) {
    val account = AccountApi(client, config)

    @OptIn(ExperimentalTime::class)
    val session = SessionManager(account, storage, now = { Clock.System.now().toEpochMilliseconds() })
    val forum = ForumApi(client, config) { session.accessToken() }
    val ai = AiGatewayApi(client, config) { session.accessToken() }

    val phoneLink = PhoneLink(
        scope = scope,
        httpClient = client,
        storage = linkStorage,
        token = { session.accessToken() },
        deviceName = config.deviceName,
    )
    val remoteChat = RemoteChatController(phoneLink, scope)

    init {
        scope.launch {
            phoneLink.start()
        }
    }
}
