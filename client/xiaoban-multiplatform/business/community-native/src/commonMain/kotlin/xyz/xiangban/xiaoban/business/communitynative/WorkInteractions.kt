package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

/** All public creation types share the post reply composer, comments and real topic likes. */
@Composable
internal fun WorkInteractionLayout(
    env: CommunityEnv, kind: String, id: Long, open: ((Route) -> Unit)?, onShare: () -> Unit, enabled: Boolean = true,
    content: @Composable (comments: @Composable () -> Unit) -> Unit,
) {
    if (!enabled) { content {}; return }
    val session by env.session.state.collectAsState()
    val loader = rememberLoader(env, "work-discussion-$kind-$id-${session.accountId}") { env.contentSocial.discussion(kind, id) }
    val state by loader.state.collectAsState()
    val scope = rememberCoroutineScope()
    var busy by remember(kind, id, session.accountId) { mutableStateOf(false) }
    var replyTopic by remember(kind, id, session.accountId) { mutableStateOf<Long?>(null) }
    var revision by remember(kind, id) { mutableStateOf(0) }
    var bookmarked by remember(kind, id, session.accountId) { mutableStateOf<Boolean?>(null) }
    var bookmarkError by remember(kind, id, session.accountId) { mutableStateOf(false) }
    var toast by remember(kind, id) { mutableStateOf<String?>(null) }
    val discussion = (state as? Load.Ready)?.value
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
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            Box(Modifier.weight(1f)) {
                content {
                    when (val current = state) {
                        is Load.Ready -> {
                            val topic = current.value.topicId
                            if (topic != null) CommentsSection(env, topic, revision, hint = "", open = open)
                            else Column(Modifier.fillMaxWidth().padding(16.dp)) {
                                Text("评论 0", style = XType.Section)
                                Text("还没有评论，聊聊你的体验吧", Modifier.padding(top = 12.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3)
                            }
                        }
                        is Load.Failed -> Banner(current.message, Modifier.padding(12.dp), warn = true, actionText = "重试", onAction = loader::reload)
                        else -> Spinner(Modifier.fillMaxWidth().height(48.dp))
                    }
                }
            }
            Hairline()
            Row(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).heightIn(min = 60.dp).padding(horizontal = 8.dp, vertical = 6.dp), verticalAlignment = Alignment.CenterVertically) {
                val canReply = discussion != null && !discussion.locked && (!session.loggedIn || discussion.canReply)
                Box(Modifier.weight(1f).height(40.dp).clip(CircleShape).background(LocalExtraColors.current.fill)
                    .clickable(enabled = canReply && !busy) { act {
                        val current = env.contentSocial.discussion(kind, id, create = true)
                        if (current.canReply) replyTopic = current.topicId else toast = "当前无法评论"
                        loader.reload()
                    } }.padding(horizontal = 12.dp), contentAlignment = Alignment.CenterStart) {
                    Text(when {
                        discussion == null -> "评论"
                        discussion.locked -> "评论已关闭"
                        !session.loggedIn -> "登录后评论"
                        !discussion.canReply -> "暂不能评论"
                        else -> "写评论…"
                    }, style = XType.Secondary, color = LocalExtraColors.current.ink3)
                }
                Row(Modifier.clip(CircleShape).clickable(enabled = discussion != null && !busy) { act {
                    val current = env.contentSocial.discussion(kind, id, create = true)
                    val topic = current.topicId ?: throw ApiException(503, "讨论暂不可用")
                    env.forum.setTopicLiked(topic, !current.liked)
                    loader.reload()
                } }.padding(horizontal = 10.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    XIcon(XIcons.Heart, tint = if (discussion?.liked == true) Brand else MaterialTheme.colorScheme.onSurface, description = if (discussion?.liked == true) "取消点赞" else "点赞")
                    Text(discussion?.likeCount?.toString() ?: "—", style = XType.Secondary)
                }
                IconTap(XIcons.Bookmark, if (bookmarked == true) "取消收藏" else "收藏", { act {
                    val current = bookmarked ?: env.contentSocial.state(kind, id).bookmarked
                    bookmarked = env.contentSocial.bookmark(kind, id, !current).bookmarked
                    bookmarkError = false
                } }, tint = if (bookmarked == true) Brand else MaterialTheme.colorScheme.onSurface)
                IconTap(XIcons.Share, "分享作品", onShare)
            }
            if (bookmarkError) Text("收藏状态读取失败，点击收藏可重试", Modifier.padding(horizontal = 12.dp, vertical = 4.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        OverlayTransition(replyTopic != null) { panel ->
            replyTopic?.let { topic -> NativeReplyComposer(env, topic, { replyTopic = null }, {
                replyTopic = null; revision++; loader.reload(); toast = "评论已提交"
            }, panel) }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
internal fun PostLikeAction(env: CommunityEnv, post: Post, open: ((Route) -> Unit)?) {
    val session by env.session.state.collectAsState()
    val scope = rememberCoroutineScope()
    var liked by remember(post.id, post.liked, session.accountId) { mutableStateOf(post.liked) }
    var count by remember(post.id, post.likeCount, session.accountId) { mutableStateOf(post.likeCount) }
    var busy by remember(post.id, session.accountId) { mutableStateOf(false) }
    var error by remember(post.id) { mutableStateOf<String?>(null) }
    Column {
        Row(Modifier.clickable(enabled = !busy) {
            if (!session.loggedIn) { open?.invoke(Route.Login); return@clickable }
            busy = true; error = null
            scope.launch {
                try { val next = env.forum.setPostLiked(post.id, !liked); count = next.likeCount; liked = next.liked }
                catch (e: CancellationException) { throw e }
                catch (e: Exception) { error = messageOf(e) }
                finally { busy = false }
            }
        }.padding(vertical = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            XIcon(XIcons.Heart, tint = if (liked) Brand else LocalExtraColors.current.ink3, size = 14.dp, description = if (liked) "取消评论点赞" else "点赞评论")
            Text(if (count > 0) "$count" else "赞", style = XType.Caption, color = if (liked) Brand else LocalExtraColors.current.ink3)
        }
        error?.let { Text(it, style = XType.Caption, color = Danger) }
    }
}
