package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import com.multiplatform.webview.request.RequestInterceptor
import com.multiplatform.webview.request.WebRequest
import com.multiplatform.webview.request.WebRequestInterceptResult
import com.multiplatform.webview.web.LoadingState
import com.multiplatform.webview.web.WebContent
import com.multiplatform.webview.web.WebViewNavigator
import com.multiplatform.webview.web.WebViewState
import com.multiplatform.webview.web.rememberWebViewNavigator
import xyz.xiangban.xiaoban.common.webview.FullscreenWebView

@Composable
internal fun MicroGalleryTab(forumBase: String, open: (Route) -> Unit) {
    LazyVerticalGrid(
        GridCells.Fixed(2),
        Modifier.fillMaxSize(),
        contentPadding = FeedPadding,
        verticalArrangement = Arrangement.spacedBy(10.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        items(
            MicroGalleryExhibit.entries,
            key = { it.id },
            span = { GridItemSpan(if (it == MicroGalleryExhibit.Aquarium) maxLineSpan else 1) },
        ) { exhibit ->
            Column(
                Modifier.fillMaxWidth().outlinedCard(12.dp)
                    .clickable { open(Route.GalleryExhibit(exhibit.id)) },
            ) {
                CommunityCover(exhibit.coverUrl(forumBase))
                Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(exhibit.title, style = XType.Section, fontWeight = FontWeight.SemiBold)
                    Text(exhibit.description, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

@Composable
internal fun GalleryExhibitScreen(forumBase: String, exhibit: MicroGalleryExhibit, back: () -> Unit, open: (Route) -> Unit) {
    var loadAttempt by remember(exhibit) { mutableIntStateOf(0) }
    var showPhoto by rememberSaveable(exhibit.id) { mutableStateOf(false) }

    Column(Modifier.fillMaxSize()) {
        ScreenBar(exhibit.title, onBack = back, trailing = {
            LinkText(if (showPhoto) "试用 3D" else "看图片", {
                showPhoto = !showPhoto
                if (!showPhoto) loadAttempt++
            }, Modifier.padding(end = 8.dp))
        })
        Box(Modifier.weight(1f).fillMaxWidth()) {
            if (showPhoto) {
                GalleryExhibitPhoto(forumBase, exhibit, "图片预览")
            } else {
                key(exhibit, loadAttempt) {
                    GalleryExhibitPlayer(
                        url = exhibit.previewUrl(forumBase),
                        cover = { message ->
                            GalleryExhibitPhoto(forumBase, exhibit, message) { loadAttempt++ }
                        },
                        back = back,
                        openAquarium = { open(Route.GalleryExhibit(MicroGalleryExhibit.Aquarium.id)) },
                    )
                }
            }
        }
    }
}

@Composable
private fun GalleryExhibitPhoto(
    forumBase: String,
    exhibit: MicroGalleryExhibit,
    message: String,
    retry: (() -> Unit)? = null,
) {
    val url = exhibit.coverUrl(forumBase)
    var failed by remember(url) { mutableStateOf(false) }
    Column(
        Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background).padding(16.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        if (!failed) {
            AsyncImage(
                model = url,
                contentDescription = exhibit.title,
                imageLoader = rememberImageLoader(),
                contentScale = ContentScale.Fit,
                modifier = Modifier.weight(1f, fill = false).fillMaxWidth(),
                onError = { failed = true },
            )
        }
        Text(
            if (failed) "展品图片暂时无法加载" else message,
            modifier = Modifier.padding(top = 12.dp),
            style = XType.Caption,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
        )
        if (retry != null) {
            GhostPillButton("重试 3D", retry, Modifier.padding(top = 8.dp), height = 44.dp)
        }
    }
}

@Composable
private fun GalleryExhibitPlayer(
    url: String,
    cover: @Composable (String) -> Unit,
    back: () -> Unit,
    openAquarium: () -> Unit,
) {
    val state = remember(url) {
        WebViewState(WebContent.Url(url)).apply { webSettings.isJavaScriptEnabled = true }
    }
    var failure by remember(url) { mutableStateOf<String?>(null) }
    val loaded = state.loadingState == LoadingState.Finished
    val mainError = state.errorsForCurrentRequest.any { it.isFromMainFrame }
    val currentUrl = state.lastLoadedUrl
    val onBack by rememberUpdatedState(back)
    val onAquarium by rememberUpdatedState(openAquarium)
    val navigator = rememberWebViewNavigator(requestInterceptor = remember(url) {
        GalleryPreviewInterceptor(url) { action ->
            if (state.lastLoadedUrl == url) {
                when (action) {
                    "gallery:back" -> onBack()
                    "gallery:aquarium" -> onAquarium()
                    "gallery:unavailable" -> failure = "当前无法显示 3D 预览，已切换为展品图片。"
                }
            }
        }
    })

    LaunchedEffect(loaded, mainError, currentUrl) {
        if (!loaded || mainError) return@LaunchedEffect
        if (currentUrl != url) {
            failure = "展品预览加载失败"
            return@LaunchedEffect
        }
        navigator.evaluateJavaScript("""
            (() => {
                const status = document.querySelector('#status, #preview-status');
                if (!status) return 'unavailable';
                const report = action => { location.href = '$galleryActionPrefix' + action; };
                window.openCommunityExhibit = report;
                const failed = () => status.dataset.previewFailed === 'true' || status.textContent.trim().startsWith('无法');
                if (failed()) return 'failed';
                const observer = new MutationObserver(() => {
                    if (!failed()) return;
                    observer.disconnect();
                    report('gallery:unavailable');
                });
                observer.observe(status, { attributes: true, childList: true, characterData: true, subtree: true });
                return 'preview';
            })();
        """.trimIndent()) { result ->
            when (result.trim('"')) {
                "failed" -> failure = "当前无法显示 3D 预览，已切换为展品图片。"
                "unavailable" -> failure = "展品预览加载失败"
            }
        }
    }

    val message = failure ?: if (mainError) "展品预览加载失败" else null
    if (message != null) {
        cover(message)
    } else {
        Box(Modifier.fillMaxSize()) {
            FullscreenWebView(
                state = state,
                modifier = Modifier.fillMaxSize(),
                captureBackPresses = false,
                navigator = navigator,
            )
            when (val loading = state.loadingState) {
                LoadingState.Initializing -> LinearProgressIndicator(Modifier.fillMaxWidth())
                is LoadingState.Loading -> LinearProgressIndicator({ loading.progress }, Modifier.fillMaxWidth())
                LoadingState.Finished -> Unit
            }
        }
    }
}

// Cancel these requests inside the WebView and send the action to the native navigation stack.
private const val galleryActionPrefix = "https://gallery.xiaoban.invalid/"

private class GalleryPreviewInterceptor(
    private val url: String,
    private val onAction: (String) -> Unit,
) : RequestInterceptor {
    override fun onInterceptUrlRequest(request: WebRequest, navigator: WebViewNavigator): WebRequestInterceptResult {
        if (request.url == url) return WebRequestInterceptResult.Allow
        if (request.isForMainFrame && request.url.startsWith(galleryActionPrefix)) {
            onAction(request.url.removePrefix(galleryActionPrefix))
        }
        return WebRequestInterceptResult.Reject
    }
}
