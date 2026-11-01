package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

internal fun contentCardRoute(kind: String, card: CommunityContentCard): Route =
    if (kind == "scheme") Route.Scheme(card.id) else if (card.kind in CapabilityPackKinds) Route.Ability(card.id) else Route.Pack(card.id)

@Composable
internal fun ContentSocialActions(env: CommunityEnv, kind: String, id: Long, open: ((Route) -> Unit)?) {
    val session by env.session.state.collectAsState()
    if (!session.loggedIn) {
        GhostPillButton("登录后收藏或举报", { open?.invoke(Route.Login) }, Modifier.padding(top = 10.dp), enabled = open != null)
        return
    }
    val loader = rememberLoader(env, "content-social-$kind-$id-${session.accountId}") { env.contentSocial.state(kind, id) }
    val state by loader.state.collectAsState()
    val scope = rememberCoroutineScope()
    var busy by remember(session.accountId, kind, id) { mutableStateOf(false) }
    var error by remember(session.accountId, kind, id) { mutableStateOf<String?>(null) }
    Column(Modifier.padding(top = 10.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        when (val current = state) {
            is Load.Ready -> Row(horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                GhostPillButton(if (current.value.bookmarked) "取消收藏" else "收藏", {
                    if (busy) return@GhostPillButton
                    busy = true; error = null
                    scope.launch {
                        try { env.contentSocial.bookmark(kind, id, !current.value.bookmarked); loader.reload() }
                        catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                        finally { busy = false }
                    }
                }, enabled = !busy)
                Box(Modifier.padding(top = 14.dp)) { NativeReportAction(env, kind, id, open) }
            }
            is Load.Failed -> Banner(if (current.notFound) "内容已不可用，无法收藏或举报" else current.message, warn = true, actionText = "重试", onAction = loader::reload)
            else -> Text("正在读取收藏状态…", style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        error?.let { Text(it, color = Danger, style = XType.Secondary) }
    }
}

@Composable
internal fun ContentBookmarkFeed(env: CommunityEnv, kind: String, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    var page by rememberSaveable(kind) { mutableStateOf(1) }
    val revision by env.contentBookmarkRevision.collectAsState()
    val loader = rememberLoader(env, "content-bookmarks-$kind-$page-${session.accountId}-$revision") { env.contentSocial.bookmarks(kind, page) }
    val state by loader.state.collectAsState()
    val scope = rememberCoroutineScope()
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    error?.let { Banner(it, Modifier.padding(12.dp), warn = true) }
    Box(Modifier.fillMaxSize()) { LoadBox(state, loader::reload) { result ->
        if (result.items.isEmpty()) CenterMessage(if (page > 1) "这一页没有收藏" else "还没有收藏${when (kind) { "pack" -> "作品"; "game" -> "游戏"; else -> "方案" }}", if (page > 1) "上一页" else "刷新", { if (page > 1) page-- else loader.reload() })
        else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            item { GhostPillButton("刷新收藏", loader::reload, Modifier.fillMaxWidth()) }
            items(result.items, key = { "${it.kind}-${it.id}" }) { entry -> WhiteCard(padding = 14.dp) {
                Text(entry.content.title, Modifier.fillMaxWidth().clickable { open(contentCardRoute(entry.kind, entry.content)) }, style = XType.Section, fontWeight = FontWeight.SemiBold)
                if (entry.kind == "pack") entry.content.version?.let { Text("版本 v$it", style = XType.Caption, color = LocalExtraColors.current.ink3) }
                if (entry.content.kind == "game") GameTrialButton(env, entry.id, open = open, modifier = Modifier.padding(top = 8.dp))
                GhostPillButton("取消收藏", {
                    if (busy) return@GhostPillButton
                    busy = true; error = null
                    scope.launch { try { env.contentSocial.bookmark(entry.kind, entry.id, false); loader.reload() }
                        catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                        finally { busy = false } }
                }, Modifier.padding(top = 8.dp), enabled = !busy)
            } }
            item { Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                GhostPillButton("上一页", { page-- }, Modifier.weight(1f), enabled = page > 1)
                Text("第 $page 页", Modifier.padding(top = 10.dp), style = XType.Caption)
                GhostPillButton("下一页", { page++ }, Modifier.weight(1f), enabled = result.hasMore)
            } }
        }
    } }
}
