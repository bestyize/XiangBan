package xyz.xiangban.xiaoban.common.webview

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.multiplatform.webview.web.WebViewNavigator
import com.multiplatform.webview.web.WebViewState

@Composable
expect fun FullscreenWebView(
    state: WebViewState,
    navigator: WebViewNavigator,
    modifier: Modifier = Modifier,
    captureBackPresses: Boolean = true,
)
