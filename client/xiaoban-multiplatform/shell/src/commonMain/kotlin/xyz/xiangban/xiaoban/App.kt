package xyz.xiangban.xiaoban

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import xyz.xiangban.xiaoban.business.communitynative.AppPrefs
import xyz.xiangban.xiaoban.business.communitynative.CommunityEnv
import xyz.xiangban.xiaoban.business.communitynative.InMemoryPrefs
import xyz.xiangban.xiaoban.business.communitynative.CommunityNativeApp
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions
import xyz.xiangban.xiaoban.common.community.AgentBridge
import xyz.xiangban.xiaoban.common.community.ChatHistoryFiles
import xyz.xiangban.xiaoban.common.community.InMemoryChatHistoryFiles
import xyz.xiangban.xiaoban.common.community.InMemoryTokenStorage
import xyz.xiangban.xiaoban.common.community.LocalSpeech
import xyz.xiangban.xiaoban.common.community.ReminderScheduler
import xyz.xiangban.xiaoban.common.community.TokenStorage
import xyz.xiangban.xiaoban.common.link.InMemoryLinkStorage
import xyz.xiangban.xiaoban.common.link.InMemoryRemoteCache
import xyz.xiangban.xiaoban.common.link.LinkStorage
import xyz.xiangban.xiaoban.common.link.RemoteCache

/** 令牌存储和系统能力由各平台入口传入；不传就用内存存储（退出应用即登出）、无分享。 */
@Composable
fun App(
    storage: TokenStorage = InMemoryTokenStorage(),
    platform: PlatformActions? = null,
    linkStorage: LinkStorage = InMemoryLinkStorage(),
    remoteCache: RemoteCache = InMemoryRemoteCache(),
    agentBridge: AgentBridge? = null,
    agentDir: String? = null,
    reminders: ReminderScheduler? = null,
    speech: LocalSpeech? = null,
    prefs: AppPrefs = InMemoryPrefs(),
    chatFiles: ChatHistoryFiles = InMemoryChatHistoryFiles(),
) {
    val env = remember(storage, platform, linkStorage, remoteCache, agentBridge, agentDir, reminders, speech, prefs, chatFiles) {
        CommunityEnv(storage = storage, platform = platform, linkStorage = linkStorage, remoteCache = remoteCache, agentBridge = agentBridge, agentDir = agentDir, reminders = reminders, speech = speech, prefs = prefs, chatFiles = chatFiles)
    }
    CommunityNativeApp(env)
}
