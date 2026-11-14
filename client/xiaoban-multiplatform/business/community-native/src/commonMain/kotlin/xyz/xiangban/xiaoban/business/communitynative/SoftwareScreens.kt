package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.layout.ContentScale
import coil3.compose.AsyncImage
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.common.community.*
import xiaoban.business.community_native.generated.resources.Res

private val SoftwareCategories = listOf(
    "all" to "全部", "ai" to "AI", "media" to "影音", "network" to "网络",
    "tools" to "工具", "productivity" to "效率", "development" to "开发",
    "games" to "游戏", "other" to "其它",
)
private val SoftwareSorts = listOf("stars" to "收藏数", "updated" to "最近更新", "hot" to "近期热门")

private suspend inline fun <reified T> softwareResource(name: String): T = withContext(Dispatchers.Default) {
    CommunityJson.decodeFromString(Res.readBytes("files/$name.json").decodeToString())
}

private suspend fun softwareCatalog(): List<SoftwareEntry> = softwareResource("software-directory")

@Composable
internal fun SoftwareDirectoryTab(env: CommunityEnv, open: (Route) -> Unit) {
    val catalog = rememberLoader(env, "software-catalog", ::softwareCatalog)
    val summaries = rememberLoader(env, "software-summaries") { env.forum.softwareSummaries() }
    val state by catalog.state.collectAsState()
    val summaryState by summaries.state.collectAsState()
    var category by rememberSaveable { mutableStateOf("all") }
    var sort by rememberSaveable { mutableStateOf("stars") }
    var limit by rememberSaveable { mutableIntStateOf(48) }
    val listState = rememberLazyListState()

    Column(Modifier.fillMaxSize()) {
        SoftwareDirectoryFilters(
            category = category,
            sort = sort,
            onCategory = {
                category = it
                limit = 48
                listState.requestScrollToItem(0)
            },
            onSort = {
                sort = it
                limit = 48
                listState.requestScrollToItem(0)
            },
        )
        LoadBox(state, catalog::reload) { entries ->
            val snapshots = (summaryState as? Load.Ready)?.value.orEmpty()
            val visible = remember(entries, snapshots, category, sort) {
                entries.filter { category == "all" || it.category == category }
                    .sortedWith(softwareDirectoryOrder(snapshots, sort))
            }
            LazyColumn(state = listState, contentPadding = FeedPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                if (summaryState is Load.Failed) item {
                    Banner("同步数据暂不可用，正在显示收录资料", warn = true, actionText = "重试", onAction = summaries::reload)
                }
                if (sort == "hot" && visible.none { snapshots[it.project]?.starGain != null }) item {
                    Text("暂无近期星标变化数据，按更新排序", style = XType.Caption)
                }
                items(visible.take(limit), key = { it.project }) { entry ->
                    SoftwareCard(entry, snapshots[entry.project]) { open(Route.Software(entry.project)) }
                }
                if (visible.size > limit) item { GhostPillButton("加载更多", { limit += 48 }) }
            }
        }
    }
}

@Composable
private fun SoftwareDirectoryFilters(
    category: String,
    sort: String,
    onCategory: (String) -> Unit,
    onSort: (String) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val categoryScroll = rememberLazyListState()
    Row(
        Modifier.fillMaxWidth().padding(horizontal = 12.dp).padding(top = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        LazyRow(
            Modifier.weight(1f).selectableGroup().drawWithContent {
                drawContent()
                val fadeWidth = 18.dp.toPx().coerceAtMost(size.width / 2)
                val edgeSize = Size(fadeWidth, size.height)
                val rtl = layoutDirection == LayoutDirection.Rtl
                val fadeLeft = if (rtl) categoryScroll.canScrollForward else categoryScroll.canScrollBackward
                val fadeRight = if (rtl) categoryScroll.canScrollBackward else categoryScroll.canScrollForward
                if (fadeLeft) drawRect(
                    Brush.horizontalGradient(listOf(colors.background, Color.Transparent), endX = fadeWidth),
                    size = edgeSize,
                )
                if (fadeRight) drawRect(
                    Brush.horizontalGradient(listOf(Color.Transparent, colors.background), startX = size.width - fadeWidth, endX = size.width),
                    topLeft = Offset(size.width - fadeWidth, 0f),
                    size = edgeSize,
                )
            },
            state = categoryScroll,
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            items(SoftwareCategories, key = { it.first }) { (key, label) ->
                val active = key == category
                Box(
                    Modifier.defaultMinSize(minWidth = 44.dp, minHeight = 44.dp).clip(CircleShape)
                        .selectable(selected = active, role = Role.Tab) { onCategory(key) },
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        label,
                        Modifier.background(if (active) extra.brandSoft else Color.Transparent, CircleShape)
                            .padding(horizontal = 9.dp, vertical = 6.dp),
                        style = XType.Secondary,
                        color = if (active) extra.brandInk else colors.onSurfaceVariant,
                        fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal,
                        maxLines = 1,
                    )
                }
            }
        }
        SoftwareSortMenu(sort, onSort)
    }
}

@Composable
private fun SoftwareSortMenu(sort: String, onSort: (String) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    val colors = MaterialTheme.colorScheme
    val label = SoftwareSorts.firstOrNull { it.first == sort }?.second ?: SoftwareSorts.first().second
    Box {
        Box(
            Modifier.heightIn(min = 44.dp).clip(CircleShape)
                .clickable(role = Role.Button, onClickLabel = "更改排序") { expanded = true },
            contentAlignment = Alignment.Center,
        ) {
            Row(
                Modifier.outlinedCard(50.dp).widthIn(min = 90.dp).padding(horizontal = 9.dp, vertical = 7.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(5.dp, Alignment.CenterHorizontally),
            ) {
                Text(label, style = XType.Secondary, fontWeight = FontWeight.Medium, maxLines = 1)
                XIcon(if (expanded) XIcons.ChevronUp else XIcons.ChevronDown, size = 14.dp, tint = colors.onSurfaceVariant)
            }
        }
        DropdownMenu(
            expanded = expanded,
            onDismissRequest = { expanded = false },
            shape = RoundedCornerShape(12.dp),
            containerColor = colors.surface,
        ) {
            SoftwareSorts.forEach { (key, title) ->
                val active = key == sort
                DropdownMenuItem(
                    modifier = Modifier.semantics { selected = active },
                    text = { Text(title, style = XType.Body) },
                    trailingIcon = { if (active) XIcon(XIcons.Check, size = 16.dp, tint = LocalExtraColors.current.brandInk) },
                    onClick = {
                        expanded = false
                        if (!active) onSort(key)
                    },
                )
            }
        }
    }
}

@Composable
private fun SoftwareCard(entry: SoftwareEntry, snapshot: SoftwareSummary?, open: () -> Unit) {
    Column(Modifier.fillMaxWidth().outlinedCard(12.dp).clickable(onClick = open).padding(14.dp),
        verticalArrangement = Arrangement.spacedBy(7.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Box(Modifier.size(34.dp).clip(RoundedCornerShape(8.dp))) {
                Avatar(entry.displayName, 34.dp, showInitial = true)
                val icon = entry.icon.ifBlank { "https://github.com/${entry.project.substringBefore('/')}.png?size=80" }
                AsyncImage(icon, contentDescription = null, imageLoader = rememberImageLoader(),
                    modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Fit)
            }
            Column(Modifier.weight(1f)) {
                Text(entry.displayName, style = XType.Body, fontWeight = FontWeight.SemiBold)
                Text(entry.project, style = XType.Caption)
            }
            (snapshot?.stars ?: entry.stars)?.let { Text("★ ${compactCount(it.coerceAtMost(Int.MAX_VALUE.toLong()).toInt())}", style = XType.Caption) }
        }
        if (entry.summary.isNotBlank()) Text(entry.summary, style = XType.Secondary, maxLines = 2, overflow = TextOverflow.Ellipsis)
        val version = if (snapshot == null) entry.version else snapshot.version.orEmpty()
        Text((listOf(version, entry.language) + entry.platforms).filter { it.isNotBlank() }.joinToString(" · "), style = XType.Caption)
        (snapshot?.updatedAt ?: entry.updatedAt)?.let { Text("更新于 ${it.take(10)}", style = XType.Caption) }
    }
}

@Composable
internal fun SoftwareDetailScreen(env: CommunityEnv, project: String, back: () -> Unit, open: (Route) -> Unit) {
    val catalog = rememberLoader(env, project) { softwareCatalog().firstOrNull { it.project == project } }
    val snapshot = rememberLoader(env, "release:$project") { env.forum.softwareSnapshot(project) }
    val state by catalog.state.collectAsState()
    val releases by snapshot.state.collectAsState()
    val initial = rememberLoader(env, "initial-release:$project") {
        softwareResource<Map<String, SoftwareRelease>>("software-initial-releases")[project]
    }
    val initialState by initial.state.collectAsState()
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    var platform by rememberSaveable(project) { mutableStateOf("all") }
    var selected by rememberSaveable(project) { mutableStateOf<String?>(null) }
    var limit by rememberSaveable(project) { mutableIntStateOf(15) }
    val discussion = rememberRepoComments(env, project.substringBefore('/'), project.substringAfter('/'), open,
        enabled = (state as? Load.Ready)?.value != null)

    fun scrollToComments() {
        scope.launch {
            val index = listState.layoutInfo.totalItemsCount - discussion.itemCount
            if (index >= 0) listState.animateScrollToItem(index)
        }
    }

    var downloadNotice by remember(project) { mutableStateOf<String?>(null) }
    CommentScaffold(env, discussion) {
        ScreenBar("软件详情", back, trailing = {
            if ((state as? Load.Ready)?.value != null) {
                IconTap(XIcons.Message, "查看评论", ::scrollToComments)
            }
        })
        downloadNotice?.let { Banner(it, warn = false) }
        LoadBox(state, catalog::reload) { entry ->
            if (entry == null) {
                CenterMessage("软件未收录")
                return@LoadBox
            }
            val session by env.session.state.collectAsState()
            val (saved, setSaved) = rememberPrefFlag(env, "fav_repo:$project")
            val data = (releases as? Load.Ready)?.value
            val history = data?.history.orEmpty()
            val bundled = (initialState as? Load.Ready)?.value
            val usingBundled = selected == null && data == null && bundled != null && releases !is Load.Loading
            val release = when {
                selected != null -> history.firstOrNull { it.version == selected }
                usingBundled -> bundled
                else -> data?.latest
            }
            val packages = remember(release, entry, platform) {
                release?.packages(entry).orEmpty().filter { platform == "all" || it.platform == platform }
            }
            LazyColumn(
                modifier = Modifier.weight(1f),
                state = listState,
                contentPadding = PaddingValues(start = 12.dp, top = 8.dp, end = 12.dp, bottom = 16.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                item { SoftwareCard(entry, null) { env.platform?.openInBrowser("https://github.com/${entry.project}") } }
                item { Text(if (selected == null) "最新版本" else "版本 ${selected.orEmpty()}", style = XType.Section) }
                val failure = releases as? Load.Failed
                if (failure != null) item {
                    Banner(failure.message, warn = true, actionText = "重试", onAction = snapshot::reload)
                }
                when {
                    release != null -> {
                        item {
                            Text(listOfNotNull(release.version, release.publishedAt?.take(10),
                                "预发布".takeIf { release.prerelease }, "收录时版本".takeIf { usingBundled }).joinToString(" · "))
                        }
                        item { PageChipRow(listOf("all" to "全部平台") + entry.platforms.map { it to it }, platform, { platform = it }) }
                        items(packages, key = { it.asset.url }) { pkg ->
                            Column(Modifier.fillMaxWidth().outlinedCard(12.dp).padding(12.dp)) {
                                Text(pkg.asset.name, style = XType.Body)
                                Text(listOfNotNull(pkg.platform, pkg.architecture, pkg.asset.size?.let(::formatDownloadSize)).joinToString(" · "), style = XType.Caption)
                                GhostPillButton("下载官方安装包", { env.platform?.openInBrowser(pkg.asset.url) }, enabled = env.platform != null)
                            }
                        }
                        if (packages.isEmpty()) item { Text("此版本没有符合条件的安装包", style = XType.Caption) }
                    }
                    releases is Load.Loading || initialState is Load.Loading ->
                        item { Spinner(Modifier.fillMaxWidth().height(54.dp)) }
                    else -> item { Text("暂无正式版安装包，可查看 GitHub 官方发布页", style = XType.Secondary) }
                }
                item {
                    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        GhostPillButton("GitHub 官方发布页", { env.platform?.openInBrowser(release?.let { entry.releaseUrl(it.version) } ?: entry.releasesUrl) }, enabled = env.platform != null)
                        GhostPillButton("项目说明", {
                            open(Route.Repo(project.substringBefore('/'), project.substringAfter('/')))
                        })
                    }
                }
                if (entry.summary.isNotBlank()) item {
                    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text("软件简介", style = XType.Section)
                        Text(entry.summary, style = XType.Body)
                    }
                }
                item { Text("历史版本", style = XType.Section) }
                if (releases is Load.Ready && history.isEmpty()) item { Text("暂无已同步的历史版本", style = XType.Secondary) }
                if (selected != null) item { LinkText("回到最新版本", { selected = null; platform = "all"; scope.launch { listState.scrollToItem(1) } }) }
                items(history.take(limit), key = { it.version }) { previous ->
                    Row(Modifier.fillMaxWidth().outlinedCard(10.dp).clickable {
                        selected = previous.version
                        platform = "all"
                        scope.launch { listState.scrollToItem(1) }
                    }.padding(12.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                        Text(previous.version, style = XType.Body)
                        Text(previous.publishedAt.orEmpty().take(10), style = XType.Caption)
                    }
                }
                if (history.size > limit) item { GhostPillButton("更多历史版本", { limit += 15 }) }
                plainCommentItems(env, discussion, open, horizontalPadding = 0.dp)
            }
            CommentBar(if (session.loggedIn) "写评论…" else "登录后评论", onWrite = { discussion.begin() }) {
                if (env.platform != null) {
                    IconTap(XIcons.ExternalLink, "在 GitHub 查看项目", {
                        env.platform.openInBrowser("https://github.com/${entry.project}")
                    })
                }
                IconTap(XIcons.Star, if (saved) "取消收藏" else "收藏", { setSaved(!saved) },
                    tint = if (saved) Brand else MaterialTheme.colorScheme.onSurface)
                CommentCountAction(discussion.total, ::scrollToComments)
            }
        }
    }
}
