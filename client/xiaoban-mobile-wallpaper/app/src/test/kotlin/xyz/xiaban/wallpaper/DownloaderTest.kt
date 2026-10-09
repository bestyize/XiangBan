package xyz.xiaban.wallpaper

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import xyz.xiaban.wallpaper.data.CopyFile
import xyz.xiaban.wallpaper.data.Delivery
import xyz.xiaban.wallpaper.data.DownloadStore
import xyz.xiaban.wallpaper.data.Downloader
import xyz.xiaban.wallpaper.data.GallerySink
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.MediaProbe
import xyz.xiaban.wallpaper.data.Remote
import xyz.xiaban.wallpaper.data.SiteException
import xyz.xiaban.wallpaper.data.SiteItem
import xyz.xiaban.wallpaper.data.SourceFile
import xyz.xiaban.wallpaper.data.Verified
import java.io.File
import java.io.IOException
import java.security.MessageDigest
import java.util.concurrent.CancellationException

private val PNG = byteArrayOf(0x89.toByte(), 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 1, 2, 3, 4, 5, 6, 7, 8)
private val OTHER_PNG = PNG.copyOf().also { it[10] = 99 }
private val HTML = "<html>blocked</html>".toByteArray()

private fun sha(bytes: ByteArray) = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }

private val ITEM = SiteItem(
    id = "pexels_1", kind = MediaKind.IMAGE, title = "Beach", author = "a", avatarUrl = "", width = 100, height = 100, color = 0, likes = 0,
    coverUrl = "https://a.test/c.jpg", detailUrl = "https://a.test/d.jpg", fullUrl = "https://a.test/f.jpg", videoUrl = "", tags = emptyList(),
)

/** A site whose behaviour each test scripts; it records every request so the order of attempts can be checked. */
private class FakeRemote(
    var withPrefer: Delivery,
    var plain: Delivery? = null,
    var sourceBody: () -> ByteArray = { PNG },
    var copyBody: () -> ByteArray = { PNG },
) : Remote {
    val calls = mutableListOf<String>()

    override fun delivery(id: String, preferSource: Boolean): Delivery {
        calls += if (preferSource) "manifest(prefer=source)" else "manifest(plain)"
        return if (preferSource) withPrefer else plain ?: throw SiteException("没有")
    }

    override fun downloadCopy(copy: CopyFile, target: File, progress: (Long, Long) -> Unit, cancelled: () -> Boolean) {
        calls += "copy"
        val body = copyBody()
        target.writeBytes(body)
        if (body.size.toLong() != copy.bytes || sha(body) != copy.sha256) {
            target.delete()
            throw SiteException("下载内容校验失败，已丢弃")
        }
    }

    override fun downloadSource(
        source: SourceFile, target: File, onTotal: (Long) -> Unit, progress: (Long, Long) -> Unit, cancelled: () -> Boolean,
    ): Verified.Result {
        calls += "source"
        val body = sourceBody()
        onTotal(body.size.toLong())
        target.writeBytes(body)
        return Verified.Result(sha(body), body.size.toLong())
    }
}

class DownloaderTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val saved = mutableListOf<String>()
    private val gallery = object : GallerySink {
        override fun save(file: File, kind: MediaKind, mime: String, displayName: String): String {
            saved += "$displayName:${file.readBytes().size}"
            return "content://media/${saved.size}"
        }
    }
    private val probe: (File, MediaKind) -> MediaProbe.Size = { file, _ ->
        if (file.readBytes().firstOrNull() != 0x89.toByte()) throw SiteException("系统无法解码这张图片")
        MediaProbe.Size(10, 20)
    }
    private val store by lazy { DownloadStore(folder.root.resolve("downloads.json")) { true } }
    private val temp by lazy { folder.root.resolve("tmp") }

    private fun copyFile(body: ByteArray) = CopyFile("https://s.test/uploads/a.png", sha(body), body.size.toLong(), "image/png")
    private fun source(url: String = "https://cdn.test/a.png") = SourceFile(url, "image/png")
    private fun delivery(copy: CopyFile?, src: SourceFile?) = Delivery("pexels_1", "Beach", MediaKind.IMAGE, copy, src)

    private fun run(remote: FakeRemote, cancelled: () -> Boolean = { false }) =
        Downloader(remote, gallery, store, temp, probe).run(ITEM, onStage = {}, onSize = {}, onProgress = {}, cancelled = cancelled, now = { 7L })

    private fun leftovers() = temp.listFiles()?.size ?: 0

    @Test
    fun theProvidersLinkIsUsedAndOurCopyIsNeverTouched() {
        val remote = FakeRemote(delivery(copyFile(PNG), source()))
        val result = run(remote)
        assertEquals(listOf("manifest(prefer=source)", "source"), remote.calls)
        assertFalse(result.alreadySaved)
        assertEquals(sha(PNG), result.entry.sha256)
        assertEquals(1, saved.size)
        assertEquals(0, leftovers())
    }

    @Test
    fun aSourceFileIsNotComparedWithTheHashOfOurCopy() {
        val remote = FakeRemote(delivery(copyFile(PNG), source()), sourceBody = { OTHER_PNG })
        val result = run(remote)
        assertEquals("the stored hash describes the file that was really saved", sha(OTHER_PNG), result.entry.sha256)
        assertEquals(listOf("manifest(prefer=source)", "source"), remote.calls)
    }

    @Test
    fun aBlockedSourceThatReturnsAWebPageFallsBackToOurCopy() {
        val remote = FakeRemote(delivery(copyFile(PNG), source()), sourceBody = { HTML })
        val result = run(remote)
        assertEquals(listOf("manifest(prefer=source)", "source", "copy"), remote.calls)
        assertEquals(sha(PNG), result.entry.sha256)
        assertEquals(1, saved.size)
        assertEquals(0, leftovers())
    }

    @Test
    fun aSourceThatCannotBeReachedFallsBackToOurCopy() {
        val remote = FakeRemote(delivery(copyFile(PNG), source()), sourceBody = { throw SiteException("网站返回错误（403）") })
        run(remote)
        assertEquals(listOf("manifest(prefer=source)", "source", "copy"), remote.calls)
    }

    @Test
    fun aRawIoFailureOfTheSourceAlsoFallsBack() {
        val remote = FakeRemote(delivery(copyFile(PNG), source()), sourceBody = { throw IOException("reset") })
        run(remote)
        assertEquals(listOf("manifest(prefer=source)", "source", "copy"), remote.calls)
    }

    @Test
    fun whenTheSiteHasNoCopyYetItIsAskedToMakeOneAfterTheSourceFails() {
        val made = copyFile(PNG)
        val remote = FakeRemote(delivery(null, source()), plain = delivery(made, null), sourceBody = { HTML })
        val result = run(remote)
        assertEquals(listOf("manifest(prefer=source)", "source", "manifest(plain)", "copy"), remote.calls)
        assertEquals(sha(PNG), result.entry.sha256)
    }

    @Test
    fun anOldSiteWithoutSourceLinksBehavesExactlyAsBefore() {
        val remote = FakeRemote(delivery(copyFile(PNG), null))
        val result = run(remote)
        assertEquals(listOf("manifest(prefer=source)", "copy"), remote.calls)
        assertEquals(sha(PNG), result.entry.sha256)
    }

    @Test
    fun ourCopyIsStillVerifiedAndABadOneIsRefused() {
        val remote = FakeRemote(delivery(copyFile(PNG), null), copyBody = { OTHER_PNG })
        try {
            run(remote)
            fail("a copy that does not match its manifest must not be saved")
        } catch (e: SiteException) {
            assertTrue(e.message, e.message!!.contains("校验"))
        }
        assertTrue(saved.isEmpty())
        assertEquals(0, leftovers())
    }

    @Test
    fun whenEveryWayFailsTheMostUsefulReasonIsReported() {
        val remote = FakeRemote(delivery(copyFile(PNG), source()), sourceBody = { HTML }, copyBody = { OTHER_PNG })
        try {
            run(remote)
            fail("expected an error")
        } catch (e: SiteException) {
            assertTrue(e.message, e.message!!.contains("校验"))
        }
        assertEquals(listOf("manifest(prefer=source)", "source", "copy"), remote.calls)
        assertEquals(0, leftovers())
    }

    @Test
    fun cancellingStopsAtOnceWithoutTryingTheNextWay() {
        val remote = FakeRemote(delivery(copyFile(PNG), source()), sourceBody = { throw CancellationException("已取消") })
        try {
            run(remote)
            fail("expected cancellation")
        } catch (_: CancellationException) {
        }
        assertEquals(listOf("manifest(prefer=source)", "source"), remote.calls)
        assertEquals(0, leftovers())
    }

    @Test
    fun aWallpaperAlreadyInTheGalleryIsNotFetchedAgain() {
        val remote = FakeRemote(delivery(copyFile(PNG), source()))
        run(remote)
        remote.calls.clear()
        val again = run(remote)
        assertTrue(again.alreadySaved)
        assertTrue("no network at all", remote.calls.isEmpty())
        assertEquals(1, saved.size)
    }

    @Test
    fun theSameFileFromAnotherRouteIsNotSavedTwice() {
        val first = FakeRemote(delivery(copyFile(PNG), source()))
        run(first)
        val other = ITEM.copy(id = "another_id")
        val second = FakeRemote(Delivery("another_id", "Beach", MediaKind.IMAGE, copyFile(PNG), source()))
        val result = Downloader(second, gallery, store, temp, probe).run(other, {}, {}, {}, { false }, { 8L })
        assertTrue(result.alreadySaved)
        assertEquals(1, saved.size)
        assertNull(null)
    }

    @Test
    fun sizeIsReportedFromTheCopyOrFromTheSourceDownload() {
        val sizes = mutableListOf<Long>()
        val remote = FakeRemote(delivery(null, source()))
        Downloader(remote, gallery, store, temp, probe).run(ITEM, {}, { sizes += it }, {}, { false }, { 1L })
        assertTrue("got $sizes", sizes.contains(PNG.size.toLong()))
    }
}
