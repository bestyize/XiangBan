package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.FeaturedIdea
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.isIdeaPlayerId

@Composable
internal fun IdeaWorkScreen(env: CommunityEnv, id: String, back: () -> Unit, open: (Route) -> Unit) = CommunityDetail {
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
        return@CommunityDetail
    }

    val discussion = rememberPlainComments(
        env, "idea:$id", open,
        fetch = { env.forum.ideaComments(id, it) },
        publish = { body, parent, images -> env.forum.commentIdea(id, body, parent, images) },
        setLiked = { comment, liked -> env.forum.likeIdeaComment(id, comment, liked) },
        delete = { env.forum.deleteIdeaComment(id, it) },
    )
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    var sharing by rememberSaveable(id) { mutableStateOf(false) }
    val images = remember(work.cover, work.images) {
        (listOf(work.cover) + work.images).filter(String::isNotBlank).map(env.forum::imageUrl).distinct()
    }

    CommentScaffold(env, discussion) {
        DetailAuthorBar(work.author, back, onShare = { sharing = true })
        LazyColumn(Modifier.weight(1f).fillMaxWidth(), state = listState, contentPadding = PaddingValues(bottom = 16.dp)) {
            item(key = "gallery") { DetailGallery(images, work.title) }
            item(key = "work") { IdeaWorkContent(env, work, images.size > 1, open) }
            plainCommentItems(env, discussion, open)
        }
        CommentBar("说点什么…", onWrite = { discussion.begin() }) {
            CommentCountAction(discussion.total) { scope.launch { listState.animateScrollToItem(2) } }
        }
    }
    OverlayTransition(sharing) { panel ->
        ShareSheet(env, work.title, work.author, env.forum.imageUrl(work.href), onDismiss = { sharing = false },
            onCopied = { discussion.toast = "链接已复制" }, panelModifier = panel)
    }
}

@Composable
private fun IdeaWorkContent(env: CommunityEnv, work: FeaturedIdea, hasDots: Boolean, open: (Route) -> Unit) {
    val exhibit = work.kind == "exhibit"
    val official = work.site?.let(::webPageRoute)
    val repository = work.repository?.let(::webPageRoute)
    val runtime = work.player?.takeIf(::isIdeaPlayerId)?.let { id ->
        if (env.config.runtimeV2 && env.platform != null) Route.IdeaRuntime(id, work.title)
        else webPageRoute(env.forum.imageUrl("/create/run/ideas/$id"))
    }
    val launch = runtime ?: official
    val controls = work.controls?.let { it.mobile?.takeIf(String::isNotBlank) ?: it.desktop?.takeIf(String::isNotBlank) }
    val hasCredits = exhibit || (official != null && runtime != null) || repository != null
    var showCredits by rememberSaveable(work.id) { mutableStateOf(false) }

    DetailBody(hasDots) {
        DetailHeading(work.title)
        Text(work.introduction?.takeIf(String::isNotBlank) ?: work.description, style = XType.DetailBody)
        DetailTags(listOfNotNull(if (exhibit) "3D 模型" else work.label, "小伴创意", "交互作品"))
        if (launch != null) {
            val label = when {
                exhibit -> "体验 3D 模型"
                runtime != null -> "打开${work.title}"
                else -> "访问作品官网"
            }
            DetailInfoStrip("体验", label, XIcons.SparkleSmall, onClick = { open(launch) })
        }
        if (controls != null || hasCredits) {
            DetailInfoStrip("说明", controls ?: "作品来源", onClick = if (hasCredits) ({ showCredits = !showCredits }) else null,
                expanded = showCredits)
        }
        if (showCredits && hasCredits) {
            if (exhibit) DetailInfoStrip("来源", "微观展厅", onClick = { open(Route.MicroGallery) })
            else if (official != null && runtime != null) DetailInfoStrip("来源", "作品官网", onClick = { open(official) })
            if (repository != null) DetailInfoStrip("源码", "查看源码", onClick = { open(repository) })
        }
        DetailMetadata("收录于 ${work.date.take(10)} 社区")
    }
}
