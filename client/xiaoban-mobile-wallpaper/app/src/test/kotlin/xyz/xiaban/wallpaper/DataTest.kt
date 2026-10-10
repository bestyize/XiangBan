package xyz.xiaban.wallpaper

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Assert.fail
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import xyz.xiaban.wallpaper.data.Attempt
import xyz.xiaban.wallpaper.data.CopyFile
import xyz.xiaban.wallpaper.data.Delivery
import xyz.xiaban.wallpaper.data.DownloadPlan
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.SourceFile
import xyz.xiaban.wallpaper.data.DownloadStore
import xyz.xiaban.wallpaper.data.FavoriteStore
import xyz.xiaban.wallpaper.data.GalleryNaming
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.MediaRules
import xyz.xiaban.wallpaper.data.MediaSniffer
import xyz.xiaban.wallpaper.data.SiteException
import xyz.xiaban.wallpaper.data.SiteItem
import xyz.xiaban.wallpaper.data.SiteParser
import xyz.xiaban.wallpaper.data.Verified
import xyz.xiaban.wallpaper.data.parseHexColor
import xyz.xiaban.wallpaper.data.phoneReason
import xyz.xiaban.wallpaper.data.resolveLink
import java.util.concurrent.CancellationException

private fun bytes(vararg v: Int) = ByteArray(v.size) { v[it].toByte() }

private const val SITE = "https://wallpaper.xiangban.xyz"

private const val SHA = "1060b0b29e3e4ef1ba88af173e61d6891bb7d8a7b2000a4dacac5e5cdd6db911"

private const val DELIVERY = """{"items":[{"author":"NASA","id":"site-abc","kind":"image","resource":{"bytes":1537444,
"mime":"image/jpeg","sha256":"$SHA","url":"https://wallpaper.xiangban.xyz/uploads/desktop-1060.jfif"},"title":"Nebula","version":"1.0.0"}],"revision":"x","schemaVersion":1}"""

private const val IMAGE_ITEM = """{"id":"pixabay_1","media_type":"image","title":"Pink Wallpaper","tags":["pink","flowers"],
"resolution":{"width":6000,"height":4000},"dominant_color":"#7c2d12",
"preview_url":"https://pixabay.com/get/big_1280.jpg","regular_url":"https://pixabay.com/get/big_1280.jpg",
"downloads":{"small":"https://pixabay.com/get/small_640.jpg","medium":"https://pixabay.com/get/big_1280.jpg","thumb":"https://cdn.pixabay.com/t_150.jpg"},
"author":{"name":"Hans","avatar_url":"https://cdn.pixabay.com/user/a.jpg"},"stats":{"likes":6756}}"""

private const val VIDEO_ITEM = """{"id":"vid_1","media_type":"video","title":"Ocean","resolution":{"width":1920,"height":1080},
"preview_url":"https://thumb.test/poster.jpg",
"downloads":{"original":"https://up.test/o.ogv","large":"https://up.test/l.ogv","medium":"https://up.test/m.480p.webm?x=1","small":"https://up.test/s.240p.webm"},
"author":{"name":"Wiki"},"stats":{"likes":12}}"""

private const val SELF_HOSTED_VIDEO = """{"id":"pexels_v_1","media_type":"video","title":"Ocean","resolution":{"width":1920,"height":1080},
"preview_url":"/uploads/desktop-aaa.jfif","regular_url":"/uploads/desktop-bbb.mp4",
"downloads":{"original":"/uploads/desktop-bbb.mp4","large":"/uploads/desktop-bbb.mp4","medium":"/uploads/desktop-bbb.mp4",
"small":"/uploads/desktop-aaa.jfif","thumb":"/uploads/desktop-aaa.jfif"},"author":{"name":"Pexels","avatar_url":"/uploads/desktop-av.jfif"}}"""

class SiteParserTest {
    @Test
    fun completesTheSitesOwnRelativeLinksWithItsAddress() {
        val item = SiteParser.page("""{"items":[$SELF_HOSTED_VIDEO]}""", "https://site.test").items.single()
        assertEquals("https://site.test/uploads/desktop-aaa.jfif", item.coverUrl)
        assertEquals("https://site.test/uploads/desktop-av.jfif", item.avatarUrl)
    }

    @Test
    fun relativeLinksNeverEscapeTheSite() {
        val tricky = SELF_HOSTED_VIDEO.replace("/uploads/desktop-aaa.jfif", "//evil.test/x.jpg")
        val item = SiteParser.page("""{"items":[$tricky]}""", "https://site.test").items.singleOrNull()
        assertNull(item)
    }

    @Test
    fun relativeLinksNeedAnHttpsSite() {
        assertTrue(SiteParser.page("""{"items":[$SELF_HOSTED_VIDEO]}""", "http://site.test").items.isEmpty())
    }

    @Test
    fun readsAnImageCardWithEverythingTheGridNeeds() {
        val item = SiteParser.page("""{"items":[$IMAGE_ITEM],"has_more":true,"page":2}""", SITE).items.single()
        assertEquals("pixabay_1", item.id)
        assertEquals(MediaKind.IMAGE, item.kind)
        assertEquals("https://pixabay.com/get/small_640.jpg", item.coverUrl)
        assertEquals("https://pixabay.com/get/big_1280.jpg", item.detailUrl)
        assertEquals("Hans", item.author)
        assertEquals("https://cdn.pixabay.com/user/a.jpg", item.avatarUrl)
        assertEquals(6756, item.likes)
        assertEquals(listOf("pink", "flowers"), item.tags)
        assertEquals(0xFF7C2D12.toInt(), item.color)
        assertEquals(6000, item.width)
    }

    @Test
    fun pagesReportWhetherThereIsMore() {
        val page = SiteParser.page("""{"items":[$IMAGE_ITEM],"has_more":true,"page":2}""", SITE)
        assertTrue(page.hasMore)
        assertEquals(2, page.page)
        assertFalse(SiteParser.page("""{"items":[],"has_more":false}""", SITE).hasMore)
    }

    @Test
    fun aVideoCardUsesItsPoster() {
        val item = SiteParser.page("""{"items":[$VIDEO_ITEM]}""", SITE).items.single()
        assertEquals(MediaKind.VIDEO, item.kind)
        assertEquals("https://thumb.test/poster.jpg", item.coverUrl)
    }

    @Test
    fun skipsCardsItCannotShowSafely() {
        val page = SiteParser.page(
            """{"items":[
              {"id":"audio","media_type":"audio","title":"x","preview_url":"https://a.test/a.jpg"},
              {"id":"a/b","media_type":"image","preview_url":"https://a.test/a.jpg"},
              {"id":"plain","media_type":"image","preview_url":"http://insecure.test/a.jpg"},
              {"id":"nopic","media_type":"image"},
              $IMAGE_ITEM]}""",
            SITE,
        )
        assertEquals(listOf("pixabay_1"), page.items.map { it.id })
    }

    @Test
    fun aVideoCardKeepsItsPosterAsThePictureAndTheFileAsTheVideo() {
        val item = SiteParser.page("""{"items":[$SELF_HOSTED_VIDEO]}""", "https://site.test").items.single()
        assertEquals("https://site.test/uploads/desktop-aaa.jfif", item.detailUrl)
        assertEquals("https://site.test/uploads/desktop-aaa.jfif", item.fullUrl)
        assertEquals("https://site.test/uploads/desktop-bbb.mp4", item.videoUrl)
    }

    @Test
    fun picksTheSmallestPlayableVideoAndNeverAFormatThePhoneCannotPlay() {
        val item = SiteParser.page("""{"items":[$VIDEO_ITEM]}""", SITE).items.single()
        assertEquals("https://up.test/m.480p.webm?x=1", item.videoUrl)
        val onlyOgv = VIDEO_ITEM.replace(".480p.webm?x=1", ".ogv").replace("s.240p.webm", "s.ogv").replace("l.ogv", "l.ogv")
        assertEquals("", SiteParser.page("""{"items":[$onlyOgv]}""", SITE).items.single().videoUrl)
    }

    @Test
    fun anImageCardOffersALargerPictureForFullScreenViewing() {
        val withLarge = IMAGE_ITEM.replace(""""downloads":{""", """"downloads":{"large":"https://pixabay.com/get/large_1920.jpg","original":"https://pixabay.com/get/o.jpg",""")
        val item = SiteParser.page("""{"items":[$withLarge]}""", SITE).items.single()
        assertEquals("https://pixabay.com/get/large_1920.jpg", item.fullUrl)
        assertEquals("", item.videoUrl)
        val plain = SiteParser.page("""{"items":[$IMAGE_ITEM]}""", SITE).items.single()
        assertEquals("https://pixabay.com/get/big_1280.jpg", plain.fullUrl)
    }

    @Test
    fun readsCategories() {
        val list = SiteParser.categories("""{"categories":[{"id":"nature","keyword":"k","name":"自然"}]}""")
        assertEquals("nature", list.single().id)
        assertEquals("自然", list.single().name)
    }

    @Test
    fun acceptsAValidDelivery() {
        val d = SiteParser.delivery(DELIVERY, SITE)
        assertEquals(MediaKind.IMAGE, d.kind)
        assertEquals(1537444L, d.copy!!.bytes)
        assertEquals("image/jpeg", d.copy!!.mime)
        assertNull(d.source)
    }

    private fun rejects(text: String, part: String) {
        try {
            SiteParser.delivery(text, SITE)
            fail("should have been rejected: $part")
        } catch (e: SiteException) {
            assertTrue(e.message, e.message!!.contains(part))
        }
    }

    @Test
    fun rejectsUnsafeOrInconsistentDeliveries() {
        rejects(DELIVERY.replace("\"schemaVersion\":1", "\"schemaVersion\":2"), "协议")
        rejects(DELIVERY.replace("https://wallpaper", "http://wallpaper"), "不安全")
        rejects(DELIVERY.replace(SHA, "ABC"), "校验值")
        rejects(DELIVERY.replace("\"image/jpeg\"", "\"video/mp4\""), "不支持")
        rejects(DELIVERY.replace("1537444", "999999999999"), "大小")
        rejects("not json", "无效")
    }

    @Test
    fun acceptsADeliveryWhoseFileIsASitePath() {
        val d = SiteParser.delivery(DELIVERY.replace("https://wallpaper.xiangban.xyz/uploads", "/uploads"), SITE)
        assertEquals("https://wallpaper.xiangban.xyz/uploads/desktop-1060.jfif", d.copy!!.url)
    }

    @Test
    fun aDeliveryCanOfferTheProvidersLinkNextToOurCopy() {
        val both = DELIVERY.replace(""""title":"Nebula"""", """"source":{"url":"https://images.pexels.test/a.jpg","mime":"image/jpeg"},"title":"Nebula"""")
        val d = SiteParser.delivery(both, SITE)
        assertEquals("https://images.pexels.test/a.jpg", d.source!!.url)
        assertEquals(1537444L, d.copy!!.bytes)
    }

    @Test
    fun aDeliveryMayCarryOnlyTheProvidersLinkBecauseNoCopyExistsYet() {
        val sourceOnly = """{"items":[{"id":"x","kind":"video","title":"V","source":{"url":"https://cdn.test/v.mp4"}}],"schemaVersion":1}"""
        val d = SiteParser.delivery(sourceOnly, SITE)
        assertEquals("https://cdn.test/v.mp4", d.source!!.url)
        assertNull(d.copy)
        assertEquals(MediaKind.VIDEO, d.kind)
    }

    @Test
    fun anUnsafeProviderLinkIsIgnoredAndTheCopyIsUsedInstead() {
        val unsafe = DELIVERY.replace(""""title":"Nebula"""", """"source":{"url":"http://plain.test/a.jpg"},"title":"Nebula"""")
        val d = SiteParser.delivery(unsafe, SITE)
        assertNull(d.source)
        assertEquals(1537444L, d.copy!!.bytes)
    }

    @Test
    fun aDeliveryWithNothingToDownloadIsRejected() {
        rejects("""{"items":[{"id":"x","kind":"image","title":"t"}],"schemaVersion":1}""", "没有")
        rejects("""{"items":[{"id":"x","kind":"image","title":"t","source":{"url":"http://plain.test/a.jpg"}}],"schemaVersion":1}""", "没有")
    }

    @Test
    fun readsTheFallbackPicturesOfOurOwnCopy() {
        val withFallback = IMAGE_ITEM.removeSuffix("}") + ""","fallback":{"preview_url":"/uploads/p.jpg","regular_url":"/uploads/r.jpg",
          "downloads":{"large":"/uploads/l.jpg","small":"/uploads/s.jpg","thumb":"/uploads/t.jpg"}}}"""
        val item = SiteParser.page("""{"items":[$withFallback]}""", SITE).items.single()
        assertEquals("$SITE/uploads/s.jpg", item.fallbackCoverUrl)
        assertEquals("$SITE/uploads/r.jpg", item.fallbackDetailUrl)
        assertEquals("$SITE/uploads/l.jpg", item.fallbackFullUrl)
        assertEquals("", item.fallbackVideoUrl)
    }

    @Test
    fun withoutAFallbackTheFallbackFieldsAreEmpty() {
        val item = SiteParser.page("""{"items":[$IMAGE_ITEM]}""", SITE).items.single()
        assertEquals("", item.fallbackCoverUrl)
        assertEquals("", item.fallbackDetailUrl)
        assertEquals("", item.fallbackFullUrl)
        assertEquals("", item.fallbackVideoUrl)
    }

    @Test
    fun aVideoFallbackIsTheFileOfOurCopy() {
        val video = VIDEO_ITEM.removeSuffix("}") + ""","fallback":{"preview_url":"/uploads/poster.jpg",
          "downloads":{"original":"/uploads/v.mp4","medium":"/uploads/v.mp4","small":"/uploads/poster.jpg"}}}"""
        val item = SiteParser.page("""{"items":[$video]}""", SITE).items.single()
        assertEquals("$SITE/uploads/v.mp4", item.fallbackVideoUrl)
        assertEquals("$SITE/uploads/poster.jpg", item.fallbackCoverUrl)
    }

    @Test
    fun parsesHexColours() {
        assertEquals(0xFF7C2D12.toInt(), parseHexColor("#7c2d12"))
        assertEquals(0, parseHexColor("7c2d"))
        assertEquals(0, parseHexColor("#zzzzzz"))
        assertEquals(0, parseHexColor(null))
    }
}

class ResolveLinkTest {
    @Test
    fun completesOnlyThePathsThatBelongToTheSite() {
        assertEquals("https://s.test/uploads/a.jpg", resolveLink("/uploads/a.jpg", "https://s.test"))
        assertEquals("https://s.test/uploads/a.jpg", resolveLink("/uploads/a.jpg", "https://s.test/"))
        assertEquals("https://cdn.test/a.jpg", resolveLink("https://cdn.test/a.jpg", "https://s.test"))
        assertEquals("", resolveLink("//evil.test/a.jpg", "https://s.test"))
        assertEquals("", resolveLink("http://cdn.test/a.jpg", "https://s.test"))
        assertEquals("", resolveLink("javascript:alert(1)", "https://s.test"))
        assertEquals("", resolveLink("uploads/a.jpg", "https://s.test"))
        assertEquals("", resolveLink(null, "https://s.test"))
        assertEquals("", resolveLink("/uploads/a.jpg", "http://s.test"))
    }
}

class VerifiedTest {
    @get:Rule
    val folder = TemporaryFolder()

    @Test
    fun hashesAndCountsTheBytesThatWereWritten() {
        val target = folder.newFile()
        val result = Verified.copy("abc".byteInputStream(), target, limit = 10)
        assertEquals("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", result.sha256)
        assertEquals(3L, result.bytes)
        assertEquals("abc", target.readText())
    }

    @Test
    fun stopsAndDeletesTheFileWhenTooLarge() {
        val target = folder.newFile()
        try {
            Verified.copy(ByteArray(100).inputStream(), target, limit = 50)
            fail("expected a size error")
        } catch (_: SiteException) {
        }
        assertFalse(target.exists())
    }

    @Test
    fun cancelRemovesThePartialFile() {
        val target = folder.newFile()
        try {
            Verified.copy(ByteArray(10).inputStream(), target, limit = 50, cancelled = { true })
            fail("expected cancellation")
        } catch (_: CancellationException) {
        }
        assertFalse(target.exists())
    }
}

class SnifferTest {
    @Test
    fun recognisesSupportedFormatsFromTheirHeaders() {
        assertEquals("image/png", MediaSniffer.mime(bytes(0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a)))
        assertEquals("image/jpeg", MediaSniffer.mime(bytes(0xff, 0xd8, 0xff, 0xe0)))
        assertEquals("image/gif", MediaSniffer.mime("GIF89a".toByteArray()))
        assertEquals("image/bmp", MediaSniffer.mime("BM....".toByteArray()))
        assertEquals("image/webp", MediaSniffer.mime("RIFF\u0000\u0000\u0000\u0000WEBPVP8 ".toByteArray()))
        assertEquals("video/mp4", MediaSniffer.mime(bytes(0, 0, 0, 0x20) + "ftypisom".toByteArray()))
        assertEquals("image/avif", MediaSniffer.mime(bytes(0, 0, 0, 0x20) + "ftypavif".toByteArray()))
        assertEquals("video/webm", MediaSniffer.mime(bytes(0x1a, 0x45, 0xdf, 0xa3, 0x01)))
    }

    @Test
    fun refusesEverythingElse() {
        assertNull(MediaSniffer.mime("<svg xmlns=...>".toByteArray()))
        assertNull(MediaSniffer.mime("#!/bin/sh".toByteArray()))
        assertNull(MediaSniffer.mime(ByteArray(0)))
    }

    @Test
    fun mapsMimeTypesToKindsAndExtensions() {
        assertEquals(MediaKind.VIDEO, MediaRules.kindOf("video/webm"))
        assertEquals(MediaKind.IMAGE, MediaRules.kindOf("image/jpeg; charset=x"))
        assertNull(MediaRules.kindOf("image/svg+xml"))
        assertEquals("jpg", MediaRules.extension("image/jpeg"))
        assertTrue(MediaRules.isHttps("https://a.test/x"))
        assertFalse(MediaRules.isHttps("http://a.test/x"))
        assertFalse(MediaRules.isHttps("https://user:pw@a.test/x"))
        assertFalse(MediaRules.isHttps("https://a.test/x#frag"))
    }
}

private fun item(id: String, likes: Int = 1) = SiteItem(
    id = id, kind = MediaKind.IMAGE, title = "标题 $id", author = "作者", avatarUrl = "https://a.test/av.jpg",
    width = 1080, height = 1920, color = 0xFF112233.toInt(), likes = likes, coverUrl = "https://a.test/c.jpg",
    detailUrl = "https://a.test/d.jpg", fullUrl = "https://a.test/f.jpg", videoUrl = "",
    fallbackCoverUrl = "https://a.test/fc.jpg", fallbackDetailUrl = "https://a.test/fd.jpg",
    fallbackFullUrl = "https://a.test/ff.jpg", fallbackVideoUrl = "https://a.test/fv.mp4", tags = listOf("a", "b"),
)

class FavoriteStoreTest {
    @get:Rule
    val folder = TemporaryFolder()

    private fun store() = FavoriteStore(folder.root.resolve("fav/favorites.json"))

    @Test
    fun togglesAndKeepsNewestFirst() {
        val store = store()
        assertTrue(store.toggle(item("one"), now = 1))
        assertTrue(store.toggle(item("two"), now = 2))
        assertEquals(listOf("two", "one"), store.list().map { it.item.id })
        assertTrue(store.contains("one"))
        assertFalse(store.toggle(item("one"), now = 3))
        assertFalse(store.contains("one"))
        assertEquals(listOf("two"), store.list().map { it.item.id })
    }

    @Test
    fun survivesReopeningWithEveryFieldIntact() {
        store().toggle(item("keep", likes = 42), now = 7)
        val back = store().list().single()
        assertEquals(item("keep", likes = 42), back.item)
        assertEquals(7L, back.savedAt)
    }

    @Test
    fun favouritesSavedByAnOlderVersionStillLoad() {
        val file = folder.root.resolve("old.json")
        file.writeText(
            """[{"item":{"id":"v1","kind":"video","title":"t","author":"a","avatar":"","w":1920,"h":1080,"color":0,"likes":0,
            "cover":"https://a.test/c.jpg","detail":"https://a.test/c.jpg","video":"https://a.test/v.mp4","tags":[]},"at":9}]""",
        )
        val back = FavoriteStore(file).list().single()
        assertEquals("v1", back.item.id)
        assertEquals(MediaKind.VIDEO, back.item.kind)
    }

    @Test
    fun oldFavouritesGetTheNewFieldsFilledIn() {
        val file = folder.root.resolve("older.json")
        file.writeText(
            """[{"item":{"id":"v1","kind":"video","title":"t","author":"a","avatar":"","w":1920,"h":1080,"color":0,"likes":0,
            "cover":"https://a.test/c.jpg","detail":"https://a.test/file.mp4","tags":[]},"at":9},
            {"item":{"id":"i1","kind":"image","title":"t","author":"a","avatar":"","w":10,"h":10,"color":0,"likes":0,
            "cover":"https://a.test/c.jpg","detail":"https://a.test/d.jpg","tags":[]},"at":8}]""",
        )
        val (video, image) = FavoriteStore(file).list().map { it.item }
        assertEquals("a video file is never used as a picture", "https://a.test/c.jpg", video.detailUrl)
        assertEquals("https://a.test/c.jpg", video.fullUrl)
        assertEquals("", video.videoUrl)
        assertEquals("https://a.test/d.jpg", image.fullUrl)
    }

    @Test
    fun anUnreadableFileMeansNoFavouritesNotACrash() {
        val file = folder.root.resolve("favorites.json")
        file.writeText("{{{ not json")
        assertTrue(FavoriteStore(file).list().isEmpty())
    }
}

class DownloadStoreTest {
    @get:Rule
    val folder = TemporaryFolder()

    private val present = mutableSetOf("content://a", "content://b")

    private fun store() = DownloadStore(folder.root.resolve("downloads.json")) { it in present }

    private fun entry(id: String, uri: String, sha: String = id.padEnd(64, '0')) =
        Downloaded(id, MediaKind.IMAGE, "t$id", uri, sha, 10, 20, 5)

    @Test
    fun addsNewestFirstAndReplacesTheSameWallpaper() {
        val store = store()
        store.add(entry("a", "content://a"))
        store.add(entry("b", "content://b"))
        store.add(entry("a", "content://a", sha = "f".repeat(64)))
        assertEquals(listOf("a", "b"), store.list().map { it.id })
        assertEquals("f".repeat(64), store.list().first().sha256)
    }

    @Test
    fun forgetsEntriesWhoseGalleryFileIsGone() {
        val store = store()
        store.add(entry("a", "content://a"))
        store.add(entry("b", "content://b"))
        present.remove("content://a")
        assertEquals(listOf("b"), store.list().map { it.id })
        assertEquals(listOf("b"), store().list().map { it.id })
    }

    @Test
    fun findsADownloadByContentHashAndRemovesById() {
        val store = store()
        store.add(entry("a", "content://a", sha = "c".repeat(64)))
        assertEquals("a", store.findByHash("c".repeat(64))?.id)
        assertNull(store.findByHash("d".repeat(64)))
        store.remove("a")
        assertTrue(store.list().isEmpty())
    }
}

class GalleryNamingTest {
    @Test
    fun buildsReadableUniqueFileNames() {
        assertEquals("Pink Wallpaper_1060b0b2.jpg", GalleryNaming.fileName("Pink Wallpaper", SHA, "jpg"))
        assertEquals("樱花_1060b0b2.mp4", GalleryNaming.fileName("樱花", SHA, "mp4"))
    }

    @Test
    fun stripsCharactersThatBreakPathsAndFallsBackWhenNothingIsLeft() {
        assertEquals("abc_1060b0b2.png", GalleryNaming.fileName("../a/b*c?", SHA, "png"))
        assertEquals("xiaoban_1060b0b2.png", GalleryNaming.fileName("///???", SHA, "png"))
        assertTrue(GalleryNaming.fileName("x".repeat(200), SHA, "png").length < 60)
    }

    @Test
    fun picksTheAlbumFolderByKind() {
        assertEquals("Pictures/小伴壁纸", GalleryNaming.folder(MediaKind.IMAGE))
        assertEquals("Movies/小伴壁纸", GalleryNaming.folder(MediaKind.VIDEO))
    }
}

class SizeLimitTest {
    @Test
    fun aProvidersFileMayBeLargerThanOurOwnCopyCanBe() {
        val largestSeen = 693L * 1024 * 1024
        assertTrue(largestSeen > MediaRules.MAX_SITE_BYTES)
        assertTrue(largestSeen < MediaRules.MAX_SOURCE_BYTES)
    }
}

class PhoneReasonTest {
    @Test
    fun theDesktopSizeRefusalIsRestatedForThePhone() {
        assertEquals("文件超过 256 MB，网站暂时无法提供下载", phoneReason("桌面壁纸不能超过 256 MiB"))
    }

    @Test
    fun theFormatRefusalKeepsItsDetailButDropsTheDesktopWording() {
        assertEquals(
            "媒体格式暂不支持下载（video/quicktime，1024 字节）",
            phoneReason("媒体格式不适用于桌面壁纸（video/quicktime，1024 字节）"),
        )
    }

    @Test
    fun otherReasonsAreShownAsSent() {
        assertEquals("媒体源 example.com 返回 HTTP 403", phoneReason("媒体源 example.com 返回 HTTP 403"))
    }
}

class DownloadPlanTest {
    private fun delivery(copy: CopyFile?, source: SourceFile?) = Delivery("x", "t", MediaKind.IMAGE, copy, source)
    private val copy = CopyFile("https://s.test/uploads/a.jpg", "a".repeat(64), 10, "image/jpeg")
    private val source = SourceFile("https://cdn.test/a.jpg", "image/jpeg")

    @Test
    fun theProvidersLinkIsTriedFirstAndOurCopyIsTheFallback() {
        val plan = DownloadPlan.attempts(delivery(copy, source))
        assertEquals(2, plan.size)
        assertTrue(plan[0] is Attempt.Source)
        assertTrue(plan[1] is Attempt.Copy)
    }

    @Test
    fun whenNoCopyExistsYetTheLastResortIsToAskTheSiteToMakeOne() {
        val plan = DownloadPlan.attempts(delivery(null, source))
        assertEquals(2, plan.size)
        assertTrue(plan[0] is Attempt.Source)
        assertTrue(plan[1] is Attempt.AskForCopy)
    }

    @Test
    fun anOldSiteWithoutASourceBehavesExactlyAsBefore() {
        val plan = DownloadPlan.attempts(delivery(copy, null))
        assertEquals(1, plan.size)
        assertTrue(plan[0] is Attempt.Copy)
    }
}
