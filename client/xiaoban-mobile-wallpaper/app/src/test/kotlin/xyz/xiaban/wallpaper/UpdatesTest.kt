package xyz.xiaban.wallpaper

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import xyz.xiaban.wallpaper.data.Announcement
import xyz.xiaban.wallpaper.data.SeenAnnouncements
import xyz.xiaban.wallpaper.data.SiteException
import xyz.xiaban.wallpaper.data.UpdateRules
import java.io.File

private const val REV = "a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90"

private fun item(extra: String = "", id: String = "n1", revision: String = REV) =
    """{"id":"$id","title":"标题","body":"内容","revision":"$revision"$extra}"""

private fun list(vararg items: String, audience: String = "android") =
    """{"schemaVersion":1,"audience":"$audience","items":[${items.joinToString(",")}]}"""

private fun notice(id: String, skippable: Boolean, revision: String = REV) =
    Announcement(id, "标题", "内容", revision, skippable, "确定", "取消", null)

class UpdateParseTest {
    @Test
    fun updateReadsVersionAndNotes() {
        val update = UpdateRules.update("""{"schemaVersion":1,"version":"1.2.0","notes":"体验优化","publishedAt":"2026-10-09T00:00:00Z","platforms":{}}""")
        assertEquals("1.2.0", update.version)
        assertEquals("体验优化", update.notes)
    }

    @Test
    fun updateRefusesAnythingThatIsNotAVersion() {
        for (body in listOf("""{"schemaVersion":1,"version":"latest"}""", """{"schemaVersion":1}""", """{"schemaVersion":2,"version":"1.0.0"}""", "<html>", "")) {
            try {
                UpdateRules.update(body)
                fail("accepted: $body")
            } catch (_: SiteException) {
            }
        }
    }

    @Test
    fun downloadsLinkPointsAtTheDownloadSectionOfTheSite() {
        assertEquals("https://wallpaper.xiangban.xyz/#desktop-downloads", UpdateRules.downloadsLink("https://wallpaper.xiangban.xyz"))
        assertEquals("https://wallpaper.xiangban.xyz/#desktop-downloads", UpdateRules.downloadsLink("https://wallpaper.xiangban.xyz/"))
    }
}

class AnnouncementParseTest {
    @Test
    fun aMinimalNoticeIsSkippableWithDefaultLabelsAndNoLink() {
        val n = UpdateRules.announcements(list(item())).single()
        assertEquals("n1", n.id)
        assertEquals("标题", n.title)
        assertEquals("内容", n.body)
        assertTrue(n.skippable)
        assertEquals("确定", n.confirmText)
        assertEquals("取消", n.cancelText)
        assertNull(n.confirmLink)
    }

    @Test
    fun aFullNoticeKeepsItsDialogFieldsAndAFragmentInTheLink() {
        val n = UpdateRules.announcements(
            list(item(""","skippable":false,"confirmText":"去下载","cancelText":"稍后","confirmUrl":"https://wallpaper.xiangban.xyz/#desktop-downloads"""")),
        ).single()
        assertFalse(n.skippable)
        assertEquals("去下载", n.confirmText)
        assertEquals("稍后", n.cancelText)
        assertEquals("https://wallpaper.xiangban.xyz/#desktop-downloads", n.confirmLink)
    }

    @Test
    fun jsonNullsAndBlankLabelsFallBackToDefaults() {
        val n = UpdateRules.announcements(list(item(""","confirmText":null,"cancelText":"  ","confirmUrl":null"""))).single()
        assertEquals("确定", n.confirmText)
        assertEquals("取消", n.cancelText)
        assertNull(n.confirmLink)
    }

    @Test
    fun onlyHttpsLinksWithoutCredentialsAreKept() {
        for (link in listOf("http://example.com/a", "javascript:alert(1)", "intent://scan/#Intent;scheme=zxing;end", "market://details?id=x", "https://user:pw@example.com/", "https:///nohost", "not a link")) {
            val n = UpdateRules.announcements(list(item(""","confirmUrl":"$link""""))).single()
            assertNull("kept: $link", n.confirmLink)
        }
        assertEquals("https://example.com/a?b=1#c", UpdateRules.announcements(list(item(""","confirmUrl":"https://example.com/a?b=1#c""""))).single().confirmLink)
    }

    @Test
    fun anOverlongLabelFallsBackToTheDefault() {
        val n = UpdateRules.announcements(list(item(""","confirmText":"这个按钮文字实在是太长了没法放""""))).single()
        assertEquals("确定", n.confirmText)
    }

    @Test
    fun badItemsAreSkippedAndTheRestKept() {
        val bad = listOf(
            item(id = ""),
            item(revision = "not-a-hash"),
            """{"id":"x","title":"","body":"","revision":"$REV"}""",
            "7",
        )
        val good = item(id = "ok")
        val parsed = UpdateRules.announcements(list(*(bad + good).toTypedArray()))
        assertEquals(listOf("ok"), parsed.map { it.id })
    }

    @Test
    fun aListNotBuiltForThisAppIsRefused() {
        // An older site ignores ?audience and answers with the desktop's list without naming it, or naming "desktop".
        for (body in listOf("""{"schemaVersion":1,"items":[${item()}]}""", list(item(), audience = "desktop"), """{"schemaVersion":2,"audience":"android","items":[]}""", """{"schemaVersion":1,"audience":"android"}""", "oops")) {
            try {
                UpdateRules.announcements(body)
                fail("accepted: $body")
            } catch (_: SiteException) {
            }
        }
    }

    @Test
    fun anEmptyListIsFine() {
        assertTrue(UpdateRules.announcements(list()).isEmpty())
    }
}

class AnnouncementSelectionTest {
    @Test
    fun aSkippableNoticeIsShownOnceThenNotAgain() {
        val n = notice("tip", skippable = true)
        assertEquals(listOf(n), UpdateRules.toShow(listOf(n)) { _, _ -> false })
        assertTrue(UpdateRules.toShow(listOf(n)) { id, rev -> id == "tip" && rev == REV }.isEmpty())
    }

    @Test
    fun anEditedNoticeCountsAsNewBecauseItsRevisionChanged() {
        val edited = notice("tip", skippable = true, revision = "f".repeat(64))
        assertEquals(listOf(edited), UpdateRules.toShow(listOf(edited)) { id, rev -> id == "tip" && rev == REV })
    }

    @Test
    fun aMandatoryNoticeComesBackEveryLaunchEvenIfSeen() {
        val must = notice("must", skippable = false)
        assertEquals(listOf(must), UpdateRules.toShow(listOf(must)) { _, _ -> true })
    }

    @Test
    fun mandatoryNoticesComeFirstThenUnseenSkippableOnesInSiteOrder() {
        val a = notice("a", skippable = true)
        val b = notice("b", skippable = false)
        val c = notice("c", skippable = true)
        val d = notice("d", skippable = false)
        val seen = setOf("c")
        assertEquals(listOf("b", "d", "a"), UpdateRules.toShow(listOf(a, b, c, d)) { id, _ -> id in seen }.map { it.id })
    }
}

class SeenAnnouncementsTest {
    @get:Rule
    val folder = TemporaryFolder()

    private fun store() = SeenAnnouncements(File(folder.root, "seen.json"))

    @Test
    fun whatWasMarkedIsRememberedAcrossRestarts() {
        store().add("tip", REV)
        val again = store()
        assertTrue(again.has("tip", REV))
        assertFalse(again.has("tip", "e".repeat(64)))
        assertFalse(again.has("other", REV))
    }

    @Test
    fun markingTwiceKeepsOneEntry() {
        val s = store()
        s.add("tip", REV)
        s.add("tip", REV)
        assertEquals(1, org.json.JSONArray(File(folder.root, "seen.json").readText()).length())
    }

    @Test
    fun onlyTheMostRecentEntriesAreKept() {
        val s = store()
        for (i in 0 until SeenAnnouncements.LIMIT + 5) s.add("n$i", REV)
        assertFalse(s.has("n0", REV))
        assertFalse(s.has("n4", REV))
        assertTrue(s.has("n5", REV))
        assertTrue(s.has("n${SeenAnnouncements.LIMIT + 4}", REV))
    }

    @Test
    fun aDamagedFileReadsAsNothingSeen() {
        File(folder.root, "seen.json").writeText("[{\"id\":\"tip\",")
        val s = store()
        assertFalse(s.has("tip", REV))
        s.add("tip", REV)
        assertTrue(s.has("tip", REV))
    }
}
