package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.multiplatform.webview.request.RequestInterceptor
import com.multiplatform.webview.request.WebRequest
import com.multiplatform.webview.request.WebRequestInterceptResult
import com.multiplatform.webview.web.LoadingState
import com.multiplatform.webview.web.WebContent
import com.multiplatform.webview.web.WebView
import com.multiplatform.webview.web.WebViewNavigator
import com.multiplatform.webview.web.WebViewState
import com.multiplatform.webview.web.rememberWebViewNavigator
import io.ktor.http.Url

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun EmbeddedWebPage(url: String, close: () -> Unit, platform: PlatformActions?) {
    val state = remember(url) {
        WebViewState(WebContent.Url(url)).apply {
            webSettings.isJavaScriptEnabled = true
            webSettings.androidWebSettings.domStorageEnabled = true
        }
    }
    val navigator = rememberWebViewNavigator(requestInterceptor = WebPageInterceptor)
    var menuOpen by remember(url) { mutableStateOf(false) }
    val currentUrl = state.lastLoadedUrl?.let(::webPageRoute)?.url ?: url
    val title = state.pageTitle?.takeIf { it.isNotBlank() } ?: Url(currentUrl).host

    Column(Modifier.fillMaxSize()) {
        ScreenBar(title, onBack = { if (navigator.canGoBack) navigator.navigateBack() else close() }, trailing = {
            if (platform != null) IconTap(XIcons.More, "网页菜单", { menuOpen = true })
        })
        Box(Modifier.weight(1f).fillMaxWidth()) {
            WebView(state = state, navigator = navigator, modifier = Modifier.fillMaxSize())
            when (val loading = state.loadingState) {
                is LoadingState.Loading -> LinearProgressIndicator(progress = { loading.progress }, modifier = Modifier.fillMaxWidth())
                LoadingState.Initializing -> LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
                LoadingState.Finished -> Unit
            }
            if (state.errorsForCurrentRequest.any { it.isFromMainFrame }) {
                CenterMessage("网页加载失败", "重试", { navigator.reload() }, Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface))
            }
        }
    }

    if (menuOpen && platform != null) {
        ModalBottomSheet(
            onDismissRequest = { menuOpen = false },
            containerColor = MaterialTheme.colorScheme.surface.copy(alpha = 0.94f),
            scrimColor = Color.Black.copy(alpha = 0.32f),
        ) {
            val colors = ListItemDefaults.colors(containerColor = Color.Transparent)
            Column(Modifier.padding(bottom = 12.dp)) {
                ListItem(
                    headlineContent = { Text("从浏览器打开") },
                    leadingContent = { XIcon(XIcons.ExternalLink) },
                    modifier = Modifier.clickable { menuOpen = false; platform.openInBrowser(currentUrl) },
                    colors = colors,
                )
                ListItem(
                    headlineContent = { Text("分享") },
                    leadingContent = { XIcon(XIcons.Share) },
                    modifier = Modifier.clickable { menuOpen = false; platform.share(title, currentUrl) },
                    colors = colors,
                )
            }
        }
    }
}

private object WebPageInterceptor : RequestInterceptor {
    override fun onInterceptUrlRequest(request: WebRequest, navigator: WebViewNavigator): WebRequestInterceptResult =
        if (!request.isForMainFrame || webPageRoute(request.url) != null) WebRequestInterceptResult.Allow
        else WebRequestInterceptResult.Reject
}
