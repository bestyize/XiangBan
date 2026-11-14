package xyz.xiangban.xiaoban.common.webview

import android.app.Activity
import android.graphics.Color
import android.view.View
import android.view.ViewGroup
import android.webkit.WebChromeClient.CustomViewCallback
import android.widget.FrameLayout
import androidx.activity.compose.BackHandler
import androidx.activity.compose.LocalActivity
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.multiplatform.webview.web.AccompanistWebChromeClient
import com.multiplatform.webview.web.PlatformWebViewParams
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
    val activity = LocalActivity.current
    val chromeClient = remember(activity) { activity?.let(::FullscreenWebChromeClient) }
    val params = remember(chromeClient) { chromeClient?.let { PlatformWebViewParams(chromeClient = it) } }
    WebView(
        state = state,
        navigator = navigator,
        modifier = modifier,
        captureBackPresses = captureBackPresses,
        platformWebViewParams = params,
    )
    if (chromeClient != null) {
        // Register after the WebView so Back exits fullscreen before navigating web history.
        BackHandler(enabled = chromeClient.isFullscreen, onBack = chromeClient::requestExit)
        DisposableEffect(chromeClient) {
            onDispose { chromeClient.dispose() }
        }
    }
}

private class FullscreenWebChromeClient(private val activity: Activity) : AccompanistWebChromeClient() {
    private class FullscreenView(
        val overlay: FrameLayout,
        val callback: CustomViewCallback,
        val visibleBars: Int,
        val barsBehavior: Int,
    )

    private val statusBar = WindowInsetsCompat.Type.statusBars()
    private val navigationBar = WindowInsetsCompat.Type.navigationBars()
    private val systemBars = statusBar or navigationBar
    private var fullscreen by mutableStateOf<FullscreenView?>(null)
    private var disposed = false
    val isFullscreen get() = fullscreen != null

    override fun onShowCustomView(view: View, callback: CustomViewCallback) {
        if (disposed || isFullscreen || activity.isFinishing || activity.isDestroyed) {
            callback.onCustomViewHidden()
            return
        }

        val window = activity.window
        val controller = WindowCompat.getInsetsController(window, window.decorView)
        val insets = ViewCompat.getRootWindowInsets(window.decorView)
        val visibleBars = (if (insets?.isVisible(statusBar) != false) statusBar else 0) or
            (if (insets?.isVisible(navigationBar) != false) navigationBar else 0)
        val overlay = FrameLayout(activity).apply {
            setBackgroundColor(Color.BLACK)
            isClickable = true
            addView(view, FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT))
        }
        fullscreen = FullscreenView(overlay, callback, visibleBars, controller.systemBarsBehavior)
        // Chromium draws fullscreen into the supplied view while the original WebView stays attached.
        window.addContentView(overlay, ViewGroup.LayoutParams(MATCH_PARENT, MATCH_PARENT))
        controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        controller.hide(systemBars or WindowInsetsCompat.Type.ime())
    }

    override fun onHideCustomView() {
        val view = fullscreen ?: return
        fullscreen = null
        (view.overlay.parent as? ViewGroup)?.removeView(view.overlay)
        view.overlay.removeAllViews()

        val window = activity.window
        val controller = WindowCompat.getInsetsController(window, window.decorView)
        controller.systemBarsBehavior = view.barsBehavior
        controller.show(view.visibleBars)
        controller.hide(systemBars and view.visibleBars.inv())
    }

    fun requestExit() {
        fullscreen?.callback?.onCustomViewHidden()
    }

    fun dispose() {
        disposed = true
        requestExit()
        onHideCustomView()
    }

    private companion object {
        const val MATCH_PARENT = ViewGroup.LayoutParams.MATCH_PARENT
    }
}
