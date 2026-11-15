package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.CapabilityPackKinds
import xyz.xiangban.xiaoban.common.community.ForumCategories
import xyz.xiangban.xiaoban.common.community.ForumSection
import xyz.xiangban.xiaoban.common.community.IdeaFeedEntry
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Page
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.cardTitle
import xyz.xiangban.xiaoban.common.community.kindLabel
import xyz.xiangban.xiaoban.common.community.mergeIdeaFeedPage

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun IdeasFeed(
    env: CommunityEnv,
    categoryState: Load<ForumCategories>,
    reloadCategories: () -> Unit,
    open: (Route) -> Unit,
) {
    var selectedPage by rememberSaveable { mutableIntStateOf(1) }
    val page = selectedPage
    val gridState = rememberLazyGridState()
    val session by env.session.state.collectAsState()
    val catalog = rememberLoader(env, "ideas-catalog") { env.forum.featuredIdeas() }
    val packs = rememberLoader(env, Triple("ideas-packs", page, session.accountId)) {
        env.forum.packs(sort = "new", page = page)
    }
    val schemes = rememberLoader(env, Triple("ideas-schemes", page, session.accountId)) { env.content.schemes(page) }
    val category = (categoryState as? Load.Ready)?.value?.boardsIn(ForumSection.Ideas)?.joinToString(",") { it.slug }
    val topics = if (category != null) {
        rememberLoader(env, listOf("ideas-topics", page, session.accountId, category)) {
            if (category.isEmpty()) Page<Topic>()
            else env.forum.topics(sort = "new", category = category, page = page, home = false, activeOnly = true)
        }
    } else null

    val catalogState by catalog.state.collectAsState()
    val packState by packs.state.collectAsState()
    val schemeState by schemes.state.collectAsState()
    val topicState = topics?.state?.collectAsState()?.value ?: (categoryState as? Load.Failed ?: Load.Loading)
    val showFeatured = page == 1
    val featured = if (showFeatured) (catalogState as? Load.Ready)?.value.orEmpty() else emptyList()
    val feed = remember(featured, packState, schemeState, topicState) {
        mergeIdeaFeedPage(
            featured, (packState as? Load.Ready)?.value, (schemeState as? Load.Ready)?.value,
            (topicState as? Load.Ready)?.value,
        )
    }
    val communityStates = listOf(packState, schemeState, topicState)
    val loading = (showFeatured && catalogState is Load.Loading) || communityStates.any { it is Load.Loading }
    val failure = when {
        communityStates.any { it is Load.Failed } -> "部分创意内容加载失败"
        showFeatured && catalogState is Load.Failed -> "精选作品加载失败"
        else -> null
    }

    fun refresh() {
        if (showFeatured) catalog.reload()
        packs.reload()
        schemes.reload()
        if (topics == null) reloadCategories() else topics.reload()
    }

    PullToRefreshBox(isRefreshing = loading, onRefresh = ::refresh, modifier = Modifier.fillMaxSize()) {
        BoxWithConstraints(Modifier.fillMaxSize()) {
            val columns = when {
                maxWidth >= 960.dp -> 4
                maxWidth >= 720.dp -> 3
                maxWidth >= 310.dp -> 2
                else -> 1
            }
            val cardArrangement = Arrangement.spacedBy(if (columns == 2) 4.dp else 8.dp)
            val gridPadding = if (columns == 2) {
                PaddingValues(
                    start = 4.dp, end = 4.dp,
                    top = FeedPadding.calculateTopPadding(), bottom = FeedPadding.calculateBottomPadding(),
                )
            } else FeedPadding
            LazyVerticalGrid(
                columns = GridCells.Fixed(columns), state = gridState, modifier = Modifier.fillMaxSize(),
                contentPadding = gridPadding, horizontalArrangement = cardArrangement,
                verticalArrangement = cardArrangement,
            ) {
                if (!loading) {
                    if (failure != null) item(span = { GridItemSpan(maxLineSpan) }) {
                        Banner(failure, warn = true, actionText = "重试", onAction = ::refresh)
                    }
                    items(feed.entries, key = { it.key }) { entry ->
                        IdeaFeedCard(entry, env, open)
                    }
                    if (feed.entries.isEmpty() && failure == null) item(span = { GridItemSpan(maxLineSpan) }) {
                        CenterMessage("暂时没有创意内容", modifier = Modifier.fillMaxWidth().height(150.dp))
                    }
                    if (page > 1 || feed.hasMore) item(span = { GridItemSpan(maxLineSpan) }) {
                        NotePager(page, feed.hasMore) {
                            selectedPage = it
                            gridState.requestScrollToItem(0)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun IdeaFeedCard(entry: IdeaFeedEntry, env: CommunityEnv, open: (Route) -> Unit) {
    when (entry) {
        is IdeaFeedEntry.Featured -> with(entry.item) {
            val runtime = player
            IdeaCard(title, description, env.forum.imageUrl(cover), label, author, date.take(10)) {
                when {
                    kind == "gallery" -> open(Route.MicroGallery)
                    runtime != null -> open(Route.IdeaRuntime(runtime, title))
                    else -> webPageRoute(env.forum.imageUrl(href))?.let(open)
                }
            }
        }
        is IdeaFeedEntry.Work -> with(entry.pack) {
            IdeaCard(
                title, summary, coverUrl?.let(env.forum::imageUrl), kindLabel(kind), author?.nickname, agoText(updatedAt),
            ) {
                open(if (kind in CapabilityPackKinds) Route.Ability(id) else Route.Pack(id))
            }
        }
        is IdeaFeedEntry.Plan -> with(entry.scheme) {
            IdeaCard(title, summary, null, "桌面方案", author.nickname, agoText(createdAt)) { open(Route.Scheme(id)) }
        }
        is IdeaFeedEntry.Post -> with(entry.topic) {
            IdeaCard(
                cardTitle, excerpt, images.firstOrNull()?.let(env.forum::imageUrl), category.name, author.nickname, agoText(createdAt),
            ) {
                open(Route.Topic(id))
            }
        }
    }
}

@Composable
private fun IdeaCard(
    title: String,
    description: String,
    cover: String?,
    label: String?,
    author: String?,
    date: String,
    onClick: () -> Unit,
) {
    val coverRatio = 3f / 2f
    Column(Modifier.fillMaxWidth().outlinedCard(8.dp).clickable(onClick = onClick)) {
        Box(Modifier.fillMaxWidth().aspectRatio(coverRatio).background(LocalExtraColors.current.fill)) {
            if (!cover.isNullOrBlank()) CommunityCover(cover, initialAspectRatio = coverRatio)
            if (!label.isNullOrBlank()) {
                val labelScale = 2f / 3f
                val caption = XType.Caption
                Text(
                    label,
                    Modifier.align(Alignment.TopEnd).padding(6.dp)
                        .background(Color(0xFF606060), RoundedCornerShape(5.dp * labelScale))
                        .padding(horizontal = 5.dp * labelScale, vertical = 2.dp * labelScale),
                    color = Color.White,
                    style = caption.copy(fontSize = caption.fontSize * labelScale, lineHeight = caption.lineHeight * labelScale),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        Column(
            modifier = Modifier.padding(start = 12.dp, top = 8.dp, end = 12.dp, bottom = 15.dp),
            verticalArrangement = Arrangement.spacedBy(5.dp),
        ) {
            Text(title, style = XType.Section, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (description.isNotBlank()) Text(
                description, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 2, overflow = TextOverflow.Ellipsis,
            )
            if (!author.isNullOrBlank() || date.isNotBlank()) {
                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(author.orEmpty(), Modifier.weight(1f), style = XType.Caption, color = LocalExtraColors.current.ink3,
                        maxLines = 1, overflow = TextOverflow.Ellipsis)
                    if (date.isNotBlank()) Text(date, style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 1)
                }
            }
        }
    }
}
