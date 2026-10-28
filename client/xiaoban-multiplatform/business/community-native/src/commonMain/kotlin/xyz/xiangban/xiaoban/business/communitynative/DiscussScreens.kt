package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.DiscussController
import xyz.xiangban.xiaoban.common.community.Loader
import xyz.xiangban.xiaoban.common.community.Post
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.htmlToPlain
import xyz.xiangban.xiaoban.common.community.messageOf

private val Sorts = listOf("latest" to "最新回复", "new" to "最新发布", "hot" to "热门")

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DiscussScreen(env: CommunityEnv, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { DiscussController(env.forum, scope) }
    val state by controller.state.collectAsState()
    var sort by rememberSaveable { mutableStateOf("latest") }
    LaunchedEffect(sort) { controller.select(sort) }

    Column(Modifier.fillMaxSize()) {
        PageHeader("讨论")
        Row(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(horizontal = 6.dp)) {
            Sorts.forEach { (key, label) -> Chip(label, sort == key) { sort = key } }
        }
        Hairline()
        PullToRefreshBox(isRefreshing = state.loading && state.topics.isEmpty(), onRefresh = controller::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.loading && state.topics.isEmpty() -> Spinner()
                state.error != null && state.topics.isEmpty() -> CenterMessage(state.error.orEmpty(), "重试", controller::refresh)
                state.topics.isEmpty() -> CenterMessage("还没有帖子")
                else -> LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
                    items(state.topics, key = { it.id }) { topic -> TopicRow(topic) { open(Route.Topic(topic.id)) }; Hairline(Modifier.padding(start = Gutter)) }
                    item {
                        when {
                            state.error != null -> TextButton(onClick = controller::loadMore) { Text("加载失败，点这里重试") }
                            state.hasMore -> { LaunchedEffect(state.topics.size) { controller.loadMore() }; Spinner(Modifier.fillMaxWidth().height(48.dp)) }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun TopicRow(topic: Topic, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(Gutter), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        TitleText(topic.title)
        if (topic.excerpt.isNotBlank()) Text(topic.excerpt, color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp, maxLines = 2)
        MetaText(listOfNotNull(topic.category.name, topic.author.nickname, "${topic.replyCount} 回复").joinToString(" · "))
    }
}

/** 帖子详情：正文加楼层，只读。回复、发帖暂时到网页端。 */
@Composable
fun TopicDetailScreen(env: CommunityEnv, id: Long, back: () -> Unit) {
    val scope = rememberCoroutineScope()
    val topicLoader = remember(env, id) { Loader(scope) { env.forum.topic(id) } }
    val topicState by topicLoader.state.collectAsState()
    LaunchedEffect(topicLoader) { topicLoader.reload() }

    var posts by remember(id) { mutableStateOf<List<Post>>(emptyList()) }
    var page by remember(id) { mutableStateOf(0) }
    var more by remember(id) { mutableStateOf(true) }
    var postsError by remember(id) { mutableStateOf<String?>(null) }
    var postsBusy by remember(id) { mutableStateOf(false) }

    suspend fun loadPosts() {
        if (postsBusy || !more) return
        postsBusy = true
        postsError = null
        try {
            val next = env.forum.posts(id, page + 1)
            posts = posts + next.items
            page += 1
            more = next.hasMore
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (e: Exception) {
            postsError = messageOf(e)
        } finally {
            postsBusy = false
        }
    }
    val retry = remember { mutableStateOf(0) }
    LaunchedEffect(id, retry.value) { loadPosts() }

    Column(Modifier.fillMaxSize()) {
        PageHeader("帖子", onBack = back)
        LoadBox(topicState, topicLoader::reload) { topic ->
            LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
                item {
                    Column(Modifier.padding(Gutter), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(topic.title, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                        MetaText(listOf(topic.category.name, topic.author.nickname, topic.createdAt.take(10)).filter { it.isNotBlank() }.joinToString(" · "))
                        Text(htmlToPlain(topic.bodyHtml), fontSize = 15.sp)
                        OutlinedButton({ env.platform?.openUrl(env.forum.topicUrl(topic.id)) }, enabled = env.platform != null, shape = CardShape) { Text("在网页打开，回复或点赞") }
                    }
                    Hairline()
                }
                items(posts, key = { it.id }) { post -> PostRow(post); Hairline(Modifier.padding(start = Gutter)) }
                item {
                    when {
                        postsError != null -> TextButton(onClick = { retry.value += 1 }) { Text("评论加载失败，点这里重试") }
                        more -> { LaunchedEffect(posts.size) { loadPosts() }; Spinner(Modifier.fillMaxWidth().height(48.dp)) }
                    }
                }
            }
        }
    }
}

@Composable
private fun PostRow(post: Post) {
    Column(Modifier.fillMaxWidth().padding(Gutter), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        MetaText("${post.floor} 楼 · ${post.author.nickname} · ${post.createdAt.take(10)}")
        when {
            post.blocked -> Text("你已屏蔽该用户", color = LocalExtraColors.current.ink3, fontSize = 14.sp)
            post.status != "normal" -> Text("该评论已被隐藏", color = LocalExtraColors.current.ink3, fontSize = 14.sp)
            else -> Text(htmlToPlain(post.bodyHtml), fontSize = 15.sp)
        }
    }
}
