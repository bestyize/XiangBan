package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

/** All public creation types share the post reply composer, comments and real topic likes. */
@Composable
internal fun WorkInteractionLayout(
    env: CommunityEnv, kind: String, id: Long, open: ((Route) -> Unit)?, onShare: () -> Unit, enabled: Boolean = true,
    linkedTopicId: Long? = null,
    content: @Composable (comments: @Composable () -> Unit) -> Unit,
) {
    if (!enabled) { content {}; return }
    val session by env.session.state.collectAsState()
    val detail = LocalCommunityDetail.current
    val loader = rememberLoader(env, listOf("work-discussion", kind, id, session.accountId, linkedTopicId)) {
        try { env.contentSocial.discussion(kind, id) }
        catch (e: ApiException) {
            if (e.status != 404 || linkedTopicId == null) throw e
            val topic = env.forum.topic(linkedTopicId)
            ContentDiscussion(topic.id, topic.likeCount, topic.replyCount, topic.viewer?.liked == true,
                topic.viewer?.canReply == true, topic.locked == true)
        }
    }
    val state by loader.state.collectAsState()
    val scope = rememberCoroutineScope()
    var busy by remember(kind, id, session.accountId) { mutableStateOf(false) }
    var replyTopic by remember(kind, id, session.accountId) { mutableStateOf<Long?>(null) }
    var replyingTo by remember(kind, id, session.accountId) { mutableStateOf<Post?>(null) }
    var editing by remember(kind, id, session.accountId) { mutableStateOf<Post?>(null) }
    var revision by remember(kind, id) { mutableStateOf(0) }
    var bookmarked by remember(kind, id, session.accountId) { mutableStateOf<Boolean?>(null) }
    var bookmarkError by remember(kind, id, session.accountId) { mutableStateOf(false) }
    var toast by remember(kind, id) { mutableStateOf<String?>(null) }
    var lastDiscussion by remember(kind, id, session.accountId) { mutableStateOf<ContentDiscussion?>(null) }
    val discussion = (state as? Load.Ready)?.value
    val shownDiscussion = discussion ?: lastDiscussion
    LaunchedEffect(discussion) { if (discussion != null) lastDiscussion = discussion }
    val commentAnchor = remember(kind, id) { BringIntoViewRequester() }
    val canReply = discussion != null && !discussion.locked && (!session.loggedIn || discussion.canReply)
    val bookmarkRevision by env.contentBookmarkRevision.collectAsState()
    LaunchedEffect(kind, id, session.accountId, revision, bookmarkRevision) {
        if (session.loggedIn) {
            try { bookmarked = env.contentSocial.state(kind, id).bookmarked; bookmarkError = false }
            catch (e: CancellationException) { throw e }
            catch (_: Exception) { bookmarkError = true }
        }
    }
    fun act(block: suspend () -> Unit) {
        if (!session.loggedIn) { open?.invoke(Route.Login); return }
        if (busy) return
        busy = true
        scope.launch {
            try { block() }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { toast = messageOf(e) }
            finally { busy = false }
        }
    }
    fun beginReply(parent: Post? = null) {
        if (!canReply) return
        act {
            val current = discussion
            val target = if (current.topicId == null) env.contentSocial.discussion(kind, id, create = true) else current
            if (target.canReply && target.topicId != null) {
                replyingTo = parent
                replyTopic = target.topicId
                if (current.topicId == null) loader.reload()
            } else toast = "当前无法评论"
        }
    }
    fun dismissComposer() {
        replyTopic = null
        replyingTo = null
        editing = null
    }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            Box(Modifier.weight(1f)) {
                content {
                    val topic = shownDiscussion?.topicId
                    val anchor = Modifier.bringIntoViewRequester(commentAnchor)
                    when {
                        topic != null -> CommentsSection(
                            env, topic, revision, open,
                            onReply = if (canReply) ::beginReply else null,
                            onEdit = { editing = it; replyTopic = topic },
                            anchor = anchor,
                        )
                        state is Load.Failed -> Banner((state as Load.Failed).message, Modifier.padding(12.dp).then(anchor),
                            warn = true, actionText = "重试", onAction = loader::reload)
                        shownDiscussion != null -> Column(Modifier.fillMaxWidth().padding(16.dp)) {
                            if (detail) Hairline()
                            CommentSectionTitle(0, anchor)
                            if (detail && canReply) DetailCommentPrompt(env) { beginReply() }
                            Text("还没有评论", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        else -> Spinner(Modifier.fillMaxWidth().height(48.dp).then(anchor))
                    }
                }
            }
            CommentBar(
                when {
                    shownDiscussion == null -> "评论"
                    shownDiscussion.locked -> "评论已关闭"
                    !session.loggedIn -> if (detail) "说点什么…" else "登录后评论"
                    !shownDiscussion.canReply -> "暂不能评论"
                    else -> if (detail) "说点什么…" else "写评论…"
                },
                onWrite = { beginReply() },
                enabled = canReply && !busy,
            ) {
                val likeWork: () -> Unit = { act {
                    val current = discussion?.takeIf { it.topicId != null } ?: env.contentSocial.discussion(kind, id, create = true)
                    val topic = current.topicId ?: throw ApiException(503, "讨论暂不可用")
                    env.forum.setTopicLiked(topic, !current.liked)
                    loader.reload()
                } }
                val toggleBookmark: () -> Unit = { act {
                    val current = bookmarked ?: env.contentSocial.state(kind, id).bookmarked
                    bookmarked = env.contentSocial.bookmark(kind, id, !current).bookmarked
                    env.contentBookmarkRevision.value++
                    bookmarkError = false
                } }
                val liked = shownDiscussion?.liked == true
                val likeLabel = if (liked) "取消点赞" else "点赞"
                val saveLabel = if (bookmarked == true) "取消收藏" else "收藏"
                if (detail) {
                    DetailDockAction(XIcons.Heart, shownDiscussion?.likeCount?.let(::compactCount), likeLabel, likeWork,
                        tint = if (liked) androidx.compose.ui.graphics.Color(0xFFE11D48) else MaterialTheme.colorScheme.onSurface,
                        enabled = discussion != null && !busy)
                    DetailDockAction(XIcons.Star, if (bookmarked == true) "已收藏" else "收藏", saveLabel, toggleBookmark,
                        tint = if (bookmarked == true) Amber else MaterialTheme.colorScheme.onSurface)
                } else {
                    Row(Modifier.clip(CircleShape).clickable(enabled = discussion != null && !busy, onClick = likeWork)
                        .heightIn(min = 44.dp).padding(horizontal = 6.dp), verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                        XIcon(XIcons.Heart, tint = if (liked) Brand else MaterialTheme.colorScheme.onSurface, size = 20.dp, description = likeLabel)
                        Text(shownDiscussion?.likeCount?.let(::compactCount) ?: "—", style = XType.Secondary)
                    }
                    IconTap(XIcons.Bookmark, saveLabel, toggleBookmark, tint = if (bookmarked == true) Brand else MaterialTheme.colorScheme.onSurface)
                }
                CommentCountAction(shownDiscussion?.replyCount) { scope.launch { commentAnchor.bringIntoView() } }
                if (!detail) IconTap(XIcons.Share, "分享作品", onShare)
            }
            if (bookmarkError) Text("收藏状态读取失败，点击收藏可重试", Modifier.padding(horizontal = 12.dp, vertical = 4.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        replyTopic?.let { topic ->
            NativeReplyComposer(env, topic, ::dismissComposer, onSent = {
                toast = if (editing != null) "评论已保存" else "评论已发布"
                if (editing == null) lastDiscussion = shownDiscussion?.copy(replyCount = shownDiscussion.replyCount + 1)
                dismissComposer()
                revision++
                loader.reload()
            }, parent = replyingTo, editingPost = editing)
        }
        Toast(toast) { toast = null }
    }
}

@Composable
internal fun PostLikeAction(env: CommunityEnv, post: Post, open: ((Route) -> Unit)?, onError: (String?) -> Unit) {
    val session by env.session.state.collectAsState()
    val scope = rememberCoroutineScope()
    var liked by remember(post.id, post.liked, session.accountId) { mutableStateOf(post.liked) }
    var count by remember(post.id, post.likeCount, session.accountId) { mutableStateOf(post.likeCount) }
    var busy by remember(post.id, session.accountId) { mutableStateOf(false) }
    CommentLikeAction(liked, count, enabled = !busy, onLike = {
        if (!session.loggedIn) open?.invoke(Route.Login)
        else if (!busy) {
            busy = true
            onError(null)
            scope.launch {
                try {
                    val next = env.forum.setPostLiked(post.id, !liked)
                    count = next.likeCount
                    liked = next.liked
                } catch (e: CancellationException) { throw e }
                catch (e: Exception) { onError(messageOf(e)) }
                finally { busy = false }
            }
        }
    })
}
