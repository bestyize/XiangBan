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
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

@Composable
internal fun GameTrialButton(env: CommunityEnv, id: Long, version: Int? = null, open: ((Route) -> Unit)?, modifier: Modifier = Modifier) {
    val scope = rememberCoroutineScope()
    val busy by env.communityGameBusy.collectAsState()
    var loading by remember(id, version) { mutableStateOf(false) }
    var error by remember(id, version) { mutableStateOf<String?>(null) }
    Column(modifier) {
        if (env.platform?.communityGamesSupported != true) {
            Text("当前平台暂不支持试玩，可在 Android 或电脑上打开", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            return@Column
        }
        PillButton(if (loading) "正在准备试玩…" else "试玩", {
            if (busy) return@PillButton
            env.communityGameBusy.value = true; loading = true; error = null
            scope.launch {
                try {
                    // Recheck publication and exact version even on cache hits. Removed works cannot run from stale cards.
                    val pack = env.forum.pack(id)
                    val chosen = version ?: pack.version
                    require(pack.kind == "game" && pack.status == "published") { "游戏已不可用" }
                    val info = env.forum.gameDownload(id, chosen)
                    info.validate(pack, chosen)
                    val launched = env.platform.playCommunityGame(info, pack.title, bookmark = { next ->
                        if (!env.session.state.value.loggedIn) throw ApiException(401, "请先登录后收藏")
                        if (next == null) env.contentSocial.state("pack", id).bookmarked
                        else env.contentSocial.bookmark("pack", id, next).bookmarked.also { env.contentBookmarkRevision.value++ }
                    }, login = { open?.invoke(Route.Login) })
                    check(launched) { "无法打开试玩，请重试" }
                } catch (e: CancellationException) { throw e }
                catch (e: Exception) { error = messageOf(e) }
                finally { loading = false; env.communityGameBusy.value = false }
            }
        }, enabled = !busy)
        error?.let { Text(it, Modifier.padding(top = 6.dp), fontSize = 12.sp, color = Danger) }
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
                    Text(pack.title, Modifier.clickable { open?.invoke(Route.Pack(id)) }, fontSize = 14.sp, fontWeight = FontWeight.Medium)
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
        Text("已收藏", Modifier.clickable { recent = false }.padding(vertical = 8.dp), fontSize = 13.sp, color = if (!recent) Brand else LocalExtraColors.current.ink3)
        Text("最近玩过", Modifier.clickable { recent = true }.padding(vertical = 8.dp), fontSize = 13.sp, color = if (recent) Brand else LocalExtraColors.current.ink3)
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
        item { Text("本机最近试玩", fontSize = 12.sp, color = LocalExtraColors.current.ink3) }
        items(games, key = { it.packId }) { game -> WhiteCard(padding = 14.dp) {
            Text(game.title, Modifier.fillMaxWidth().clickable { open(Route.Pack(game.packId)) }, fontSize = 15.sp, fontWeight = FontWeight.Medium)
            GameTrialButton(env, game.packId, game.version, open, Modifier.padding(top = 8.dp))
        } }
    }
}
