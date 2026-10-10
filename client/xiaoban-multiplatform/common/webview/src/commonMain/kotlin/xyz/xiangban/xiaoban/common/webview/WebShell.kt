package xyz.xiangban.xiaoban.common.webview

import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
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

/** Shows a website full screen: progress bar, retry on load failure, back navigation, external links in the system browser. */
@Composable
fun WebShell(config: WebShellConfig, modifier: Modifier = Modifier) {
    val palette = if (isSystemInDarkTheme()) config.dark else config.light
    val uriHandler = LocalUriHandler.current
    val state = remember(config) {
        WebViewState(WebContent.Url(config.homeUrl)).also {
            it.webSettings.isJavaScriptEnabled = true
            it.webSettings.backgroundColor = palette.background
            // The site keeps its login token in localStorage.
            it.webSettings.androidWebSettings.domStorageEnabled = true
        }
    }
    val navigator = rememberWebViewNavigator(
        requestInterceptor = remember(config, uriHandler) { ExternalLinkInterceptor(config) { uriHandler.openUri(it) } },
    )

    Box(modifier.fillMaxSize().background(palette.background).safeDrawingPadding()) {
        WebView(state = state, navigator = navigator, modifier = Modifier.fillMaxSize())
        when (val loading = state.loadingState) {
            is LoadingState.Loading -> LinearProgressIndicator(progress = { loading.progress }, modifier = Modifier.fillMaxWidth())
            LoadingState.Initializing -> LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
            LoadingState.Finished -> Unit
        }
        if (state.errorsForCurrentRequest.any { it.isFromMainFrame }) LoadFailed(palette, navigator)
    }
}

@Composable
private fun LoadFailed(palette: WebShellPalette, navigator: WebViewNavigator) {
    Column(
        Modifier.fillMaxSize().background(palette.background).padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text("连不上相伴，检查一下网络再试", style = MaterialTheme.typography.bodyLarge, color = palette.foreground)
        Button(
            onClick = { navigator.reload() },
            colors = ButtonDefaults.buttonColors(containerColor = palette.accent, contentColor = palette.background),
            modifier = Modifier.padding(top = 16.dp),
        ) { Text("重试") }
    }
}

private class ExternalLinkInterceptor(
    private val config: WebShellConfig,
    private val openExternally: (String) -> Unit,
) : RequestInterceptor {
    override fun onInterceptUrlRequest(request: WebRequest, navigator: WebViewNavigator): WebRequestInterceptResult {
        if (!request.isForMainFrame || config.keepsInShell(request.url)) return WebRequestInterceptResult.Allow
        openExternally(request.url)
        return WebRequestInterceptResult.Reject
    }
}
