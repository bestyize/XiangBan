package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
        TabStrip(navigationCategories.map { it.key to it.label }, selected.key) { category = it; filter = "" }
        PageChipRow(listOf("" to "全部") + selected.filters.map { it.key to it.label }, filter, { filter = it }, Modifier.padding(horizontal = 12.dp, vertical = 12.dp))
        PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
            LoadBox(state, loader::reload) { directory ->
                val entries = remember(directory, category, filter) { directory.navigationDirectory().filter { it.matchesNavigation(category, filter) } }
                LazyVerticalGrid(GridCells.Adaptive(160.dp), Modifier.fillMaxSize(), contentPadding = FeedPadding,
                    horizontalArrangement = Arrangement.spacedBy(10.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
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
    Column(Modifier.fillMaxWidth().heightIn(min = 158.dp).outlinedCard(12.dp).clickable(onClick = onClick).padding(14.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            BrandTile(BrandLogos.find(link.logo) ?: BrandLogos.find(link.name) ?: BrandLogos.find(link.vendor), link.name, size = 34.dp, corner = 9.dp)
            Column(Modifier.weight(1f)) {
                Text(link.name, style = XType.Secondary, fontWeight = FontWeight.Normal, maxLines = 2, overflow = TextOverflow.Ellipsis)
                Text(link.vendor.ifBlank { navigationDomain(link.url) }, Modifier.padding(top = 2.dp), style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
        }
        Text(link.summary, Modifier.padding(top = 12.dp).heightIn(min = 38.dp), style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 2, overflow = TextOverflow.Ellipsis)
        Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
            if (link.navigationType.isNotBlank()) GrayTag(link.navigationType, compact = true)
            if (link.sample) Text("示例", style = XType.Caption, color = Brand)
        }
    }
}
