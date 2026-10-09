package xyz.xiaban.wallpaper

import org.junit.Assert.assertEquals
import org.junit.Test
import xyz.xiaban.wallpaper.ui.coverAspect
import xyz.xiaban.wallpaper.ui.downloadLabel
import xyz.xiaban.wallpaper.ui.likesText
import xyz.xiaban.wallpaper.ui.sizeText

class FormatTest {
    @Test
    fun likesAreShortenedLikeTheOriginalApp() {
        assertEquals("0", likesText(0))
        assertEquals("9999", likesText(9_999))
        assertEquals("1万", likesText(10_000))
        assertEquals("1.2万", likesText(12_345))
        assertEquals("98.8万", likesText(988_000))
        assertEquals("1亿", likesText(100_000_000))
        assertEquals("0", likesText(-5))
    }

    @Test
    fun coversKeepTheTrueShapeOfThePicture() {
        assertEquals(16f / 9f, coverAspect(3840, 2160), 0.0001f)
        assertEquals(4f / 3f, coverAspect(4896, 3672), 0.0001f)
        assertEquals(1.5f, coverAspect(6000, 4000), 0.0001f)
        assertEquals(1440f / 3120f, coverAspect(1440, 3120), 0.0001f)
        assertEquals(1f, coverAspect(800, 800), 0.0001f)
    }

    @Test
    fun differentShapesGiveDifferentHeightsInAColumn() {
        val column = 540f
        val wide = column / coverAspect(3840, 2160)
        val standard = column / coverAspect(4896, 3672)
        val tall = column / coverAspect(1440, 3120)
        assertEquals(true, wide < standard && standard < tall)
    }

    @Test
    fun unknownSizesGetTheCataloguesUsualShapeAndAbsurdOnesAreBounded() {
        assertEquals(1.5f, coverAspect(0, 0), 0.0001f)
        assertEquals(1.5f, coverAspect(100, 0), 0.0001f)
        assertEquals(0.25f, coverAspect(10, 10_000), 0.0001f)
        assertEquals(4f, coverAspect(10_000, 10), 0.0001f)
    }

    @Test
    fun fileSizesAreReadable() {
        assertEquals("512 B", sizeText(512))
        assertEquals("2 KB", sizeText(2048))
        assertEquals("1.5 MB", sizeText(1_572_864))
    }
}

class DownloadLabelTest {
    @Test
    fun showsPercentAndSizesWhileDownloading() {
        assertEquals("下载中 35% · 38.5 MB / 110.0 MB", downloadLabel(0.35f, 110L * 1024 * 1024))
        assertEquals("下载中 0% · 0 B / 2 KB", downloadLabel(0f, 2048))
        assertEquals("下载中 100% · 2 KB / 2 KB", downloadLabel(1f, 2048))
    }

    @Test
    fun fallsBackToPercentWhenTheSizeIsUnknownAndClampsOddValues() {
        assertEquals("下载中 40%", downloadLabel(0.4f, 0))
        assertEquals("下载中 100% · 2 KB / 2 KB", downloadLabel(1.7f, 2048))
        assertEquals("下载中 0% · 0 B / 2 KB", downloadLabel(-1f, 2048))
    }
}

class BackoffTest {
    @Test
    fun waitsLongerEachTimeUpToACap() {
        assertEquals(1_200L, xyz.xiaban.wallpaper.data.Backoff.delayMillis(0, null))
        assertEquals(2_400L, xyz.xiaban.wallpaper.data.Backoff.delayMillis(1, null))
        assertEquals(4_800L, xyz.xiaban.wallpaper.data.Backoff.delayMillis(2, null))
        assertEquals(8_000L, xyz.xiaban.wallpaper.data.Backoff.delayMillis(5, null))
    }

    @Test
    fun followsTheServersOwnHintButNeverWaitsTooLong() {
        assertEquals(3_000L, xyz.xiaban.wallpaper.data.Backoff.delayMillis(0, "3"))
        assertEquals(8_000L, xyz.xiaban.wallpaper.data.Backoff.delayMillis(0, "30"))
        assertEquals(1_200L, xyz.xiaban.wallpaper.data.Backoff.delayMillis(0, "soon"))
        assertEquals(1_200L, xyz.xiaban.wallpaper.data.Backoff.delayMillis(0, "999"))
    }
}
