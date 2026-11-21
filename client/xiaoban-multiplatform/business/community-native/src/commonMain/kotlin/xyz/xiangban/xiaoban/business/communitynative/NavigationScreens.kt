package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun NavTab(env: CommunityEnv) {
    var category by rememberSaveable { mutableStateOf("models") }
    var filter by rememberSaveable { mutableStateOf("") }
    val selected = navigationCategories.firstOrNull { it.key == category } ?: navigationCategories.first()
    val loader = rememberLoader(env, "nav") { env.forum.navLinks() }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        CommunityTabRow(navigationCategories.map { it.key to it.label }, selected.key, filter = true) { category = it; filter = "" }
        CommunityTabRow(listOf("" to "全部") + selected.filters.map { it.key to it.label }, filter,
            Modifier.padding(horizontal = 6.dp), filter = true) { filter = it }
        PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
            LoadBox(state, loader::reload) { directory ->
                val entries = remember(directory, category, filter) { directory.navigationDirectory().filter { it.matchesNavigation(category, filter) } }
                LazyVerticalGrid(GridCells.Adaptive(150.dp), Modifier.fillMaxSize(), contentPadding = CommunityFeedPadding,
                    horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    if (entries.isEmpty()) item(span = { GridItemSpan(maxLineSpan) }) { CenterMessage("这个分类还没有已发布入口", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                    items(entries, key = { it.key }) { link -> NavigationCard(link) { navigationUrl(link.url)?.let { env.platform?.openUrl(it) } } }
                    if (entries.isNotEmpty()) item(span = { GridItemSpan(maxLineSpan) }) { FootNote("点击卡片，在浏览器打开官方入口") }
                }
            }
        }
    }
}

@Composable
private fun NavigationCard(link: NavLink, onClick: () -> Unit) {
    Column(Modifier.fillMaxWidth().communityFeedCard().clickable(onClick = onClick).padding(11.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(9.dp)) {
            Box(Modifier.size(34.dp).background(feedFill, RoundedCornerShape(10.dp)), contentAlignment = Alignment.Center) {
                val logo = BrandLogos.find(link.logo) ?: BrandLogos.find(link.name) ?: BrandLogos.find(link.vendor)
                if (logo != null) XIcon(logo.icon, tint = logo.color, size = 24.dp)
                else Text(link.name.take(1), style = feedText(15), color = Brand, fontWeight = FontWeight.Bold)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                Text(link.name, style = feedText(14), color = feedInk, fontWeight = FontWeight(650), maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(link.vendor, style = feedText(11.5, 16.1), color = feedMuted, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        if (link.summary.isNotBlank()) Text(link.summary, Modifier.padding(top = 11.dp), style = feedText(12, 18.6),
            color = feedSecondary, maxLines = 2, overflow = TextOverflow.Ellipsis)
        Row(Modifier.fillMaxWidth().padding(top = 11.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(navigationDomain(link.url), Modifier.weight(1f), style = feedText(11), color = feedMuted, fontFamily = FontFamily.Monospace,
                maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (link.navigationType.isNotBlank()) FeedTag(link.navigationType, Modifier.widthIn(max = 65.dp))
        }
    }
}
