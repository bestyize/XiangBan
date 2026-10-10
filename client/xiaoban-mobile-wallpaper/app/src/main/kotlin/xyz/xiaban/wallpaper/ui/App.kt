package xyz.xiaban.wallpaper.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.lazy.staggeredgrid.rememberLazyStaggeredGridState
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.LocalContentColor
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
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
private val TAB_GLYPHS = listOf(Glyph.Image, Glyph.Video, Glyph.Heart, Glyph.User)

@Composable
fun App(themeMode: ThemeMode, onThemeMode: (ThemeMode) -> Unit) {
    val app: AppViewModel = viewModel()
    val images: FeedViewModel = viewModel(key = "image", factory = viewModelFactory { initializer { FeedViewModel(app.site, MediaKind.IMAGE) } })
    val videos: FeedViewModel = viewModel(key = "video", factory = viewModelFactory { initializer { FeedViewModel(app.site, MediaKind.VIDEO) } })

    var tab by rememberSaveable { mutableIntStateOf(0) }
    var settingsOpen by rememberSaveable { mutableStateOf(false) }
    val imageGrid = rememberLazyListState()
    val videoGrid = rememberLazyListState()
    val favoriteGrid = rememberLazyListState()
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
    val loginOpen by app.loginOpen.collectAsState()
    val notice by app.announcement.collectAsState()
    // Keep showing the last page while it slides out, after the selection has already been cleared.
    var lastDetail by remember { mutableStateOf<SiteItem?>(null) }
    if (detail != null) lastDetail = detail

    CompositionLocalProvider(LocalContentColor provides Palette.Text) {
    Box(Modifier.fillMaxSize()) {
        Box(Modifier.fillMaxSize().background(Palette.Page)) {
            when (tab) {
                0 -> FeedScreen(images, app, imageGrid)
                1 -> FeedScreen(videos, app, videoGrid)
                2 -> FavoritesScreen(app, favoriteGrid)
                else -> MineScreen(app, mineGrid, onOpenSettings = { settingsOpen = true })
            }
            Box(Modifier.align(Alignment.BottomCenter)) { TabBar(tab) { tab = it } }
        }
        AnimatedVisibility(
            visible = settingsOpen,
            enter = slideInHorizontally { it / 3 } + fadeIn(),
            exit = slideOutHorizontally { it / 3 } + fadeOut(),
        ) {
            SettingsScreen(app, themeMode, onThemeMode, onClose = { settingsOpen = false })
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
        SnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter).navigationBarsPadding().padding(bottom = 104.dp))
        if (loginOpen) LoginDialog(app, onDismiss = app::closeLogin)
        notice?.let {
            // Keyed so a second notice starts at the top of its text instead of inheriting the first one's scroll.
            key(it.id) { AnnouncementDialog(it, onConfirm = rememberAnnouncementConfirm(app, it), onCancel = app::dismissAnnouncement) }
        }
    }
    }
}

@Composable
private fun TabBar(selected: Int, onSelect: (Int) -> Unit) {
    val shape = RoundedCornerShape(32.dp)
    Box(Modifier.fillMaxWidth().navigationBarsPadding().padding(start = 24.dp, end = 24.dp, top = 6.dp, bottom = 12.dp)) {
        Row(
            Modifier.fillMaxWidth().height(64.dp)
                .shadow(18.dp, shape, ambientColor = Palette.Float, spotColor = Palette.Float)
                .background(Palette.Card, shape)
                .padding(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            TAB_LABELS.forEachIndexed { index, label ->
                val on = index == selected
                val weight by animateFloatAsState(if (on) 1.9f else 1f, label = "tabWeight")
                Box(
                    Modifier.weight(weight).fillMaxHeight().clip(CircleShape)
                        .then(if (on) Modifier.background(Palette.Brand) else Modifier)
                        .clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) { onSelect(index) },
                    contentAlignment = Alignment.Center,
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        GlyphIcon(TAB_GLYPHS[index], if (on) Color.White else Palette.Sub)
                        if (on) {
                            Spacer(Modifier.width(6.dp))
                            Text(label, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Bold, maxLines = 1)
                        }
                    }
                }
            }
        }
    }
}
