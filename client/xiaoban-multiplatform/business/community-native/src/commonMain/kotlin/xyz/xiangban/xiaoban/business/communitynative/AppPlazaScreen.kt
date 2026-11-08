package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import coil3.ImageLoader
import coil3.compose.AsyncImage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

/** Plaza sections deliberately exclude desktop ideas and unsupported app kinds. */
@Composable
internal fun AppPlazaScreen(env: CommunityEnv, official: List<MobileGame>, local: List<MobileGame>, covers: Map<String, ByteArray>,
    imageLoader: ImageLoader, loading: Boolean, errors: List<String>, reload: () -> Unit,
    onOfficial: (MobileGame) -> Unit, onAllGames: () -> Unit, onAllTools: () -> Unit, open: (Route) -> Unit) {
    val games = official.filterNot { BuiltinMobileGames.isTool(it.id) }
    val tools = official.filter { BuiltinMobileGames.isTool(it.id) }
    var filter by rememberSaveable { mutableStateOf("推荐") }
    var page by rememberSaveable(filter) { mutableStateOf(1) }
    var entries by remember(filter) { mutableStateOf(emptyList<Pack>()) }
    val session by env.session.state.collectAsState()
    val loader = rememberLoader(env, "app-plaza-$filter-$page-${session.accountId}") {
        env.forum.packs(kind = if (filter == "工具") "app" else "game", sort = if (filter == "最新") "latest" else "hot", page = page)
    }
    val state by loader.state.collectAsState()
    val picksLoader = rememberLoader(env, "app-plaza-picks-${session.accountId}") { env.forum.packs(featured = true) }
    val picks by picksLoader.state.collectAsState()
    LaunchedEffect(state) {
        (state as? Load.Ready)?.value?.let { result ->
            entries = ((if (page == 1) emptyList() else entries) + result.items).distinctBy { it.id }
        }
    }
    val recentIds = env.platform?.recentGames?.collectAsState()?.value.orEmpty()
    val recentCommunity = env.platform?.recentCommunityGames?.collectAsState()?.value.orEmpty()
    val available = official.map { AppShortcut("builtin", it.id, it.name) } + local.map { AppShortcut("local", it.id, it.name) }
    val recent = recentAppShortcuts(recentIds, available, recentCommunity)
    var message by remember { mutableStateOf<String?>(null) }
    val ratios = remember(filter) { mutableStateMapOf<Long, Float>() }
    val columns = appWaterfallColumns(entries) { pack -> 1f / (ratios[pack.id] ?: (4f / 3f)) + .6f }
    Box(Modifier.fillMaxSize()) {
        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(bottom = NavInset), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (recent.isNotEmpty()) {
                item { LazyRow(Modifier.padding(top = 4.dp), contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    items(recent, key = { it.key }) { item -> AppShortcutTile(env, item, Modifier.width(64.dp)) {
                        if (item.source == "community") open(Route.Pack(item.id.toLong()))
                        else if (env.platform?.openGame(item.id) != true) message = "当前平台无法打开游戏"
                    } }
                } }
            }
            item { PlazaHeading("精选游戏", "全部", onAllGames.takeIf { games.isNotEmpty() }) }
            if (loading && games.isEmpty()) item { Spinner(Modifier.fillMaxWidth().height(80.dp)) }
            if (!loading && games.isEmpty()) item { Text("当前平台暂无官方游戏", Modifier.padding(horizontal = 16.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3) }
            item {
                LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    items(games, key = { it.id }) { game ->
                        FeaturedGameCard(game, covers["builtin:${game.id}"], imageLoader, Modifier.width(124.dp)) { onOfficial(game) }
                    }
                }
            }
            if (errors.isNotEmpty()) item { Banner(errors.joinToString("\n"), Modifier.padding(horizontal = 16.dp), warn = true, actionText = "重试", onAction = reload) }
            if (tools.isNotEmpty()) {
                item { PlazaHeading("精选工具", "全部", onAllTools) }
                item {
                    Row(Modifier.padding(horizontal = 16.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        tools.take(4).forEach { tool -> AppShortcutTile(env, AppShortcut("builtin", tool.id, tool.name), Modifier.weight(1f)) { onOfficial(tool) } }
                        repeat(4 - tools.take(4).size) { Spacer(Modifier.weight(1f)) }
                    }
                }
            }
            val pickItems = (picks as? Load.Ready)?.value?.items.orEmpty()
            if (pickItems.isNotEmpty()) {
                item { PlazaHeading("创作者精选") }
                item {
                    LazyRow(contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        items(pickItems, key = { it.id }) { pack -> CreatorPickCard(env, pack, Modifier.width(260.dp)) { open(Route.Pack(pack.id)) } }
                    }
                }
            }
            item { PlazaHeading("社区作品") }
            item {
                Row(Modifier.padding(horizontal = 16.dp).horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    listOf("推荐", "最新", "游戏", "工具").forEach { label -> PlazaChip(label, label == filter) { filter = label } }
                }
            }
            run {
                item {
                    Row(Modifier.padding(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        columns.forEach { column -> Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            column.forEach { pack -> key(pack.id) { AppCommunityCard(env, pack, onRatio = { ratios[pack.id] = it }, open = open) } }
                        } }
                    }
                }
                when (val current = state) {
                    is Load.Ready -> item {
                        if (entries.isEmpty()) Text(if (filter == "工具") "还没有社区工具" else "还没有社区作品", Modifier.padding(24.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3)
                        else if (current.value.hasMore) AppActionButton("加载更多", { page++ }, Modifier.padding(horizontal = 16.dp).fillMaxWidth())
                        else Text("已看完所有作品", Modifier.padding(16.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                    }
                    is Load.Failed -> item { Banner(current.message, Modifier.padding(horizontal = 16.dp), warn = true, actionText = "重试", onAction = loader::reload) }
                    else -> item { Spinner(Modifier.fillMaxWidth().height(64.dp)) }
                }
            }
        }
        Toast(message) { message = null }
    }
}

@Composable
private fun PlazaHeading(title: String, action: String? = null, onAction: (() -> Unit)? = null) {
    Row(Modifier.fillMaxWidth().padding(start = 16.dp, end = 8.dp, top = 12.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(title, Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
        if (action != null && onAction != null) Row(Modifier.heightIn(min = 32.dp).clip(CircleShape).clickable(role = Role.Button, onClick = onAction).padding(horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(action, style = XType.Caption, color = LocalExtraColors.current.ink3)
            XIcon(XIcons.ChevronRight, size = 14.dp, tint = LocalExtraColors.current.ink3)
        }
    }
}

/** Shared by the plaza row and the full list; only the cover is clipped so the two text lines never lose their corners. */
@Composable
internal fun FeaturedGameCard(game: MobileGame, cover: ByteArray?, imageLoader: ImageLoader, modifier: Modifier, onClick: () -> Unit) {
    Column(modifier.clickable(role = Role.Button, onClick = onClick).padding(bottom = 4.dp)) {
        Box(Modifier.fillMaxWidth().aspectRatio(3f / 4f).outlinedCard(16.dp), contentAlignment = Alignment.Center) {
            XIcon(XIcons.Gamepad, size = 24.dp, tint = LocalExtraColors.current.brandInk)
            cover?.let { bytes -> AsyncImage(bytes, null, imageLoader = imageLoader, contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize()) }
            Text("官方", Modifier.align(Alignment.BottomEnd).padding(8.dp).clip(CircleShape).background(MaterialTheme.colorScheme.surface).padding(horizontal = 8.dp, vertical = 4.dp), style = XType.Caption)
        }
        Text(game.name, Modifier.padding(top = 8.dp), style = XType.Secondary, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Text(BuiltinMobileGames.genre(game.id), Modifier.padding(top = 4.dp), style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 1)
    }
}

@Composable
internal fun PlazaChip(label: String, active: Boolean, onClick: () -> Unit) {
    Text(label, Modifier.height(34.dp).clip(CircleShape).background(if (active) LocalExtraColors.current.brandSoft else LocalExtraColors.current.fill)
        .semantics { selected = active }.clickable(role = Role.Tab, onClick = onClick).padding(horizontal = 12.dp, vertical = 8.dp),
        style = XType.Secondary, color = if (active) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurface)
}

/** 创作者精选：16:9 封面、作者、两行简介。 */
@Composable
private fun CreatorPickCard(env: CommunityEnv, pack: Pack, modifier: Modifier, onClick: () -> Unit) {
    Column(modifier.outlinedCard(16.dp).clickable(role = Role.Button, onClick = onClick)) {
        Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
            XIcon(XIcons.Gamepad, size = 24.dp, tint = LocalExtraColors.current.ink3)
            pack.coverUrl?.let { url -> AsyncImage(env.forum.imageUrl(url), null, imageLoader = rememberImageLoader(), contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize()) }
        }
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Avatar(pack.author?.nickname ?: "作者", 18.dp, showInitial = true)
                Text(pack.author?.nickname ?: "社区作者", style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text(pack.title, style = XType.Section, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (pack.summary.isNotBlank()) Text(pack.summary, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
        }
    }
}

/** 精选工具「全部」：四列图标网格。 */
@Composable
internal fun FeaturedToolsPage(env: CommunityEnv, tools: List<MobileGame>, back: () -> Unit, onOpen: (MobileGame) -> Unit) {
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        ScreenBar("精选工具", back)
        LazyColumn(Modifier.fillMaxSize().navigationBarsPadding(), contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            items(tools.chunked(4)) { row ->
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    row.forEach { tool -> AppShortcutTile(env, AppShortcut("builtin", tool.id, tool.name), Modifier.weight(1f)) { onOpen(tool) } }
                    repeat(4 - row.size) { Spacer(Modifier.weight(1f)) }
                }
            }
        }
    }
}

/** 精选游戏「全部」：三列封面，按方向筛选。 */
@Composable
internal fun FeaturedGamesPage(games: List<MobileGame>, covers: Map<String, ByteArray>, imageLoader: ImageLoader, back: () -> Unit, onOpen: (MobileGame) -> Unit) {
    var filter by rememberSaveable { mutableStateOf("全部") }
    val shown = games.filter { filter == "全部" || (filter == "横屏") == (it.orientation == "landscape") }
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        ScreenBar("精选游戏", back)
        LazyColumn(Modifier.fillMaxSize().navigationBarsPadding(), contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            item { Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) { listOf("全部", "竖屏", "横屏").forEach { label -> PlazaChip(label, label == filter) { filter = label } } } }
            if (shown.isEmpty()) item { Text("没有${filter}游戏", Modifier.padding(vertical = 24.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3) }
            items(shown.chunked(3)) { row ->
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    row.forEach { game -> FeaturedGameCard(game, covers["builtin:${game.id}"], imageLoader, Modifier.weight(1f)) { onOpen(game) } }
                    repeat(3 - row.size) { Spacer(Modifier.weight(1f)) }
                }
            }
        }
    }
}

/** Same hierarchy as VisualTopicCard, with real pack and linked discussion counts. */
@Composable
private fun AppCommunityCard(env: CommunityEnv, pack: Pack, onRatio: (Float) -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var liked by remember(pack.id, session.accountId) { mutableStateOf(false) }
    var count by remember(pack.id, pack.likeCount, session.accountId) { mutableIntStateOf(pack.likeCount) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var ratio by remember(pack.coverUrl) { mutableFloatStateOf(4f / 3f) }
    Column(Modifier.fillMaxWidth().outlinedCard(16.dp).clickable { open(Route.Pack(pack.id)) }) {
        Box(Modifier.fillMaxWidth().aspectRatio(ratio).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
            XIcon(XIcons.Gamepad, size = 24.dp, tint = LocalExtraColors.current.ink3)
            pack.coverUrl?.let { url -> AsyncImage(env.forum.imageUrl(url), null, imageLoader = rememberImageLoader(), contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize(), onSuccess = {
                if (it.result.image.height > 0) {
                    ratio = (it.result.image.width.toFloat() / it.result.image.height).coerceIn(.75f, 16f / 9f)
                    onRatio(ratio)
                }
            }) }
            Row(Modifier.align(Alignment.BottomStart).fillMaxWidth().background(MaterialTheme.colorScheme.surface.copy(alpha = .9f)).padding(8.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Stat(XIcons.Play, compactCount(pack.applyCount.coerceAtMost(Int.MAX_VALUE.toLong()).toInt()))
                Stat(XIcons.Message, compactCount(pack.replyCount))
            }
        }
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(pack.title, style = XType.Secondary, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                // PackAuthor has no avatar URL; use the shared circular initial avatar.
                Avatar(pack.author?.nickname ?: "作者", 20.dp, showInitial = true)
                Text(pack.author?.nickname ?: "社区作者", Modifier.weight(1f), style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Row(Modifier.heightIn(min = 32.dp).clickable(enabled = !busy) {
                    if (!session.loggedIn) { open(Route.Login); return@clickable }
                    busy = true
                    scope.launch {
                        try {
                            val discussion = env.contentSocial.discussion("pack", pack.id, create = true)
                            val topicId = discussion.topicId ?: error("暂时无法点赞")
                            val result = env.forum.setTopicLiked(topicId, !discussion.liked)
                            liked = result.liked; count = result.likeCount; error = null
                        } catch (e: CancellationException) { throw e }
                        catch (e: Exception) { error = messageOf(e) }
                        finally { busy = false }
                    }
                }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    XIcon(XIcons.Heart, size = 16.dp, tint = if (liked) LocalExtraColors.current.brandInk else LocalExtraColors.current.ink3, description = if (liked) "取消点赞" else "点赞")
                    Text(compactCount(count), style = XType.Caption, color = LocalExtraColors.current.ink3)
                }
            }
            error?.let { Text(it, style = XType.Caption, color = Danger) }
        }
    }
}
