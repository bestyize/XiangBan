package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.*
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Loader
import xyz.xiangban.xiaoban.common.community.Page
import xyz.xiangban.xiaoban.common.community.NewsComment
import xyz.xiangban.xiaoban.common.community.messageOf

@Composable
internal fun SoftwareRepoComments(
    env: CommunityEnv,
    project: String,
    loader: Loader<Page<NewsComment>>,
    page: Int,
    selectPage: (Int) -> Unit,
    openComposer: Boolean,
    onComposerOpened: () -> Unit,
    open: (Route) -> Unit,
) {
    val owner = project.substringBefore('/')
    val repo = project.substringAfter('/')
    val session by env.session.state.collectAsState()
    val scope = rememberCoroutineScope()
    val state by loader.state.collectAsState()
    val loaded = (state as? Load.Ready)?.value
    var composing by remember(project, session.accountId) { mutableStateOf(false) }
    var replyTo by remember(project, session.accountId) { mutableStateOf<NewsComment?>(null) }
    var draft by rememberSaveable(project, session.accountId) { mutableStateOf("") }
    var deleting by remember(project, session.accountId) { mutableStateOf<NewsComment?>(null) }
    var busy by remember(project, session.accountId) { mutableStateOf(false) }
    var error by remember(project, session.accountId) { mutableStateOf<String?>(null) }

    fun begin(parent: NewsComment? = null) {
        if (!session.loggedIn) { open(Route.Login); return }
        error = null
        replyTo = parent
        composing = true
    }

    LaunchedEffect(openComposer) {
        if (openComposer) {
            onComposerOpened()
            begin()
        }
    }

    fun mutate(nextPage: Int = page, onDone: () -> Unit = {}, request: suspend () -> Unit) {
        if (busy) return
        if (!session.loggedIn) { open(Route.Login); return }
        busy = true
        error = null
        scope.launch {
            try {
                request()
                onDone()
                if (page == nextPage) loader.reload() else selectPage(nextPage)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                error = messageOf(e)
            } finally {
                busy = false
            }
        }
    }

    Column(Modifier.fillMaxWidth().padding(top = 14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Hairline()
        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            Text(loaded?.let { "评论 · ${it.total}" } ?: "评论", Modifier.weight(1f), style = XType.Section)
            TextButton(onClick = { begin() }, enabled = !busy) { Text("写评论") }
        }
        when (val result = state) {
            is Load.Ready -> {
                if (result.value.items.isEmpty()) Text("还没有评论，来说说你怎么用它的。", style = XType.Secondary)
                result.value.items.forEach { comment ->
                    key(comment.id) {
                        CuratedComment(env, comment,
                            parentName = result.value.items.firstOrNull { it.id == comment.replyTo }?.author?.nickname,
                            onAuthor = { open(Route.User(comment.author.uid)) },
                            onLike = { mutate { env.forum.likeRepoComment(owner, repo, comment.id, !comment.liked) } },
                            onReply = { begin(comment) },
                            onDelete = { deleting = comment },
                        )
                    }
                }
                if (page > 1 || result.value.hasMore) NotePager(page, result.value.hasMore, selectPage)
            }
            is Load.Failed -> Banner(result.message, warn = true, actionText = "重试", onAction = loader::reload)
            else -> Spinner(Modifier.fillMaxWidth().height(48.dp))
        }
        if (!composing && deleting == null) error?.let { Text(it, color = Danger, style = XType.Caption) }
    }

    if (composing) AlertDialog(
        onDismissRequest = { if (!busy) { composing = false; error = null } },
        title = { Text(replyTo?.let { "回复 ${it.author.nickname}" } ?: "写评论") },
        text = {
            Column {
                OutlinedTextField(draft, { if (it.length <= 2000) draft = it }, enabled = !busy,
                    minLines = 2, maxLines = 5, placeholder = { Text("分享你的看法，2–2000 字") })
                error?.let { Text(it, color = Danger, style = XType.Caption) }
            }
        },
        confirmButton = {
            TextButton(enabled = !busy && draft.trim().length >= 2, onClick = {
                mutate(nextPage = 1, onDone = { draft = ""; composing = false; replyTo = null }) {
                    env.forum.commentRepo(owner, repo, draft.trim(), replyTo?.id)
                }
            }) { Text(if (busy) "发送中…" else "发送") }
        },
        dismissButton = { TextButton(enabled = !busy, onClick = { composing = false; error = null }) { Text("取消") } },
    )

    deleting?.let { comment ->
        AlertDialog(
            onDismissRequest = { if (!busy) { deleting = null; error = null } },
            title = { Text("删除评论？") },
            text = { Text(error ?: "删除后这条评论将不再显示。") },
            confirmButton = {
                TextButton(enabled = !busy, onClick = {
                    val nextPage = if (page > 1 && loaded?.items?.size == 1) page - 1 else page
                    mutate(nextPage, onDone = { deleting = null }) { env.forum.deleteRepoComment(owner, repo, comment.id) }
                }) { Text("删除") }
            },
            dismissButton = { TextButton(enabled = !busy, onClick = { deleting = null; error = null }) { Text("取消") } },
        )
    }
}
