package xyz.xiangban.xiaoban.business.community

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import xyz.xiangban.xiaoban.common.webview.WebShell
import xyz.xiangban.xiaoban.common.webview.WebShellConfig
import xyz.xiangban.xiaoban.common.webview.WebShellPalette

/**
 * The community front end (chat.xiangban.xyz) in a native frame: read-only browsing of 周刊, 资源, 讨论 and 项目.
 * The page has its own mobile tab bar and share actions (copy link), so the shell adds nothing of its own yet.
 * Colors are the site's own tokens (services/server-forum/web/src/styles/base.css): brand red on black, white and gray.
 */
val CommunityWebConfig = WebShellConfig(
    homeUrl = "https://chat.xiangban.xyz",
    internalHosts = setOf("chat.xiangban.xyz"),
    light = WebShellPalette(background = Color(0xFFF5F5F5), foreground = Color(0xFF333333), accent = Color(0xFFFF2442)),
    dark = WebShellPalette(background = Color(0xFF111111), foreground = Color(0xFFF5F5F5), accent = Color(0xFFFF2442)),
)

@Composable
fun CommunityApp(modifier: Modifier = Modifier) = WebShell(CommunityWebConfig, modifier)
