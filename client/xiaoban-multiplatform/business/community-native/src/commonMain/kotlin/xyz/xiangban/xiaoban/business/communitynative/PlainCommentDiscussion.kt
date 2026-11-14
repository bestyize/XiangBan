package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.NewsComment
import xyz.xiangban.xiaoban.common.community.Page
import xyz.xiangban.xiaoban.common.community.messageOf

/** Shared discussion for editorial articles and open-source projects. */
@Stable
internal class PlainCommentDiscussion(
    private val scope: CoroutineScope,
    private val loggedIn: Boolean,
    private val open: (Route) -> Unit,
    private val fetch: suspend (Int) -> Page<NewsComment>,
    val publish: suspend (String, Long?, List<String>) -> Unit,
    private val setLiked: suspend (Long, Boolean) -> Unit,
    private val delete: suspend (Long) -> Unit,
) {
    var comments by mutableStateOf(emptyList<NewsComment>())
        private set
    var total by mutableStateOf<Int?>(null)
        private set
    var hasMore by mutableStateOf(false)
        private set
    var loading by mutableStateOf(true)
        private set
    var error by mutableStateOf<String?>(null)
        private set
    var composing by mutableStateOf(false)
        private set
    var replyTo by mutableStateOf<NewsComment?>(null)
        private set
    var deleteTarget by mutableStateOf<NewsComment?>(null)
    var deleting by mutableStateOf(false)
        private set
    var deleteError by mutableStateOf<String?>(null)
        private set
    var toast by mutableStateOf<String?>(null)
    private var page = 0
    private var lastRequest = 1
    private var loadJob: Job? = null
    private val pendingLikes = mutableSetOf<Long>()

    // Header and footer always occupy one lazy item each.
    val itemCount get() = comments.size + 2

    fun refresh() = load(1)
    fun retry() = load(lastRequest)
    fun more() {
        if (!loading && hasMore) load(page + 1)
    }

    private fun load(next: Int) {
        loadJob?.cancel()
        lastRequest = next
        loading = true
        error = null
        loadJob = scope.launch {
            try {
                val result = fetch(next)
                comments = if (next == 1) result.items else (comments + result.items).distinctBy { it.id }
                page = result.page
                total = result.total
                hasMore = result.hasMore
                loading = false
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                error = messageOf(e)
                loading = false
            }
        }
    }

    fun begin(parent: NewsComment? = null) {
        if (!loggedIn) { open(Route.Login); return }
        replyTo = parent
        composing = true
    }

    fun dismissComposer() { composing = false }

    fun published() {
        composing = false
        toast = "评论已发布"
        refresh()
    }

    fun like(comment: NewsComment) {
        if (!loggedIn) { open(Route.Login); return }
        if (!pendingLikes.add(comment.id)) return
        scope.launch {
            val next = !comment.liked
            try {
                setLiked(comment.id, next)
                comments = comments.map {
                    if (it.id == comment.id && it.liked != next) it.copy(liked = next, likeCount = (it.likeCount + if (next) 1 else -1).coerceAtLeast(0))
                    else it
                }
                if (loading) retry()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                toast = messageOf(e)
            } finally {
                pendingLikes.remove(comment.id)
            }
        }
    }

    fun cancelDelete() {
        if (!deleting) {
            deleteTarget = null
            deleteError = null
        }
    }

    fun confirmDelete() {
        val target = deleteTarget ?: return
        if (deleting) return
        deleting = true
        deleteError = null
        scope.launch {
            try {
                delete(target.id)
                deleteTarget = null
                refresh()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                deleteError = messageOf(e)
            } finally {
                deleting = false
            }
        }
    }
}

@Composable
internal fun rememberPlainComments(
    env: CommunityEnv,
    thread: Any,
    open: (Route) -> Unit,
    fetch: suspend (Int) -> Page<NewsComment>,
    publish: suspend (String, Long?, List<String>) -> Unit,
    setLiked: suspend (Long, Boolean) -> Unit,
    delete: suspend (Long) -> Unit,
    enabled: Boolean = true,
): PlainCommentDiscussion {
    val session by env.session.state.collectAsState()
    return key(env, thread, session.accountId, session.loggedIn) {
        val scope = rememberCoroutineScope()
        val discussion = remember { PlainCommentDiscussion(scope, session.loggedIn, open, fetch, publish, setLiked, delete) }
        LaunchedEffect(discussion, enabled) { if (enabled) discussion.refresh() }
        discussion
    }
}

@Composable
internal fun rememberRepoComments(
    env: CommunityEnv, owner: String, repo: String, open: (Route) -> Unit, enabled: Boolean = true,
): PlainCommentDiscussion = rememberPlainComments(
    env, "$owner/$repo", open,
    fetch = { env.forum.repoComments(owner, repo, it) },
    publish = { text, parent, images -> env.forum.commentRepo(owner, repo, text, parent, images) },
    setLiked = { id, liked -> env.forum.likeRepoComment(owner, repo, id, liked) },
    delete = { env.forum.deleteRepoComment(owner, repo, it) },
    enabled = enabled,
)

internal fun LazyListScope.plainCommentItems(
    env: CommunityEnv,
    discussion: PlainCommentDiscussion,
    open: (Route) -> Unit,
    horizontalPadding: Dp = 16.dp,
) {
    val padding = Modifier.padding(horizontal = horizontalPadding)
    item(key = "comments-head") {
        Column(Modifier.padding(top = 16.dp)) {
            Hairline()
            CommentSectionTitle(discussion.total, padding.padding(top = 8.dp))
            if (discussion.comments.isEmpty() && !discussion.loading && discussion.error == null) {
                Text("还没有评论", padding.padding(vertical = 8.dp), style = XType.Secondary,
                    color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
    val comments = discussion.comments
    val names = comments.associate { it.id to it.author.nickname }
    items(comments, key = { "comment-${it.id}" }) { comment ->
        PlainCommentRow(
            env, comment, comment.replyTo?.let(names::get),
            onAuthor = { open(Route.User(comment.author.uid)) },
            onLike = { discussion.like(comment) },
            onReply = { discussion.begin(comment) },
            onDelete = { discussion.deleteTarget = comment },
            modifier = padding,
        )
    }
    item(key = "comments-foot") {
        CommentListFooter(
            discussion.loading, discussion.error, discussion::retry,
            onMore = discussion::more.takeIf { discussion.hasMore }, modifier = padding,
        )
    }
}

@Composable
internal fun CommentScaffold(
    env: CommunityEnv,
    discussion: PlainCommentDiscussion,
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().then(modifier), content = content)
        PlainCommentOverlays(env, discussion)
    }
}

@Composable
private fun PlainCommentOverlays(env: CommunityEnv, discussion: PlainCommentDiscussion) {
    key(discussion) {
        PlainCommentComposer(
            env, discussion.composing, discussion.replyTo, discussion::dismissComposer, discussion::published, discussion.publish,
        )
    }
    if (discussion.deleteTarget != null) {
        AlertDialog(
            onDismissRequest = discussion::cancelDelete,
            title = { Text("删除评论？") },
            text = { Text(discussion.deleteError ?: "删除后这条评论将不再显示。") },
            confirmButton = {
                TextButton(onClick = discussion::confirmDelete, enabled = !discussion.deleting) {
                    Text(if (discussion.deleting) "删除中…" else "删除")
                }
            },
            dismissButton = {
                TextButton(onClick = discussion::cancelDelete, enabled = !discussion.deleting) { Text("取消") }
            },
        )
    }
    Toast(discussion.toast) { discussion.toast = null }
}
