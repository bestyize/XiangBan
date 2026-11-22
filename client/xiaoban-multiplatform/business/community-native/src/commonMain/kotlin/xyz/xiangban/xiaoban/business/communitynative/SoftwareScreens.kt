package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.draw.clip
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import xyz.xiangban.xiaoban.common.share.ShareContent
import xyz.xiangban.xiaoban.common.share.ShareExtraAction
import xyz.xiangban.xiaoban.common.share.ShareIconKey
import xyz.xiangban.xiaoban.common.share.ShareThumbKind
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
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
            LazyColumn(state = listState, contentPadding = CommunityFeedPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
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
    Row(
        Modifier.fillMaxWidth().padding(horizontal = 6.dp).padding(top = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        CommunityTabRow(SoftwareCategories, category, Modifier.weight(1f), filter = true, onSelect = onCategory)
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
    val shape = RoundedCornerShape(10.dp)
    var iconFailed by remember(entry.project, entry.icon) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth().communityFeedCard().clickable(onClick = open).padding(12.dp),
        verticalArrangement = Arrangement.spacedBy(9.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Box(Modifier.size(38.dp).clip(shape).background(feedFill).border(1.dp, LocalExtraColors.current.hairline, shape),
                contentAlignment = Alignment.Center) {
                if (iconFailed) {
                    Text(entry.displayName.take(1), style = feedText(13.5, 18.9), fontWeight = FontWeight.Bold, color = feedInk)
                } else {
                    val icon = entry.icon.ifBlank { "https://github.com/${entry.project.substringBefore('/')}.png?size=80" }
                    AsyncImage(icon, contentDescription = null, imageLoader = rememberSoftwareImageLoader(),
                        modifier = Modifier.size(24.dp), contentScale = ContentScale.Fit, onError = { iconFailed = true })
                }
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(entry.displayName, Modifier.weight(1f), style = feedText(14.5, 20.3), color = feedInk,
                        fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    (snapshot?.stars ?: entry.stars)?.let { Text("★ ${formatStars(it)}", style = feedText(11.5, 16.1), color = feedMuted) }
                }
                Text(entry.project, style = feedText(11), color = feedMuted, fontFamily = FontFamily.Monospace,
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        if (entry.summary.isNotBlank()) Text(entry.summary, style = feedText(12, 18.0), color = feedSecondary,
            maxLines = 2, overflow = TextOverflow.Ellipsis)
        val version = if (snapshot == null) entry.version else snapshot.version.orEmpty()
        if (version.isNotBlank() || entry.language.isNotBlank()) {
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                val badge = RoundedCornerShape(6.dp)
                if (version.isNotBlank()) Text(version,
                    Modifier.background(LocalExtraColors.current.brandSoft, badge).padding(horizontal = 7.dp, vertical = 2.dp),
                    style = feedText(11), color = LocalExtraColors.current.brandInk, fontFamily = FontFamily.Monospace, fontWeight = FontWeight.SemiBold,
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (entry.language.isNotBlank()) {
                    Row(Modifier.background(feedFill, badge).padding(horizontal = 7.dp, vertical = 2.dp),
                        horizontalArrangement = Arrangement.spacedBy(5.dp), verticalAlignment = Alignment.CenterVertically) {
                        Dot(softwareLanguageColor(entry.language), 6.dp)
                        Text(entry.language, style = feedText(11), color = feedSecondary, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                }
            }
        }
        Column {
            Hairline()
            Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Row(horizontalArrangement = Arrangement.spacedBy(7.dp), verticalAlignment = Alignment.CenterVertically) {
                    entry.platforms.forEach { platform -> SoftwarePlatformMark(platform) }
                }
                (snapshot?.updatedAt ?: entry.updatedAt)?.takeIf(String::isNotBlank)?.let {
                    Text("更新于 ${it.take(10)}", style = feedText(11), color = feedMuted, maxLines = 1)
                }
            }
        }
    }
}

private fun softwareLanguageColor(language: String): Color = when (language.lowercase()) {
    "python" -> Color(0xFF3572A5)
    "typescript" -> Color(0xFF3178C6)
    "javascript" -> Color(0xFFF1E05A)
    "c++" -> Color(0xFFF34B7D)
    "c#" -> Color(0xFF178600)
    "rust" -> Color(0xFFDEA584)
    "go" -> Color(0xFF00ADD8)
    "java" -> Color(0xFFB07219)
    else -> Color(0xFF94A3B8)
}

private val softwarePlatformPaths = mapOf(
    "Windows" to "M1 2.3 7.3 1.4v6.1H1V2.3Zm7.2-1L15 .3v7.2H8.2V1.3ZM1 8.5h6.3v6.1L1 13.7V8.5Zm7.2 0H15v7.2l-6.8-1V8.5Z",
    "macOS" to "M11.6 8.4c0-1.7 1.4-2.5 1.5-2.6-.8-1.2-2.1-1.4-2.5-1.4-1.1-.1-2.1.6-2.7.6-.5 0-1.4-.6-2.3-.6-1.2 0-2.3.7-2.9 1.7-1.2 2.1-.3 5.2.9 6.9.6.8 1.3 1.8 2.2 1.7.9 0 1.2-.6 2.3-.6s1.4.6 2.3.6c1.1 0 1.6-.8 2.2-1.7.7-1 1-1.9 1-2-.1 0-2-.8-2-2.6ZM9.9 3.3c.5-.6.8-1.4.7-2.2-.7 0-1.6.5-2.1 1.1-.4.5-.8 1.3-.7 2.1.8.1 1.6-.4 2.1-1Z",
    "Linux" to "M8 1.5c1.7 0 2.7 1.2 2.7 3 0 .8-.2 1.5.4 2.3l1.7 2.3c.7 1 .4 2.8-.8 3.6-.8.6-2.2.8-4 .8s-3.2-.2-4-.8c-1.2-.8-1.5-2.6-.8-3.6l1.7-2.3c.6-.8.4-1.5.4-2.3 0-1.8 1-3 2.7-3Z",
    "Android" to "M11.6 4.7 12.8 2.6a.35.35 0 1 0-.6-.35L11 4.4A5.5 5.5 0 0 0 5 4.4L3.8 2.25a.35.35 0 0 0-.6.35l1.2 2.1A4.7 4.7 0 0 0 2 8.7h12a4.7 4.7 0 0 0-2.4-4Zm-5.8 2a.6.6 0 1 1 0-1.2.6.6 0 0 1 0 1.2Zm4.4 0a.6.6 0 1 1 0-1.2.6.6 0 0 1 0 1.2ZM2.8 9.6v3.7a1 1 0 0 0 1 1h.8v1.2a.7.7 0 0 0 1.4 0v-1.2h4v1.2a.7.7 0 0 0 1.4 0v-1.2h.8a1 1 0 0 0 1-1V9.6H2.8Z",
)

@Composable
private fun SoftwarePlatformMark(platform: String) {
    val key = softwarePlatformPaths.keys.firstOrNull { it.equals(platform, ignoreCase = true) }
    val path = softwarePlatformPaths[key]
    if (path == null) {
        Text(platform, style = feedText(11), color = feedMuted, maxLines = 1)
        return
    }
    val icon = remember(key) {
        ImageVector.Builder(platform, 13.dp, 13.dp, 16f, 16f)
            .addPath(PathParser().parsePathString(path).toNodes(), fill = SolidColor(Color.Black)).build()
    }
    val tint = when (key) {
        "Windows" -> Color(0xFF0078D4)
        "Linux" -> Color(0xFFD97706)
        "Android" -> Color(0xFF22C55E)
        else -> feedInk
    }
    XIcon(icon, tint = tint, size = 13.dp, description = platform)
}

@Composable
internal fun SoftwareDetailScreen(env: CommunityEnv, project: String, back: () -> Unit, open: (Route) -> Unit) = CommunityDetail {
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
    var sharing by rememberSaveable(project) { mutableStateOf(false) }
    CommentScaffold(env, discussion) {
        if ((state as? Load.Ready)?.value == null) ScreenBar("软件详情", back)
        downloadNotice?.let { Banner(it, warn = false) }
        LoadBox(state, catalog::reload) { entry ->
            if (entry == null) {
                CenterMessage("软件未收录")
                return@LoadBox
            }
            DetailAuthorBar(entry.project, back, onShare = { sharing = true })
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
                contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 16.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                item {
                    DetailBody(horizontalPadding = 0.dp) {
                        DetailHeading(entry.displayName)
                        if (entry.summary.isNotBlank()) Text(entry.summary, style = XType.DetailBody)
                        DetailTags(listOf("开源软件", entry.language) + entry.platforms)
                        DetailInfoStrip("下载", "查看官方安装包", XIcons.Download, onClick = { scope.launch { listState.animateScrollToItem(1) } })
                        webPageRoute("https://github.com/${entry.project}")?.let { github ->
                            DetailInfoStrip("GitHub", entry.project, XIcons.ExternalLink, onClick = { open(github) })
                        }
                        DetailMetadata(listOfNotNull(data?.latest?.version ?: entry.version.takeIf(String::isNotBlank),
                            (data?.latest?.publishedAt ?: entry.updatedAt)?.let { "更新于 ${it.take(10)}" }).joinToString(" · "))
                    }
                }
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
            CommentBar("说点什么…", onWrite = { discussion.begin() }) {
                DetailDockAction(XIcons.Star, if (saved) "已收藏" else "收藏", if (saved) "取消收藏" else "收藏",
                    onClick = { setSaved(!saved) }, tint = if (saved) Amber else MaterialTheme.colorScheme.onSurface)
                CommentCountAction(discussion.total, ::scrollToComments)
            }
        }
    }
    OverlayTransition(sharing) { panel ->
        val entry = (state as? Load.Ready)?.value
        if (entry != null) {
            val repoUrl = "https://github.com/${entry.project}"
            val extras = remember(repoUrl, env.platform) {
                if (env.platform != null) listOf(
                    ShareExtraAction("copyRepo", "复制仓库", ShareIconKey.Code) {
                        env.platform.copy(repoUrl)
                        discussion.toast = "仓库地址已复制"
                        "已复制 GitHub 仓库地址"
                    },
                ) else emptyList()
            }
            DirectShareSheet(
                env = env,
                content = ShareContent(
                    title = entry.displayName,
                    url = repoUrl,
                    summary = entry.summary,
                    author = entry.project.substringBefore('/'),
                    dateLabel = entry.language,
                    kindLabel = "开源软件",
                    badge = "开源",
                    thumbKind = ShareThumbKind.Repo,
                    thumbMonogram = entry.displayName.firstOrNull()?.uppercaseChar()?.toString(),
                ),
                onDismiss = { sharing = false },
                onCopied = { discussion.toast = "链接已复制" },
                extraActions = extras,
                panelModifier = panel,
            )
        }
    }
}
