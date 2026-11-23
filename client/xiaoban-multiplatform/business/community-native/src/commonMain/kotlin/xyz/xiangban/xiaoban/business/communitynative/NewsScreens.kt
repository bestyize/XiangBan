package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
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
import xyz.xiangban.xiaoban.common.share.ShareContent
import xyz.xiangban.xiaoban.common.share.ShareExtraAction
import xyz.xiangban.xiaoban.common.share.ShareIconKey
import xyz.xiangban.xiaoban.common.share.ShareThumbKind

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
fun NewsDetailScreen(env: CommunityEnv, key: String, back: () -> Unit, open: (Route) -> Unit, frontier: Boolean = false) = CommunityDetail {
    val scope = rememberCoroutineScope()
    var article by remember(key, frontier) { mutableStateOf<NewsItem?>(null) }
    var error by remember(key, frontier) { mutableStateOf<String?>(null) }
    var loadingArticle by remember(key, frontier) { mutableStateOf(false) }
    var sharing by remember(key, frontier) { mutableStateOf(false) }
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

    val news = article
    if (news == null) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar(if (frontier) "小伴前沿" else "AI 日报", onBack = back)
            val body = Modifier.fillMaxWidth().weight(1f)
            if (error != null) CenterMessage("文章加载失败：$error", "重试", ::loadArticle, body)
            else Spinner(body)
        }
        return@CommunityDetail
    }

    val fallbackAuthor = if (frontier) "小伴前沿" else "AI 日报"
    val images = remember(news.coverUrl) {
        listOfNotNull(news.coverUrl.takeIf(String::isNotBlank)?.let(env.forum::imageUrl))
    }

    CommentScaffold(env, discussion) {
        DetailAuthorBar(news.source.ifBlank { fallbackAuthor }, back, onShare = { sharing = true })
        LazyColumn(Modifier.weight(1f).fillMaxWidth(), state = listState, contentPadding = PaddingValues(bottom = 16.dp)) {
            item(key = "gallery") { DetailGallery(images, news.title) }
            item(key = "article") {
                DetailBody {
                    DetailHeading(news.title)
                    if (news.url.isNotBlank()) {
                        DetailPrimaryCta("阅读来源原文", XIcons.ExternalLink, onClick = { env.platform?.openUrl(news.url) })
                    }
                    if (news.summary.isNotBlank()) {
                        Text(
                            news.summary,
                            Modifier.padding(bottom = 16.dp).fillMaxWidth().clip(RoundedCornerShape(12.dp))
                                .background(LocalExtraColors.current.fill).padding(horizontal = 14.dp, vertical = 12.dp),
                            style = XType.DetailBody,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    if (news.bodyHtml.isNotBlank()) EditorialBody(env, news.bodyHtml)
                    else Text("这篇内容暂未整理站内正文。", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                    val whenText = agoText(news.sourcePublishedAt ?: news.publishedAt)
                    DetailMetadata(
                        listOf(
                            news.source,
                            if (news.timeBasis == "collected") "采集于 $whenText" else whenText,
                            if (news.sample) "示例" else "",
                        ).filter { it.isNotBlank() }.joinToString(" · ")
                    )
                }
            }
            plainCommentItems(env, discussion, open)
        }
        CommentBar("说点什么…", onWrite = { discussion.begin() }) {
            CommentCountAction(discussion.total) { scope.launch { listState.animateScrollToItem(2) } }
        }
    }
    OverlayTransition(sharing) { panel ->
        val shareUrl = news.url.takeIf(String::isNotBlank)
            ?: env.forum.imageUrl(if (frontier) "/frontier/${news.key}" else "/daily/${news.key}")
        val plainBody = remember(news) {
            if (news.bodyHtml.isNotBlank()) htmlToPlain(news.bodyHtml).trim() else news.summary.trim()
        }
        val extras = remember(plainBody, env.platform) {
            if (plainBody.isNotBlank() && env.platform != null) listOf(
                ShareExtraAction("copyFull", "复制全文", ShareIconKey.CopyText) {
                    env.platform.copy(plainBody)
                    discussion.toast = "已复制全文"
                    "已复制完整正文"
                },
            ) else emptyList()
        }
        DirectShareSheet(
            env = env,
            content = ShareContent(
                title = news.title,
                url = shareUrl,
                summary = news.summary,
                author = news.source.ifBlank { "小伴编辑部" },
                dateLabel = agoText(news.sourcePublishedAt ?: news.publishedAt),
                kindLabel = if (frontier) "小伴前沿" else "AI 日报",
                badge = if (frontier) "前沿" else "日报",
                coverUrl = news.coverUrl.takeIf(String::isNotBlank),
                thumbKind = ShareThumbKind.Daily,
            ),
            onDismiss = { sharing = false },
            onCopied = { discussion.toast = "链接已复制" },
            extraActions = extras,
            panelModifier = panel,
        )
    }
}
