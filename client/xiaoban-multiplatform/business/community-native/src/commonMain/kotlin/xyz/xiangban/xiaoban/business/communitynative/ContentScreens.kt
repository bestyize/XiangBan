package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as gridItems
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.DiscussController
import xyz.xiangban.xiaoban.common.community.FeedController
import xyz.xiangban.xiaoban.common.community.FeedState
import xyz.xiangban.xiaoban.common.community.Loader
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.PackKinds
import xyz.xiangban.xiaoban.common.community.Post
import xyz.xiangban.xiaoban.common.community.Project
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.VisualKinds
import xyz.xiangban.xiaoban.common.community.WeeklyEntry
import xyz.xiangban.xiaoban.common.community.WeeklyIssue
import xyz.xiangban.xiaoban.common.community.formatSize
import xyz.xiangban.xiaoban.common.community.htmlToPlain
import xyz.xiangban.xiaoban.common.community.kindLabel
import xyz.xiangban.xiaoban.common.community.messageOf

// ───────────── 共用部件 ─────────────

private val AvatarColors = listOf(0xFF8FA9C9L, 0xFFF4A6A0L, 0xFF9FBFCFL, 0xFFD9C7A8L, 0xFFE8C547L, 0xFF9BD1B4L)

/** 没有头像图时按名字取一个固定的底色圆点。 */
@Composable
fun Avatar(name: String, size: androidx.compose.ui.unit.Dp = 18.dp, showInitial: Boolean = false) {
    val color = Color(AvatarColors[(name.hashCode() and 0x7fffffff) % AvatarColors.size])
    Box(Modifier.size(size).clip(CircleShape).background(color), contentAlignment = Alignment.Center) {
        if (showInitial) Text(name.take(1), color = Color.White, fontSize = (size.value * 0.45f).sp, fontWeight = FontWeight.SemiBold)
    }
}

/** 栏目条：白底、文字加红色短下划线，选中的加粗。 */
@Composable
fun TabStrip(tabs: List<Pair<String, String>>, selected: String, onSelect: (String) -> Unit) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        LazyRow(Modifier.fillMaxWidth().height(40.dp), contentPadding = PaddingValues(horizontal = 16.dp), horizontalArrangement = Arrangement.spacedBy(24.dp), verticalAlignment = Alignment.CenterVertically) {
            items(tabs, key = { it.first }) { (key, label) ->
                val on = key == selected
                Box(Modifier.fillMaxHeight().clickable { onSelect(key) }, contentAlignment = Alignment.Center) {
                    Text(label, fontSize = 14.sp, fontWeight = if (on) FontWeight.Bold else FontWeight.Normal, color = if (on) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant)
                    Box(Modifier.align(Alignment.BottomCenter).padding(bottom = 3.dp).size(width = 18.dp, height = 3.dp).clip(RoundedCornerShape(2.dp)).background(if (on) Brand else Color.Transparent))
                }
            }
        }
        Hairline()
    }
}

@Composable
private fun LoadMoreFooter(hasMore: Boolean, error: String?, count: Int, onMore: () -> Unit, onRetry: () -> Unit) {
    when {
        error != null -> Text("加载失败，点这里重试", Modifier.fillMaxWidth().clickable(onClick = onRetry).padding(16.dp), color = Brand, fontSize = 13.sp)
        hasMore -> {
            // 这一项滚到可见才会进入组合，以此触发下一页；条数变化后再次触发。
            LaunchedEffect(count) { onMore() }
            Spinner(Modifier.fillMaxWidth().height(48.dp))
        }
    }
}

/** 底部短暂提示（“链接已复制”）。 */
@Composable
fun Toast(text: String?, onGone: () -> Unit) {
    if (text == null) return
    LaunchedEffect(text) { delay(1600); onGone() }
    Box(Modifier.fillMaxSize().padding(bottom = 120.dp), contentAlignment = Alignment.BottomCenter) {
        Text(text, Modifier.clip(RoundedCornerShape(18.dp)).background(Color.Black.copy(alpha = 0.78f)).padding(horizontal = 16.dp, vertical = 8.dp), color = Color.White, fontSize = 13.sp)
    }
}

/** 分享弹层：预览卡片加“系统分享”“复制链接”。手机上分享的是网页链接，对方在电脑上打开后再用小伴桌面版。 */
@Composable
private fun ShareSheet(env: CommunityEnv, title: String, sub: String, url: String, heading: String = "分享", hint: String? = null, onDismiss: () -> Unit, onCopied: () -> Unit) {
    BottomSheet(onDismiss) {
        Text(heading, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        Row(Modifier.padding(top = 12.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.background).padding(10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(48.dp).clip(RoundedCornerShape(6.dp)).background(Brand.copy(alpha = 0.12f)), contentAlignment = Alignment.Center) { XIcon(XIcons.Link, tint = Brand, size = 24.dp) }
            Column(Modifier.weight(1f)) {
                Text(title, fontSize = 14.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (sub.isNotBlank()) Text(sub, Modifier.padding(top = 2.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        if (hint != null) Text(hint, Modifier.padding(top = 12.dp), fontSize = 13.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        val platform = env.platform
        Row(Modifier.padding(top = 16.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            GhostPillButton("复制链接", { platform?.copy(url); onDismiss(); onCopied() }, Modifier.weight(1f), enabled = platform != null)
            PillButton("分享…", { platform?.share(title, url); onDismiss() }, Modifier.weight(1f), enabled = platform != null)
        }
        GhostPillButton("取消", onDismiss, Modifier.padding(top = 10.dp).fillMaxWidth())
    }
}

@Composable
private fun <T> rememberLoader(env: CommunityEnv, key: Any, fetch: suspend () -> T): Loader<T> {
    val scope = rememberCoroutineScope()
    val loader = remember(env, key) { Loader(scope, fetch) }
    LaunchedEffect(loader) { loader.reload() }
    return loader
}

// ───────────── 社区首页（信息流） ─────────────

private val FeedTabs: List<Pair<String, String>> = listOf("" to "全部", "weekly" to "周刊", "projects" to "项目") + PackKinds

internal fun packMeta(pack: Pack): String =
    listOfNotNull(kindLabel(pack.kind), pack.author?.nickname, pack.trustLabel.takeIf { pack.trustLevel < 2 }, "${pack.applyCount} 次应用".takeIf { pack.applyCount > 0 }).joinToString(" · ")

private val FeedState.isEmptyList: Boolean get() = packs.isEmpty() && weekly.isEmpty() && projects.isEmpty()

/** 社区首页：栏目条加信息流。图片优先的类型用双列封面卡，文字为主的占满一行。 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeedScreen(env: CommunityEnv, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { FeedController(env.forum, scope) }
    val state by controller.state.collectAsState()
    var tab by rememberSaveable { mutableStateOf("") }
    LaunchedEffect(tab) { controller.select(tab) }

    Column(Modifier.fillMaxSize()) {
        TabStrip(FeedTabs, tab) { tab = it }
        PullToRefreshBox(isRefreshing = state.loading && state.isEmptyList, onRefresh = controller::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.loading && state.isEmptyList -> Spinner()
                state.error != null && state.isEmptyList -> CenterMessage(state.error.orEmpty(), "重试", controller::refresh)
                state.isEmpty -> CenterMessage("这里还没有内容")
                tab == "weekly" -> WeeklyList(state, open, controller::loadMore, controller::refresh)
                tab == "projects" -> ProjectList(state.projects, open)
                else -> PackGrid(state, open, controller::loadMore, controller::refresh)
            }
        }
    }
}

@Composable
private fun PackGrid(state: FeedState, open: (Route) -> Unit, onMore: () -> Unit, onRetry: () -> Unit) {
    LazyVerticalGrid(
        GridCells.Fixed(2), Modifier.fillMaxSize(),
        contentPadding = PaddingValues(8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        gridItems(state.packs, key = { it.id }, span = { pack -> GridItemSpan(if (pack.kind in VisualKinds) 1 else 2) }) { pack ->
            if (pack.kind in VisualKinds) VisualCard(pack) { open(Route.Pack(pack.id)) } else TextCard(pack) { open(Route.Pack(pack.id)) }
        }
        item(span = { GridItemSpan(2) }) { LoadMoreFooter(state.hasMore, state.error, state.packs.size, onMore, onRetry) }
    }
}

@Composable
private fun VisualCard(pack: Pack, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick)) {
        Box {
            Cover(pack.coverUrl, pack.id, Modifier.fillMaxWidth().aspectRatio(4f / 3f))
            Text(kindLabel(pack.kind), Modifier.padding(8.dp).clip(RoundedCornerShape(6.dp)).background(Color.Black.copy(alpha = 0.45f)).padding(horizontal = 6.dp, vertical = 2.dp), color = Color.White, fontSize = 10.sp)
        }
        Column(Modifier.padding(start = 8.dp, end = 8.dp, top = 8.dp, bottom = 10.dp)) {
            Text(pack.title, fontSize = 14.sp, fontWeight = FontWeight.Medium, lineHeight = 20.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(Modifier.padding(top = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                Avatar(pack.author?.nickname ?: "?")
                Text(pack.author?.nickname ?: "匿名", Modifier.weight(1f), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (pack.applyCount > 0) Text("${pack.applyCount}", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
        }
    }
}

@Composable
private fun TextCard(pack: Pack, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(start = 10.dp, end = 10.dp, top = 12.dp, bottom = 10.dp)) {
        Text(kindLabel(pack.kind), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
        Text(pack.title, Modifier.padding(top = 6.dp), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, lineHeight = 21.sp)
        if (pack.summary.isNotBlank()) Text(pack.summary, Modifier.padding(top = 4.dp), fontSize = 13.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
        Row(Modifier.padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
            Avatar(pack.author?.nickname ?: "?")
            Text(listOfNotNull(pack.author?.nickname, pack.trustLabel.takeIf { pack.trustLevel < 2 }, "${pack.applyCount} 次应用".takeIf { pack.applyCount > 0 }).joinToString(" · "), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

@Composable
private fun WeeklyList(state: FeedState, open: (Route) -> Unit, onMore: () -> Unit, onRetry: () -> Unit) {
    LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        items(state.weekly, key = { it.week }) { issue -> WeeklyRow(issue) { open(Route.Weekly(issue.week)) }; Hairline(Modifier.padding(start = 16.dp)) }
        item { LoadMoreFooter(state.hasMore, state.error, state.weekly.size, onMore, onRetry) }
    }
}

@Composable
private fun WeeklyRow(issue: WeeklyIssue, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(issue.week, Modifier.clip(RoundedCornerShape(10.dp)).background(Brand).padding(horizontal = 10.dp, vertical = 2.dp), color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            Text(issue.publishedAt?.take(10).orEmpty(), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Text(issue.title, Modifier.padding(top = 8.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold, lineHeight = 22.sp)
        if (issue.summary.isNotBlank()) Text(issue.summary, Modifier.padding(top = 4.dp), fontSize = 13.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun ProjectList(projects: List<Project>, open: (Route) -> Unit) {
    LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        items(projects, key = { it.slug }) { project ->
            Column(Modifier.fillMaxWidth().clickable { open(Route.Project(project.slug)) }.padding(16.dp)) {
                Text(project.name, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                if (project.summary.isNotBlank()) Text(project.summary, Modifier.padding(top = 4.dp), fontSize = 13.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
            Hairline(Modifier.padding(start = 16.dp))
        }
    }
}

// ───────────── 讨论 / 资源 ─────────────

private val Sorts = listOf("latest" to "最新回复", "new" to "最新发布", "hot" to "热门")

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DiscussScreen(env: CommunityEnv, open: (Route) -> Unit, category: String? = null, emptyText: String = "还没有帖子") {
    val scope = rememberCoroutineScope()
    val controller = remember(env, category) { DiscussController(env.forum, scope, category) }
    val state by controller.state.collectAsState()
    var sort by rememberSaveable { mutableStateOf("latest") }
    LaunchedEffect(sort) { controller.select(sort) }

    Column(Modifier.fillMaxSize()) {
        TabStrip(Sorts, sort) { sort = it }
        PullToRefreshBox(isRefreshing = state.loading && state.topics.isEmpty(), onRefresh = controller::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.loading && state.topics.isEmpty() -> Spinner()
                state.error != null && state.topics.isEmpty() -> CenterMessage(state.error.orEmpty(), "重试", controller::refresh)
                state.topics.isEmpty() -> CenterMessage(emptyText)
                else -> LazyColumn(Modifier.fillMaxSize().padding(top = 8.dp).background(MaterialTheme.colorScheme.surface)) {
                    items(state.topics, key = { it.id }) { topic -> TopicRow(topic) { open(Route.Topic(topic.id)) }; Hairline(Modifier.padding(start = 16.dp)) }
                    item { LoadMoreFooter(state.hasMore, state.error, state.topics.size, controller::loadMore, controller::loadMore) }
                }
            }
        }
    }
}

@Composable
internal fun TopicRow(topic: Topic, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(16.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
            if (topic.pinned) Text("置顶", fontSize = 11.sp, fontWeight = FontWeight.SemiBold, color = Brand)
            if (topic.featured) Text("精华", fontSize = 11.sp, fontWeight = FontWeight.SemiBold, color = Brand)
            Text(topic.category.name, fontSize = 11.sp, color = LocalExtraColors.current.ink3)
        }
        Text(topic.title, Modifier.padding(top = 6.dp), fontSize = 15.sp, fontWeight = FontWeight.Medium, lineHeight = 22.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
        if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 4.dp), fontSize = 13.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Row(Modifier.padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Avatar(topic.author.nickname, 16.dp)
            Text(topic.author.nickname, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            Text("·", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            Text("${topic.replyCount} 回复", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        }
    }
}

// ───────────── 搜索 ─────────────

/** 搜索作品和帖子。没有输入时显示本次打开应用以来的搜索历史。 */
@Composable
fun SearchScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    var query by rememberSaveable { mutableStateOf("") }
    var history by rememberSaveable { mutableStateOf("") }
    var packs by remember { mutableStateOf<List<Pack>>(emptyList()) }
    var topics by remember { mutableStateOf<List<Topic>>(emptyList()) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var searched by remember { mutableStateOf(false) }

    fun search(text: String) {
        val q = text.trim()
        if (q.isEmpty() || busy) return
        query = q
        history = (listOf(q) + history.split('\n').filter { it.isNotBlank() && it != q }).take(8).joinToString("\n")
        busy = true; error = null; searched = true
        scope.launch {
            try {
                val p = runCatching { env.forum.packs(q = q).items }
                val t = runCatching { env.forum.searchTopics(q).items }
                packs = p.getOrDefault(emptyList())
                topics = t.getOrDefault(emptyList())
                if (p.isFailure && t.isFailure) error = messageOf(p.exceptionOrNull() ?: t.exceptionOrNull() ?: RuntimeException())
            } finally {
                busy = false
            }
        }
    }

    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        Row(Modifier.fillMaxWidth().statusBarsPadding().padding(start = 16.dp, end = 8.dp, top = 8.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically) {
                TextBox(query, { query = it }, Modifier.weight(1f), placeholder = "搜索作品、帖子", keyboardType = KeyboardType.Text)
            }
            PillButton("搜索", { search(query) }, enabled = query.isNotBlank() && !busy, height = 40.dp)
            Text("取消", Modifier.clickable(onClick = back).padding(horizontal = 8.dp, vertical = 12.dp), fontSize = 15.sp)
        }
        Hairline()
        when {
            busy -> Spinner()
            error != null -> CenterMessage(error.orEmpty(), "重试", onAction = { search(query) })
            !searched -> Column(Modifier.fillMaxWidth().padding(horizontal = 16.dp)) {
                val items = history.split('\n').filter { it.isNotBlank() }
                if (items.isEmpty()) {
                    Text("输入关键词，搜索社区里的作品和帖子。", Modifier.padding(top = 20.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                } else {
                    Row(Modifier.fillMaxWidth().padding(top = 16.dp, bottom = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text("搜索历史", Modifier.weight(1f), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                        Text("清空", Modifier.clickable { history = "" }.padding(4.dp), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        items.forEach { h -> Text(h, Modifier.clip(CircleShape).background(LocalExtraColors.current.fill).clickable { search(h) }.padding(horizontal = 14.dp, vertical = 7.dp), fontSize = 13.sp) }
                    }
                }
            }
            packs.isEmpty() && topics.isEmpty() -> CenterMessage("没有找到和“$query”相关的内容")
            else -> Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                if (packs.isNotEmpty()) {
                    GroupLabel("作品")
                    packs.forEachIndexed { i, pack ->
                        if (i > 0) Hairline(Modifier.padding(start = 16.dp))
                        Column(Modifier.fillMaxWidth().clickable { open(Route.Pack(pack.id)) }.padding(16.dp)) {
                            Text(pack.title, fontSize = 15.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Text(packMeta(pack), Modifier.padding(top = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        }
                    }
                }
                if (topics.isNotEmpty()) {
                    GroupLabel("帖子")
                    topics.forEachIndexed { i, topic ->
                        if (i > 0) Hairline(Modifier.padding(start = 16.dp))
                        TopicRow(topic) { open(Route.Topic(topic.id)) }
                    }
                }
            }
        }
    }
}

// ───────────── 详情页 ─────────────

/** 作品详情：顶部封面（带返回和分享），标题、标签、作者、简介、更新记录、评论；底部“发到电脑”。 */
@Composable
fun PackDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit) {
    val loader = rememberLoader(env, id) { env.forum.pack(id) }
    val state by loader.state.collectAsState()
    var share by remember { mutableStateOf(false) }
    var toast by remember { mutableStateOf<String?>(null) }
    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        LoadBox(state, loader::reload) { pack ->
            Column(Modifier.fillMaxSize()) {
                Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())) {
                    Box(Modifier.fillMaxWidth().height(232.dp)) {
                        Cover(pack.coverUrl, pack.id, Modifier.fillMaxSize())
                        CircleOverlayButton(XIcons.ChevronLeft, "返回", back, Modifier.align(Alignment.TopStart).padding(start = 12.dp, top = 12.dp))
                        CircleOverlayButton(XIcons.Share, "分享", { share = true }, Modifier.align(Alignment.TopEnd).padding(end = 12.dp, top = 12.dp))
                    }
                    Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp)) {
                        Text(pack.title, fontSize = 22.sp, fontWeight = FontWeight.SemiBold, lineHeight = 29.sp)
                        Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            Text(kindLabel(pack.kind), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            Text(pack.trustLabel, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            if (pack.pureData) Text("纯数据", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            if (pack.applyCount > 0) Text("${pack.applyCount} 次应用", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                        }
                        Row(Modifier.padding(top = 16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                            Avatar(pack.author?.nickname ?: "?", 36.dp, showInitial = true)
                            Column(Modifier.weight(1f)) {
                                Text(pack.author?.nickname ?: "匿名", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                                Text("v${pack.version}" + (pack.versions.firstOrNull { it.version == pack.version }?.size?.takeIf { it > 0 }?.let { " · " + formatSize(it) } ?: "") + pack.updatedAt.take(10).takeIf { it.isNotBlank() }?.let { " · 更新于 $it" }.orEmpty(), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                        if (pack.summary.isNotBlank()) Text(pack.summary, Modifier.padding(top = 16.dp), fontSize = 15.sp, lineHeight = 26.sp)
                        if (pack.tags.isNotEmpty()) Text(pack.tags.joinToString("  ") { "#$it" }, Modifier.padding(top = 12.dp), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        val changes = pack.versions.filter { it.changelog.isNotBlank() }
                        if (changes.isNotEmpty()) {
                            Text("更新记录", Modifier.padding(top = 24.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                            changes.sortedByDescending { it.version }.forEach { v ->
                                Text(v.label.ifBlank { "v${v.version}" }, Modifier.padding(top = 10.dp), fontSize = 13.sp, fontWeight = FontWeight.Medium)
                                Text(v.changelog, Modifier.padding(top = 2.dp), fontSize = 13.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    }
                    pack.topicId?.let { CommentsSection(env, it) }
                }
                Hairline()
                Box(Modifier.fillMaxWidth().height(72.dp).padding(horizontal = 12.dp), contentAlignment = Alignment.Center) {
                    PillButton("发到电脑", { share = true }, Modifier.fillMaxWidth())
                }
            }
            if (share) {
                ShareSheet(
                    env, pack.title, kindLabel(pack.kind) + (pack.author?.nickname?.let { " · $it" } ?: ""), env.forum.packUrl(pack.id),
                    heading = "发到电脑",
                    hint = "手机上不能直接应用到桌面。把链接发到电脑（比如微信的“文件传输助手”），在电脑上用浏览器打开，再用桌面版小伴一键应用。",
                    onDismiss = { share = false }, onCopied = { toast = "链接已复制" },
                )
            }
        }
        if (state !is xyz.xiangban.xiaoban.common.community.Load.Ready<*>) {
            Box(Modifier.align(Alignment.TopStart).padding(start = 4.dp, top = 4.dp)) { IconTap(XIcons.ChevronLeft, "返回", back) }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
private fun CircleOverlayButton(icon: androidx.compose.ui.graphics.vector.ImageVector, description: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Box(modifier.size(40.dp).clip(CircleShape).background(Color.Black.copy(alpha = 0.4f)).clickable(onClick = onClick), contentAlignment = Alignment.Center) {
        XIcon(icon, tint = Color.White, size = 22.dp, description = description)
    }
}

/** 作品对应的讨论帖里的评论，分页加载。[reloadKey] 变化时从头重新加载（发完回复后用）。 */
@Composable
private fun CommentsSection(env: CommunityEnv, topicId: Long, reloadKey: Int = 0, hint: String = "评论和点赞请在网页端进行：") {
    var posts by remember(topicId, reloadKey) { mutableStateOf<List<Post>>(emptyList()) }
    var page by remember(topicId, reloadKey) { mutableStateOf(0) }
    var more by remember(topicId, reloadKey) { mutableStateOf(true) }
    var error by remember(topicId, reloadKey) { mutableStateOf<String?>(null) }
    var busy by remember(topicId, reloadKey) { mutableStateOf(false) }
    var retry by remember { mutableStateOf(0) }

    suspend fun load() {
        if (busy || !more) return
        busy = true; error = null
        try {
            val next = env.forum.posts(topicId, page + 1)
            posts = posts + next.items
            page += 1
            more = next.hasMore
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (e: Exception) {
            error = messageOf(e)
        } finally {
            busy = false
        }
    }
    LaunchedEffect(topicId, retry, reloadKey) { load() }

    Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 24.dp, bottom = 16.dp)) {
        Text("评论 ${posts.size}${if (more && posts.isNotEmpty()) "+" else ""}", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
        posts.forEach { post -> PostItem(post) }
        when {
            error != null -> Text("评论加载失败，点这里重试", Modifier.padding(top = 12.dp).clickable { retry += 1 }, color = Brand, fontSize = 13.sp)
            more -> { LaunchedEffect(posts.size) { load() }; Spinner(Modifier.fillMaxWidth().height(48.dp)) }
            posts.isEmpty() -> Text("还没有评论", Modifier.padding(top = 12.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
        }
        if (hint.isNotBlank()) {
            Text(hint, Modifier.padding(top = 12.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            LinkText("在网页打开讨论 ›", { env.platform?.openUrl(env.forum.topicUrl(topicId)) })
        }
    }
}

@Composable
private fun PostItem(post: Post) {
    Row(Modifier.padding(top = 14.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Avatar(post.author.nickname, 30.dp, showInitial = true)
        Column(Modifier.weight(1f)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(post.author.nickname, fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                Text("${post.floor} 楼 · ${post.createdAt.take(10)}", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            val body = when {
                post.blocked -> "你已屏蔽该用户"
                post.status != "normal" -> "该评论已被隐藏"
                else -> htmlToPlain(post.bodyHtml)
            }
            Text(body, Modifier.padding(top = 2.dp), fontSize = 14.sp, lineHeight = 22.sp, color = if (post.blocked || post.status != "normal") LocalExtraColors.current.ink3 else MaterialTheme.colorScheme.onSurface)
        }
    }
}

/** 帖子详情：标题、作者、正文，下面是回复；底栏可以点赞、收藏、写回复（需要登录）。 */
@Composable
fun TopicDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val loader = rememberLoader(env, id) { env.forum.topic(id) }
    val state by loader.state.collectAsState()
    var share by remember { mutableStateOf(false) }
    var replying by remember { mutableStateOf(false) }
    var toast by remember { mutableStateOf<String?>(null) }
    var reloadKey by remember { mutableStateOf(0) }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("帖子", onBack = back, trailing = { IconTap(XIcons.Share, "分享", { share = true }) })
            LoadBox(state, loader::reload) { topic ->
                var liked by remember(topic.id) { mutableStateOf(topic.viewer?.liked == true) }
                var bookmarked by remember(topic.id) { mutableStateOf(topic.viewer?.bookmarked == true) }
                var likeCount by remember(topic.id) { mutableStateOf(topic.likeCount) }
                fun guarded(block: suspend () -> Unit) {
                    if (!session.loggedIn) { open(Route.Login); return }
                    scope.launch { try { block() } catch (e: kotlinx.coroutines.CancellationException) { throw e } catch (e: Exception) { toast = messageOf(e) } }
                }
                Column(Modifier.fillMaxSize()) {
                    Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())) {
                        Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 16.dp)) {
                            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                if (topic.pinned) Text("置顶", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = Brand)
                                Text(topic.category.name, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                            Text(topic.title, Modifier.padding(top = 10.dp), fontSize = 20.sp, fontWeight = FontWeight.SemiBold, lineHeight = 28.sp)
                            Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                Avatar(topic.author.nickname, 32.dp, showInitial = true)
                                Column {
                                    Text(topic.author.nickname, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                                    Text("发布于 ${topic.createdAt.take(10)}", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                            }
                            Text(htmlToPlain(topic.bodyHtml), Modifier.padding(top = 12.dp), fontSize = 15.sp, lineHeight = 26.sp)
                        }
                        Column(Modifier.padding(top = 8.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface)) { CommentsSection(env, id, reloadKey, hint = "") }
                    }
                    Hairline()
                    Row(Modifier.fillMaxWidth().height(60.dp).background(MaterialTheme.colorScheme.surface).padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                        Box(
                            Modifier.weight(1f).height(40.dp).padding(horizontal = 4.dp).clip(CircleShape).background(LocalExtraColors.current.fill).clickable { if (session.loggedIn) replying = true else open(Route.Login) }.padding(horizontal = 16.dp),
                            contentAlignment = Alignment.CenterStart,
                        ) { Text(if (session.loggedIn) "写回复…" else "登录后回复", fontSize = 14.sp, color = LocalExtraColors.current.ink3) }
                        Row(Modifier.clip(CircleShape).clickable {
                            guarded {
                                val next = !liked
                                env.forum.setTopicLiked(topic.id, next)
                                liked = next; likeCount = (likeCount + if (next) 1 else -1).coerceAtLeast(0)
                            }
                        }.padding(horizontal = 10.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            XIcon(XIcons.Heart, tint = if (liked) Brand else MaterialTheme.colorScheme.onSurface, description = if (liked) "取消点赞" else "点赞")
                            Text("$likeCount", fontSize = 13.sp, color = if (liked) Brand else MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        IconTap(XIcons.Bookmark, if (bookmarked) "取消收藏" else "收藏", {
                            guarded { val next = !bookmarked; env.forum.setTopicBookmarked(topic.id, next); bookmarked = next; toast = if (next) "已收藏" else "已取消收藏" }
                        }, tint = if (bookmarked) Brand else MaterialTheme.colorScheme.onSurface)
                    }
                }
            }
        }
        if (replying) {
            ReplySheet(env, id, onDismiss = { replying = false }, onSent = { replying = false; reloadKey += 1; toast = "已发送" })
        }
        if (share) {
            val topic = (state as? xyz.xiangban.xiaoban.common.community.Load.Ready<Topic>)?.value
            ShareSheet(env, topic?.title ?: "帖子", topic?.category?.name.orEmpty(), env.forum.topicUrl(id), onDismiss = { share = false }, onCopied = { toast = "链接已复制" })
        }
        Toast(toast) { toast = null }
    }
}

/** 写回复的弹层：多行输入加发送，失败时把服务端的原因显示在输入框下面。 */
@Composable
private fun ReplySheet(env: CommunityEnv, topicId: Long, onDismiss: () -> Unit, onSent: () -> Unit) {
    val scope = rememberCoroutineScope()
    var text by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    BottomSheet(onDismiss) {
        Text("回复", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        AreaBox(text, { text = it }, Modifier.padding(top = 12.dp), placeholder = "写下你的回复（至少 2 个字）…", minHeight = 120.dp, enabled = !busy)
        error?.let { Text(it, Modifier.padding(top = 8.dp), fontSize = 13.sp, color = Brand) }
        Row(Modifier.padding(top = 14.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            GhostPillButton("取消", onDismiss, Modifier.weight(1f))
            PillButton(if (busy) "发送中…" else "发送", {
                busy = true; error = null
                scope.launch {
                    try { env.forum.reply(topicId, text.trim()); onSent() } catch (e: kotlinx.coroutines.CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) } finally { busy = false }
                }
            }, Modifier.weight(1f), enabled = !busy && text.trim().length >= 2)
        }
    }
}

@Composable
fun WeeklyDetailScreen(env: CommunityEnv, week: String, open: (Route) -> Unit, back: () -> Unit) {
    val loader = rememberLoader(env, week) { env.forum.weekly(week) }
    val state by loader.state.collectAsState()
    var share by remember { mutableStateOf(false) }
    var toast by remember { mutableStateOf<String?>(null) }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("周刊", onBack = back, trailing = { IconTap(XIcons.Share, "分享", { share = true }) })
            LoadBox(state, loader::reload) { detail ->
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).background(MaterialTheme.colorScheme.surface)) {
                    Column(Modifier.padding(16.dp)) {
                        Text(detail.week, Modifier.clip(RoundedCornerShape(10.dp)).background(Brand).padding(horizontal = 10.dp, vertical = 2.dp), color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                        Text(detail.title, Modifier.padding(top = 10.dp), fontSize = 20.sp, fontWeight = FontWeight.SemiBold, lineHeight = 28.sp)
                        if (detail.summary.isNotBlank()) Text(detail.summary, Modifier.padding(top = 6.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    Hairline()
                    detail.entries().forEachIndexed { i, entry ->
                        WeeklyEntryRow(i + 1, entry, open)
                        Hairline(Modifier.padding(start = 16.dp))
                    }
                }
                if (share) ShareSheet(env, detail.title, detail.week, env.forum.weeklyUrl(detail.week), onDismiss = { share = false }, onCopied = { toast = "链接已复制" })
            }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
private fun WeeklyEntryRow(rank: Int, entry: WeeklyEntry, open: (Route) -> Unit) {
    when (entry) {
        is WeeklyEntry.PackEntry -> EntryColumn(rank, entry.title, entry.summary, entry.note, kindLabel(entry.kind)) { open(Route.Pack(entry.packId)) }
        is WeeklyEntry.TopicEntry -> EntryColumn(rank, entry.title, entry.summary, entry.note, "讨论") { open(Route.Topic(entry.topicId)) }
        is WeeklyEntry.TextEntry -> EntryColumn(rank, entry.title.orEmpty(), "", entry.note, null, null)
    }
}

@Composable
private fun EntryColumn(rank: Int, title: String, summary: String, note: String, label: String?, onClick: (() -> Unit)? = {}) {
    val base = if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier
    Row(base.fillMaxWidth().padding(16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Text("$rank", Modifier.width(24.dp), fontSize = 20.sp, fontWeight = FontWeight.SemiBold, color = if (rank <= 3) Brand else MaterialTheme.colorScheme.onSurfaceVariant)
        Column(Modifier.weight(1f)) {
            if (title.isNotBlank()) Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, lineHeight = 22.sp)
            if (summary.isNotBlank()) Text(summary, Modifier.padding(top = 4.dp), fontSize = 13.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (note.isNotBlank()) Text(note, Modifier.padding(top = 4.dp), fontSize = 14.sp, lineHeight = 22.sp)
            if (label != null) Text(label, Modifier.padding(top = 8.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
        }
    }
}

@Composable
fun ProjectDetailScreen(env: CommunityEnv, slug: String, back: () -> Unit) {
    val loader = rememberLoader(env, slug) { env.forum.project(slug) }
    val state by loader.state.collectAsState()
    var share by remember { mutableStateOf(false) }
    var toast by remember { mutableStateOf<String?>(null) }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("项目", onBack = back, trailing = { IconTap(XIcons.Share, "分享", { share = true }) })
            LoadBox(state, loader::reload) { project ->
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).background(MaterialTheme.colorScheme.surface).padding(16.dp)) {
                    Text(project.name, fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
                    if (project.summary.isNotBlank()) Text(project.summary, Modifier.padding(top = 8.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    project.links.forEach { link ->
                        GhostPillButton(link.title, { env.platform?.openUrl(link.url) }, Modifier.padding(top = 12.dp).fillMaxWidth(), enabled = env.platform != null)
                    }
                    val body = htmlToPlain(project.bodyHtml)
                    if (body.isNotBlank()) Text(body, Modifier.padding(top = 16.dp), fontSize = 15.sp, lineHeight = 26.sp)
                }
                if (share) ShareSheet(env, project.name, "项目", env.forum.projectUrl(project.slug), onDismiss = { share = false }, onCopied = { toast = "链接已复制" })
            }
        }
        Toast(toast) { toast = null }
    }
}
