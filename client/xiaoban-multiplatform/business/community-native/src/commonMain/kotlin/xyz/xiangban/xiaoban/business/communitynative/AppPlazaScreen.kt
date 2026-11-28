package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

private const val FeaturedToolSlots = 4
private val FeaturedInset = 16.dp
private val CardGap = 10.dp

/** Plaza sections deliberately exclude desktop ideas and unsupported app kinds. */
@Composable
internal fun AppPlazaScreen(
    env: CommunityEnv,
    official: List<MobileGame>,
    loading: Boolean,
    errors: List<String>,
    reload: () -> Unit,
    featuredShortcuts: List<AppShortcut>,
    featuredTools: Load<Unit>,
    retryFeaturedTools: () -> Unit,
    onOfficial: (MobileGame) -> Unit,
    onOfficialDetails: (MobileGame) -> Unit,
    onAllGames: () -> Unit,
    onAllTools: () -> Unit,
    open: (Route) -> Unit,
) {
    val games = official.filterNot { BuiltinMobileGames.isTool(it.id) }
    val tools = official.filter { BuiltinMobileGames.isTool(it.id) }
    var selectedFilter by rememberSaveable { mutableStateOf("精选") }
    val filter = if (selectedFilter == "推荐") "精选" else selectedFilter
    val featuredOnly = filter == "精选"
    val session by env.session.state.collectAsState()
    var page by remember(filter, session.accountId) { mutableStateOf(1) }
    var entries by remember(filter, session.accountId) { mutableStateOf(emptyList<Pack>()) }
    val requestedPage = page
    val loader = rememberLoader(env, "app-plaza-$filter-$requestedPage-${session.accountId}") {
        env.forum.packs(
            kind = when (filter) { "精选" -> null; "工具" -> "app"; else -> "game" },
            sort = when (filter) { "精选" -> null; "最新" -> "latest"; else -> "hot" },
            page = requestedPage,
            featured = featuredOnly,
        )
    }
    val state by key(loader) { loader.state.collectAsState() }
    val toolShortcuts = (tools.map { AppShortcut("builtin", it.id, it.name) } + featuredShortcuts).take(FeaturedToolSlots)
    LaunchedEffect(loader, state) {
        (state as? Load.Ready)?.value?.let { result ->
            val items = if (featuredOnly) result.items.filter { it.featured && (it.kind == "game" || it.kind == "app") } else result.items
            entries = ((if (requestedPage == 1) emptyList() else entries) + items).distinctBy { it.id }
        }
    }
    Box(Modifier.fillMaxSize()) {
        LazyColumn(
            Modifier.fillMaxSize(),
            contentPadding = PaddingValues(bottom = NavInset),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            // 1. 精选游戏：方案 A 精装三行横滑榜单卡（复用 1:1 图标，零额外封面素材）
            item { PlazaHeading("精选游戏", "全部", onAllGames.takeIf { games.isNotEmpty() }, topPadding = 8.dp) }
            if (loading && games.isEmpty()) item { Spinner(Modifier.fillMaxWidth().height(80.dp)) }
            if (!loading && games.isEmpty()) item {
                Text("当前平台暂无官方游戏", Modifier.padding(horizontal = FeaturedInset), style = XType.Secondary, color = LocalExtraColors.current.ink3)
            }
            if (games.isNotEmpty()) item {
                val columns = games.chunked(3)
                BoxWithConstraints(Modifier.fillMaxWidth()) {
                    val cardWidth = if (columns.size > 1) (maxWidth - FeaturedInset * 2 - 40.dp).coerceIn(264.dp, 324.dp)
                    else maxWidth - FeaturedInset * 2
                    LazyRow(
                        contentPadding = PaddingValues(horizontal = FeaturedInset),
                        horizontalArrangement = Arrangement.spacedBy(CardGap),
                    ) {
                        items(columns, key = { it.first().id }) { columnGames ->
                            FeaturedGameColumnCard(
                                env = env,
                                games = columnGames,
                                modifier = Modifier.width(cardWidth),
                                onDetails = onOfficialDetails,
                                onOpen = onOfficial,
                            )
                        }
                    }
                }
            }
            if (errors.isNotEmpty()) item {
                Banner(errors.joinToString("\n"), Modifier.padding(horizontal = FeaturedInset), warn = true, actionText = "重试", onAction = reload)
            }

            // 2. 精选工具：2×2 精致功能卡（1:1 图标 + 名称 + 一句话用途）
            if (toolShortcuts.isNotEmpty() || featuredTools !is Load.Ready) {
                item { PlazaHeading("精选工具", "全部", onAllTools) }
                if (toolShortcuts.isNotEmpty()) {
                    items(toolShortcuts.chunked(2), key = { it.first().key }) { row ->
                        Row(
                            Modifier.fillMaxWidth().padding(horizontal = FeaturedInset),
                            horizontalArrangement = Arrangement.spacedBy(CardGap),
                        ) {
                            row.forEach { shortcut ->
                                FeaturedToolCard(env, shortcut, tools, Modifier.weight(1f), onOfficialDetails, onOfficial, open)
                            }
                            if (row.size == 1) Spacer(Modifier.weight(1f))
                        }
                    }
                }
                featuredToolsStatus(featuredTools, featuredShortcuts.isNotEmpty(), retryFeaturedTools, Modifier.padding(horizontal = FeaturedInset))
            }

            // 3. 社区作品：净化 3:2 双列卡（去黑渐变遮罩、补 0.5dp 封面底线、标题旁轻标）
            item { PlazaHeading("社区作品") }
            item {
                Row(
                    Modifier.padding(horizontal = FeaturedInset).horizontalScroll(rememberScrollState()),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    listOf("精选", "最新", "游戏", "工具").forEach { label ->
                        PlazaChip(label, label == filter) { selectedFilter = label }
                    }
                }
            }
            run {
                items(entries.chunked(2), key = { it.first().id }) { row ->
                    Row(
                        Modifier.fillMaxWidth().padding(horizontal = FeaturedInset),
                        horizontalArrangement = Arrangement.spacedBy(CardGap),
                    ) {
                        row.forEach { pack -> key(pack.id) { AppCommunityCard(env, pack, Modifier.weight(1f), open) } }
                        if (row.size == 1) Spacer(Modifier.weight(1f))
                    }
                }
                when (val current = state) {
                    is Load.Ready -> item {
                        if (current.value.hasMore) AppActionButton("加载更多", { page = requestedPage + 1 }, Modifier.padding(horizontal = FeaturedInset).fillMaxWidth())
                        else if (entries.isEmpty()) {
                            val emptyMessage = when (filter) { "精选" -> "暂无精选作品"; "工具" -> "还没有社区工具"; else -> "还没有社区作品" }
                            Text(emptyMessage, Modifier.padding(24.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3)
                        } else Text("已看完所有作品", Modifier.padding(horizontal = FeaturedInset, vertical = 12.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                    }
                    is Load.Failed -> item { Banner(current.message, Modifier.padding(horizontal = FeaturedInset), warn = true, actionText = "重试", onAction = loader::reload) }
                    else -> item { Spinner(Modifier.fillMaxWidth().height(64.dp)) }
                }
            }
        }
    }
}

@Composable
private fun PlazaHeading(
    title: String,
    action: String? = null,
    onAction: (() -> Unit)? = null,
    topPadding: androidx.compose.ui.unit.Dp = 10.dp,
) {
    Row(
        Modifier.fillMaxWidth().padding(start = FeaturedInset, end = 8.dp, top = topPadding),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(title, Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
        if (action != null && onAction != null) Row(
            Modifier.heightIn(min = 32.dp).clip(CircleShape).clickable(role = Role.Button, onClick = onAction).padding(horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            Text(action, style = XType.Caption, color = LocalExtraColors.current.ink3)
            XIcon(XIcons.ChevronRight, size = 14.dp, tint = LocalExtraColors.current.ink3)
        }
    }
}

/** 方案 A：三行一组的白色细框游戏榜单卡（仅依赖 1:1 图标）。 */
@Composable
private fun FeaturedGameColumnCard(
    env: CommunityEnv,
    games: List<MobileGame>,
    modifier: Modifier,
    onDetails: (MobileGame) -> Unit,
    onOpen: (MobileGame) -> Unit,
) {
    val extra = LocalExtraColors.current
    Column(modifier.outlinedCard(16.dp).padding(horizontal = 12.dp, vertical = 4.dp)) {
        games.forEachIndexed { index, game ->
            if (index > 0) Box(Modifier.fillMaxWidth().padding(start = 63.dp).height(0.5.dp).background(extra.hairline))
            FeaturedGameRow(env, game, onDetails = { onDetails(game) }, onOpen = { onOpen(game) })
        }
    }
}

@Composable
private fun FeaturedGameRow(
    env: CommunityEnv,
    game: MobileGame,
    onDetails: () -> Unit,
    onOpen: () -> Unit,
) {
    val extra = LocalExtraColors.current
    Row(
        Modifier.fillMaxWidth()
            .combinedClickable(onLongClickLabel = "查看作品详情", onLongClick = onDetails, onClick = onOpen)
            .padding(vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(11.dp),
    ) {
        AppShortcutIcon(env, AppShortcut("builtin", game.id, game.name), Modifier.size(52.dp), corner = 14.dp)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(game.name, style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(
                "${BuiltinMobileGames.genre(game.id)} · ${gameTagline(game)}",
                style = XType.Caption,
                color = extra.ink3,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        PlazaOpenPill("打开", onOpen)
    }
}

@Composable
private fun PlazaOpenPill(label: String, onClick: () -> Unit, primary: Boolean = false) {
    val extra = LocalExtraColors.current
    Box(
        Modifier.height(30.dp).clip(CircleShape)
            .background(if (primary) extra.brandSoft else extra.fill)
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = 13.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            label,
            style = XType.Caption,
            fontWeight = FontWeight.SemiBold,
            color = if (primary) extra.brandInk else MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
        )
    }
}

private fun gameTagline(game: MobileGame): String = when (game.id) {
    "block-puzzle" -> "8×8 经典消除连击"
    "dusk-glider" -> "乘日落气流收集微光"
    "ink-blade" -> "极简水墨一击必杀"
    BuiltinMobileGames.PAINTING_ADVENTURE -> "跃入油画横版冒险"
    "water-sort" -> "45 关调饮分类手帐"
    "pro-cube" -> "3D 真实手感计时"
    "jade-gomoku" -> "水墨玉石人机对弈"
    "merge-2048" -> "顺滑数字合成挑战"
    "snake-arena" -> "无尽成长冲分挑战"
    else -> game.description.trim().ifBlank { if (game.orientation == "landscape") "横屏离线可玩" else "竖屏离线可玩" }
}

private fun toolSubtitle(shortcut: AppShortcut, tools: List<MobileGame>): String = when {
    shortcut.source == "builtin" -> when (shortcut.id) {
        "mbti" -> "48 题人格报告"
        BuiltinMobileGames.REMOTE_CONTROL -> "电视空调红外遥控"
        BuiltinMobileGames.QUICK_LAUNCH -> "桌面快捷启动"
        BuiltinMobileGames.LIVE_WALLPAPERS -> "雨窗天光 · 五套氛围"
        else -> tools.firstOrNull { it.id == shortcut.id }?.description?.trim().orEmpty().ifBlank { "内置实用工具" }
    }
    shortcut.slug == "wancisou" -> "公开资源检索"
    else -> "社区精选工具"
}

@Composable
private fun FeaturedToolCard(
    env: CommunityEnv,
    shortcut: AppShortcut,
    tools: List<MobileGame>,
    modifier: Modifier,
    onDetails: (MobileGame) -> Unit,
    onOpen: (MobileGame) -> Unit,
    open: (Route) -> Unit,
) {
    if (shortcut.source == "builtin") {
        val tool = tools.first { it.id == shortcut.id }
        ToolCardBody(
            env = env,
            shortcut = shortcut,
            subtitle = toolSubtitle(shortcut, tools),
            modifier = modifier,
            onDetails = { onDetails(tool) },
            onClick = { onOpen(tool) },
        )
    } else key(shortcut.key) {
        val openDetails = { open(Route.Pack(shortcut.id.toLong())) }
        CommunityWorkLaunch(env, shortcut.id.toLong(), openGameDetails = null) { loading, _, error, launch ->
            ToolCardBody(
                env = env,
                shortcut = shortcut,
                subtitle = when {
                    loading -> "正在打开…"
                    error != null -> error
                    else -> toolSubtitle(shortcut, tools)
                },
                subtitleError = error != null,
                modifier = modifier,
                onDetails = openDetails,
                onClick = launch,
            )
        }
    }
}

@Composable
private fun ToolCardBody(
    env: CommunityEnv,
    shortcut: AppShortcut,
    subtitle: String,
    modifier: Modifier,
    subtitleError: Boolean = false,
    onDetails: (() -> Unit)? = null,
    onClick: () -> Unit,
) {
    Row(
        modifier.outlinedCard(16.dp)
            .combinedClickable(onLongClickLabel = "查看作品详情", onLongClick = onDetails, onClick = onClick)
            .padding(11.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        AppShortcutIcon(env, shortcut, Modifier.size(42.dp), corner = 11.dp)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(shortcut.title, style = XType.Secondary, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(
                subtitle,
                style = XType.Caption,
                color = if (subtitleError) Danger else LocalExtraColors.current.ink3,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
internal fun PlazaChip(label: String, active: Boolean, onClick: () -> Unit) {
    val extra = LocalExtraColors.current
    val borderColor = if (active) Color(ThemeState.palette.btn).copy(alpha = if (ThemeState.dark) 0.4f else 0.25f) else extra.hairline
    Box(
        Modifier.height(32.dp).clip(CircleShape)
            .background(if (active) extra.brandSoft else MaterialTheme.colorScheme.surface)
            .border(1.dp, borderColor, CircleShape)
            .semantics { selected = active }
            .clickable(role = Role.Tab, onClick = onClick)
            .padding(horizontal = 13.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            label,
            style = XType.Secondary,
            fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal,
            color = if (active) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
        )
    }
}

/** 精选工具「全部」：方案 A 精修列表，展示用途说明与打开入口。 */
@Composable
internal fun FeaturedToolsPage(
    env: CommunityEnv,
    tools: List<MobileGame>,
    back: () -> Unit,
    featuredShortcuts: List<AppShortcut>,
    featuredTools: Load<Unit>,
    retryFeaturedTools: () -> Unit,
    onDetails: (MobileGame) -> Unit,
    onOpen: (MobileGame) -> Unit,
    open: (Route) -> Unit,
) {
    val extra = LocalExtraColors.current
    val builtinShortcuts = tools.map { AppShortcut("builtin", it.id, it.name) }
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        ScreenBar("精选工具", back)
        LazyColumn(
            Modifier.fillMaxSize().navigationBarsPadding(),
            contentPadding = PaddingValues(start = FeaturedInset, end = FeaturedInset, top = 8.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            if (builtinShortcuts.isNotEmpty()) {
                item {
                    Column(Modifier.fillMaxWidth().outlinedCard(16.dp).padding(horizontal = 12.dp, vertical = 4.dp)) {
                        builtinShortcuts.forEachIndexed { index, shortcut ->
                            val tool = tools.first { it.id == shortcut.id }
                            if (index > 0) Box(Modifier.fillMaxWidth().padding(start = 56.dp).height(0.5.dp).background(extra.hairline))
                            Row(
                                Modifier.fillMaxWidth()
                                    .combinedClickable(onLongClickLabel = "查看作品详情", onLongClick = { onDetails(tool) }, onClick = { onOpen(tool) })
                                    .padding(vertical = 10.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(11.dp),
                            ) {
                                AppShortcutIcon(env, shortcut, Modifier.size(44.dp), corner = 12.dp)
                                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                                    Text(tool.name, style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                    Text(toolSubtitle(shortcut, tools), style = XType.Caption, color = extra.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                }
                                PlazaOpenPill("打开", { onOpen(tool) })
                            }
                        }
                    }
                }
            }
            if (featuredShortcuts.isNotEmpty()) {
                item { Text("社区精选工具", Modifier.padding(top = 4.dp), style = XType.Caption, color = extra.ink3) }
                item {
                    Column(Modifier.fillMaxWidth().outlinedCard(16.dp).padding(horizontal = 12.dp, vertical = 4.dp)) {
                        featuredShortcuts.forEachIndexed { index, shortcut ->
                            if (index > 0) Box(Modifier.fillMaxWidth().padding(start = 56.dp).height(0.5.dp).background(extra.hairline))
                            key(shortcut.key) {
                                val openDetails = { open(Route.Pack(shortcut.id.toLong())) }
                                CommunityWorkLaunch(env, shortcut.id.toLong(), openGameDetails = null) { loading, _, error, launch ->
                                    Row(
                                        Modifier.fillMaxWidth()
                                            .combinedClickable(onLongClickLabel = "查看作品详情", onLongClick = openDetails, onClick = launch)
                                            .padding(vertical = 10.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(11.dp),
                                    ) {
                                        AppShortcutIcon(env, shortcut, Modifier.size(44.dp), corner = 12.dp)
                                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                                            Text(shortcut.title, style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                            Text(
                                                when {
                                                    loading -> "正在打开…"
                                                    error != null -> error
                                                    else -> toolSubtitle(shortcut, tools)
                                                },
                                                style = XType.Caption,
                                                color = if (error != null) Danger else extra.ink3,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                            )
                                        }
                                        PlazaOpenPill("打开", launch)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            featuredToolsStatus(featuredTools, featuredShortcuts.isNotEmpty(), retryFeaturedTools)
        }
    }
}

private fun LazyListScope.featuredToolsStatus(state: Load<Unit>, hasTools: Boolean, retry: () -> Unit, modifier: Modifier = Modifier) {
    when (state) {
        Load.Loading -> if (!hasTools) item {
            Text("正在加载社区精选工具…", modifier, style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        is Load.Failed -> item {
            Banner("社区精选工具加载失败", modifier.fillMaxWidth(), desc = state.message, warn = true, actionText = "重试", onAction = retry)
        }
        is Load.Ready -> Unit
    }
}

/** 精选游戏「全部」：方案 A 精修列表，按方向筛选。 */
@Composable
internal fun FeaturedGamesPage(
    env: CommunityEnv,
    games: List<MobileGame>,
    back: () -> Unit,
    onDetails: (MobileGame) -> Unit,
    onOpen: (MobileGame) -> Unit,
) {
    var filter by rememberSaveable { mutableStateOf("全部") }
    val shown = games.filter { filter == "全部" || (filter == "横屏") == (it.orientation == "landscape") }
    val extra = LocalExtraColors.current
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        ScreenBar("精选游戏", back)
        LazyColumn(
            Modifier.fillMaxSize().navigationBarsPadding(),
            contentPadding = PaddingValues(start = FeaturedInset, end = FeaturedInset, top = 8.dp, bottom = 24.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    listOf("全部", "竖屏", "横屏").forEach { label -> PlazaChip(label, label == filter) { filter = label } }
                }
            }
            if (shown.isEmpty()) item {
                Text("没有${filter}游戏", Modifier.padding(vertical = 24.dp), style = XType.Secondary, color = extra.ink3)
            } else item {
                Column(Modifier.fillMaxWidth().outlinedCard(16.dp).padding(horizontal = 12.dp, vertical = 4.dp)) {
                    shown.forEachIndexed { index, game ->
                        if (index > 0) Box(Modifier.fillMaxWidth().padding(start = 63.dp).height(0.5.dp).background(extra.hairline))
                        FeaturedGameRow(env, game, onDetails = { onDetails(game) }, onOpen = { onOpen(game) })
                    }
                }
            }
        }
    }
}

/** Clean 3:2 community work card: unobstructed cover with hairline divider and inline featured badge. */
@Composable
private fun AppCommunityCard(env: CommunityEnv, pack: Pack, modifier: Modifier, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var liked by rememberSaveable(pack.id, session.accountId) { mutableStateOf(false) }
    var count by rememberSaveable(pack.id, pack.likeCount, session.accountId) { mutableStateOf(pack.likeCount) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val extra = LocalExtraColors.current
    Column(modifier.outlinedCard(16.dp).clickable { open(Route.Pack(pack.id)) }) {
        Box(Modifier.fillMaxWidth().aspectRatio(3f / 2f).background(extra.fill), contentAlignment = Alignment.Center) {
            XIcon(XIcons.Gamepad, size = 24.dp, tint = extra.ink3)
            pack.coverUrl?.let { url ->
                AsyncImage(
                    env.forum.imageUrl(url),
                    null,
                    imageLoader = rememberImageLoader(),
                    contentScale = ContentScale.Crop,
                    modifier = Modifier.fillMaxSize(),
                )
            }
        }
        Box(Modifier.fillMaxWidth().height(0.5.dp).background(extra.hairline))
        Column(Modifier.padding(horizontal = 10.dp, vertical = 10.dp), verticalArrangement = Arrangement.spacedBy(7.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    pack.title,
                    Modifier.weight(1f),
                    style = XType.Secondary,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                val badgeText = if (pack.featured) "精选" else if (pack.kind == "app") "工具" else "游戏"
                Text(
                    badgeText,
                    Modifier.clip(CircleShape)
                        .background(if (pack.featured) extra.brandSoft else extra.fill)
                        .padding(horizontal = 6.dp, vertical = 1.dp),
                    style = XType.Caption.copy(fontSize = 10.5.sp, lineHeight = 15.sp),
                    fontWeight = if (pack.featured) FontWeight.SemiBold else FontWeight.Normal,
                    color = if (pack.featured) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                )
            }
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                // PackAuthor has no avatar URL; use the shared circular initial avatar.
                Avatar(pack.author?.nickname ?: "作者", 18.dp, showInitial = true)
                Text(
                    pack.author?.nickname ?: "社区作者",
                    Modifier.weight(1f),
                    style = XType.Caption,
                    color = extra.ink3,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Row(
                    Modifier.heightIn(min = 28.dp).clickable(enabled = !busy) {
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
                    },
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(3.dp),
                ) {
                    XIcon(
                        XIcons.Heart,
                        size = 14.dp,
                        tint = if (liked) extra.brandInk else extra.ink3,
                        description = if (liked) "取消点赞" else "点赞",
                    )
                    Text(compactCount(count), style = XType.Caption, color = extra.ink3)
                }
            }
            error?.let { Text(it, style = XType.Caption, color = Danger) }
        }
    }
}
