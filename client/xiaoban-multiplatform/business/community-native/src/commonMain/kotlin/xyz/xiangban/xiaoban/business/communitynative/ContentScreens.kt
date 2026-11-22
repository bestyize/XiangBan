package xyz.xiangban.xiaoban.business.communitynative

import xyz.xiangban.xiaoban.common.community.isCasual
import xyz.xiangban.xiaoban.common.community.cardTitle
import xyz.xiangban.xiaoban.common.community.casualArticle

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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as gridItems
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.TextButton
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.common.community.BodyBlock
import xyz.xiangban.xiaoban.common.community.DiscussController
import xyz.xiangban.xiaoban.common.community.FeedController
import xyz.xiangban.xiaoban.common.community.FeedState
import xyz.xiangban.xiaoban.common.community.Loader
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.PackKinds
import xyz.xiangban.xiaoban.common.community.Post
import xyz.xiangban.xiaoban.common.community.Project
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.VisualKinds
import xyz.xiangban.xiaoban.common.community.WeeklyEntry
import xyz.xiangban.xiaoban.common.community.formatSize
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.compactCount
import xyz.xiangban.xiaoban.common.community.kindLabel
import xyz.xiangban.xiaoban.common.community.linkedCommunityPacks
import xyz.xiangban.xiaoban.common.community.messageOf
import xyz.xiangban.xiaoban.common.community.parseBody
import xyz.xiangban.xiaoban.common.community.replyCanEdit

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
fun TabStrip(tabs: List<Pair<String, String>>, selected: String, centered: Boolean = false, onSelect: (String) -> Unit) {
    val scrollState = rememberLazyListState()
    if (centered) LaunchedEffect(selected, tabs) {
        val index = tabs.indexOfFirst { it.first == selected }
        if (index >= 0) scrollState.animateScrollToItem(index)
    }
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        LazyRow(Modifier.fillMaxWidth().height(32.dp), state = scrollState, contentPadding = PaddingValues(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(24.dp, if (centered) Alignment.CenterHorizontally else Alignment.Start),
            verticalAlignment = Alignment.CenterVertically) {
            items(tabs, key = { it.first }) { (key, label) ->
                val on = key == selected
                Box(Modifier.fillMaxHeight().clickable { onSelect(key) }, contentAlignment = Alignment.Center) {
                    Text(label, Modifier.padding(bottom = 3.dp), style = XType.Secondary, fontWeight = if (on) FontWeight.Bold else FontWeight.Normal, color = if (on) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant)
                    Box(Modifier.align(Alignment.BottomCenter).padding(bottom = 1.dp).size(width = 18.dp, height = 3.dp).clip(RoundedCornerShape(2.dp)).background(if (on) Brand else Color.Transparent))
                }
            }
        }
        Hairline()
    }
}

@Composable
internal fun LoadMoreFooter(hasMore: Boolean, error: String?, count: Int, onMore: () -> Unit, onRetry: () -> Unit) {
    when {
        error != null -> Text("加载失败，点这里重试", Modifier.fillMaxWidth().clickable(onClick = onRetry).padding(16.dp), color = Danger, style = XType.Body)
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
        Text(text, Modifier.clip(RoundedCornerShape(18.dp)).background(Color.Black.copy(alpha = 0.78f)).padding(horizontal = 16.dp, vertical = 8.dp), color = Color.White, style = XType.Body)
    }
}

/** 分享弹层：预览卡片加“系统分享”“复制链接”。手机上分享的是网页链接，对方在电脑上打开后再用小伴桌面版。 */
@Composable
internal fun ShareSheet(env: CommunityEnv, title: String, sub: String, url: String, heading: String = "分享", hint: String? = null, onDismiss: () -> Unit, onCopied: () -> Unit, panelModifier: Modifier = Modifier) {
    CommunityShareSheet(env, url, heading, hint, onDismiss, onCopied, panelModifier)
}

@Composable
internal fun <T> rememberLoader(env: CommunityEnv, key: Any, fetch: suspend () -> T): Loader<T> {
    val scope = rememberCoroutineScope()
    val loader = remember(env, key) { Loader(scope, fetch) }
    LaunchedEffect(loader) { loader.reload() }
    return loader
}

// ───────────── 作品摘要 ─────────────

internal fun packMeta(pack: Pack): String =
    listOfNotNull(kindLabel(pack.kind), pack.author?.nickname, pack.trustLabel.takeIf { pack.trustLevel < 2 }, "${pack.applyCount} 次应用".takeIf { pack.applyCount > 0 }).joinToString(" · ")

// ───────────── 帖子行 ─────────────

@Composable
internal fun TopicRow(topic: Topic, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(16.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
            if (topic.pinned) Text("置顶", style = XType.Caption, fontWeight = FontWeight.SemiBold, color = Brand)
            if (topic.featured) Text("精华", style = XType.Caption, fontWeight = FontWeight.SemiBold, color = Brand)
            Text(topic.category.name, style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        Text(topic.cardTitle, Modifier.padding(top = 6.dp), style = XType.Body, fontWeight = FontWeight.Medium, maxLines = 2, overflow = TextOverflow.Ellipsis)
        if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 4.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Row(Modifier.padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Avatar(topic.author.nickname, 16.dp)
            Text(topic.author.nickname, style = XType.Caption, color = LocalExtraColors.current.ink3)
            Text("·", style = XType.Caption, color = LocalExtraColors.current.ink3)
            Text("${topic.replyCount} 回复", style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
    }
}

// ───────────── 详情页 ─────────────

/** 社区作品详情，应用、游戏和能力作品沿用各自的内容与操作。 */
@Composable
fun PackDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit, open: ((Route) -> Unit)? = null, chat: xyz.xiangban.xiaoban.common.community.CloudChatController? = null) = CommunityDetail {
    val loader = rememberLoader(env, "pack-detail-$id") { env.forum.pack(id, recordView = true) }
    val state by loader.state.collectAsState()
    var share by remember { mutableStateOf(false) }
    var toast by remember { mutableStateOf<String?>(null) }
    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        LoadBox(state, loader::reload) { pack ->
            if (pack.kind in setOf("game", "app") && chat != null) {
                AppPostContent(env, chat, pack, back, open)
                return@LoadBox
            }
            if (pack.kind in xyz.xiangban.xiaoban.common.community.CapabilityPackKinds) {
                CapabilityDetailContent(env, pack, back, open = open)
                return@LoadBox
            }
            Column(Modifier.fillMaxSize()) {
                val author = pack.author
                DetailAuthorBar(author?.nickname ?: "社区作品", back, onShare = { share = true },
                    onAuthor = if (author != null && open != null) ({ open(Route.User(author.uid)) }) else null) {
                    author?.let { DetailAuthorFollow(env, it.uid, open) }
                }
                WorkInteractionLayout(env, "pack", pack.id, open, { share = true }, enabled = pack.status == "published",
                    linkedTopicId = pack.topicId) { comments ->
                    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                        DetailGallery(listOfNotNull(pack.coverUrl?.takeIf(String::isNotBlank)?.let(env.forum::imageUrl)), pack.title)
                        DetailBody {
                            DetailHeading(pack.title)
                            if (pack.summary.isNotBlank()) Text(pack.summary, style = XType.DetailBody)
                            DetailTags(pack.tags)
                            if (pack.kind in setOf("game", "app") && pack.status == "published") GameTrialButton(env, pack.id, open = open, modifier = Modifier.padding(top = 16.dp))
                            DetailMetadata(listOfNotNull(kindLabel(pack.kind), "v${pack.version}",
                                pack.versions.firstOrNull { it.version == pack.version }?.size?.takeIf { it > 0 }?.let(::formatSize),
                                pack.updatedAt.take(10).takeIf { it.isNotBlank() }?.let { "更新于 $it" },
                                "纯数据".takeIf { pack.pureData }, "${pack.applyCount} 次应用".takeIf { pack.applyCount > 0 }).joinToString(" · "))
                            if (pack.status == "published") NativeReportAction(env, "pack", pack.id, open)
                            val changes = pack.versions.filter { it.changelog.isNotBlank() }
                            if (changes.isNotEmpty()) {
                                Text("更新记录", Modifier.padding(top = 24.dp), style = XType.Section, fontWeight = FontWeight.SemiBold)
                                changes.sortedByDescending { it.version }.forEach { v ->
                                    Text(v.label.ifBlank { "v${v.version}" }, Modifier.padding(top = 10.dp), style = XType.Body, fontWeight = FontWeight.Medium)
                                    Text(v.changelog, Modifier.padding(top = 2.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                            }
                        }
                        comments()
                    }
                }
            }
            OverlayTransition(share) { panel ->
                ShareSheet(
                    env, pack.title, kindLabel(pack.kind) + (pack.author?.nickname?.let { " · $it" } ?: ""), env.forum.packUrl(pack.id),
                    heading = "分享作品",
                    hint = "分享作品链接，也可以在电脑上打开查看与使用。",
                    onDismiss = { share = false }, onCopied = { toast = "链接已复制" },
                    panelModifier = panel,
                )
            }
        }
        if (state !is xyz.xiangban.xiaoban.common.community.Load.Ready<*>) {
            Box(Modifier.align(Alignment.TopStart).statusBarsPadding().padding(start = 4.dp, top = 4.dp)) { IconTap(XIcons.ChevronLeft, "返回", back) }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
internal fun CircleOverlayButton(icon: androidx.compose.ui.graphics.vector.ImageVector, description: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Box(modifier.size(40.dp).clip(CircleShape).background(Color.Black.copy(alpha = 0.4f)).clickable(onClick = onClick), contentAlignment = Alignment.Center) {
        XIcon(icon, tint = Color.White, size = 22.dp, description = description)
    }
}

/** 求助帖的回答区需要的信息：被采纳的是哪条、谁是题主、当前读者能不能采纳，以及采纳的动作。 */
internal class HelpAnswers(val acceptedPostId: Long?, val authorUid: String, val canAccept: Boolean, val onAccept: (Post) -> Unit)

private data class DiscussionBody(val blocks: List<BodyBlock>, val linkedWorks: List<Long>)
private data class PreparedTopic(val topic: Topic, val body: DiscussionBody)
private data class PreparedPost(val post: Post, val body: DiscussionBody)

private fun parseDiscussionBody(env: CommunityEnv, html: String, excluded: Set<Long> = emptySet()) = DiscussionBody(
    blocks = parseBody(html, env.config.forumBase),
    linkedWorks = linkedCommunityPacks(html, env.config).filterNot { it in excluded },
)

/** 作品对应的讨论帖里的评论，分页加载。[reloadKey] 变化时从头重新加载（发完回复后用）。 */
@Composable
internal fun CommentsSection(
    env: CommunityEnv,
    topicId: Long,
    reloadKey: Int,
    open: ((Route) -> Unit)?,
    onReply: ((Post?) -> Unit)?,
    onEdit: (Post) -> Unit,
    help: HelpAnswers? = null,
    topicForEditing: Topic? = null,
    anchor: Modifier = Modifier,
) {
    val session by env.session.state.collectAsState()
    val scope = rememberCoroutineScope()
    val detail = LocalCommunityDetail.current
    var newestFirst by remember(topicId, session.accountId, detail) { mutableStateOf(detail) }
    var editTopic by remember(topicId, reloadKey, session.accountId) { mutableStateOf(topicForEditing) }
    var posts by remember(topicId, reloadKey, session.accountId, newestFirst) { mutableStateOf<List<PreparedPost>>(emptyList()) }
    var page by remember(topicId, reloadKey, session.accountId, newestFirst) { mutableStateOf(0) }
    var total by remember(topicId, reloadKey, session.accountId, newestFirst) { mutableStateOf<Int?>(null) }
    var more by remember(topicId, reloadKey, session.accountId, newestFirst) { mutableStateOf(true) }
    var error by remember(topicId, reloadKey, session.accountId, newestFirst) { mutableStateOf<String?>(null) }
    var loading by remember(topicId, reloadKey, session.accountId, newestFirst) { mutableStateOf(true) }
    var pageJob by remember { mutableStateOf<Job?>(null) }
    DisposableEffect(topicId, reloadKey, session.accountId, newestFirst) {
        onDispose { pageJob?.cancel() }
    }

    suspend fun load() {
        loading = true
        error = null
        try {
            val requested = if (page == 0) 1 else if (newestFirst) page - 1 else page + 1
            val first = env.forum.posts(topicId, requested)
            val next = if (page == 0 && newestFirst && first.hasMore) {
                env.forum.posts(topicId, (first.total - 1) / first.pageSize + 1)
            } else first
            val ordered = if (newestFirst) next.items.asReversed() else next.items
            val prepared = withContext(Dispatchers.Default) {
                ordered.map { post ->
                    val body = if (!post.blocked && post.status == "normal") {
                        parseDiscussionBody(env, post.bodyHtml, post.workRefs.map { it.id }.toSet())
                    } else DiscussionBody(emptyList(), emptyList())
                    PreparedPost(post, body)
                }
            }
            posts = (posts + prepared).distinctBy { it.post.id }
            page = next.page
            total = next.total
            more = if (newestFirst) next.page > 1 else next.hasMore
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (e: Exception) {
            error = messageOf(e)
        } finally {
            loading = false
        }
    }
    LaunchedEffect(topicId, reloadKey, session.accountId, newestFirst) { load() }

    val hasOwnPost = posts.any { it.post.mine }
    LaunchedEffect(topicId, session.accountId, hasOwnPost, topicForEditing) {
        editTopic = topicForEditing ?: if (session.loggedIn && hasOwnPost) {
            try { env.forum.topic(topicId) }
            catch (e: kotlinx.coroutines.CancellationException) { throw e }
            catch (_: Exception) { null }
        } else null
    }
    val authors = remember(posts) { posts.associate { it.post.floor to it.post } }
    Column(Modifier.padding(start = 16.dp, end = 16.dp, top = if (detail) 4.dp else 24.dp, bottom = 16.dp)) {
        if (detail) Hairline()
        CommentSectionTitle(total, if (detail) anchor.padding(top = 6.dp) else anchor, label = if (help != null) "回答" else "评论",
            newestFirst = newestFirst, onSort = if (detail && total != null && !loading) ({ newestFirst = !newestFirst }) else null)
        if (detail && onReply != null) DetailCommentPrompt(env) { onReply(null) }
        posts.forEach { (post, body) ->
            key(post.id) {
                val parent = post.replyToFloor?.let { floor ->
                    val name = authors[floor]?.takeUnless { it.blocked }?.author?.nickname
                    "回复 ${name ?: "$floor 楼"}"
                }
                PostItem(
                    env, post, body, parent, open = open,
                    onReply = onReply?.let { reply -> { reply(post) } },
                    onEdit = if (session.loggedIn && replyCanEdit(post, session.accountId, editTopic)) ({ onEdit(post) }) else null,
                    accepted = help?.acceptedPostId == post.id,
                    onAccept = if (help != null && help.canAccept && post.author.uid != help.authorUid) ({ help.onAccept(post) }) else null,
                )
            }
        }
        if (total == 0) Text(if (help != null) "还没有人回答" else "还没有评论",
            Modifier.padding(vertical = 8.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        val loadMore: () -> Unit = {
            if (pageJob?.isActive != true) pageJob = scope.launch { load() }
        }
        CommentListFooter(loading, error, loadMore, loadMore.takeIf { more && page > 0 })
    }
}

@Composable
private fun PostItem(
    env: CommunityEnv,
    post: Post,
    body: DiscussionBody,
    parent: String?,
    accepted: Boolean,
    onAccept: (() -> Unit)?,
    open: ((Route) -> Unit)?,
    onEdit: (() -> Unit)?,
    onReply: (() -> Unit)?,
) {
    val visible = !post.blocked && post.status == "normal"
    var likeError by remember(post.id) { mutableStateOf<String?>(null) }
    CommentRow(
        env, post.author, "${post.floor} 楼 · ${agoText(post.createdAt)}", parent.takeIf { visible },
        onAuthor = open?.let { navigate -> { navigate(Route.User(post.author.uid)) } },
        onReply = onReply.takeIf { visible },
        like = if (visible) ({ PostLikeAction(env, post, open) { likeError = it } }) else null,
        actions = {
            if (visible) {
                if (onEdit != null && !accepted) CommentTextAction("编辑", onEdit)
                NativeReportAction(env, "post", post.id, open)
            }
        },
    ) {
        if (!visible) {
            Text(if (post.blocked) "你已屏蔽该用户" else "该评论已被隐藏", style = XType.Secondary,
                color = MaterialTheme.colorScheme.onSurfaceVariant)
        } else {
            val bodyStyle = if (LocalCommunityDetail.current) XType.DetailComment else XType.Body
            RichBody(body.blocks, fontSize = bodyStyle.fontSize, lineHeight = bodyStyle.lineHeight)
            ReplyContentCards(env, post.workRefs, open)
            LinkedWorkCards(env, body.linkedWorks, open)
            ForumAttachments(env, post.attachments, open)
        }
        likeError?.let { Text(it, style = XType.Caption, color = Danger) }
        if (accepted) Row(Modifier.padding(top = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            XIcon(XIcons.CheckBold, tint = Success, size = 16.dp)
            Text("已采纳", style = XType.Caption, color = Success)
        }
        if (visible && onAccept != null) PillButton("采纳这个回答", onAccept, Modifier.padding(top = 10.dp).fillMaxWidth(), height = 40.dp)
    }
}

/** 帖子详情：标题、作者、正文，下面是回复；底栏可以点赞、收藏、写回复（需要登录）。 */
@Composable
fun TopicDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit, open: (Route) -> Unit, showComments: Boolean = false) = CommunityDetail {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val loader = rememberLoader(env, listOf(id, session.accountId)) {
        val topic = env.forum.topic(id)
        withContext(Dispatchers.Default) { PreparedTopic(topic, parseDiscussionBody(env, topic.bodyHtml)) }
    }
    val state by loader.state.collectAsState()
    var share by remember { mutableStateOf(false) }
    var replying by remember(id, session.accountId) { mutableStateOf(false) }
    var replyingTo by remember(id, session.accountId) { mutableStateOf<Post?>(null) }
    var editing by remember(id, session.accountId) { mutableStateOf<Post?>(null) }
    var confirmClose by remember(id) { mutableStateOf(false) }
    var closingHelp by remember(id) { mutableStateOf(false) }
    var toast by remember { mutableStateOf<String?>(null) }
    var reloadKey by remember { mutableStateOf(0) }
    val commentAnchor = remember(id) { BringIntoViewRequester() }
    var jumpToComments by rememberSaveable(id, showComments) { mutableStateOf(showComments) }
    var commentsPlaced by remember(id) { mutableStateOf(false) }
    var commentCount by remember(id) { mutableStateOf<Int?>(null) }
    fun beginReply(parent: Post? = null) {
        if (!session.loggedIn) { open(Route.Login); return }
        replyingTo = parent
        replying = true
    }
    fun dismissComposer() {
        replying = false
        editing = null
        replyingTo = null
    }
    LaunchedEffect(jumpToComments, commentsPlaced) {
        if (jumpToComments && commentsPlaced) {
            commentAnchor.bringIntoView()
            jumpToComments = false
        }
    }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            if (state !is Load.Ready<*>) ScreenBar("帖子", onBack = back)
            LoadBox(state, loader::reload) { (topic, body) ->
                val images = remember(topic.images) { topic.images.filter(String::isNotBlank).map(env.forum::imageUrl).distinct() }
                DetailAuthorBar(topic.author.nickname, back, onShare = { share = true },
                    onAuthor = { open(Route.User(topic.author.uid)) }, avatar = { TopicAuthorAvatar(env, topic.author, 30.dp) }) {
                    DetailAuthorFollow(env, topic.author.uid, open)
                }
                var liked by remember(topic.id, session.accountId, topic.viewer?.liked) { mutableStateOf(topic.viewer?.liked == true) }
                var bookmarked by remember(topic.id, session.accountId, topic.viewer?.bookmarked) { mutableStateOf(topic.viewer?.bookmarked == true) }
                var likeCount by remember(topic.id, session.accountId, topic.likeCount) { mutableStateOf(topic.likeCount) }
                fun guarded(block: suspend () -> Unit) {
                    if (!session.loggedIn) { open(Route.Login); return }
                    scope.launch { try { block() } catch (e: kotlinx.coroutines.CancellationException) { throw e } catch (e: Exception) { toast = messageOf(e) } }
                }
                Column(Modifier.fillMaxSize()) {
                    Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState())) {
                        DetailGallery(images, topic.title)
                        DetailBody(hasDots = images.size > 1) {
                            DetailHeading(topic.title)
                            if (topic.pinned || topic.help != null) {
                                Row(Modifier.padding(bottom = 8.dp), horizontalArrangement = Arrangement.spacedBy(10.dp),
                                    verticalAlignment = Alignment.CenterVertically) {
                                    if (topic.pinned) Text("置顶", style = XType.Caption, fontWeight = FontWeight.SemiBold, color = Brand)
                                    topic.help?.let { help ->
                                        HelpStatusTag(help)
                                        Text(helpRemain(help), style = XType.Caption, color = LocalExtraColors.current.ink3)
                                    }
                                }
                            }
                            if (topic.help?.status == "open" && topic.author.uid == session.accountId) {
                                TextButton(onClick = { confirmClose = true }, enabled = !closingHelp) { Text("关闭求助") }
                            }
                            if (confirmClose) AlertDialog(
                                onDismissRequest = { if (!closingHelp) confirmClose = false },
                                title = { Text("关闭求助？") },
                                text = { Text("尚未发放的悬赏积分将退还。") },
                                confirmButton = {
                                    TextButton(enabled = !closingHelp, onClick = {
                                        guarded {
                                            closingHelp = true
                                            try {
                                                env.forum.closeHelp(topic.id)
                                                confirmClose = false
                                                loader.reload()
                                            } finally {
                                                closingHelp = false
                                            }
                                        }
                                    }) { Text("关闭求助") }
                                },
                                dismissButton = {
                                    TextButton(enabled = !closingHelp, onClick = { confirmClose = false }) { Text("取消") }
                                },
                            )
                            RichBody(body.blocks, fontSize = XType.DetailBody.fontSize, lineHeight = XType.DetailBody.lineHeight,
                                galleryImages = images, onCopy = { text -> env.platform?.copy(text); toast = "已复制全文" })
                            LinkedWorkCards(env, body.linkedWorks, open)
                            ForumAttachments(env, topic.attachments, open)
                            topic.help?.takeIf { it.bounty > 0 }?.let { Text("悬赏 ${it.bounty} 分", Modifier.padding(top = 12.dp), style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = Amber) }
                            topic.poll?.let { PollView(env, topic, it, open) { msg -> toast = msg } }
                            DetailTags(listOf(topic.category.name) + topic.tags) { open(Route.Articles(topic.category.slug, topic.category.name)) }
                            val stamp = topic.editedAt?.let { "编辑于 ${agoText(it)}" } ?: "发布于 ${agoText(topic.createdAt)}"
                            DetailMetadata(listOfNotNull(stamp, "${topic.viewCount} 次浏览", "已锁定".takeIf { topic.locked == true },
                                "阅读等级 ${topic.minReadLevel}".takeIf { topic.minReadLevel > 0 }).joinToString(" · "))
                            NativeReportAction(env, "topic", topic.id, open)
                        }
                        val answers = topic.help?.let {
                            HelpAnswers(
                                acceptedPostId = it.acceptedPostId, authorUid = topic.author.uid,
                                canAccept = topic.viewer?.canEdit == true && it.status == "open",
                                onAccept = { post -> guarded { env.forum.acceptAnswer(topic.id, post.id); toast = "已采纳"; loader.reload() } },
                            )
                        }
                        val anchor = Modifier.bringIntoViewRequester(commentAnchor).onGloballyPositioned { commentsPlaced = true }
                        Column(Modifier.fillMaxWidth()) {
                            CommentsSection(
                                env, id, reloadKey, open,
                                onReply = ::beginReply.takeIf { topic.locked != true && topic.viewer?.canReply != false },
                                onEdit = { editing = it }, help = answers, topicForEditing = topic, anchor = anchor,
                            )
                        }
                    }
                    CommentBar(
                        when {
                            topic.locked == true -> "评论已关闭"
                            topic.viewer?.canReply == false && session.loggedIn -> "暂不能评论"
                            topic.help != null -> "写回答…"
                            else -> "说点什么…"
                        },
                        onWrite = { beginReply() },
                        enabled = topic.locked != true && topic.viewer?.canReply != false,
                    ) {
                        DetailDockAction(XIcons.Heart, compactCount(likeCount), if (liked) "取消点赞" else "点赞", onClick = {
                            guarded {
                                val result = env.forum.setTopicLiked(topic.id, !liked)
                                liked = result.liked
                                likeCount = result.likeCount
                            }
                        }, tint = if (liked) Color(0xFFE11D48) else MaterialTheme.colorScheme.onSurface)
                        DetailDockAction(XIcons.Star, if (bookmarked) "已收藏" else "收藏",
                            if (bookmarked) "取消收藏" else "收藏", onClick = {
                                guarded {
                                    val next = !bookmarked
                                    env.forum.setTopicBookmarked(topic.id, next)
                                    bookmarked = next
                                    toast = if (next) "已收藏" else "已取消收藏"
                                }
                            }, tint = if (bookmarked) Amber else MaterialTheme.colorScheme.onSurface)
                        CommentCountAction(commentCount ?: topic.replyCount) { jumpToComments = true }
                    }
                }
            }
        }
        if (replying || editing != null) {
            NativeReplyComposer(env, id, ::dismissComposer, onSent = {
                toast = if (editing != null) "评论已保存" else "评论已发布"
                if (editing == null) {
                    val topic = (state as? Load.Ready)?.value?.topic
                    commentCount = (commentCount ?: topic?.replyCount ?: 0) + 1
                }
                dismissComposer()
                reloadKey += 1
            }, parent = replyingTo, editingPost = editing)
        }
        OverlayTransition(share) { panel ->
            val topic = (state as? Load.Ready)?.value?.topic
            ShareSheet(env, topic?.cardTitle ?: "帖子", topic?.category?.name.orEmpty(), env.forum.topicUrl(id), onDismiss = { share = false }, onCopied = { toast = "链接已复制" }, panelModifier = panel)
        }
        Toast(toast) { toast = null }
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
                        Text(detail.week, Modifier.clip(RoundedCornerShape(10.dp)).background(Brand).padding(horizontal = 10.dp, vertical = 2.dp), color = Color.White, style = XType.Caption, fontWeight = FontWeight.SemiBold)
                        Text(detail.title, Modifier.padding(top = 10.dp), style = XType.Headline, fontWeight = FontWeight.SemiBold)
                        if (detail.summary.isNotBlank()) Text(detail.summary, Modifier.padding(top = 6.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    Hairline()
                    detail.entries().forEachIndexed { i, entry ->
                        WeeklyEntryRow(i + 1, entry, open)
                        Hairline(Modifier.padding(start = 16.dp))
                    }
                }
                OverlayTransition(share) { panel -> ShareSheet(env, detail.title, detail.week, env.forum.weeklyUrl(detail.week), onDismiss = { share = false }, onCopied = { toast = "链接已复制" }, panelModifier = panel) }
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
        Text("$rank", Modifier.width(24.dp), style = XType.Title, fontWeight = FontWeight.SemiBold, color = if (rank <= 3) Brand else MaterialTheme.colorScheme.onSurfaceVariant)
        Column(Modifier.weight(1f)) {
            if (title.isNotBlank()) Text(title, style = XType.Section, fontWeight = FontWeight.SemiBold)
            if (summary.isNotBlank()) Text(summary, Modifier.padding(top = 4.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (note.isNotBlank()) Text(note, Modifier.padding(top = 4.dp), style = XType.Body)
            if (label != null) Text(label, Modifier.padding(top = 8.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
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
                    Text(project.name, style = XType.Headline, fontWeight = FontWeight.SemiBold)
                    if (project.summary.isNotBlank()) Text(project.summary, Modifier.padding(top = 8.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    project.links.forEach { link ->
                        GhostPillButton(link.title, { env.platform?.openUrl(link.url) }, Modifier.padding(top = 12.dp).fillMaxWidth(), enabled = env.platform != null)
                    }
                    RichBody(env, project.bodyHtml, Modifier.padding(top = 16.dp))
                }
                OverlayTransition(share) { panel -> ShareSheet(env, project.name, "项目", env.forum.projectUrl(project.slug), onDismiss = { share = false }, onCopied = { toast = "链接已复制" }, panelModifier = panel) }
            }
        }
        Toast(toast) { toast = null }
    }
}
