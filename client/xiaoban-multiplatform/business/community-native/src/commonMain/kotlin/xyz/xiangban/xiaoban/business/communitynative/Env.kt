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
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import xyz.xiangban.xiaoban.common.community.AccountApi
import xyz.xiangban.xiaoban.common.community.AgentBridge
import xyz.xiangban.xiaoban.common.community.AgentRuntime
import xyz.xiangban.xiaoban.common.community.ReminderScheduler
import xyz.xiangban.xiaoban.common.community.LocalSpeech
import xyz.xiangban.xiaoban.common.community.SpeechModels
import xyz.xiangban.xiaoban.common.community.VoicePlatform
import xyz.xiangban.xiaoban.common.community.SetReminderTool
import xyz.xiangban.xiaoban.common.community.postChatLines
import xyz.xiangban.xiaoban.common.community.AiGatewayApi
import xyz.xiangban.xiaoban.common.community.ChatHistoryFiles
import xyz.xiangban.xiaoban.common.community.InMemoryChatHistoryFiles
import xyz.xiangban.xiaoban.common.community.CommunityConfig
import xyz.xiangban.xiaoban.common.community.ForumApi
import xyz.xiangban.xiaoban.common.community.InMemoryTokenStorage
import xyz.xiangban.xiaoban.common.community.SessionManager
import xyz.xiangban.xiaoban.common.community.TokenStorage
import xyz.xiangban.xiaoban.common.link.InMemoryLinkStorage
import xyz.xiangban.xiaoban.common.link.InMemoryRemoteCache
import xyz.xiangban.xiaoban.common.link.LinkStorage
import xyz.xiangban.xiaoban.common.link.RemoteCache
import xyz.xiangban.xiaoban.common.link.PhoneLink
import xyz.xiangban.xiaoban.common.link.RemoteChatController

/** 各平台才能做的事：复制到剪贴板、系统分享面板、用浏览器打开链接。不提供时对应按钮不可用。 */
interface PlatformActions : VoicePlatform {
    /** 未接系统文件选择器的平台（当前 iOS）明确降级到网页，不申请全盘访问权限。 */
    val forumFilePickerSupported: Boolean get() = false
    suspend fun pickForumFile(image: Boolean, maxBytes: Long): xyz.xiangban.xiaoban.common.community.ForumUploadFile? = null
    fun copy(text: String)
    fun share(title: String, url: String)
    fun openUrl(url: String)
    /** 结束应用（最外面一页按返回）；平台不支持就什么也不做。 */
    fun exitApp() {}
    /** 横屏全屏的页面（散点图）进入时锁成横屏，离开时恢复；平台不支持就什么也不做。 */
    fun setLandscape(landscape: Boolean) {}
    /** 打开系统里这个应用的设置页（让用户去开麦克风权限）；平台不支持就返回 false。 */
    fun openAppSettings(): Boolean = false
    /** 朗读一段文字；平台不支持就返回 false。 */
    fun speak(text: String): Boolean = false

    /** 用系统朗读念一段回复（按媒体音量，不是通话音量），念完回调 [onDone]；平台不支持返回 false。 */
    fun speakReply(text: String, onDone: (Boolean) -> Unit): Boolean = speakText(text, onDone)

    /** 开始录音（16kHz、单声道、16 位小端 PCM）；第一次会先向用户要麦克风权限。返回是否已经开始录。 */
    suspend fun startRecording(): Boolean = false
    /** 结束录音，拿到录下来的声音；没在录就是 null。 */
    fun stopRecording(): ByteArray? = null
}

/** 界面偏好（外观、主题色）的存放处；平台用自己的轻量存储实现，不提供就只在本次运行里记着。 */
interface AppPrefs {
    fun get(key: String): String?
    fun put(key: String, value: String)
}

class InMemoryPrefs : AppPrefs {
    private val values = mutableMapOf<String, String>()
    override fun get(key: String) = values[key]
    override fun put(key: String, value: String) { values[key] = value }
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
    val remoteCache: RemoteCache = InMemoryRemoteCache(),
    val scope: CoroutineScope = CoroutineScope(Dispatchers.Default + SupervisorJob()),
    /** 手机里的智能体内核（Android 上是 Rust 库）；没有就是纯云端聊天。 */
    agentBridge: AgentBridge? = null,
    /** 智能体保存对话的文件夹。 */
    agentDir: String? = null,
    /** 系统闹钟提醒；智能体的“设提醒”和追踪页的“提醒我”都用它。没有就不提供提醒。 */
    val reminders: ReminderScheduler? = null,
    /** 手机本地语音识别（安卓上是 sherpa-onnx 的 SenseVoice）；没有就只用云端和电脑上的识别。 */
    val speech: LocalSpeech? = null,
    val prefs: AppPrefs = InMemoryPrefs(),
    /** 小伴AI 的对话历史存在哪里；不提供就只在这次打开应用期间记着。 */
    val chatFiles: ChatHistoryFiles = InMemoryChatHistoryFiles(),
) {
    /** 对话历史是不是会留到下次打开应用。 */
    val chatsKept: Boolean get() = chatFiles !is InMemoryChatHistoryFiles

    /** 朗读回复：回复下面的“朗读”、自动朗读和设置里的试听都走这一个，来源、音色、语速按声音设置。 */
    val reader: ReplyReader by lazy { ReplyReader(this) }

    val account = AccountApi(client, config)

    @OptIn(ExperimentalTime::class)
    private var accountChanged: (String?, String?) -> Unit = { _, _ -> }
    val session = SessionManager(account, storage, now = { Clock.System.now().toEpochMilliseconds() }, onAccountChanged = { old, new -> accountChanged(old, new) })
    val forum = ForumApi(client, config) { session.accessToken() }
    val content = xyz.xiangban.xiaoban.common.community.ContentPublishingApi(client, config) { session.accessToken() }
    val social = xyz.xiangban.xiaoban.common.community.SocialApi(client, config) { session.accessToken() }
    val challenges = xyz.xiangban.xiaoban.common.community.ChallengesApi(client, config) { session.accessToken() }
    val ai = AiGatewayApi(client, config) { session.accessToken() }

    @OptIn(ExperimentalTime::class)
    val agent: AgentRuntime? = if (agentBridge != null && agentDir != null) {
        val tools = listOfNotNull(reminders?.let { SetReminderTool(it) { Clock.System.now().toEpochMilliseconds() } })
        AgentRuntime(agentBridge, { body, onLine -> ai.postChatLines(body, onLine) }, tools, agentDir, AGENT_SYSTEM_PROMPT, scope)
    } else null

    val phoneLink = PhoneLink(
        scope = scope,
        httpClient = client,
        storage = linkStorage,
        token = { session.accessToken() },
        deviceName = config.deviceName,
        baseUrl = config.linkBase,
    )
    /** 本地语音识别模型的下载与状态。 */
    val speechModels: SpeechModels? = speech?.let { SpeechModels(client, it.files) }

    val remoteChat = RemoteChatController(phoneLink, scope, remoteCache, session.state.value.accountId)

    init {
        val resetLock = Mutex()
        accountChanged = { old, new ->
            remoteChat.changeAccount(new)
            if (old != null && old != new) remoteCache.clearAccount(old)
            scope.launch { resetLock.withLock { phoneLink.resetForAccount() } }
        }
        scope.launch {
            phoneLink.start()
        }
    }
}

private const val AGENT_SYSTEM_PROMPT =
    "你是小伴，用户随身的温和、真诚、靠谱的 AI 伙伴。回答直接清晰、重点突出，默认使用简体中文。" +
        "你可以用工具帮用户做事；会改动手机上东西的操作，系统会先请用户确认。用户拒绝了就不要再试，直接告诉用户没有执行。"
