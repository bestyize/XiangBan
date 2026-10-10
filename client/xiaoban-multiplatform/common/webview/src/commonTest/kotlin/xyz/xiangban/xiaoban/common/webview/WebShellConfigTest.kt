package xyz.xiangban.xiaoban.common.webview

import androidx.compose.ui.graphics.Color
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class WebShellConfigTest {
    private val palette = WebShellPalette(Color.White, Color.Black, Color.Red)
    private val config = WebShellConfig("https://xiaban.xyz", setOf("xiaban.xyz"), palette, palette)

    @Test
    fun `site pages and subdomains stay in the shell`() {
        assertTrue(config.keepsInShell("https://xiaban.xyz/"))
        assertTrue(config.keepsInShell("https://xiaban.xyz/games/snake-arena?from=card#top"))
        assertTrue(config.keepsInShell("https://m.XIABAN.xyz/x"))
        assertTrue(config.keepsInShell("http://xiaban.xyz:8080/x"))
    }

    @Test
    fun `other sites and non web schemes leave the shell`() {
        assertFalse(config.keepsInShell("https://example.com/"))
        assertFalse(config.keepsInShell("mailto:me@xiaban.xyz"))
        assertFalse(config.keepsInShell("tel:123"))
        assertFalse(config.keepsInShell("intent://xiaban.xyz/#Intent;end"))
    }

    @Test
    fun `lookalike hosts do not count as the site`() {
        assertFalse(config.keepsInShell("https://evilxiaban.xyz/"))
        assertFalse(config.keepsInShell("https://xiaban.xyz.evil.example/"))
        assertFalse(config.keepsInShell("https://xiaban.xyz@evil.example/"))
        assertFalse(config.keepsInShell("https://evil.example/?next=https://xiaban.xyz/"))
    }
}
