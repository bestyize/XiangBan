package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
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
            if (heading) Text(htmlToPlain(content), Modifier.padding(top = 8.dp), style = XType.Title, fontWeight = FontWeight.Bold)
            else RichBody(env, content, fontSize = 16.sp, lineHeight = 29.sp)
        }
    }
}

@Composable
fun NewsDetailScreen(env: CommunityEnv, key: String, back: () -> Unit, open: (Route) -> Unit, frontier: Boolean = false) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var article by remember(key, frontier) { mutableStateOf<NewsItem?>(null) }
    var error by remember(key, frontier) { mutableStateOf<String?>(null) }
    var loadingArticle by remember(key, frontier) { mutableStateOf(false) }
    val listState = rememberLazyListState()
    val discussion = rememberPlainComments(
        env, frontier to key, open,
        fetch = { env.forum.newsComments(key, it, frontier) },
        publish = { body, parent, images -> env.forum.commentNews(key, body, parent, frontier, images) },
        setLiked = { id, liked -> env.forum.likeNewsComment(key, id, liked, frontier) },
        delete = { env.forum.deleteNewsComment(key, it, frontier) },
    )

    fun loadArticle() {
        if (loadingArticle) return
        loadingArticle = true; error = null
        scope.launch {
            try { article = env.forum.news(key, frontier) }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { error = messageOf(e) }
            finally { loadingArticle = false }
        }
    }
    LaunchedEffect(key, frontier) { loadArticle() }

    CommentScaffold(env, discussion, Modifier.background(MaterialTheme.colorScheme.surface)) {
        ScreenBar(if (frontier) "小伴前沿" else "AI 日报", onBack = back)
        LazyColumn(Modifier.weight(1f), state = listState, contentPadding = PaddingValues(bottom = 16.dp)) {
            if (article == null) item {
                if (error != null) CenterMessage("文章加载失败：$error", "重试", ::loadArticle, Modifier.fillMaxWidth().padding(32.dp))
                else Spinner(Modifier.fillMaxWidth().height(120.dp))
            }
            article?.let { news ->
                item {
                    Column(Modifier.fillMaxWidth().padding(horizontal = 22.dp, vertical = 20.dp)) {
                        Text(news.title, style = XType.Headline, fontWeight = FontWeight.Bold)
                        Spacer(Modifier.height(14.dp))
                        val whenText = agoText(news.sourcePublishedAt ?: news.publishedAt)
                        Text(listOf(news.source, if (news.timeBasis == "collected") "采集于 $whenText" else whenText, if (news.sample) "示例" else "").filter { it.isNotBlank() }.joinToString(" · "), style = XType.Secondary, color = LocalExtraColors.current.ink3)
                        if (news.coverUrl.isNotBlank()) {
                            Spacer(Modifier.height(22.dp))
                            Cover(news.coverUrl, news.key.hashCode().toLong() and 0x7fffffff, Modifier.fillMaxWidth().aspectRatio(16f / 9f))
                        }
                        if (news.summary.isNotBlank()) {
                            Spacer(Modifier.height(24.dp))
                            Text(news.summary, Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(LocalExtraColors.current.fill).padding(16.dp), style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        Spacer(Modifier.height(26.dp))
                        if (news.bodyHtml.isNotBlank()) EditorialBody(env, news.bodyHtml)
                        else Text("这篇内容暂未整理站内正文。", style = XType.Body, color = LocalExtraColors.current.ink3)
                        if (news.url.isNotBlank()) Text("查看原始来源 ↗", Modifier.padding(top = 24.dp).clickable { env.platform?.openUrl(news.url) }.padding(vertical = 6.dp), color = Brand, style = XType.Secondary)
                    }
                }
                plainCommentItems(env, discussion, open)
            }
        }
        if (article != null) {
            CommentBar(if (session.loggedIn) "写评论…" else "登录后评论", onWrite = { discussion.begin() }) {
                CommentCountAction(discussion.total) { scope.launch { listState.animateScrollToItem(1) } }
            }
        }
    }
}
