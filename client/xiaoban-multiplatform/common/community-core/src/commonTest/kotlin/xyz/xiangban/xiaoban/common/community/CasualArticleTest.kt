package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class CasualArticleTest {
    @Test fun categoryAncestryAndUntitledFeedLabelsRemainConsistent() {
        val note = Topic(1, "", excerpt = "记录今天的小事", category = CategoryRef(2, "living-note", "生活", "casual"), author = PublicUser("u", "作者"))
        assertTrue(note.isCasual)
        assertEquals("记录今天的小事", note.cardTitle)
        assertEquals("", note.title)
        assertEquals(false, note.copy(category = CategoryRef(2, "casual-moved", "教程", "learn")).isCasual)
        assertTrue(note.copy(category = CategoryRef(2, "casual-fun", "吃喝玩乐")).isCasual)
    }
    @Test fun galleryKeepsAllImagesInOrderAndBodyWithoutImageDuplicates() {
        val html = "<p>第一段</p><img src='/api/images/cover' alt='封面'>" +
            (1..5).joinToString("") { "<img src='/api/images/$it'>" } +
            "<p>第二段</p><img src='/api/images/cover'>"
        val article = casualArticle(html, "https://chat.xiangban.xyz")
        assertEquals(6, article.images.size)
        assertEquals("https://chat.xiangban.xyz/api/images/cover", article.images.first().src)
        assertEquals("https://chat.xiangban.xyz/api/images/5", article.images.last().src)
        assertEquals(listOf("第一段", "第二段"), article.paragraphs.map { it.text })
    }
    @Test fun unsafeImagesDoNotMoveIntoTheGalleryAndTextOnlyNotesStillWork() {
        val article = casualArticle("<p>文字生活</p><img src='javascript:alert(1)'><img src='data:image/png;base64,abc'>", "https://chat.xiangban.xyz")
        assertTrue(article.images.isEmpty())
        assertEquals("文字生活", article.paragraphs.single().text)
    }
}
