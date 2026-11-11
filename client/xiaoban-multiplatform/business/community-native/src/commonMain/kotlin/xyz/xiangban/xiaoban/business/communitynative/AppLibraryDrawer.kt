package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.*

@Composable
internal fun AppLibraryDrawer(
    env: CommunityEnv, available: List<AppShortcut>, loading: Boolean, errors: List<String>,
    reload: () -> Unit, dismiss: () -> Unit, open: (Route) -> Unit,
) {
    val session by env.session.state.collectAsState()
    val bookmarkRevision by env.contentBookmarkRevision.collectAsState()
    var page by rememberSaveable(session.accountId, bookmarkRevision) { mutableIntStateOf(1) }
    var bookmarks by remember(session.accountId) { mutableStateOf(emptyList<AppShortcut>()) }
    val bookmarkLoader = rememberLoader(env, "app-drawer-favorites-${session.accountId}-$page-$bookmarkRevision") {
        if (session.loggedIn) {
            val games = env.contentSocial.bookmarks("game", page)
            val apps = env.contentSocial.bookmarks("app", page)
            (games.items + apps.items).sortedByDescending { it.createdAt } to (games.hasMore || apps.hasMore)
        } else emptyList<ContentBookmark>() to false
    }
    val bookmarkState by bookmarkLoader.state.collectAsState()
    LaunchedEffect(bookmarkState) {
        (bookmarkState as? Load.Ready)?.value?.let { result -> bookmarks = ((if (page == 1) emptyList() else bookmarks) + result.first.map {
            AppShortcut("community", it.id.toString(), it.content.title, it.content.coverUrl)
        }).distinctBy { it.key } }
    }
    val recentIds = env.platform?.recentGames?.collectAsState()?.value.orEmpty()
    val community = env.platform?.recentCommunityGames?.collectAsState()?.value.orEmpty()
    val recent = recentAppShortcuts(recentIds, available, community)
    val favorites = env.appLibrary.favorites.mapNotNull { saved -> available.firstOrNull { it.key == saved.key } } + bookmarks
    var message by remember { mutableStateOf<String?>(null) }
    fun choose(item: AppShortcut) {
        if (item.source == "community") { dismiss(); open(Route.Pack(item.id.toLong())) }
        else if (env.platform?.openGame(item.id) == true) dismiss() else message = "当前平台无法打开游戏"
    }
    Column(Modifier.fillMaxSize().clip(RoundedCornerShape(bottomStart = 16.dp, bottomEnd = 16.dp))
        .background(MaterialTheme.colorScheme.surface).statusBarsPadding()
        .padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 16.dp)) {
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("最近用过", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
                IconTap(XIcons.Close, "收起最近用过", dismiss)
            }
            if (loading) Spinner(Modifier.fillMaxWidth().height(48.dp))
            else if (recent.isEmpty()) Text("还没有使用记录", style = XType.Secondary, color = LocalExtraColors.current.ink3)
            AppShortcutGrid(env, recent, recentItems = true, choose = ::choose)
            if (errors.isNotEmpty()) Banner(errors.joinToString(" · "), warn = true, actionText = "重试", onAction = reload)
            Text("我的收藏", Modifier.padding(top = 12.dp), style = XType.Section, fontWeight = FontWeight.SemiBold)
            AppShortcutGrid(env, favorites, choose = ::choose)
            when (val current = bookmarkState) {
                is Load.Ready -> {
                    if (favorites.isEmpty()) Text("还没有收藏的应用", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                    if (current.value.second) AppActionButton("加载更多收藏", { page++ }, Modifier.fillMaxWidth())
                }
                is Load.Failed -> Banner(current.message, warn = true, actionText = "重试", onAction = bookmarkLoader::reload)
                else -> Spinner(Modifier.fillMaxWidth().height(48.dp))
            }
            if (!session.loggedIn) Text("登录后查看社区收藏", Modifier.clickable { dismiss(); open(Route.Login) }.padding(vertical = 8.dp), style = XType.Secondary, color = LocalExtraColors.current.brandInk)
            message?.let { Text(it, style = XType.Caption, color = Danger) }
        }
    }
}

@Composable
private fun AppShortcutGrid(env: CommunityEnv, items: List<AppShortcut>, recentItems: Boolean = false, choose: (AppShortcut) -> Unit) {
    items.chunked(4).forEach { row -> Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        row.forEach { item -> key(item.key) {
            if (recentItems && item.source == "community") {
                CommunityShortcutTile(env, item, Modifier.weight(1f), showGameDetails = true) { choose(item) }
            } else AppShortcutTile(env, item, Modifier.weight(1f)) { choose(item) }
        } }
        repeat(4 - row.size) { Spacer(Modifier.weight(1f)) }
    } }
}
