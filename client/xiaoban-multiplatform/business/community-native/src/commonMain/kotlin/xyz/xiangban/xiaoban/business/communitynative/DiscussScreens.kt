package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.Category
import xyz.xiangban.xiaoban.common.community.DiscussController
import xyz.xiangban.xiaoban.common.community.FeedController
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.PackKinds
import xyz.xiangban.xiaoban.common.community.Poll
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.compactCount
import xyz.xiangban.xiaoban.common.community.messageOf

/** 讨论的三个二级栏目。 */
internal enum class DiscussSub(val key: String, val label: String) {
    Create("create", "创作"), Learn("learn", "学习和教程"), Casual("casual", "休闲");

    companion object {
        fun byKey(key: String) = entries.firstOrNull { it.key == key } ?: Create
    }
}

/** 底部留出悬浮导航的高度；列表可以滚到导航下面。 */
internal val FeedPadding = PaddingValues(start = 8.dp, end = 8.dp, top = 8.dp, bottom = 24.dp + NavInset)

// ───────────── 通用帖子流 ─────────────

/**
 * 一页帖子流：下拉刷新、到底翻页。置顶的帖子（[pinnedBar] 为真时）收成顶部一条公告，其余按 [card] 画。
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
        LazyColumn(Modifier.fillMaxSize(), contentPadding = FeedPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
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
        Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(horizontal = 12.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        XIcon(XIcons.Pin, tint = Brand, size = 16.dp)
        Text("置顶", fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = Brand)
        Text(topic.title, Modifier.weight(1f), fontSize = 14.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
    }
}

/** 一条帖子卡片：作者和时间、标题、摘要，带图时显示配图（一张在右边，多张排成一行），带投票时显示选项。 */
@Composable
internal fun TopicCard(env: CommunityEnv, topic: Topic, open: (Route) -> Unit, onMessage: (String) -> Unit, tag: String? = topic.category.name) {
    val images = topic.images.map { env.forum.imageUrl(it) }
    WhiteCard(Modifier.clickable { open(Route.Topic(topic.id)) }) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Avatar(topic.author.nickname, 32.dp, showInitial = true)
            Column(Modifier.weight(1f)) {
                Text(topic.author.nickname, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(agoText(topic.createdAt), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
            if (!tag.isNullOrEmpty()) GrayTag(tag)
        }
        if (images.size == 1) {
            Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Column(Modifier.weight(1f)) { TitleAndExcerpt(topic) }
                Cover(images[0], topic.id, Modifier.size(88.dp))
            }
        } else {
            Column(Modifier.padding(top = 12.dp)) { TitleAndExcerpt(topic) }
            if (images.size >= 2) {
                Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    images.take(3).forEachIndexed { i, url -> Cover(url, topic.id + i, Modifier.weight(1f).aspectRatio(1f)) }
                    repeat(3 - images.take(3).size) { Spacer(Modifier.weight(1f)) }
                }
            }
        }
        topic.poll?.let { PollView(env, topic, it, open, onMessage) }
        Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(18.dp), verticalAlignment = Alignment.CenterVertically) {
            Stat(XIcons.Eye, compactCount(topic.viewCount))
            Stat(XIcons.Message, compactCount(topic.replyCount))
            Stat(XIcons.Heart, compactCount(topic.likeCount))
        }
    }
}

@Composable
private fun TitleAndExcerpt(topic: Topic) {
    Text(topic.title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, lineHeight = 23.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
    if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 4.dp), fontSize = 14.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
}

@Composable
internal fun Stat(icon: androidx.compose.ui.graphics.vector.ImageVector, text: String) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        XIcon(icon, tint = LocalExtraColors.current.ink3, size = 16.dp)
        Text(text, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

/** 投票：投过之前点选项投票（要登录），投过之后显示各项占比，自己选的那项加粗。 */
@Composable
internal fun PollView(env: CommunityEnv, topic: Topic, initial: Poll, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var poll by remember(topic.id) { mutableStateOf(initial) }
    var busy by remember(topic.id) { mutableStateOf(false) }
    val voted = poll.myVote != null
    Column(Modifier.padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        poll.options.forEachIndexed { i, option ->
            val mine = poll.myVote == i
            val pct = poll.percent(i)
            Box(
                Modifier.fillMaxWidth().height(40.dp).clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill)
                    .clickable(enabled = !voted && !busy) {
                        if (!session.loggedIn) { open(Route.Login); return@clickable }
                        busy = true
                        scope.launch {
                            try { poll = env.forum.votePoll(topic.id, i) } catch (e: kotlinx.coroutines.CancellationException) { throw e } catch (e: Exception) { onMessage(messageOf(e)) } finally { busy = false }
                        }
                    },
            ) {
                if (voted) Box(Modifier.fillMaxSize(fraction = pct / 100f).background(if (mine) Brand.copy(alpha = 0.22f) else DataBlue.copy(alpha = 0.12f)))
                Row(Modifier.fillMaxSize().padding(horizontal = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(option.text, Modifier.weight(1f), fontSize = 14.sp, fontWeight = if (mine) FontWeight.Bold else FontWeight.Normal, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    if (voted) Text("$pct%", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                }
            }
        }
        Text(if (voted) "${poll.total} 人参与投票" else "${poll.total} 人参与投票 · 点选项投票", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

// ───────────── 创作 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CreateTab(env: CommunityEnv, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { FeedController(env.forum, scope) }
    val state by controller.state.collectAsState()
    var kind by rememberSaveable { mutableStateOf("") }
    LaunchedEffect(kind) { controller.select(kind) }
    Column(Modifier.fillMaxSize()) {
        PageChipRow(listOf("" to "全部") + PackKinds, kind, { kind = it }, Modifier.padding(horizontal = 8.dp, vertical = 8.dp))
        PullToRefreshBox(isRefreshing = state.loading && state.packs.isEmpty(), onRefresh = controller::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.loading && state.packs.isEmpty() -> Spinner()
                state.error != null && state.packs.isEmpty() -> CenterMessage(state.error.orEmpty(), "重试", controller::refresh)
                state.isEmpty -> CenterMessage("这里还没有作品")
                else -> PackGrid(state, open, controller::loadMore, controller::refresh, bottomPadding = 24.dp + NavInset)
            }
        }
    }
}

// ───────────── 学习和教程 ─────────────

/** 学习和教程：技术交流和学习资料两个板块，按话题标签筛。标签在论坛里是小写、空格换成连字符。 */
private val LearnTags = listOf("" to "全部", "claude-code" to "Claude Code", "codex" to "Codex", "提示词" to "提示词", "工作流" to "工作流", "本地部署" to "本地部署", "ai-硬件" to "AI 硬件")

@Composable
private fun LearnTab(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    var tag by rememberSaveable { mutableStateOf("") }
    val controller = remember(env, tag) { DiscussController(env.forum, scope, category = "tech,share-learn", tag = tag.ifEmpty { null }, home = false) }
    TopicFeed(controller, "这个标签下还没有教程", open, pinnedBar = true, header = {
        PageChipRow(LearnTags, tag, { tag = it })
    }) { TopicCard(env, it, open, onMessage) }
}

// ───────────── 休闲 ─────────────

@Composable
private fun CasualTab(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    val categories = rememberLoader(env, "categories") { env.forum.categories() }
    val loaded by categories.state.collectAsState()
    var slug by rememberSaveable { mutableStateOf("casual") }
    val children = (loaded as? Load.Ready<List<Category>>)?.value.orEmpty().let { all ->
        val parent = all.firstOrNull { it.slug == "casual" }
        all.filter { parent != null && it.parentId == parent.id }
    }
    val controller = remember(env, slug) { DiscussController(env.forum, scope, category = slug, home = false) }
    TopicFeed(controller, "这里还没有帖子", open, pinnedBar = true, header = {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            PageChipRow(listOf("casual" to "全部") + children.map { it.slug to it.name }, slug, { slug = it })
            if (slug == "casual-chat") Text("闲聊灌水不进首页，也不加积分", fontSize = 12.sp, color = LocalExtraColors.current.ink3, modifier = Modifier.padding(horizontal = 8.dp))
        }
    }) { TopicCard(env, it, open, onMessage) }
}

/** 讨论栏目。[toast] 用来把投票失败之类的提示交给外层统一显示。 */
@Composable
internal fun DiscussTab(env: CommunityEnv, sub: DiscussSub, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    when (sub) {
        DiscussSub.Create -> CreateTab(env, open)
        DiscussSub.Learn -> LearnTab(env, open, onMessage)
        DiscussSub.Casual -> CasualTab(env, open, onMessage)
    }
}
