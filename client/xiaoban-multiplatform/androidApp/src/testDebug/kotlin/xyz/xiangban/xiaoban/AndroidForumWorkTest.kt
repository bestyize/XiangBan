package xyz.xiangban.xiaoban

import java.util.zip.ZipInputStream
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import xyz.xiangban.xiaoban.common.community.ForumUploadFile
import xyz.xiangban.xiaoban.common.community.MobileGameFile
import xyz.xiangban.xiaoban.common.community.readMobileGameManifest

class AndroidForumWorkTest {
    @Test
    fun htmlAndHtmArePackagedForTheRuntime() {
        for (name in listOf("小游戏.html", "小工具.HTM")) {
            val raw = ForumUploadFile(name, "text/html", "<!doctype html><p>作品内容</p>".encodeToByteArray())
            val result = prepareForumWork(raw)
            assertEquals(name, result.originalName)
            assertEquals(name.substringBeforeLast('.'), result.title)
            assertEquals("app", result.kind)
            assertTrue(result.file.name.endsWith(".zip"))
            val archive = ZipInputStream(result.file.bytes.inputStream()).use { zip ->
                buildMap {
                    var entry = zip.nextEntry
                    while (entry != null) {
                        put(entry.name, zip.readBytes().decodeToString())
                        entry = zip.nextEntry
                    }
                }
            }
            assertEquals(setOf("manifest.json", "index.html", "icon.svg", "cover.svg"), archive.keys)
            assertEquals("app", readMobileGameManifest(archive.getValue("manifest.json")).kind)
            assertTrue(archive.getValue("index.html").startsWith("<!doctype html><script>"))
            assertTrue(archive.getValue("index.html").contains("window.xb?.ready?.()"))
        }
    }

    @Test
    fun zipUsesItsRootManifestAndKeepsTheSelectedBytes() {
        val file = zip("manifest.json", """{"kind":"game","name":"纸飞机","description":"点击屏幕飞行"}""")
        val work = prepareForumWork(file)
        assertEquals("纸飞机", work.title)
        assertEquals("点击屏幕飞行", work.summary)
        assertEquals("game", work.kind)
        assertArrayEquals(file.bytes, work.file.bytes)
    }

    @Test
    fun longNamesKeepTheirExtension() {
        val file = ForumUploadFile("作".repeat(150) + ".htm", "text/html", "<p>Hello</p>".encodeToByteArray())
        val work = prepareForumWork(file)
        assertTrue(work.originalName.endsWith(".htm"))
        assertEquals(120, work.originalName.length)
    }

    @Test
    fun unrelatedFilesAndMalformedWorksAreRejected() {
        val invalid = listOf(
            ForumUploadFile("notes.txt", "text/plain", byteArrayOf(1)),
            ForumUploadFile("bad.htm", "text/html", byteArrayOf(0xff.toByte())),
            ForumUploadFile("broken.zip", "application/zip", byteArrayOf(0x50, 0x4b)),
            zip("folder/manifest.json", """{"kind":"game"}"""),
            zip("manifest.json", """{"kind":"unknown"}"""),
            zip("manifest.json", " ".repeat(65 * 1024)),
        )
        invalid.forEach { file -> assertThrows(IllegalArgumentException::class.java) { prepareForumWork(file) } }
    }

    private fun zip(path: String, manifest: String): ForumUploadFile = ForumUploadFile(
        "作品.ZIP", "application/octet-stream", RuntimeV2Archive.encode(listOf(MobileGameFile(path, manifest))),
    )
}
