package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.staggeredgrid.LazyVerticalStaggeredGrid
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridCells
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridItemSpan
import androidx.compose.foundation.lazy.staggeredgrid.items as staggeredItems
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.Category
import xyz.xiangban.xiaoban.common.community.DiscussController
import xyz.xiangban.xiaoban.common.community.FeedController
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.Poll
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.VisualKinds
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.compactCount
import xyz.xiangban.xiaoban.common.community.kindLabel
import xyz.xiangban.xiaoban.common.community.messageOf

/** 讨论的三个二级栏目。 */
internal enum class DiscussSub(val key: String, val label: String) {
    Create("create", "创作"), Learn("learn", "学习和教程"), Casual("casual", "休闲");

    companion object {
        fun byKey(key: String) = entries.firstOrNull { it.key == key } ?: Create
    }
}

/** 白底卡片之间留 4dp、两边留 12dp；底部让出悬浮导航的高度。 */
internal val FeedPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 4.dp, bottom = 24.dp + NavInset)
private val Gap = 4.dp

// ───────────── 通用帖子流 ─────────────

/**
 * 一页帖子流：下拉刷新、到底翻页。[pinnedBar] 为真时把置顶的帖子收成顶部一条公告，其余按 [card] 画。
 * [header] 放在列表最上面（筛选胶囊、统计卡）。
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun TopicFeed(
    controller: DiscussController,
    emptyText: String,
    open: (Route) -> Unit,
    modifier: Modifier = Modifier,
    pinnedBar: Boolean = false,
    header: (@Composable () -> Unit)? = null,
    card: @Composable (Topic) -> Unit,
) {
    val state by controller.state.collectAsState()
    LaunchedEffect(controller) { controller.select("latest") }
    val pinned = if (pinnedBar) state.topics.firstOrNull { it.pinned } else null
    val rows = if (pinned != null) state.topics.filter { it.id != pinned.id } else state.topics
    PullToRefreshBox(isRefreshing = state.loading && state.topics.isEmpty(), onRefresh = controller::refresh, modifier = modifier.fillMaxSize()) {
        LazyColumn(Modifier.fillMaxSize(), contentPadding = FeedPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
            if (header != null) item { header() }
            if (pinned != null) item { PinnedBar(pinned) { open(Route.Topic(pinned.id)) } }
            when {
                state.loading && state.topics.isEmpty() -> item { Spinner(Modifier.fillMaxWidth().height(160.dp)) }
                state.error != null && state.topics.isEmpty() -> item { CenterMessage(state.error.orEmpty(), "重试", controller::refresh, Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                rows.isEmpty() -> item { CenterMessage(emptyText, modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                else -> {
                    items(rows, key = { it.id }) { card(it) }
                    item { LoadMoreFooter(state.hasMore, state.error, state.topics.size, controller::loadMore, controller::loadMore) }
                }
            }
        }
    }
}

@Composable
private fun PinnedBar(topic: Topic, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        XIcon(XIcons.Pin, tint = Brand, size = 16.dp)
        Text("置顶", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = Brand)
        Text(topic.title, Modifier.weight(1f), fontSize = 14.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
    }
}

/**
 * 一条帖子：作者和时间、右上角标签；标题和摘要（有一张配图时配图在文字左边，88dp 见方）；
 * 两张以上的图排成三宫格；有投票时显示各项占比；底下是 浏览 / 回复 / 点赞。
 */
@Composable
internal fun TopicCard(env: CommunityEnv, topic: Topic, open: (Route) -> Unit, onMessage: (String) -> Unit, tagLabel: String? = null) {
    val images = topic.images.map { env.forum.imageUrl(it) }
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable { open(Route.Topic(topic.id)) }.padding(14.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Avatar(topic.author.nickname, 28.dp)
            Column(Modifier.weight(1f)) {
                Text(topic.author.nickname, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(agoText(topic.createdAt), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
            GrayTag(tagLabel ?: topic.tags.firstOrNull() ?: topic.category.name)
        }
        Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            if (images.size == 1) Cover(images[0], topic.id, Modifier.size(88.dp))
            Column(Modifier.weight(1f)) {
                Text(topic.title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, lineHeight = 23.sp, maxLines = 3, overflow = TextOverflow.Ellipsis)
                if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 4.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
        }
        if (images.size >= 2) {
            Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                images.take(3).forEachIndexed { i, url -> Cover(url, topic.id + i, Modifier.weight(1f).height(104.dp)) }
                repeat(3 - images.take(3).size) { Spacer(Modifier.weight(1f)) }
            }
        }
        topic.poll?.let { PollView(env, topic, it, open, onMessage) }
        Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(16.dp), verticalAlignment = Alignment.CenterVertically) {
            Stat(XIcons.Eye, compactCount(topic.viewCount))
            Stat(XIcons.Message, compactCount(topic.replyCount))
            Stat(XIcons.Heart, compactCount(topic.likeCount))
        }
    }
}

@Composable
internal fun Stat(icon: ImageVector, text: String) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        XIcon(icon, tint = LocalExtraColors.current.ink3, size = 16.dp)
        Text(text, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

/**
 * 投票：每项一条按占比填充的横条，右边写百分比，领先的（或自己选的）加粗并用浅蓝填充；底下写参与人数。
 * 还没投过时点一项就投（要登录）。
 */
@Composable
internal fun PollView(env: CommunityEnv, topic: Topic, initial: Poll, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var poll by remember(topic.id) { mutableStateOf(initial) }
    var busy by remember(topic.id) { mutableStateOf(false) }
    val voted = poll.myVote != null
    val lead = poll.myVote ?: poll.options.indices.maxByOrNull { poll.options[it].votes }?.takeIf { poll.total > 0 }
    val fillOn = if (ThemeState.dark) DataBlue.copy(alpha = 0.28f) else Color(0xFFDDE6F3)
    val fillOff = LocalExtraColors.current.line.copy(alpha = if (ThemeState.dark) 0.5f else 0.6f)
    Column(Modifier.padding(top = 10.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        poll.options.forEachIndexed { i, option ->
            val pct = poll.percent(i)
            val on = lead == i
            Box(
                Modifier.fillMaxWidth().height(34.dp).clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill)
                    .clickable(enabled = !voted && !busy) {
                        if (!session.loggedIn) { open(Route.Login); return@clickable }
                        busy = true
                        scope.launch {
                            try { poll = env.forum.votePoll(topic.id, i) } catch (e: kotlinx.coroutines.CancellationException) { throw e } catch (e: Exception) { onMessage(messageOf(e)) } finally { busy = false }
                        }
                    },
            ) {
                if (pct > 0) Box(Modifier.fillMaxHeight().fillMaxWidth(pct / 100f).background(if (on) fillOn else fillOff))
                Row(Modifier.fillMaxSize().padding(horizontal = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(option.text, Modifier.weight(1f), fontSize = 14.sp, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("$pct%", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                }
            }
        }
        Text("${poll.total} 人参与投票", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

// ───────────── 创作 ─────────────

/** 创作栏的筛选：和设计稿一致的顺序，值是内容包类型。 */
private val CreateChips = listOf("" to "全部", "effect" to "特效", "wallpaper" to "壁纸", "widget" to "组件", "game" to "游戏", "routine" to "例行", "reminder" to "提醒", "task" to "任务", "goal" to "目标", "skill" to "技能", "expert" to "专家")

/** 这些类型没有封面，卡片是文字和图标；选中它们时改成一列。 */
private val SingleColumnKinds = setOf("routine", "reminder", "task", "goal", "skill", "expert")

private fun skillTag(kind: String) = when (kind) {
    "routine" -> "例行事项"; "reminder" -> "提醒规则"; "task" -> "任务模板"; "goal" -> "目标模板"; "skill" -> "技能"; "expert" -> "专家"; "workflow" -> "工作流"; else -> kind
}

private fun skillIcon(kind: String) = when (kind) {
    "routine" -> XIcons.Clock; "reminder" -> XIcons.Bell; "task" -> XIcons.CheckBold; "goal" -> XIcons.Target; "skill" -> XIcons.Cube; "expert" -> XIcons.User; else -> XIcons.Tool
}

/** 运行时会动手做事的类型，卡片上提示“运行前需确认权限”。 */
private val NeedsPermission = setOf("routine", "task", "goal", "skill", "workflow")

/** 作品卡的封面高度在这几个值里轮换，做出瀑布流错落的样子（取自设计稿）。 */
private val CoverHeights = listOf(190, 140, 200, 150, 176)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CreateTab(env: CommunityEnv, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { FeedController(env.forum, scope) }
    val state by controller.state.collectAsState()
    var kind by rememberSaveable { mutableStateOf("") }
    LaunchedEffect(kind) { controller.select(kind) }
    val single = kind in SingleColumnKinds
    Column(Modifier.fillMaxSize()) {
        PageChipRow(CreateChips, kind, { kind = it }, Modifier.padding(start = 12.dp, end = 12.dp, top = 10.dp, bottom = 6.dp))
        PullToRefreshBox(isRefreshing = state.loading && state.packs.isEmpty(), onRefresh = controller::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.loading && state.packs.isEmpty() -> Spinner()
                state.error != null && state.packs.isEmpty() -> CenterMessage(state.error.orEmpty(), "重试", controller::refresh)
                state.isEmpty -> CenterMessage("这里还没有作品")
                else -> LazyVerticalStaggeredGrid(
                    StaggeredGridCells.Fixed(if (single) 1 else 2), Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(start = 12.dp, end = 12.dp, bottom = 24.dp + NavInset),
                    horizontalArrangement = Arrangement.spacedBy(Gap), verticalItemSpacing = Gap,
                ) {
                    staggeredItems(state.packs, key = { it.id }) { pack ->
                        if (pack.kind in VisualKinds) LookCard(pack, single) { open(Route.Pack(pack.id)) } else SkillCard(pack) { open(Route.Pack(pack.id)) }
                    }
                    item(span = StaggeredGridItemSpan.FullLine) { LoadMoreFooter(state.hasMore, state.error, state.packs.size, controller::loadMore, controller::refresh) }
                }
            }
        }
    }
}

@Composable
private fun AuthorLine(pack: Pack) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        Avatar(pack.author?.nickname ?: "?", 18.dp)
        Text(pack.author?.nickname ?: "小伴官方", Modifier.weight(1f), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
        XIcon(XIcons.Heart, tint = LocalExtraColors.current.ink3, size = 14.dp)
        Text(compactCount(pack.applyCount.toInt()), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

/** 作品卡：错落高度的封面，左上黑底类型标签，游戏右上红色「可试玩」；下面是标题和作者。 */
@Composable
internal fun LookCard(pack: Pack, single: Boolean = false, onClick: () -> Unit) {
    val h = CoverHeights[(pack.id % CoverHeights.size).toInt()] * (if (single) 1.3f else 1f)
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick)) {
        Box {
            Cover(pack.coverUrl, pack.id, Modifier.fillMaxWidth().height(h.dp))
            Text(kindLabel(pack.kind), Modifier.padding(8.dp).clip(RoundedCornerShape(4.dp)).background(Color.Black.copy(alpha = 0.45f)).padding(horizontal = 6.dp, vertical = 2.dp), fontSize = 12.sp, color = Color.White)
            if (pack.kind == "game") Row(
                Modifier.align(Alignment.TopEnd).padding(8.dp).clip(RoundedCornerShape(4.dp)).background(Brand).padding(horizontal = 6.dp, vertical = 2.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(2.dp),
            ) {
                XIcon(XIcons.Play, tint = Color.White, size = 12.dp)
                Text("可试玩", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = Color.White)
            }
        }
        Column(Modifier.padding(start = 8.dp, end = 8.dp, top = 8.dp, bottom = 10.dp)) {
            Text(pack.title, fontSize = 14.sp, fontWeight = FontWeight.Medium, lineHeight = 20.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Box(Modifier.padding(top = 8.dp)) { AuthorLine(pack) }
        }
    }
}

/** 能力卡：没有封面，蓝灰图标块加类型，标题、最多 4 行摘要、权限提示和作者。 */
@Composable
internal fun SkillCard(pack: Pack, onClick: () -> Unit) {
    val soft = if (ThemeState.dark) Color(0xFF9FB6DC).copy(alpha = 0.18f) else Color(0xFFE8EEF8)
    val ink = if (ThemeState.dark) Color(0xFF9FB6DC) else Color(0xFF3E5C8A)
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(start = 10.dp, end = 10.dp, top = 12.dp, bottom = 10.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Box(Modifier.size(24.dp).clip(RoundedCornerShape(8.dp)).background(soft), contentAlignment = Alignment.Center) { XIcon(skillIcon(pack.kind), tint = ink, size = 16.dp) }
            Text(skillTag(pack.kind), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Text(pack.title, Modifier.padding(top = 8.dp), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, lineHeight = 21.sp)
        if (pack.summary.isNotBlank()) Text(pack.summary, Modifier.padding(top = 6.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 4, overflow = TextOverflow.Ellipsis)
        if (pack.kind in NeedsPermission) Row(Modifier.padding(top = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            XIcon(XIcons.Lock, tint = LocalExtraColors.current.ink3, size = 12.dp)
            Text("运行前需确认权限", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        }
        Box(Modifier.padding(top = 10.dp)) { AuthorLine(pack) }
    }
}

// ───────────── 学习和教程 ─────────────

/** 学习和教程的话题标签（论坛里标签是小写、空格换成连字符）。 */
private val LearnTags = listOf("" to "全部", "claude-code" to "Claude Code", "codex" to "Codex", "提示词" to "提示词", "工作流" to "工作流", "本地部署" to "本地部署", "ai-硬件" to "AI 硬件")

@Composable
private fun LearnTab(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    var tag by rememberSaveable { mutableStateOf("") }
    val controller = remember(env, tag) { DiscussController(env.forum, scope, category = "tech,share-learn", tag = tag.ifEmpty { null }, home = false) }
    val labels = remember { LearnTags.toMap() }
    TopicFeed(controller, "这个标签下还没有教程", open, pinnedBar = true, header = {
        PageChipRow(LearnTags, tag, { tag = it }, Modifier.padding(top = 6.dp))
    }) { TopicCard(env, it, open, onMessage, tagLabel = it.tags.firstOrNull()?.let { t -> labels[t] ?: t } ?: it.category.name) }
}

// ───────────── 休闲 ─────────────

/** 休闲栏的筛选，顺序和设计稿一致；论坛里没有对应板块的会自动隐藏。 */
private val CasualChips = listOf(
    "casual-work" to "职场", "casual-offwork" to "下班了", "casual-indie" to "副业", "casual-gear" to "数码", "casual-loss" to "亏了吗",
    "casual-fun" to "吃喝玩乐", "casual-media" to "片单乐评", "casual-think" to "哲思", "casual-help" to "互助", "casual-chat" to "闲聊灌水",
)

@Composable
private fun CasualTab(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    val categories = rememberLoader(env, "categories") { env.forum.categories() }
    val loaded by categories.state.collectAsState()
    var slug by rememberSaveable { mutableStateOf("casual") }
    val existing = (loaded as? Load.Ready<List<Category>>)?.value.orEmpty().map { it.slug }.toSet()
    val chips = listOf("casual" to "全部") + CasualChips.filter { it.first in existing }
    val controller = remember(env, slug) { DiscussController(env.forum, scope, category = slug, home = false) }
    val labels = remember { CasualChips.toMap() }
    TopicFeed(controller, "这里还没有帖子", open, pinnedBar = true, header = {
        Column(verticalArrangement = Arrangement.spacedBy(6.dp), modifier = Modifier.padding(top = 6.dp)) {
            PageChipRow(chips, slug, { slug = it })
            if (slug == "casual-chat") Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp), modifier = Modifier.padding(horizontal = 2.dp, vertical = 4.dp)) {
                XIcon(XIcons.Info, tint = LocalExtraColors.current.ink3, size = 14.dp)
                Text("闲聊灌水不进首页，也不加积分", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
        }
    }) { TopicCard(env, it, open, onMessage, tagLabel = it.tags.firstOrNull() ?: labels[it.category.slug] ?: it.category.name) }
}

/** 讨论栏目。[onMessage] 用来把投票失败之类的提示交给外层统一显示。 */
@Composable
internal fun DiscussTab(env: CommunityEnv, sub: DiscussSub, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    when (sub) {
        DiscussSub.Create -> CreateTab(env, open)
        DiscussSub.Learn -> LearnTab(env, open, onMessage)
        DiscussSub.Casual -> CasualTab(env, open, onMessage)
    }
}
