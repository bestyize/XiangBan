package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import xyz.xiangban.xiaoban.common.community.*

/** Top drawer, separate from the existing conversation side drawer. */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
internal fun AppLibraryDrawer(env: CommunityEnv, dismiss: () -> Unit, goPlaza: () -> Unit, open: (Route) -> Unit, panel: Modifier = Modifier) {
    BackHandler(onBack = dismiss)
    var available by remember { mutableStateOf(emptyList<AppShortcut>()) }
    var catalogError by remember { mutableStateOf<String?>(null) }
    var revision by remember { mutableIntStateOf(0) }
    var loading by remember { mutableStateOf(true) }
    val session by env.session.state.collectAsState()
    val bookmarkRevision by env.contentBookmarkRevision.collectAsState()
    var page by rememberSaveable(session.accountId) { mutableIntStateOf(1) }
    var bookmarks by remember(session.accountId) { mutableStateOf(emptyList<AppShortcut>()) }
    val bookmarkLoader = rememberLoader(env, "app-drawer-favorites-${session.accountId}-$page-$bookmarkRevision") {
        if (session.loggedIn) env.contentSocial.bookmarks("game", page) else Page<ContentBookmark>()
    }
    val bookmarkState by bookmarkLoader.state.collectAsState()
    LaunchedEffect(bookmarkRevision) { page = 1 }
    LaunchedEffect(bookmarkState) {
        (bookmarkState as? Load.Ready)?.value?.let { result -> bookmarks = ((if (page == 1) emptyList() else bookmarks) + result.items.map {
            AppShortcut("community", it.id.toString(), it.content.title, it.content.coverUrl)
        }).distinctBy { it.key } }
    }
    LaunchedEffect(env, revision) {
        loading = true
        val items = mutableListOf<AppShortcut>()
        val errors = mutableListOf<String>()
        try {
            try { items += env.platform?.builtinGames().orEmpty().map { AppShortcut("builtin", it.id, it.name) } }
            catch (e: CancellationException) { throw e } catch (_: Exception) { errors += "官方目录读取失败" }
            try { items += env.games?.list().orEmpty().map { AppShortcut("local", it.id, it.name) } }
            catch (e: CancellationException) { throw e } catch (_: Exception) { errors += "本机作品读取失败" }
            available = items
            catalogError = errors.takeIf { it.isNotEmpty() }?.joinToString(" · ")
        } finally { loading = false }
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
    BoxWithConstraints(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.onSurface.copy(alpha = .18f)).clickable(indication = null, interactionSource = null) { dismiss() }) {
        Column(panel.fillMaxWidth().heightIn(max = maxHeight * .8f).clip(RoundedCornerShape(bottomStart = 16.dp, bottomEnd = 16.dp))
            .background(MaterialTheme.colorScheme.surface).clickable(indication = null, interactionSource = null) {}.statusBarsPadding().padding(16.dp)) {
            Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("最近用过", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
                    IconTap(XIcons.Close, "收起我的应用", dismiss)
                }
                if (loading) Spinner(Modifier.fillMaxWidth().height(48.dp))
                else if (recent.isEmpty()) Text("还没有使用记录", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                AppShortcutGrid(env, recent, ::choose)
                catalogError?.let { Banner(it, warn = true, actionText = "重试", onAction = { revision++ }) }
                Text("我的收藏", Modifier.padding(top = 12.dp), style = XType.Section, fontWeight = FontWeight.SemiBold)
                AppShortcutGrid(env, favorites, ::choose)
                when (val current = bookmarkState) {
                    is Load.Ready -> {
                        if (favorites.isEmpty()) Text("还没有收藏的应用", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                        if (current.value.hasMore) AppActionButton("加载更多收藏", { page++ }, Modifier.fillMaxWidth())
                    }
                    is Load.Failed -> Banner(current.message, warn = true, actionText = "重试", onAction = bookmarkLoader::reload)
                    else -> Spinner(Modifier.fillMaxWidth().height(48.dp))
                }
                if (!session.loggedIn) Text("登录后查看社区收藏", Modifier.clickable { dismiss(); open(Route.Login) }.padding(vertical = 8.dp), style = XType.Secondary, color = LocalExtraColors.current.brandInk)
                message?.let { Text(it, style = XType.Caption, color = Danger) }
            }
            Hairline(Modifier.padding(top = 12.dp))
            Row(Modifier.fillMaxWidth().height(44.dp).clickable(role = Role.Button) { dismiss(); goPlaza() },
                horizontalArrangement = Arrangement.Center, verticalAlignment = Alignment.CenterVertically) {
                Text("去应用广场", style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = LocalExtraColors.current.brandInk)
                XIcon(XIcons.ChevronRight, Modifier.padding(start = 2.dp), size = 16.dp, tint = LocalExtraColors.current.brandInk)
            }
            Box(Modifier.align(Alignment.CenterHorizontally).size(48.dp, 16.dp).clickable { dismiss() }, contentAlignment = Alignment.Center) {
                Box(Modifier.size(32.dp, 4.dp).clip(CircleShape).background(LocalExtraColors.current.line))
            }
        }
    }
}

@Composable
private fun AppShortcutGrid(env: CommunityEnv, items: List<AppShortcut>, choose: (AppShortcut) -> Unit) {
    items.chunked(4).forEach { row -> Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        row.forEach { item -> key(item.key) { AppShortcutTile(env, item, Modifier.weight(1f)) { choose(item) } } }
        repeat(4 - row.size) { Spacer(Modifier.weight(1f)) }
    } }
}
