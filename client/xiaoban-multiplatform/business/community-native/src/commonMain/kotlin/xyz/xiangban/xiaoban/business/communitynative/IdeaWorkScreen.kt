package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.FeaturedIdea
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.isIdeaPlayerId

@Composable
internal fun IdeaWorkScreen(env: CommunityEnv, id: String, back: () -> Unit, open: (Route) -> Unit) {
    val loader = rememberLoader(env, "idea-work:$id") {
        env.forum.featuredIdeas().find { it.id == id && (it.kind == "work" || it.kind == "exhibit") }
    }
    val state by loader.state.collectAsState()
    val work = (state as? Load.Ready)?.value
    if (work == null) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("创意", onBack = back)
            val body = Modifier.fillMaxWidth().weight(1f)
            when (val current = state) {
                Load.Loading -> Spinner(body)
                is Load.Failed -> CenterMessage(current.message, "重试", loader::reload, body)
                is Load.Ready -> CenterMessage("找不到这个作品", modifier = body)
            }
        }
        return
    }

    val discussion = rememberPlainComments(
        env, "idea:$id", open,
        fetch = { env.forum.ideaComments(id, it) },
        publish = { body, parent, images -> env.forum.commentIdea(id, body, parent, images) },
        setLiked = { comment, liked -> env.forum.likeIdeaComment(id, comment, liked) },
        delete = { env.forum.deleteIdeaComment(id, it) },
    )
    val session by env.session.state.collectAsState()
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()

    CommentScaffold(env, discussion, Modifier.background(MaterialTheme.colorScheme.surface)) {
        ScreenBar("创意", onBack = back)
        LazyColumn(Modifier.weight(1f), state = listState, contentPadding = PaddingValues(bottom = 16.dp)) {
            item(key = "work") { IdeaWorkContent(env, work, open) }
            plainCommentItems(env, discussion, open)
        }
        CommentBar(if (session.loggedIn) "写评论…" else "登录后评论", onWrite = { discussion.begin() }) {
            CommentCountAction(discussion.total) { scope.launch { listState.animateScrollToItem(1) } }
        }
    }
}

@Composable
private fun IdeaWorkContent(env: CommunityEnv, work: FeaturedIdea, open: (Route) -> Unit) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val exhibit = work.kind == "exhibit"
    val images = (listOf(work.cover) + work.images).filter(String::isNotBlank).map(env.forum::imageUrl).distinct()
    val player = work.player?.takeIf(::isIdeaPlayerId)
    val official = work.site?.let(::webPageRoute)
    val repository = work.repository?.let(::webPageRoute)
    val runtime = player?.let { id ->
        if (env.config.runtimeV2 && env.platform != null) Route.IdeaRuntime(id, work.title)
        else webPageRoute(env.forum.imageUrl("/create/run/ideas/$id"))
    }
    val launch = runtime ?: official

    Column(Modifier.fillMaxWidth()) {
        if (images.isNotEmpty()) IdeaWorkGallery(images, work.title)
        Column(Modifier.fillMaxWidth().padding(16.dp)) {
            val label = if (exhibit) "3D 模型" else work.label
            if (!label.isNullOrBlank()) {
                Text(label, style = XType.Secondary, color = extra.ink3)
                Spacer(Modifier.height(4.dp))
            }
            Text(work.title, style = XType.Headline, fontWeight = FontWeight.Bold)
            Text(
                work.introduction?.takeIf(String::isNotBlank) ?: work.description,
                Modifier.padding(top = 8.dp),
                style = XType.Body,
                color = colors.onSurfaceVariant,
            )
            Spacer(Modifier.height(16.dp))
            Hairline()
            Spacer(Modifier.height(8.dp))
            IdeaWorkCredit("作者", work.author)
            if (exhibit) IdeaWorkCredit("来源", "微观展厅", onClick = { open(Route.MicroGallery) })
            else if (official != null && runtime != null) IdeaWorkCredit("来源", "作品官网") { open(official) }
            if (repository != null) IdeaWorkCredit("源码", "查看源码") { open(repository) }

            if (launch != null) {
                Spacer(Modifier.height(16.dp))
                val action = when {
                    runtime == null -> "访问官网"
                    exhibit -> "体验 3D 模型"
                    else -> "立即试玩"
                }
                GhostPillButton(action, onClick = { open(launch) }, modifier = Modifier.fillMaxWidth())
                if (runtime != null) work.controls?.mobile?.takeIf(String::isNotBlank)?.let { hint ->
                    Text(hint, Modifier.align(Alignment.CenterHorizontally).padding(top = 8.dp), style = XType.Caption, color = extra.ink3)
                }
            }
        }
    }
}

@Composable
private fun IdeaWorkGallery(images: List<String>, title: String) {
    val extra = LocalExtraColors.current
    val loader = rememberImageLoader()
    if (images.size == 1) {
        AsyncImage(
            model = images[0],
            contentDescription = title + "封面",
            imageLoader = loader,
            contentScale = ContentScale.FillWidth,
            modifier = Modifier.fillMaxWidth().background(extra.fill),
        )
        return
    }

    val pager = rememberPagerState { images.size }
    var aspect by rememberSaveable(images.first()) { mutableFloatStateOf(1f) }
    Box(Modifier.fillMaxWidth()) {
        HorizontalPager(
            state = pager,
            modifier = Modifier.fillMaxWidth().aspectRatio(aspect).background(extra.fill),
            key = { images[it] },
            beyondViewportPageCount = 1,
        ) { page ->
            AsyncImage(
                model = images[page],
                contentDescription = "$title，第 ${page + 1} 张",
                imageLoader = loader,
                contentScale = ContentScale.Fit,
                modifier = Modifier.fillMaxSize(),
                onSuccess = { result ->
                    val image = result.result.image
                    if (page == 0 && image.width > 0 && image.height > 0) {
                        aspect = image.width.toFloat() / image.height
                    }
                },
            )
        }
        val pageDescription = "第 ${pager.currentPage + 1} 张，共 ${images.size} 张"
        Row(
            Modifier.align(Alignment.BottomEnd)
                .padding(15.dp)
                .semantics { contentDescription = pageDescription },
            horizontalArrangement = Arrangement.spacedBy(6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            images.indices.forEach { page ->
                val color = if (page == pager.currentPage) Color.White else Color.White.copy(alpha = .48f)
                Box(Modifier.size(5.dp).background(color, CircleShape))
            }
        }
    }
}

@Composable
private fun IdeaWorkCredit(label: String, value: String, onClick: (() -> Unit)? = null) {
    Row(
        Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text(label, style = XType.Secondary, color = LocalExtraColors.current.ink3)
        if (onClick == null) Text(value, Modifier.padding(vertical = 6.dp), style = XType.Secondary)
        else Text(
            value,
            Modifier.clickable(role = Role.Button, onClick = onClick).heightIn(min = 32.dp).padding(vertical = 7.dp),
            style = XType.Secondary,
            textDecoration = TextDecoration.Underline,
        )
    }
}
