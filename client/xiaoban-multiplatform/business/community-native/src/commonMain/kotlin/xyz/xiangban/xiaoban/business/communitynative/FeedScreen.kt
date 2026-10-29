package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items as gridItems
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.FeedController
import xyz.xiangban.xiaoban.common.community.FeedState
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.PackKinds
import xyz.xiangban.xiaoban.common.community.Project
import xyz.xiangban.xiaoban.common.community.VisualKinds
import xyz.xiangban.xiaoban.common.community.WeeklyIssue
import xyz.xiangban.xiaoban.common.community.kindLabel

private val FeedTabs: List<Pair<String, String>> = listOf("" to "全部", "weekly" to "周刊", "projects" to "项目") + PackKinds

/** 社区首页：二级 Tab 加信息流。图片优先的类型用双列封面卡，其余用通栏列表。 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeedScreen(env: CommunityEnv, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { FeedController(env.forum, scope) }
    val state by controller.state.collectAsState()
    var tab by rememberSaveable { mutableStateOf("") }
    LaunchedEffect(tab) { controller.select(tab) }

    Column(Modifier.fillMaxSize()) {
        LazyRow(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface), contentPadding = PaddingValues(horizontal = 6.dp)) {
            items(FeedTabs, key = { it.first }) { (key, label) -> Chip(label, tab == key) { tab = key } }
        }
        Hairline()
        PullToRefreshBox(isRefreshing = state.loading && state.isEmptyList, onRefresh = controller::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.loading && state.isEmptyList -> Spinner()
                state.error != null && state.isEmptyList -> CenterMessage(state.error.orEmpty(), "重试", controller::refresh)
                state.isEmpty -> CenterMessage("这里还没有内容")
                tab == "weekly" -> WeeklyList(state, open, controller::loadMore, controller::refresh)
                tab == "projects" -> ProjectList(state.projects, open)
                tab in VisualKinds -> PackGrid(state, open, controller::loadMore, controller::refresh)
                else -> PackList(state, open, controller::loadMore, controller::refresh)
            }
        }
    }
}

private val FeedState.isEmptyList: Boolean get() = packs.isEmpty() && weekly.isEmpty() && projects.isEmpty()

@Composable
private fun LoadMoreFooter(state: FeedState, count: Int, onMore: () -> Unit, onRetry: () -> Unit) {
    when {
        state.error != null -> TextButton(onClick = onRetry) { Text("加载失败，点这里重试") }
        state.hasMore -> {
            // 这一项滚到可见才会进入组合，以此触发下一页；条数变化后再次触发。
            LaunchedEffect(count) { onMore() }
            Spinner(Modifier.fillMaxWidth().height(48.dp))
        }
    }
}

@Composable
private fun PackList(state: FeedState, open: (Route) -> Unit, onMore: () -> Unit, onRetry: () -> Unit) {
    LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        items(state.packs, key = { it.id }) { pack ->
            PackRow(pack) { open(Route.Pack(pack.id)) }
            Hairline(Modifier.padding(start = Gutter))
        }
        item { LoadMoreFooter(state, state.packs.size, onMore, onRetry) }
    }
}

@Composable
private fun PackRow(pack: Pack, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(Gutter), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Cover(pack.coverUrl, pack.id, Modifier.size(width = 96.dp, height = 72.dp))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            TitleText(pack.title)
            if (pack.summary.isNotBlank()) Text(pack.summary, color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp, maxLines = 2)
            MetaText(packMeta(pack))
        }
    }
}

internal fun packMeta(pack: Pack): String =
    listOfNotNull(kindLabel(pack.kind), pack.author?.nickname, pack.trustLabel.takeIf { pack.trustLevel < 2 }, "${pack.applyCount} 次应用".takeIf { pack.applyCount > 0 }).joinToString(" · ")

@Composable
private fun PackGrid(state: FeedState, open: (Route) -> Unit, onMore: () -> Unit, onRetry: () -> Unit) {
    LazyVerticalGrid(
        GridCells.Fixed(2), Modifier.fillMaxSize(),
        contentPadding = PaddingValues(Gutter), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        gridItems(state.packs, key = { it.id }) { pack ->
            Column(Modifier.clickable { open(Route.Pack(pack.id)) }, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Cover(pack.coverUrl, pack.id, Modifier.fillMaxWidth().aspectRatio(4f / 3f))
                TitleText(pack.title, maxLines = 1)
                MetaText(packMeta(pack))
            }
        }
        item(span = { GridItemSpan(2) }) { LoadMoreFooter(state, state.packs.size, onMore, onRetry) }
    }
}

@Composable
private fun WeeklyList(state: FeedState, open: (Route) -> Unit, onMore: () -> Unit, onRetry: () -> Unit) {
    LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        items(state.weekly, key = { it.week }) { issue -> WeeklyRow(issue) { open(Route.Weekly(issue.week)) }; Hairline(Modifier.padding(start = Gutter)) }
        item { LoadMoreFooter(state, state.weekly.size, onMore, onRetry) }
    }
}

@Composable
private fun WeeklyRow(issue: WeeklyIssue, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(Gutter), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        TitleText(issue.title)
        if (issue.summary.isNotBlank()) Text(issue.summary, color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp, maxLines = 2)
        MetaText(listOfNotNull(issue.week, issue.publishedAt?.take(10)).joinToString(" · "))
    }
}

@Composable
private fun ProjectList(projects: List<Project>, open: (Route) -> Unit) {
    LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        items(projects, key = { it.slug }) { project ->
            Column(Modifier.fillMaxWidth().clickable { open(Route.Project(project.slug)) }.padding(Gutter), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                TitleText(project.name)
                if (project.summary.isNotBlank()) Text(project.summary, color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp, maxLines = 2)
            }
            Hairline(Modifier.padding(start = Gutter))
        }
    }
}
