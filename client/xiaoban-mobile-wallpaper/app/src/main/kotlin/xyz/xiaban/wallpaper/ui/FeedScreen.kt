package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridState
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridItemSpan
import androidx.compose.foundation.lazy.staggeredgrid.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiaban.wallpaper.companion.Expression
import xyz.xiaban.wallpaper.data.Category

private const val PRELOAD_AHEAD = 6

/** Pictures or videos from the website: category tabs and search on top, a two-column waterfall below. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeedScreen(feed: FeedViewModel, app: AppViewModel, grid: LazyStaggeredGridState) {
    val state by feed.state.collectAsState()
    val categories by app.categories.collectAsState()
    val favorites by app.favorites.collectAsState()
    val favoriteIds = remember(favorites) { favorites.mapTo(HashSet()) { it.item.id } }
    var searching by rememberSaveable { mutableStateOf(false) }
    var category by rememberSaveable { mutableStateOf(feed.category) }

    LaunchedEffect(grid, state.items.size) {
        snapshotFlow { grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0 }.collect { last ->
            if (state.items.isNotEmpty() && last >= state.items.size - PRELOAD_AHEAD) feed.loadMore()
        }
    }

    Column(Modifier.fillMaxSize().background(Palette.Page)) {
        Column(Modifier.background(Palette.Card).statusBarsPadding()) {
            Row(Modifier.fillMaxWidth().height(48.dp), verticalAlignment = Alignment.CenterVertically) {
                CategoryTabs(categories, category, Modifier.weight(1f)) {
                    category = it
                    feed.select(it)
                }
                Box(
                    Modifier.size(48.dp).clickable { searching = !searching },
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(if (searching) Icons.Filled.Close else Icons.Filled.Search, if (searching) "关闭搜索" else "搜索", tint = Palette.Text)
                }
            }
            if (searching) SearchField(feed.query) { feed.search(it) }
        }
        PullToRefreshBox(isRefreshing = state.refreshing, onRefresh = feed::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.items.isEmpty() && !state.answered -> Center(Expression.WAITING, "正在加载…")
                state.items.isEmpty() && state.error != null -> Center(Expression.WORRIED, state.error.orEmpty(), "点击重试", feed::retry)
                state.items.isEmpty() -> Center(Expression.CONFUSED, "没有找到壁纸，换个关键词或分类试试")
                else -> Waterfall(grid) {
                    items(state.items, key = { it.id }) { item ->
                        OnlineCard(item, item.id in favoriteIds, onFavorite = { app.toggleFavorite(item) }, onClick = { app.open(item) })
                    }
                    item(key = "footer", span = StaggeredGridItemSpan.FullLine) {
                        FeedFooter(state, feed::retry)
                    }
                }
            }
        }
    }
}

@Composable
private fun CategoryTabs(categories: List<Category>, selected: String, modifier: Modifier, onSelect: (String) -> Unit) {
    val tabs = listOf(Category("", "全部")) + categories
    Row(modifier.horizontalScroll(rememberScrollState()).padding(start = 8.dp), verticalAlignment = Alignment.CenterVertically) {
        for (tab in tabs) {
            val on = tab.id == selected
            Column(
                Modifier.clickable { onSelect(tab.id) }.padding(horizontal = 12.dp, vertical = 6.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(
                    tab.name,
                    color = if (on) Palette.Text else Palette.Sub,
                    fontSize = if (on) 17.sp else 16.sp,
                    fontWeight = if (on) FontWeight.Bold else FontWeight.Normal,
                    maxLines = 1,
                )
                Spacer(Modifier.height(3.dp))
                Box(Modifier.width(16.dp).height(3.dp).clip(RoundedCornerShape(2.dp)).background(if (on) Palette.Red else Color.Transparent))
            }
        }
    }
}

@Composable
private fun SearchField(initial: String, onSearch: (String) -> Unit) {
    var text by rememberSaveable { mutableStateOf(initial) }
    val keyboard = LocalSoftwareKeyboardController.current
    Row(
        Modifier.fillMaxWidth().padding(start = 12.dp, end = 12.dp, bottom = 8.dp).height(36.dp).clip(RoundedCornerShape(18.dp)).background(Palette.Field).padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(Icons.Filled.Search, null, Modifier.size(18.dp), tint = Palette.Sub)
        Spacer(Modifier.width(8.dp))
        Box(Modifier.weight(1f)) {
            if (text.isEmpty()) Text("搜索壁纸", color = Palette.Hint, fontSize = 14.sp)
            BasicTextField(
                value = text,
                onValueChange = { text = it },
                singleLine = true,
                textStyle = TextStyle(color = Palette.Text, fontSize = 14.sp),
                cursorBrush = SolidColor(Palette.Red),
                keyboardOptions = KeyboardOptions(imeAction = ImeAction.Search),
                keyboardActions = KeyboardActions(onSearch = {
                    onSearch(text)
                    keyboard?.hide()
                }),
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
private fun FeedFooter(state: FeedState, retry: () -> Unit) {
    Box(Modifier.fillMaxWidth().padding(vertical = 14.dp), contentAlignment = Alignment.Center) {
        when {
            state.loading -> CircularProgressIndicator(Modifier.size(20.dp), color = Palette.Red, strokeWidth = 2.dp)
            state.error != null -> Text("${state.error}，点击重试", Modifier.clickable(onClick = retry).padding(8.dp), color = Palette.Red, fontSize = 13.sp)
            !state.hasMore -> Text("到底啦", color = Palette.Hint, fontSize = 12.sp)
        }
    }
}
