package xyz.xiangban.xiaoban.business.xiaban

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import xyz.xiangban.xiaoban.common.webview.WebShell
import xyz.xiangban.xiaoban.common.webview.WebShellConfig
import xyz.xiangban.xiaoban.common.webview.WebShellPalette

/** xiaban.xyz in a native frame. Colors are the site's own light and dark tokens. */
val XiabanWebConfig = WebShellConfig(
    homeUrl = "https://xiaban.xyz",
    internalHosts = setOf("xiaban.xyz"),
    light = WebShellPalette(background = Color(0xFFF7F7F7), foreground = Color(0xFF262626), accent = Color(0xFFE51B3E)),
    dark = WebShellPalette(background = Color(0xFF141414), foreground = Color(0xFFF2F2F2), accent = Color(0xFFFF6B81)),
)

@Composable
fun XiabanApp(modifier: Modifier = Modifier) = WebShell(XiabanWebConfig, modifier)
