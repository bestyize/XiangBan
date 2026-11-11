package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

private const val CommunityWorkUnsupported = "当前平台暂不支持试玩，可在 Android 或电脑上打开"

@Composable
internal fun GameTrialButton(env: CommunityEnv, id: Long, version: Int? = null, open: ((Route) -> Unit)?, modifier: Modifier = Modifier, label: String = "试玩", appStyle: Boolean = false) {
    Column(modifier) {
        if (env.platform?.communityGamesSupported != true) {
            Text(CommunityWorkUnsupported, style = XType.Caption, color = LocalExtraColors.current.ink3)
            return@Column
        }
        CommunityWorkLaunch(env, id, version) { loading, busy, error, launchGame ->
            val buttonLabel = if (loading) "正在准备试玩…" else label
            if (appStyle) AppActionButton(buttonLabel, launchGame, Modifier.fillMaxWidth(), primary = true, enabled = !busy, icon = XIcons.Play)
            else PillButton(buttonLabel, launchGame, enabled = !busy)
            error?.let { Text(it, Modifier.padding(top = 6.dp), style = XType.Caption, color = Danger) }
        }
    }
}

@Composable
internal fun CommunityWorkLaunch(env: CommunityEnv, id: Long, version: Int? = null,
    openGameDetails: (() -> Unit)? = null,
    content: @Composable (loading: Boolean, busy: Boolean, error: String?, launch: () -> Unit) -> Unit) {
    val scope = rememberCoroutineScope()
    val busy by env.communityGameBusy.collectAsState()
    var loading by remember(id, version) { mutableStateOf(false) }
    var error by remember(id, version) { mutableStateOf<String?>(null) }
    var pendingDownload by remember(id, version) { mutableStateOf<Pair<Long, CompletableDeferred<Boolean>>?>(null) }

    content(loading, busy, error) {
        scope.launch {
            if (env.communityGameBusy.value) return@launch
            loading = true
            error = null
            try {
                openCommunityWork(env, id, version, openGameDetails) { size ->
                    val decision = CompletableDeferred<Boolean>()
                    pendingDownload = size to decision
                    decision.await()
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                error = messageOf(e)
            } finally {
                pendingDownload = null
                loading = false
            }
        }
    }
    pendingDownload?.let { (size, decision) ->
        XiaobanConfirmDialog(
            title = "使用移动网络下载？",
            message = "作品包大小为 ${formatSize(size)}。",
            confirmLabel = "下载",
            onConfirm = { decision.complete(true) },
            onDismiss = { decision.complete(false) },
        )
    }
}

/** Shared by the in-app launch buttons and Android desktop shortcuts. */
suspend fun openCommunityWork(
    env: CommunityEnv,
    id: Long,
    version: Int? = null,
    openGameDetails: (() -> Unit)? = null,
    confirmMobileDownload: suspend (Long) -> Boolean,
) {
    val platform = env.platform
    if (platform?.communityGamesSupported != true) {
        if (openGameDetails != null) return openGameDetails()
        error(CommunityWorkUnsupported)
    }
    if (!env.communityGameBusy.compareAndSet(false, true)) return
    var checkedVersion: Int? = null
    try {
        require(env.config.runtimeV2) { "当前设备暂不支持这类作品，请更新系统 WebView 或 Chrome" }
        if (platform.communityNetworkOffline()) {
            check(platform.playCachedCommunityTool(id, version)) { "没有可用的离线缓存，请联网打开一次" }
            return
        }
        val pack = env.forum.pack(id)
        if (pack.kind !in setOf("game", "app") || pack.status != "published") {
            platform.invalidateCachedCommunityTools(id, null)
            error("作品已不可用")
        }
        if (pack.kind == "game" && openGameDetails != null) return openGameDetails()
        val chosen = version ?: pack.version
        checkedVersion = chosen
        val info = env.forum.gameDownload(id, chosen)
        try {
            info.validate(pack, chosen)
        } catch (e: IllegalArgumentException) {
            platform.invalidateCachedCommunityTools(id, chosen)
            throw e
        }
        if (platform.needsRuntimeV2MobileConfirmation(info) && !confirmMobileDownload(info.size)) return
        check(platform.playCommunityRuntimeV2(info, pack)) { "无法打开作品，请重试" }
    } catch (e: ApiException) {
        if (e.status == 0 && platform.playCachedCommunityTool(id, checkedVersion ?: version)) return
        if (e.status in listOf(403, 404, 410)) platform.invalidateCachedCommunityTools(id, checkedVersion)
        throw e
    } finally {
        env.communityGameBusy.value = false
    }
}

/** Links in the first post are resolved through public pack metadata, just like structured reply cards. */
@Composable
internal fun LinkedGameCards(env: CommunityEnv, html: String, open: ((Route) -> Unit)?, excluded: List<Long> = emptyList()) {
    val ids = remember(html, env.config.forumBase, excluded) { linkedCommunityPacks(html, env.config).filterNot { it in excluded } }
    ids.forEach { id ->
        val loader = rememberLoader(env, "linked-game-$id") { env.forum.pack(id) }
        val state by loader.state.collectAsState()
        val pack = (state as? Load.Ready)?.value
        if (pack != null && pack.kind == "game" && pack.status == "published") WhiteCard(Modifier.padding(top = 10.dp), padding = 12.dp) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Cover(pack.coverUrl, pack.id, Modifier.size(64.dp).clickable { open?.invoke(Route.Pack(id)) })
                Column(Modifier.weight(1f)) {
                    Text(pack.title, Modifier.clickable { open?.invoke(Route.Pack(id)) }, style = XType.Body, fontWeight = FontWeight.Medium)
                    GameTrialButton(env, id, open = open, modifier = Modifier.padding(top = 6.dp))
                }
            }
        }
    }
}

@Composable
internal fun CommunityGameBookmarks(env: CommunityEnv, open: (Route) -> Unit) {
    var recent by rememberSaveable { mutableStateOf(false) }
    val session by env.session.state.collectAsState()
    Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(20.dp)) {
        Text("已收藏", Modifier.clickable { recent = false }.padding(vertical = 8.dp), style = XType.Secondary, color = if (!recent) Brand else LocalExtraColors.current.ink3)
        Text("最近玩过", Modifier.clickable { recent = true }.padding(vertical = 8.dp), style = XType.Secondary, color = if (recent) Brand else LocalExtraColors.current.ink3)
    }
    if (!recent) {
        if (session.loggedIn) ContentBookmarkFeed(env, "game", open)
        else CenterMessage("登录后查看收藏的游戏", "登录", { open(Route.Login) })
        return
    }
    val platform = env.platform
    if (platform?.communityGamesSupported != true) { CenterMessage("当前平台暂不支持游戏试玩"); return }
    val games by platform.recentCommunityGames.collectAsState()
    if (games.isEmpty()) { CenterMessage("还没有试玩记录"); return }
    LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        item { Text("本机最近试玩", style = XType.Caption, color = LocalExtraColors.current.ink3) }
        items(games, key = { it.packId }) { game -> WhiteCard(padding = 14.dp) {
            Text(game.title, Modifier.fillMaxWidth().clickable { open(Route.Pack(game.packId)) }, style = XType.Body, fontWeight = FontWeight.Medium)
            GameTrialButton(env, game.packId, game.version, open, Modifier.padding(top = 8.dp))
        } }
    }
}
