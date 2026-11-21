package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.CapabilityPackKinds
import coil3.compose.AsyncImage
import xyz.xiangban.xiaoban.common.community.FeaturedIdea
import xyz.xiangban.xiaoban.common.community.ForumCategories
import xyz.xiangban.xiaoban.common.community.ForumSection
import xyz.xiangban.xiaoban.common.community.IdeaFeedEntry
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Page
import xyz.xiangban.xiaoban.common.community.Topic
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
    val listState = rememberLazyListState()
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
    val featured = remember(showFeatured, catalogState) {
        if (!showFeatured) emptyList()
        else (catalogState as? Load.Ready)?.value.orEmpty()
            .sortedBy { featuredCardColors.keys.indexOf(it.id).takeIf { index -> index >= 0 } ?: Int.MAX_VALUE }
    }
    val featuredHero = featured.firstOrNull { it.id == "whale-girl" && it.cover.isNotBlank() }
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
            val rows = remember(feed.entries, columns) { feed.entries.chunked(columns) }
            LazyColumn(Modifier.fillMaxSize(), state = listState, contentPadding = CommunityFeedPadding) {
                if (!loading) {
                    if (failure != null) item(key = "error") {
                        Banner(failure, warn = true, actionText = "重试", onAction = ::refresh)
                    }
                    featuredHero?.let { hero ->
                        item(key = "spotlight") {
                            IdeaSpotlight(hero, env.forum.imageUrl(hero.cover)) { open(Route.IdeaWork(hero.id)) }
                            Spacer(Modifier.height(8.dp))
                        }
                    }
                    items(rows, key = { it.first().key }) { row ->
                        Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            row.forEach { entry -> IdeaFeedCard(entry, env, open, Modifier.weight(1f).fillMaxHeight()) }
                            repeat(columns - row.size) { Spacer(Modifier.weight(1f)) }
                        }
                        Spacer(Modifier.height(6.dp))
                    }
                    if (rows.isEmpty() && failure == null) item {
                        CenterMessage("暂时没有创意内容", modifier = Modifier.fillMaxWidth().height(150.dp))
                    }
                    if (page > 1 || feed.hasMore) item {
                        NotePager(page, feed.hasMore) {
                            selectedPage = it
                            listState.requestScrollToItem(0)
                        }
                    }
                }
            }
        }
    }
}

private val featuredCardColors = mapOf(
    "jiangshan-map" to Color(0xFFA8640C),
    "micro-gallery" to Color(0xFF07C160),
    "plush-rabbit" to Color(0xFFD93A44),
    "starry-nightrider" to Color(0xFF7152D8),
    "little-prince" to Color(0xFF2F6FDB),
    "saltline" to Color(0xFF2B2B30),
    "oil-paint" to Color(0xFFD93A44),
    "whale-girl" to Color(0xFF2F6FDB),
)

@Composable
private fun IdeaSpotlight(hero: FeaturedIdea, cover: String, open: () -> Unit) {
    Box(Modifier.fillMaxWidth().height(184.dp).clip(RoundedCornerShape(14.dp))
        .background(Color(0xFF141821)).clickable(onClick = open)) {
        AsyncImage(cover, null, imageLoader = rememberImageLoader(), contentScale = ContentScale.Crop, modifier = Modifier.matchParentSize())
        val scrim = Color(0xFF0A0E16)
        Column(
            Modifier.fillMaxSize().background(Brush.verticalGradient(
                0f to scrim.copy(alpha = .08f), .45f to scrim.copy(alpha = .22f), 1f to scrim.copy(alpha = .86f),
            )).padding(start = 20.dp, top = 16.dp, end = 20.dp, bottom = 18.dp),
            verticalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(
                listOfNotNull("✨ 本周主推", hero.label?.takeIf(String::isNotBlank)).joinToString(" · "),
                Modifier.background(Color.White.copy(alpha = .2f), CircleShape)
                    .border(1.dp, Color.White.copy(alpha = .26f), CircleShape).padding(horizontal = 7.dp, vertical = 2.dp),
                color = Color.White, style = feedText(9.5, 12.825), fontWeight = FontWeight.SemiBold, maxLines = 1,
            )
            Column {
                Text(hero.title, Modifier.padding(bottom = 4.dp), style = feedText(17, 22.1), color = Color.White,
                    fontWeight = FontWeight.Bold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                Text(hero.description, Modifier.padding(bottom = 8.dp), style = feedText(12), color = Color.White.copy(alpha = .84f),
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
                Row(Modifier.background(Color.White, CircleShape).padding(horizontal = 10.dp, vertical = 5.dp),
                    verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                    Text("立即体验", style = feedText(11), fontWeight = FontWeight.Bold, color = Color(0xFF111827))
                    XIcon(XIcons.ChevronRight, size = 13.dp, tint = Color(0xFF111827))
                }
            }
        }
    }
}

@Composable
private fun IdeaFeedCard(entry: IdeaFeedEntry, env: CommunityEnv, open: (Route) -> Unit, modifier: Modifier) {
    when (entry) {
        is IdeaFeedEntry.Featured -> with(entry.item) {
            val title = when (id) {
                "micro-gallery" -> "微观展厅 · 龙鱼水族箱"
                "oil-paint" -> "Oil Paint · 实时油画漫步"
                else -> title
            }
            val label = when (id) {
                "micro-gallery" -> "展厅"
                "plush-rabbit" -> "3D 模型"
                else -> label
            }
            CommunityWorkCard(title, cover.takeIf(String::isNotBlank)?.let(env.forum::imageUrl), label, author, date,
                modifier, featuredCardColors[id]) {
                open(if (kind == "gallery") Route.MicroGallery else Route.IdeaWork(id))
            }
        }
        is IdeaFeedEntry.Work -> with(entry.pack) {
            CommunityWorkCard(title, coverUrl?.let(env.forum::imageUrl), kindLabel(kind), author?.nickname, updatedAt, modifier) {
                open(if (kind in CapabilityPackKinds) Route.Ability(id) else Route.Pack(id))
            }
        }
        is IdeaFeedEntry.Plan -> with(entry.scheme) {
            CommunityWorkCard(title, null, "桌面方案", author.nickname, createdAt, modifier) { open(Route.Scheme(id)) }
        }
        is IdeaFeedEntry.Post -> with(entry.topic) {
            CommunityWorkCard(cardTitle, images.firstOrNull()?.let(env.forum::imageUrl), category.name, author.nickname, createdAt,
                modifier, avatarUrl = author.avatarUrl?.takeIf(String::isNotBlank)?.let(env.forum::imageUrl)) { open(Route.Topic(id)) }
        }
    }
}

@Composable
internal fun CommunityWorkCard(
    title: String,
    cover: String?,
    label: String?,
    author: String?,
    date: String,
    modifier: Modifier = Modifier,
    avatarColor: Color? = null,
    avatarUrl: String? = null,
    onClick: () -> Unit,
) {
    val loader = rememberImageLoader()
    Column(modifier.fillMaxWidth().communityFeedCard().clickable(onClick = onClick)) {
        Box(Modifier.fillMaxWidth().aspectRatio(3f / 2f).background(feedFill)) {
            if (!cover.isNullOrBlank()) AsyncImage(cover, null, imageLoader = loader, modifier = Modifier.matchParentSize(),
                contentScale = ContentScale.Crop)
            if (!label.isNullOrBlank()) Text(
                label, Modifier.padding(6.dp).background(Color(0xFF111827).copy(alpha = .56f), CircleShape)
                    .padding(horizontal = 6.dp, vertical = 1.5.dp),
                color = Color.White, style = feedText(9.5, 12.825), fontWeight = FontWeight.SemiBold,
                maxLines = 1, overflow = TextOverflow.Ellipsis,
            )
        }
        Column(Modifier.weight(1f).padding(start = 10.dp, top = 9.dp, end = 10.dp, bottom = 10.dp)) {
            Text(title, style = feedText(13, 18.2), color = feedInk, fontWeight = FontWeight(650),
                maxLines = 2, overflow = TextOverflow.Ellipsis)
            if (!author.isNullOrBlank() || date.isNotBlank()) {
                Spacer(Modifier.weight(1f))
                Row(Modifier.fillMaxWidth().padding(top = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    if (!author.isNullOrBlank()) {
                        Box(Modifier.size(20.dp).clip(CircleShape).background(avatarColor ?: Brand), contentAlignment = Alignment.Center) {
                            Text(author.trimStart('@').take(1).uppercase(), style = feedText(10), color = Color.White, fontWeight = FontWeight.Bold)
                            avatarUrl?.let { AsyncImage(it, null, imageLoader = loader, modifier = Modifier.matchParentSize(), contentScale = ContentScale.Crop) }
                        }
                    }
                    Text(author.orEmpty(), Modifier.weight(1f), style = feedText(11.5, 16.1), color = feedSecondary,
                        maxLines = 1, overflow = TextOverflow.Ellipsis)
                    if (date.isNotBlank()) Text(date.take(10).removePrefix(date.take(4) + "-"), style = feedText(11), color = feedMuted, maxLines = 1)
                }
            }
        }
    }
}
