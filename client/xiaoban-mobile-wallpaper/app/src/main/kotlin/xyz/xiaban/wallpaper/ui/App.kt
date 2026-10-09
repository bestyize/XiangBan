package xyz.xiaban.wallpaper.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.staggeredgrid.rememberLazyStaggeredGridState
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.SiteItem

private val TAB_LABELS = listOf("壁纸", "视频", "收藏", "我的")

@Composable
fun App() {
    val app: AppViewModel = viewModel()
    val images: FeedViewModel = viewModel(key = "image", factory = viewModelFactory { initializer { FeedViewModel(app.site, MediaKind.IMAGE) } })
    val videos: FeedViewModel = viewModel(key = "video", factory = viewModelFactory { initializer { FeedViewModel(app.site, MediaKind.VIDEO) } })

    var tab by rememberSaveable { mutableIntStateOf(0) }
    val imageGrid = rememberLazyStaggeredGridState()
    val videoGrid = rememberLazyStaggeredGridState()
    val favoriteGrid = rememberLazyStaggeredGridState()
    val mineGrid = rememberLazyStaggeredGridState()

    val snackbar = remember { SnackbarHostState() }
    LaunchedEffect(app) {
        app.messages.collect {
            snackbar.currentSnackbarData?.dismiss()
            snackbar.showSnackbar(it)
        }
    }

    val detail by app.detail.collectAsState()
    val viewing by app.viewer.collectAsState()
    // Keep showing the last page while it slides out, after the selection has already been cleared.
    var lastDetail by remember { mutableStateOf<SiteItem?>(null) }
    if (detail != null) lastDetail = detail

    Box(Modifier.fillMaxSize()) {
        Scaffold(
            containerColor = Palette.Page,
            bottomBar = { TabBar(tab) { tab = it } },
        ) { inner ->
            Box(Modifier.fillMaxSize().padding(bottom = inner.calculateBottomPadding())) {
                when (tab) {
                    0 -> FeedScreen(images, app, imageGrid)
                    1 -> FeedScreen(videos, app, videoGrid)
                    2 -> FavoritesScreen(app, favoriteGrid)
                    else -> MineScreen(app, mineGrid)
                }
            }
        }
        AnimatedVisibility(
            visible = detail != null,
            enter = slideInHorizontally { it / 3 } + fadeIn(),
            exit = slideOutHorizontally { it / 3 } + fadeOut(),
        ) {
            lastDetail?.let { DetailScreen(it, app) }
        }
        AnimatedVisibility(visible = viewing != null, enter = fadeIn(), exit = fadeOut()) {
            viewing?.let { Viewer(it, onClose = app::closeViewer) }
        }
        SnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter).navigationBarsPadding().padding(bottom = 76.dp))
    }
}

@Composable
private fun TabBar(selected: Int, onSelect: (Int) -> Unit) {
    Column(Modifier.fillMaxWidth().background(Palette.Card).navigationBarsPadding()) {
        Box(Modifier.fillMaxWidth().height(0.5.dp).background(Palette.Line))
        Row(Modifier.fillMaxWidth().height(52.dp)) {
            TAB_LABELS.forEachIndexed { index, label ->
                val on = index == selected
                Box(
                    Modifier.weight(1f).fillMaxHeight().clickable(
                        interactionSource = remember { MutableInteractionSource() },
                        indication = null,
                    ) { onSelect(index) },
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        label,
                        color = if (on) Palette.Text else Palette.Sub,
                        fontSize = 16.sp,
                        fontWeight = if (on) FontWeight.Bold else FontWeight.Normal,
                    )
                }
            }
        }
    }
}
