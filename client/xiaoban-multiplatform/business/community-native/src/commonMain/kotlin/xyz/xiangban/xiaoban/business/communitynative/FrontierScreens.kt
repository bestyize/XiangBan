package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
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
    Column(Modifier.fillMaxWidth().communityFeedCard(18.dp).padding(12.dp)) {
        Row(Modifier.fillMaxWidth().padding(bottom = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("前沿", Modifier.weight(1f), style = feedText(16), color = feedInk, fontWeight = FontWeight.Bold)
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
    Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).clip(RoundedCornerShape(14.dp)).background(Color(0xFF16181F))) {
        HorizontalPager(pager, Modifier.fillMaxSize(), key = { items[it].key }) { page ->
            val article = items[page]
            Box(Modifier.fillMaxSize().clickable { open(Route.FrontierArticle(article.key)) }) {
                if (article.coverUrl.isNotBlank()) Cover(env.forum.imageUrl(article.coverUrl), article.key.hashCode().toLong(), Modifier.fillMaxSize())
                Text(article.title, Modifier.align(Alignment.BottomStart).fillMaxWidth()
                    .background(Brush.verticalGradient(listOf(Color.Transparent, Color.Black.copy(alpha = .84f))))
                    .padding(start = 16.dp, top = 36.dp, end = 16.dp, bottom = 38.dp),
                    style = feedText(15.5, 21.7), fontWeight = FontWeight(650), color = Color.White,
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
                val articleMeta = listOfNotNull(article.source.takeIf(String::isNotBlank),
                    agoText(article.sourcePublishedAt ?: article.publishedAt).takeIf(String::isNotBlank),
                    "示例".takeIf { article.sample }).joinToString(" · ")
                Text(articleMeta, Modifier.align(Alignment.BottomEnd).padding(start = 106.dp, end = 14.dp, bottom = 11.dp),
                    style = feedText(11.5, 16.1), color = Color.White.copy(alpha = .8f),
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        if (items.size > 1) Row(Modifier.align(Alignment.BottomStart).padding(start = 14.dp, bottom = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            items.forEachIndexed { index, article ->
                val active = pager.currentPage == index
                Box(Modifier.size(if (active) 20.dp else 7.dp, 7.dp).clip(CircleShape)
                    .background(Color.White.copy(alpha = if (active) 1f else .48f))
                    .clickable(onClickLabel = article.title) { scope.launch { pager.animateScrollToPage(index) } })
            }
        }
    }
}
