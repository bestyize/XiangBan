package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.aspectRatio
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
import androidx.compose.foundation.layout.widthIn
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
import xyz.xiangban.xiaoban.common.community.cardTitle
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.VisualKinds
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.compactCount
import xyz.xiangban.xiaoban.common.community.kindLabel
import xyz.xiangban.xiaoban.common.community.messageOf

/** 讨论的三个二级栏目。 */
internal enum class DiscussSub(val key: String, val label: String) {
    Create("create", "创作"), Learn("learn", "学习和教程"), Casual("casual", "休闲"), All("all", "全部讨论 ›");

    companion object {
        fun byKey(key: String) = entries.firstOrNull { it.key == key } ?: Create
    }
}

/** 白底卡片之间留 8dp、两边留 12dp；底部让出悬浮导航的高度。 */
internal val FeedPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 24.dp + NavInset)
private val Gap = 8.dp

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
    initialSort: String = "latest",
    card: @Composable (Topic) -> Unit,
) {
    val state by controller.state.collectAsState()
    LaunchedEffect(controller, initialSort) { controller.select(initialSort) }
    val pinned = if (pinnedBar) state.topics.firstOrNull { it.pinned } else null
    val rows = if (pinned != null) state.topics.filter { it.id != pinned.id } else state.topics
    PullToRefreshBox(isRefreshing = state.loading && state.topics.isNotEmpty(), onRefresh = controller::refresh, modifier = modifier.fillMaxSize()) {
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
        Modifier.fillMaxWidth().outlinedCard(12.dp).clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        XIcon(XIcons.Pin, tint = Brand, size = 16.dp)
        Text("置顶", style = XType.Caption, fontWeight = FontWeight.SemiBold, color = Brand)
        Text(topic.cardTitle, Modifier.weight(1f), style = XType.Body, maxLines = 1, overflow = TextOverflow.Ellipsis)
        XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
    }
}

/** 帖子列表只展示标题、真实配图与轻量元信息，正文和投票在详情阅读。 */
@Composable
internal fun TopicCard(env: CommunityEnv, topic: Topic, open: (Route) -> Unit, onMessage: (String) -> Unit, tagLabel: String? = null) {
    val image = topic.images.firstOrNull()?.let(env.forum::imageUrl)
    Column(Modifier.fillMaxWidth().outlinedCard(12.dp).clickable { open(Route.Topic(topic.id)) }.padding(14.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.Top) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(7.dp)) {
                if (topic.pinned || topic.featured) Text(if (topic.pinned) "置顶" else "精选", style = XType.Caption, color = Brand)
                Text(topic.cardTitle, style = XType.Secondary, fontWeight = FontWeight.Normal, maxLines = 2, overflow = TextOverflow.Ellipsis)
                if (topic.excerpt.isNotBlank() && topic.excerpt != topic.cardTitle) Text(topic.excerpt, style = XType.Secondary, color = LocalExtraColors.current.ink3, maxLines = 2, overflow = TextOverflow.Ellipsis)
                tagLabel?.let { GrayTag(it, compact = true) }
            }
            if (image != null) Box(Modifier.widthIn(max = 112.dp)) { CommunityCover(image) { TopicCoverStats(topic) } }
        }
        Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Avatar(topic.author.nickname, 18.dp, showInitial = true)
            Text(topic.author.nickname, Modifier.weight(1f), style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(agoText(topic.createdAt), style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        if (image == null || topic.likeCount > 0 || topic.poll != null) Row(Modifier.padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            if (image == null) { Stat(XIcons.Eye, compactCount(topic.viewCount)); Stat(XIcons.Message, compactCount(topic.replyCount)) }
            if (topic.likeCount > 0) Stat(XIcons.Heart, compactCount(topic.likeCount))
            if (topic.poll != null) Text("投票", style = XType.Caption, color = Brand)
        }
    }
}

@Composable
internal fun Stat(icon: ImageVector, text: String) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        XIcon(icon, tint = LocalExtraColors.current.ink3, size = 16.dp)
        Text(text, style = XType.Caption, color = LocalExtraColors.current.ink3)
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
                    Text(option.text, Modifier.weight(1f), style = XType.Body, fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("$pct%", style = XType.Body, fontWeight = FontWeight.SemiBold)
                }
            }
        }
        Text("${poll.total} 人参与投票", style = XType.Caption, color = LocalExtraColors.current.ink3)
    }
}

// ───────────── 创作 ─────────────

/** 创作栏的筛选：和设计稿一致的顺序，值是内容包类型。 */
private val CreateChips = listOf("" to "全部", "effect" to "特效", "wallpaper" to "壁纸", "widget" to "组件", "game" to "游戏", "routine" to "例行", "reminder" to "提醒", "task" to "任务", "goal" to "目标", "skill" to "技能", "expert" to "专家", "workflow" to "工作流", "schemes" to "方案")

/** 这些类型没有封面，卡片是文字和图标；选中它们时改成一列。 */
private val SingleColumnKinds = setOf("routine", "reminder", "task", "goal", "skill", "expert")

private fun skillTag(kind: String) = when (kind) {
    "routine" -> "例行事项"; "reminder" -> "提醒规则"; "task" -> "任务模板"; "goal" -> "目标模板"; "skill" -> "技能"; "expert" -> "专家"; "workflow" -> "工作流"; else -> kind
}

private fun skillIcon(kind: String) = when (kind) {
    "routine" -> XIcons.Clock; "reminder" -> XIcons.Bell; "task" -> XIcons.CheckBold; "goal" -> XIcons.Target; "skill" -> XIcons.Cube; "expert" -> XIcons.User; else -> XIcons.Tool
}

/** 作品封面沿用网页的真实比例与 16:9 分类规则。 */


@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CreateTab(env: CommunityEnv, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { FeedController(env.forum, scope) }
    val state by controller.state.collectAsState()
    var kind by rememberSaveable { mutableStateOf("") }
    LaunchedEffect(kind) { if (kind != "schemes") controller.select(kind) }
    val single = kind in SingleColumnKinds
    Column(Modifier.fillMaxSize()) {
        PageChipRow(CreateChips, kind, { kind = it }, Modifier.padding(start = 12.dp, end = 12.dp, top = 10.dp, bottom = 6.dp))
        if (kind == "schemes") { NativeSchemesFeed(env, open); return@Column }
        PullToRefreshBox(isRefreshing = state.loading && state.packs.isNotEmpty(), onRefresh = controller::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.loading && state.packs.isEmpty() -> Spinner()
                state.error != null && state.packs.isEmpty() -> CenterMessage(state.error.orEmpty(), "重试", controller::refresh)
                else -> LazyVerticalStaggeredGrid(
                    if (single) StaggeredGridCells.Fixed(1) else StaggeredGridCells.Adaptive(160.dp), Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(start = 12.dp, end = 12.dp, bottom = 24.dp + NavInset),
                    horizontalArrangement = Arrangement.spacedBy(Gap), verticalItemSpacing = Gap,
                ) {
                    item(span = StaggeredGridItemSpan.FullLine) {
                        Column {
                            EditorialSection(env, if (kind.isBlank()) "create" else "create-$kind", "图文分享", open, "内容介绍、使用方法与创作经验。可导入的作品和方案在下方单独展示。")
                            Text("作品与方案", Modifier.padding(top = 18.dp, bottom = 6.dp), style = XType.Section, fontWeight = FontWeight.SemiBold)
                        }
                    }
                    staggeredItems(state.packs, key = { it.id }, span = { if (it.kind in VisualKinds) StaggeredGridItemSpan.SingleLane else StaggeredGridItemSpan.FullLine }) { pack ->
                        if (pack.kind in VisualKinds) LookCard(pack, masonry = kind == "wallpaper") { open(Route.Pack(pack.id)) } else SkillCard(pack) { open(Route.Ability(pack.id)) }
                    }
                    if (state.isEmpty) item(span = StaggeredGridItemSpan.FullLine) { CenterMessage("这里还没有作品") }
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
        Text(pack.author?.nickname ?: "社区作品", Modifier.weight(1f), style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
        if (pack.likeCount > 0) Stat(XIcons.Heart, compactCount(pack.likeCount))
        if (pack.replyCount > 0) Stat(XIcons.Message, compactCount(pack.replyCount))
    }
}

/** 视觉作品卡：16:9 封面，壁纸筛选按实际图片比例展示。 */
@Composable
internal fun LookCard(pack: Pack, masonry: Boolean = false, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().outlinedCard(12.dp).clickable(onClick = onClick)) {
        Box {
            if (!pack.coverUrl.isNullOrBlank()) CommunityCover(pack.coverUrl.orEmpty(), natural = masonry, minimum = .8f, contain = pack.kind in setOf("game", "widget"))
            else Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) { Text(kindLabel(pack.kind), style = XType.Body, color = LocalExtraColors.current.ink3) }
            Text(kindLabel(pack.kind), Modifier.padding(8.dp).clip(RoundedCornerShape(4.dp)).background(Color.Black.copy(alpha = .55f)).padding(horizontal = 6.dp, vertical = 2.dp), style = XType.Caption, color = Color.White)
        }
        Column(Modifier.padding(10.dp)) {
            Text(pack.title, style = XType.Secondary, fontWeight = FontWeight.Normal, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (pack.author != null || pack.likeCount > 0 || pack.replyCount > 0) Box(Modifier.padding(top = 10.dp)) { AuthorLine(pack) }
        }
    }
}

/** 能力卡：没有封面，显示类型、标题及可用的作者信息。 */
@Composable
internal fun SkillCard(pack: Pack, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().outlinedCard(12.dp).clickable(onClick = onClick).padding(14.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(Modifier.size(48.dp).clip(RoundedCornerShape(10.dp)).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) { XIcon(skillIcon(pack.kind), tint = LocalExtraColors.current.ink3, size = 24.dp) }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(skillTag(pack.kind), style = XType.Caption, color = LocalExtraColors.current.ink3)
            Text(pack.title, style = XType.Secondary, fontWeight = FontWeight.Normal, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (pack.summary.isNotBlank()) Text(pack.summary, style = XType.Secondary, color = LocalExtraColors.current.ink3, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (pack.author != null || pack.likeCount > 0 || pack.replyCount > 0) AuthorLine(pack)
        }
    }
}

// ───────────── 学习和教程 ─────────────

private fun learnChannel(filter: String): String = when (filter) {
    "claude-code" -> "learn-claude-code"; "codex" -> "learn-codex"; "提示词" -> "learn-prompts"; "工作流" -> "learn-workflows"; "本地部署" -> "learn-local"; "ai-硬件" -> "learn-hardware"; else -> "learn"
}

/** 学习和教程的话题标签（论坛里标签是小写、空格换成连字符）。 */
private val LearnTags = listOf("" to "全部", "claude-code" to "Claude Code", "codex" to "Codex", "提示词" to "提示词", "工作流" to "工作流", "本地部署" to "本地部署", "ai-硬件" to "AI 硬件")

@Composable
private fun LearnTab(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    var tag by rememberSaveable { mutableStateOf("") }
    val controller = remember(env, tag) { DiscussController(env.forum, scope, category = learnChannel(tag), home = false) }
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
    CasualWaterfall(env, controller, open, header = {
        Column(verticalArrangement = Arrangement.spacedBy(6.dp), modifier = Modifier.padding(top = 6.dp)) {
            PageChipRow(chips, slug, { slug = it })
            if (slug == "casual-chat") Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp), modifier = Modifier.padding(horizontal = 2.dp, vertical = 4.dp)) {
                XIcon(XIcons.Info, tint = LocalExtraColors.current.ink3, size = 14.dp)
                Text("闲聊灌水不进首页，也不加积分", style = XType.Caption, color = LocalExtraColors.current.ink3)
            }
        }
    })
}

/** 讨论栏目。[onMessage] 用来把投票失败之类的提示交给外层统一显示。 */
@Composable
internal fun DiscussTab(env: CommunityEnv, sub: DiscussSub, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    when (sub) {
        DiscussSub.Create -> CreateTab(env, open)
        DiscussSub.Learn -> LearnTab(env, open, onMessage)
        DiscussSub.Casual -> CasualTab(env, open, onMessage)
        DiscussSub.All -> AllDiscussions(env, open, onMessage)
    }
}

@Composable
private fun AllDiscussions(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { DiscussController(env.forum, scope) }
    val state by controller.state.collectAsState()
    val session by env.session.state.collectAsState()
    TopicFeed(controller, "暂无讨论", open, header = {
        PageChipRow(listOf("latest" to "最新回复", "new" to "新帖", "hot" to "热门", "featured" to "精选", "following" to "关注"), state.sort, {
            if (it == "following" && !session.loggedIn) open(Route.Login) else controller.select(it)
        })
    }) { TopicCard(env, it, open, onMessage, tagLabel = it.category.name) }
}
