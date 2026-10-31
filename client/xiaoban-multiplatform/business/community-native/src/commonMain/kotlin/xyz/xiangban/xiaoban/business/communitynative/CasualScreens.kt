package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.staggeredgrid.LazyVerticalStaggeredGrid
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridCells
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridItemSpan
import androidx.compose.foundation.lazy.staggeredgrid.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import coil3.compose.AsyncImage
import xyz.xiangban.xiaoban.common.community.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun CasualWaterfall(env: CommunityEnv, controller: DiscussController, open: (Route) -> Unit, header: @Composable () -> Unit) {
    val state by controller.state.collectAsState()
    val gridState = remember(controller) { LazyStaggeredGridState() }
    LaunchedEffect(controller) { controller.select("latest") }
    Column(Modifier.fillMaxSize()) {
        header()
        PullToRefreshBox(state.loading && state.topics.isNotEmpty(), controller::refresh, Modifier.weight(1f)) {
            LazyVerticalStaggeredGrid(StaggeredGridCells.Fixed(2), Modifier.fillMaxSize(), state = gridState,
                contentPadding = PaddingValues(start = 10.dp, end = 10.dp, top = 10.dp, bottom = NavInset + 24.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp), verticalItemSpacing = 10.dp) {
                when {
                    state.loading && state.topics.isEmpty() -> item(span = StaggeredGridItemSpan.FullLine) { Spinner(Modifier.fillMaxWidth().height(160.dp)) }
                    state.error != null && state.topics.isEmpty() -> item(span = StaggeredGridItemSpan.FullLine) { CenterMessage(state.error.orEmpty(), "重试", controller::refresh, Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                    state.topics.isEmpty() -> item(span = StaggeredGridItemSpan.FullLine) { CenterMessage("这里还没有帖子", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                    else -> {
                        items(state.topics.distinctBy { it.id }, key = { it.id }) { topic -> CasualCard(env, topic) { open(Route.Topic(topic.id)) } }
                        item(span = StaggeredGridItemSpan.FullLine) { LoadMoreFooter(state.hasMore, state.error, state.topics.size, controller::loadMore, controller::loadMore) }
                    }
                }
            }
        }
    }
}

@Composable
private fun CasualCard(env: CommunityEnv, topic: Topic, onClick: () -> Unit) {
    val url = topic.images.firstOrNull()?.let(env.forum::imageUrl)
    var ratio by rememberSaveable(url) { mutableStateOf(.75f) }
    var failed by remember(url) { mutableStateOf(false) }
    val title = topic.cardTitle
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick)) {
        Box(Modifier.fillMaxWidth().aspectRatio(if (url == null) (if (title.length > 24) .75f else 1f) else ratio)
            .background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
            if (url == null || failed) {
                Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Box(Modifier.width(24.dp).height(3.dp).background(Brand))
                    Text(title, fontSize = 19.sp, fontWeight = FontWeight.Bold, lineHeight = 28.sp, maxLines = 6, overflow = TextOverflow.Ellipsis)
                }
            }
            if (url != null && !failed) AsyncImage(url, "${title}的封面", imageLoader = rememberImageLoader(), contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(), onSuccess = {
                    val image = it.result.image
                    if (image.width > 0 && image.height > 0) ratio = (image.width.toFloat() / image.height).coerceIn(.65f, 1.5f)
                }, onError = { failed = true })
            if (topic.pinned) Text("置顶", Modifier.align(Alignment.TopStart).padding(8.dp).clip(RoundedCornerShape(4.dp)).background(Brand).padding(horizontal = 6.dp, vertical = 3.dp), color = Color.White, fontSize = 10.sp)
        }
        Column(Modifier.padding(horizontal = 10.dp, vertical = 10.dp), verticalArrangement = Arrangement.spacedBy(9.dp)) {
            Text(title, fontSize = 14.sp, lineHeight = 20.sp, fontWeight = FontWeight.Medium, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                Avatar(topic.author.nickname, 18.dp, showInitial = true)
                Text(topic.author.nickname, Modifier.weight(1f), fontSize = 10.sp, maxLines = 1, overflow = TextOverflow.Ellipsis, color = LocalExtraColors.current.ink3)
                XIcon(XIcons.Heart, size = 13.dp, tint = LocalExtraColors.current.ink3)
                Text(compactCount(topic.likeCount), fontSize = 10.sp, color = LocalExtraColors.current.ink3)
            }
        }
    }
}

/** Full article images, never the list API's three-thumbnail subset. */
@Composable
internal fun CasualGallery(env: CommunityEnv, images: List<BodyBlock.Image>) {
    if (images.isEmpty()) return
    val pager = rememberPagerState(pageCount = { images.size })
    var zoom by remember(images) { mutableStateOf<Int?>(null) }
    var ratio by remember(images.first().src) { mutableStateOf(.75f) }
    BoxWithConstraints(Modifier.fillMaxWidth().background(LocalExtraColors.current.fill)) {
        Box(Modifier.fillMaxWidth().height((maxWidth / ratio).coerceIn(180.dp, 520.dp))) {
            HorizontalPager(pager, Modifier.fillMaxSize(), key = { images[it].src }) { index ->
                var failed by remember(images[index].src) { mutableStateOf(false) }
                Box(Modifier.fillMaxSize().clickable { zoom = index }, contentAlignment = Alignment.Center) {
                    if (failed) Text("图片暂时无法加载", color = LocalExtraColors.current.ink3, fontSize = 13.sp)
                    AsyncImage(images[index].src, images[index].alt.ifBlank { "图片 ${index + 1}" }, imageLoader = rememberImageLoader(),
                        modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Fit, onError = { failed = true }, onSuccess = {
                            failed = false
                            if (index == 0 && it.result.image.height > 0) ratio = (it.result.image.width.toFloat() / it.result.image.height).coerceIn(.65f, 1.8f)
                        })
                }
            }
            if (images.size > 1) Text("${pager.currentPage + 1} / ${images.size}", Modifier.align(Alignment.BottomEnd).padding(12.dp)
                .clip(CircleShape).background(Color.Black.copy(alpha = .48f)).padding(horizontal = 10.dp, vertical = 4.dp), color = Color.White, fontSize = 12.sp)
        }
    }
    zoom?.let { initial ->
        Dialog({ zoom = null }, properties = DialogProperties(usePlatformDefaultWidth = false)) {
            val fullPager = rememberPagerState(initialPage = initial, pageCount = { images.size })
            Box(Modifier.fillMaxSize().background(Color.Black)) {
                HorizontalPager(fullPager, Modifier.fillMaxSize()) { index ->
                    AsyncImage(images[index].src, "图片 ${index + 1}", imageLoader = rememberImageLoader(), contentScale = ContentScale.Fit, modifier = Modifier.fillMaxSize())
                }
                Text("${fullPager.currentPage + 1} / ${images.size}", Modifier.align(Alignment.TopCenter).statusBarsPadding().padding(18.dp), color = Color.White, fontSize = 14.sp)
                Box(Modifier.align(Alignment.TopEnd).statusBarsPadding().padding(8.dp)) { IconTap(XIcons.Close, "关闭图片", { zoom = null }, tint = Color.White) }
            }
        }
    }
}
