package xyz.xiangban.xiaoban

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import xyz.xiangban.xiaoban.business.communitynative.CommunityEnv
import xyz.xiangban.xiaoban.business.communitynative.CommunityNativeApp
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions
import xyz.xiangban.xiaoban.common.community.AgentBridge
import xyz.xiangban.xiaoban.common.community.InMemoryTokenStorage
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
) {
    val env = remember(storage, platform, linkStorage, remoteCache, agentBridge, agentDir, reminders) {
        CommunityEnv(storage = storage, platform = platform, linkStorage = linkStorage, remoteCache = remoteCache, agentBridge = agentBridge, agentDir = agentDir, reminders = reminders)
    }
    CommunityNativeApp(env)
}
