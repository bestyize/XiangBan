package xyz.xiangban.xiaoban.common.community

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlinx.coroutines.test.runTest

class TopicDraftsTest {
    @Test
    fun restoresCoverAndPublishingReferencesInTheSameAccountAfterReopening() = runTest {
        val disk = InMemoryChatHistoryFiles()
        val cover = ForumUploadFile("cover.webp", "image/webp", byteArrayOf(1, 2, 3))
        val draft = TopicDraft(
            title = "初稿", body = "稍后继续编辑", section = 1, boardKey = "casual-fun", tags = listOf("话题"),
            attachments = listOf(ForumAttachment("attachment", "附件", 2, price = 3)),
            pollOptions = listOf("选项一", "选项二"),
            cover = TopicDraftCover("cover-1", cover.name, cover.mime,
                template = TopicDraftCoverText("封面", "副标题", "Memo", true, "封面"),
                uploaded = ForumImage("image", "/api/images/image", 3)),
            uploadedWork = TopicDraftUpload("files/upload.zip", "game", "作品", "介绍", "work.zip", 12, true),
            requestId = "f62a574a-7c95-4c2d-b177-b9f3b2ba956a",
        )
        TopicDrafts(disk).save("account-a", "new", draft, cover)
        val restarted = TopicDrafts(disk)
        val restored = requireNotNull(restarted.load("account-a", "new"))
        assertEquals(draft, restored.first)
        assertContentEquals(cover.bytes, restored.second?.bytes)
        assertNull(restarted.load("account-b", "new"))
        assertNull(restarted.load("account-a", "help"))

        val registered = draft.copy(uploadedWork = null, linkedWork = TopicDraftLink(7, "作品", "游戏"))
        restarted.save("account-a", "new", registered)
        assertEquals(registered, TopicDrafts(disk).load("account-a", "new")?.first)
        assertContentEquals(cover.bytes, TopicDrafts(disk).load("account-a", "new")?.second?.bytes)

        val other = TopicDraft(title = "另一个账号的草稿")
        restarted.save("account-b", "new", other)
        restarted.clear("account-a", "new")
        assertNull(TopicDrafts(disk).load("account-a", "new"))
        assertEquals(other, TopicDrafts(disk).load("account-b", "new")?.first)
    }

    @Test
    fun removingCoverKeepsTheRestOfTheDraft() = runTest {
        val disk = InMemoryChatHistoryFiles()
        val drafts = TopicDrafts(disk)
        val cover = ForumUploadFile("image.webp", "image/webp", byteArrayOf(4))
        val draft = TopicDraft(title = "保留正文和作品", linkedWork = TopicDraftLink(8, "链接", "工具"))
        drafts.save("owner", "new", draft.copy(cover = TopicDraftCover("preview", cover.name, cover.mime)), cover)
        drafts.save("owner", "new", draft)
        val restored = requireNotNull(TopicDrafts(disk).load("owner", "new"))
        assertEquals(draft, restored.first)
        assertNull(restored.second)
    }

    @Test
    fun unreadableCoverDoesNotLoseTheWritingOrTheUploadedImage() = runTest {
        val disk = InMemoryChatHistoryFiles()
        val draft = TopicDraft(body = "图片打开失败时仍然恢复正文",
            cover = TopicDraftCover("cover", "cover.webp", "image/webp", uploaded = ForumImage("image", "/api/images/image", 3)))
        TopicDrafts(disk).save("owner", "new", draft)
        val files = object : ChatHistoryFiles by disk {
            override fun readMedia(name: String): ByteArray = error("cover unreadable")
        }
        val restored = requireNotNull(TopicDrafts(files).load("owner", "new"))
        assertEquals(draft, restored.first)
        assertNull(restored.second)
    }
}
