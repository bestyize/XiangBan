package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.NewsItem
import xyz.xiangban.xiaoban.common.community.Page
import xyz.xiangban.xiaoban.common.community.agoText

/** 小伴前沿独立于日报；使用已发布正文，详情和留言保持站内。 */
@Composable
internal fun FrontierSection(env: CommunityEnv, state: Load<Page<NewsItem>>, retry: () -> Unit, open: (Route) -> Unit) {
    if (state is Load.Ready && state.value.items.isEmpty()) return
    WhiteCard(padding = 12.dp) {
        Row(Modifier.fillMaxWidth().padding(bottom = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            XIcon(XIcons.File, size = 18.dp)
            Text("小伴前沿", Modifier.weight(1f).padding(start = 6.dp), style = XType.Section, fontWeight = FontWeight.SemiBold)
            LinkText("查看全部", { open(Route.Frontier) })
        }
        when (state) {
            is Load.Ready -> FrontierCarousel(env, state.value.items.take(5), open)
            is Load.Failed -> Banner(state.message, warn = true, actionText = "重试", onAction = retry)
            else -> Spinner(Modifier.fillMaxWidth().height(120.dp))
        }
    }
}

@Composable
private fun FrontierCarousel(env: CommunityEnv, items: List<NewsItem>, open: (Route) -> Unit) {
    val pager = rememberPagerState { items.size }
    val scope = rememberCoroutineScope()
    Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.surfaceVariant)) {
        HorizontalPager(pager, Modifier.fillMaxSize(), key = { items[it].key }) { page ->
            val article = items[page]
            Box(Modifier.fillMaxSize().clickable { open(Route.FrontierArticle(article.key)) }) {
                if (article.coverUrl.isNotBlank()) Cover(env.forum.imageUrl(article.coverUrl), article.key.hashCode().toLong(), Modifier.fillMaxSize())
                Box(Modifier.fillMaxSize().background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = .8f)))))
                Column(Modifier.align(Alignment.BottomStart).fillMaxWidth().padding(start = 16.dp, end = 16.dp, bottom = 14.dp)) {
                    Text(article.title, style = XType.Body, color = Color.White, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Row(Modifier.fillMaxWidth().padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Spacer(Modifier.width(if (items.size > 1) (items.size * 16).dp else 0.dp))
                        Text(article.source, Modifier.weight(1f), style = XType.Caption, color = Color.White.copy(alpha = .85f), maxLines = 1, overflow = TextOverflow.Ellipsis, textAlign = androidx.compose.ui.text.style.TextAlign.End)
                        Text(agoText(article.sourcePublishedAt ?: article.publishedAt), style = XType.Caption, color = Color.White.copy(alpha = .85f), maxLines = 1)
                        if (article.sample) Text("示例", style = XType.Caption, color = Color.White)
                    }
                }
            }
        }
        if (items.size > 1) Row(Modifier.align(Alignment.BottomStart).padding(start = 10.dp, bottom = 4.dp)) {
            items.forEachIndexed { index, _ ->
                Box(Modifier.width(16.dp).height(36.dp).clickable { scope.launch { pager.animateScrollToPage(index) } }, contentAlignment = Alignment.Center) {
                    Box(Modifier.size(if (pager.currentPage == index) 10.dp else 4.dp, 4.dp).clip(RoundedCornerShape(4.dp)).background(Color.White.copy(alpha = if (pager.currentPage == index) 1f else .5f)))
                }
            }
        }
    }
}
