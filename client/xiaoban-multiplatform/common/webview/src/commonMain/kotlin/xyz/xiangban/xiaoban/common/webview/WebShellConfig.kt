package xyz.xiangban.xiaoban.common.webview

import androidx.compose.ui.graphics.Color

/** Colors the shell draws itself (system-bar insets, error and fallback pages); match the page so they look like part of it. */
data class WebShellPalette(val background: Color, val foreground: Color, val accent: Color)

/** What a web shell loads and how it frames the page. */
data class WebShellConfig(
    val homeUrl: String,
    /** Pages on these hosts (and their subdomains) stay in the shell; every other link opens in the system browser. */
    val internalHosts: Set<String>,
    val light: WebShellPalette,
    val dark: WebShellPalette,
)

internal fun WebShellConfig.keepsInShell(url: String): Boolean {
    val scheme = url.substringBefore(':', "").lowercase()
    if (scheme != "https" && scheme != "http") return false
    val host = url.substringAfter("://").substringBefore('/').substringBefore('?').substringBefore('#')
        .substringAfterLast('@').substringBefore(':').lowercase()
    return internalHosts.any { host == it || host.endsWith(".$it") }
}
