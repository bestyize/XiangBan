package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.NewsComment
import xyz.xiangban.xiaoban.common.community.NewsItem
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.htmlToPlain
import xyz.xiangban.xiaoban.common.community.messageOf

/** 文章使用服务器净化的 HTML；按标题分段，正文中的图片仍由 RichBody 展示。 */
@Composable
private fun EditorialBody(env: CommunityEnv, html: String) {
    val parts = remember(html) {
        val headings = Regex("(?is)<h[1-6][^>]*>.*?</h[1-6]>")
        buildList {
            var last = 0
            for (match in headings.findAll(html)) {
                if (match.range.first > last) add(false to html.substring(last, match.range.first))
                add(true to match.value)
                last = match.range.last + 1
            }
            if (last < html.length) add(false to html.substring(last))
        }
    }
    Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
        parts.forEach { (heading, content) ->
            if (heading) Text(htmlToPlain(content), Modifier.padding(top = 8.dp), fontSize = 20.sp, lineHeight = 29.sp, fontWeight = FontWeight.Bold)
            else RichBody(env, content, fontSize = 16.sp, lineHeight = 29.sp)
        }
    }
}

@Composable
fun NewsDetailScreen(env: CommunityEnv, key: String, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var article by remember(key) { mutableStateOf<NewsItem?>(null) }
    var error by remember(key) { mutableStateOf<String?>(null) }
    var comments by remember(key) { mutableStateOf<List<NewsComment>>(emptyList()) }
    var commentsError by remember(key) { mutableStateOf<String?>(null) }
    var commentsPage by remember(key) { mutableIntStateOf(0) }
    var commentsMore by remember(key) { mutableStateOf(false) }
    var loadingComments by remember(key) { mutableStateOf(false) }
    var loadingArticle by remember(key) { mutableStateOf(false) }
    var draft by remember(key) { mutableStateOf("") }
    var replyTo by remember(key) { mutableStateOf<NewsComment?>(null) }
    var sending by remember(key) { mutableStateOf(false) }
    var deleteTarget by remember(key) { mutableStateOf<NewsComment?>(null) }

    fun loadArticle() {
        if (loadingArticle) return
        loadingArticle = true; error = null
        scope.launch {
            try { article = env.forum.news(key) }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { error = messageOf(e) }
            finally { loadingArticle = false }
        }
    }
    fun loadComments(reset: Boolean = false) {
        if (loadingComments) return
        loadingComments = true; commentsError = null
        scope.launch {
            try {
                val result = env.forum.newsComments(key, if (reset) 1 else commentsPage + 1)
                comments = if (reset) result.items else comments + result.items
                commentsPage = result.page; commentsMore = result.hasMore
            } catch (e: CancellationException) { throw e }
            catch (e: Exception) { commentsError = messageOf(e) }
            finally { loadingComments = false }
        }
    }
    LaunchedEffect(key) { loadArticle(); loadComments(true) }

    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        ScreenBar("AI 日报", onBack = back)
        LazyColumn(Modifier.weight(1f), contentPadding = PaddingValues(bottom = 24.dp)) {
            if (article == null) item {
                if (error != null) CenterMessage("文章加载失败：$error", "重试", ::loadArticle, Modifier.fillMaxWidth().padding(32.dp))
                else Spinner(Modifier.fillMaxWidth().height(120.dp))
            }
            article?.let { news ->
                item {
                    Column(Modifier.fillMaxWidth().padding(horizontal = 22.dp, vertical = 20.dp)) {
                        Text(news.title, fontSize = 26.sp, lineHeight = 36.sp, fontWeight = FontWeight.Bold)
                        Spacer(Modifier.height(14.dp))
                        val whenText = agoText(news.sourcePublishedAt ?: news.publishedAt)
                        Text(listOf(news.source, if (news.timeBasis == "collected") "采集于 $whenText" else whenText, if (news.sample) "示例" else "").filter { it.isNotBlank() }.joinToString(" · "), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                        if (news.coverUrl.isNotBlank()) {
                            Spacer(Modifier.height(22.dp))
                            Cover(news.coverUrl, news.key.hashCode().toLong() and 0x7fffffff, Modifier.fillMaxWidth().aspectRatio(16f / 9f))
                        }
                        if (news.summary.isNotBlank()) {
                            Spacer(Modifier.height(24.dp))
                            Text(news.summary, Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(LocalExtraColors.current.fill).padding(16.dp), fontSize = 15.sp, lineHeight = 25.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        Spacer(Modifier.height(26.dp))
                        if (news.bodyHtml.isNotBlank()) EditorialBody(env, news.bodyHtml)
                        else Text("这篇内容暂未整理站内正文。", fontSize = 15.sp, lineHeight = 25.sp, color = LocalExtraColors.current.ink3)
                        if (news.url.isNotBlank()) Text("查看原始来源 ↗", Modifier.padding(top = 24.dp).clickable { env.platform?.openUrl(news.url) }.padding(vertical = 6.dp), color = Brand, fontSize = 13.sp)
                        Spacer(Modifier.height(28.dp))
                        Hairline()
                        Text("留言", Modifier.padding(top = 22.dp, bottom = 8.dp), fontSize = 19.sp, fontWeight = FontWeight.Bold)
                    }
                }
                if (comments.isEmpty() && !loadingComments && commentsError == null) item {
                    Text("还没有留言，来分享你的想法吧。", Modifier.padding(horizontal = 22.dp, vertical = 14.dp), color = LocalExtraColors.current.ink3, fontSize = 14.sp)
                }
                items(comments, key = { it.id }) { comment ->
                    Column(Modifier.fillMaxWidth().padding(horizontal = 22.dp, vertical = 12.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                            Avatar(comment.author.nickname, 32.dp, showInitial = true)
                            Column(Modifier.weight(1f)) {
                                Text(comment.author.nickname, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                                Text(agoText(comment.createdAt), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            }
                            Text("${if (comment.liked) "♥" else "♡"} ${comment.likeCount}", Modifier.clickable {
                                if (!session.loggedIn) open(Route.Login) else scope.launch {
                                    try { env.forum.likeNewsComment(key, comment.id, !comment.liked); loadComments(true) }
                                    catch (e: CancellationException) { throw e } catch (e: Exception) { commentsError = messageOf(e) }
                                }
                            }.padding(5.dp), color = if (comment.liked) Brand else LocalExtraColors.current.ink3, fontSize = 13.sp)
                        }
                        if (comment.replyTo != null) {
                            val parent = comments.firstOrNull { it.id == comment.replyTo }
                            Text("回复 ${parent?.author?.nickname ?: "留言 #${comment.replyTo}"}", Modifier.padding(start = 42.dp, top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                        }
                        Text(comment.body, Modifier.padding(start = 42.dp, top = 6.dp), fontSize = 15.sp, lineHeight = 24.sp)
                        Row(Modifier.padding(start = 42.dp, top = 8.dp), horizontalArrangement = Arrangement.spacedBy(18.dp)) {
                            Text("回复", Modifier.clickable { if (!session.loggedIn) open(Route.Login) else replyTo = comment }, color = LocalExtraColors.current.ink3, fontSize = 12.sp)
                            if (comment.mine) Text("删除", Modifier.clickable { deleteTarget = comment }, color = LocalExtraColors.current.ink3, fontSize = 12.sp)
                        }
                    }
                }
                item {
                    if (commentsError != null) Text("留言加载或操作失败：$commentsError · 重试", Modifier.clickable { loadComments(true) }.padding(22.dp), color = Danger, fontSize = 13.sp)
                    else if (commentsMore) Text("加载更多留言", Modifier.clickable { loadComments() }.padding(22.dp), color = Brand, fontSize = 14.sp)
                    if (loadingComments) Spinner(Modifier.fillMaxWidth().height(48.dp))
                }
            }
        }
        if (article != null) {
            Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).imePadding().padding(horizontal = 14.dp, vertical = 8.dp)) {
                replyTo?.let { Text("回复 ${it.author.nickname} · 取消", Modifier.clickable { replyTo = null }.padding(bottom = 6.dp), color = Brand, fontSize = 12.sp) }
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedTextField(value = draft, onValueChange = { if (it.length <= 2000) draft = it }, modifier = Modifier.weight(1f), placeholder = { Text("写留言…", fontSize = 14.sp) }, maxLines = 3, shape = RoundedCornerShape(20.dp))
                    TextButton(enabled = !sending && draft.trim().length >= 2, onClick = {
                        if (!session.loggedIn) open(Route.Login) else {
                            sending = true
                            scope.launch {
                                try { env.forum.commentNews(key, draft, replyTo?.id); draft = ""; replyTo = null; loadComments(true) }
                                catch (e: CancellationException) { throw e } catch (e: Exception) { commentsError = messageOf(e) }
                                finally { sending = false }
                            }
                        }
                    }) { Text(if (sending) "发送中" else "发送") }
                }
            }
        }
    }
    deleteTarget?.let { target ->
        AlertDialog(onDismissRequest = { deleteTarget = null }, title = { Text("删除留言？") }, text = { Text("删除后这条留言将不再显示。") }, confirmButton = {
            TextButton(onClick = { deleteTarget = null; scope.launch {
                try { env.forum.deleteNewsComment(key, target.id); loadComments(true) }
                catch (e: CancellationException) { throw e } catch (e: Exception) { commentsError = messageOf(e) }
            } }) { Text("删除") }
        }, dismissButton = { TextButton(onClick = { deleteTarget = null }) { Text("取消") } })
    }
}
