package xyz.xiangban.xiaoban.common.share

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class ShareModelsTest {
    @Test
    fun railOrderingKeepsDeleteLastAndExcludesReport() {
        val extras = listOf(
            ShareExtraAction("copyFull", "复制全文", ShareIconKey.CopyText) { "已复制" },
        )
        val authorItems = buildShareRailItems(
            systemShareEnabled = true,
            copied = false,
            extraActions = extras,
            deleteAction = ShareDeleteAction { "已删除" },
        )
        assertEquals(
            listOf("system", "copyLink", "openCard", "extra:copyFull", "browser", "delete"),
            authorItems.map { it.id },
        )
        assertEquals(ShareTileTone.Primary, authorItems.first().tone)
        assertEquals(ShareTileTone.Danger, authorItems.last().tone)
        assertTrue(authorItems.none { it.label.contains("举报") })

        val visitorItems = buildShareRailItems(
            systemShareEnabled = true,
            copied = true,
            extraActions = extras,
            deleteAction = null,
        )
        assertEquals(
            listOf("system", "copyLink", "openCard", "extra:copyFull", "browser"),
            visitorItems.map { it.id },
        )
        assertEquals("已复制", visitorItems[1].label)
        assertEquals(ShareTileTone.Copied, visitorItems[1].tone)
    }

    @Test
    fun displayUrlAndMetaFormatting() {
        assertEquals("chat.xiangban.xyz/t/128", shareDisplayUrl("https://chat.xiangban.xyz/t/128/"))
        val content = ShareContent(
            title = "小伴社区刚刚起步",
            url = "https://chat.xiangban.xyz/t/128",
            author = "read",
        )
        assertEquals("read · 公开链接 · 浏览器免登录可看", content.displayMeta)
        assertEquals("chat.xiangban.xyz/t/128", content.displayUrl)
    }
}
