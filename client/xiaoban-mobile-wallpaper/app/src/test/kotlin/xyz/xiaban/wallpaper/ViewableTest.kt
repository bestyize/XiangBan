package xyz.xiaban.wallpaper

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.SiteItem
import xyz.xiaban.wallpaper.ui.viewable

private fun site(
    kind: MediaKind, video: String = "", w: Int = 1920, h: Int = 1080,
    fbDetail: String = "", fbFull: String = "", fbVideo: String = "",
) = SiteItem(
    id = "x1", kind = kind, title = "标题", author = "作者", avatarUrl = "", width = w, height = h, color = 0, likes = 0,
    coverUrl = "https://a.test/cover.jpg", detailUrl = "https://a.test/detail.jpg", fullUrl = "https://a.test/full.jpg",
    videoUrl = video, fallbackDetailUrl = fbDetail, fallbackFullUrl = fbFull, fallbackVideoUrl = fbVideo, tags = emptyList(),
)

class ViewableTest {
    @Test
    fun anOnlineImageOpensItsSharpestPictureAndShowsTheDetailOneFirst() {
        val v = site(MediaKind.IMAGE).viewable()
        assertEquals("https://a.test/full.jpg", v.image)
        assertEquals("https://a.test/detail.jpg", v.poster)
        assertEquals(16f / 9f, v.aspect, 0.0001f)
        assertFalse(v.local)
        assertFalse(v.playable)
    }

    @Test
    fun anOnlineVideoIsPlayableOnlyWhenThePhoneCanStreamIt() {
        assertTrue(site(MediaKind.VIDEO, video = "https://a.test/v.mp4").viewable().playable)
        assertFalse(site(MediaKind.VIDEO).viewable().playable)
    }

    @Test
    fun aSavedVideoPlaysStraightFromTheGallery() {
        val v = Downloaded("id", MediaKind.VIDEO, "t", "content://media/external/video/media/7", "h", 3840, 2160, 1).viewable()
        assertTrue(v.playable)
        assertTrue(v.local)
        assertEquals("content://media/external/video/media/7", v.video)
    }

    @Test
    fun aSavedImageOpensFromTheGalleryEntry() {
        val v = Downloaded("id", MediaKind.IMAGE, "t", "content://media/external/images/media/3", "h", 1080, 1920, 1).viewable()
        assertEquals("content://media/external/images/media/3", v.image)
        assertEquals(1080f / 1920f, v.aspect, 0.0001f)
        assertFalse(v.playable)
        assertTrue(v.local)
    }

    @Test
    fun theSitesOwnCopyBecomesTheFallbackOfEveryPicture() {
        val v = site(MediaKind.IMAGE, fbDetail = "https://s.test/uploads/d.jpg", fbFull = "https://s.test/uploads/f.jpg").viewable()
        assertEquals("https://s.test/uploads/f.jpg", v.fallbackImage)
        assertEquals("https://s.test/uploads/d.jpg", v.fallbackPoster)
        assertEquals("", v.fallbackVideo)
    }

    @Test
    fun noCopyMeansNoFallback() {
        val v = site(MediaKind.IMAGE).viewable()
        assertEquals("", v.fallbackImage)
        assertEquals("", v.fallbackPoster)
    }

    @Test
    fun aVideoThatOnlyOurCopyCanPlayStillPlays() {
        val v = site(MediaKind.VIDEO, video = "", fbVideo = "https://s.test/uploads/v.mp4").viewable()
        assertTrue(v.playable)
        assertEquals("https://s.test/uploads/v.mp4", v.video)
        assertEquals("nothing left to fall back to", "", v.fallbackVideo)
    }

    @Test
    fun aPlayableVideoKeepsOurCopyAsItsFallback() {
        val v = site(MediaKind.VIDEO, video = "https://cdn.test/v.mp4", fbVideo = "https://s.test/uploads/v.mp4").viewable()
        assertEquals("https://cdn.test/v.mp4", v.video)
        assertEquals("https://s.test/uploads/v.mp4", v.fallbackVideo)
    }

    @Test
    fun savedItemsHaveNoFallbackBecauseTheyAreLocal() {
        val v = Downloaded("id", MediaKind.IMAGE, "t", "content://media/external/images/media/3", "h", 10, 10, 1).viewable()
        assertEquals("", v.fallbackImage)
        assertEquals("", v.fallbackVideo)
    }

    @Test
    fun anUnknownSizeMeansNoAspect() {
        assertEquals(0f, site(MediaKind.IMAGE, w = 0, h = 0).viewable().aspect, 0f)
    }
}
