package xyz.xiangban.xiaoban.common.webview

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.multiplatform.webview.web.WebView
import com.multiplatform.webview.web.WebViewNavigator
import com.multiplatform.webview.web.WebViewState

@Composable
actual fun FullscreenWebView(
    state: WebViewState,
    navigator: WebViewNavigator,
    modifier: Modifier,
    captureBackPresses: Boolean,
) {
    WebView(state = state, navigator = navigator, modifier = modifier, captureBackPresses = captureBackPresses)
}
