package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
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
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.Orientation

private const val PRELOAD_AHEAD = 6

/** Pictures or videos from the website: category tabs and search on top, a two-column waterfall below. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FeedScreen(feed: FeedViewModel, app: AppViewModel, grid: LazyListState) {
    val state by feed.state.collectAsState()
    val categories by app.categories.collectAsState()
    val favorites by app.favorites.collectAsState()
    val favoriteIds = remember(favorites) { favorites.mapTo(HashSet()) { it.item.id } }
    var searching by rememberSaveable { mutableStateOf(false) }
    var category by rememberSaveable { mutableStateOf(feed.category) }
    var orientation by rememberSaveable { mutableStateOf(feed.orientation) }

    LaunchedEffect(grid, state.items.size) {
        snapshotFlow { grid.layoutInfo.visibleItemsInfo.lastOrNull()?.index ?: 0 }.collect { last ->
            if (state.items.isNotEmpty() && last >= state.items.size - PRELOAD_AHEAD) feed.loadMore()
        }
    }

    Column(Modifier.fillMaxSize().background(Palette.Page)) {
        Column(Modifier.background(Palette.Page).statusBarsPadding()) {
            Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 16.dp, top = 8.dp, bottom = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text(
                        if (feed.kind == MediaKind.VIDEO) "视频" else "小伴壁纸",
                        color = Palette.Text, fontSize = 24.sp, fontWeight = FontWeight.Bold,
                    )
                    Text(
                        if (feed.kind == MediaKind.VIDEO) "挑一段会动的风景" else "今天想换哪一张",
                        color = Palette.Sub, fontSize = 13.sp,
                    )
                }
                OrientationSwitch(orientation) {
                    orientation = it
                    feed.selectOrientation(it)
                }
                Spacer(Modifier.width(8.dp))
                Box(
                    Modifier.size(44.dp).clip(CircleShape).background(Palette.Card).border(1.dp, Palette.Line, CircleShape)
                        .clickable { searching = !searching },
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(if (searching) Icons.Filled.Close else Icons.Filled.Search, if (searching) "关闭搜索" else "搜索", tint = Palette.Text)
                }
            }
            if (searching) SearchField(feed.query) { feed.search(it) }
            CategoryTabs(categories, category, Modifier.fillMaxWidth()) {
                category = it
                feed.select(it)
            }
            Spacer(Modifier.height(8.dp))
        }
        PullToRefreshBox(isRefreshing = state.refreshing, onRefresh = feed::refresh, modifier = Modifier.fillMaxSize()) {
            when {
                state.items.isEmpty() && !state.answered -> Center(Expression.WAITING, "正在加载…")
                state.items.isEmpty() && state.error != null -> Center(Expression.WORRIED, state.error.orEmpty(), "点击重试", feed::retry)
                state.items.isEmpty() -> Center(Expression.CONFUSED, "没有找到符合条件的内容，换个关键词、分类或方向试试")
                else -> FeedList(grid) {
                    items(state.items, key = { it.id }) { item ->
                        OnlineCard(item, item.id in favoriteIds, onFavorite = { app.toggleFavorite(item) }, onClick = { app.open(item) })
                    }
                    item(key = "footer") {
                        FeedFooter(state, feed::retry)
                    }
                }
            }
        }
    }
}

/** 全部 / 竖版 / 横版: a small segmented control; the site does the filtering. */
@Composable
private fun OrientationSwitch(selected: Orientation, onSelect: (Orientation) -> Unit) {
    val shape = RoundedCornerShape(18.dp)
    Row(
        Modifier.height(36.dp).clip(shape).background(Palette.Card).border(1.dp, Palette.Line, shape).padding(3.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        for (option in Orientation.entries) {
            val on = option == selected
            Box(
                Modifier.fillMaxHeight().clip(RoundedCornerShape(15.dp))
                    .then(if (on) Modifier.background(Palette.Brand) else Modifier)
                    .clickable { onSelect(option) }
                    .padding(horizontal = 10.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    option.label,
                    color = if (on) Color.White else Palette.Sub,
                    fontSize = 13.sp,
                    fontWeight = if (on) FontWeight.Bold else FontWeight.Normal,
                    maxLines = 1,
                )
            }
        }
    }
}

@Composable
private fun CategoryTabs(categories: List<Category>, selected: String, modifier: Modifier, onSelect: (String) -> Unit) {
    val tabs = listOf(Category("", "全部")) + categories
    Row(
        modifier.horizontalScroll(rememberScrollState()).padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        for (tab in tabs) {
            val on = tab.id == selected
            Box(
                Modifier.height(32.dp).clip(RoundedCornerShape(16.dp))
                    .then(
                        if (on) Modifier.background(Palette.Brand)
                        else Modifier.background(Palette.Card).border(1.dp, Palette.Line, RoundedCornerShape(16.dp)),
                    )
                    .clickable { onSelect(tab.id) }
                    .padding(horizontal = 14.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    tab.name,
                    color = if (on) Color.White else Palette.Sub,
                    fontSize = 14.sp,
                    fontWeight = if (on) FontWeight.Bold else FontWeight.Normal,
                    maxLines = 1,
                )
            }
        }
    }
}

@Composable
private fun SearchField(initial: String, onSearch: (String) -> Unit) {
    var text by rememberSaveable { mutableStateOf(initial) }
    val keyboard = LocalSoftwareKeyboardController.current
    Row(
        Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, bottom = 10.dp).height(40.dp).clip(RoundedCornerShape(20.dp)).background(Palette.Card).border(1.dp, Palette.Line, RoundedCornerShape(20.dp)).padding(horizontal = 14.dp),
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
                cursorBrush = SolidColor(Palette.Brand),
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
            state.loading -> CircularProgressIndicator(Modifier.size(20.dp), color = Palette.Brand, strokeWidth = 2.dp)
            state.error != null -> Text("${state.error}，点击重试", Modifier.clickable(onClick = retry).padding(8.dp), color = Palette.Brand, fontSize = 13.sp)
            !state.hasMore -> Text("到底啦", color = Palette.Hint, fontSize = 12.sp)
        }
    }
}
