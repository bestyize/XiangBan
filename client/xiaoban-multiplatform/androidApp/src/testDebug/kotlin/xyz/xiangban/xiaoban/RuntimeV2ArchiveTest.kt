package xyz.xiangban.xiaoban

import java.io.File
import java.io.RandomAccessFile
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream
import org.apache.commons.compress.archivers.zip.Zip64Mode
import org.apache.commons.compress.archivers.zip.ZipArchiveEntry
import org.apache.commons.compress.archivers.zip.ZipArchiveOutputStream
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder

class RuntimeV2ArchiveTest {
    @get:Rule val temporary = TemporaryFolder()

    @Test fun extractsClassicAndZip64ArchivesWithExplicitAndImplicitDirectories() {
        for (mode in listOf(Zip64Mode.Never, Zip64Mode.Always)) {
            val zip = archive(validEntries + Entry("art/", "") + Entry("lib/nested/main.mjs", "export const n = 2"), mode)
            val destination = temporary.newFolder()
            assertEquals("tool-demo", RuntimeV2Archive.extract(zip, destination, "app").id)
            assertEquals("export const n = 2", destination.resolve("lib/nested/main.mjs").readText())
            assertTrue(destination.resolve("art/icon.svg").isFile)
        }
    }

    @Test fun rejectsAmbiguousPathsAndUnixSpecialEntries() {
        listOf(
            Entry("Art/extra.svg", "x"),
            Entry("art/icon.svg/child.js", "x"),
            Entry("art/icon.svg", "duplicate"),
            Entry("art/.hidden.svg", "x"),
            Entry("../outside.js", "x"),
            Entry("a/b/c/d/e/f/g/h/i.js", "x"),
            Entry("art/a.exe", "x"),
            Entry("links.js", "../index.html", 0xa1ff),
            Entry("pipe.js", "", 0x11a4),
        ).forEach { bad -> reject(archive(validEntries + bad)) }

        val backslashZip = temporary.newFile()
        ZipOutputStream(backslashZip.outputStream()).use { output ->
            for (entry in validEntries + Entry("art\\other.svg", "x")) {
                output.putNextEntry(ZipEntry(entry.path))
                output.write(entry.body.toByteArray())
                output.closeEntry()
            }
        }
        reject(backslashZip)
    }

    @Test fun requiresExpectedKindEntryAssetsAndBoundedManifest() {
        reject(archive(validEntries), "game")
        reject(archive(validEntries.filter { it.path != "art/cover.svg" }))
        val oversized = Entry("manifest.json", validEntries.first().body.padEnd(64 * 1024 + 1))
        reject(archive(listOf(oversized) + validEntries.drop(1)))
    }

    @Test fun rejectsDirectoryCountMismatchAndTooManyExplicitEntries() {
        val zip = archive(validEntries)
        RandomAccessFile(zip, "rw").use { file ->
            file.seek(file.length() - 22 + 8)
            file.write(byteArrayOf(1, 0, 1, 0))
        }
        reject(zip)
        reject(archive(validEntries + (1..3_000).map { Entry("dir$it/", "") }))
    }

    @Test fun removesPartiallyExtractedFilesWhenPayloadCrcFails() {
        val zip = archive(validEntries + Entry("script.js", "unique-fixture-payload"), stored = true)
        val bytes = zip.readBytes()
        val marker = "unique-fixture-payload".toByteArray()
        val position = bytes.indices.first { index ->
            index + marker.size <= bytes.size && marker.indices.all { bytes[index + it] == marker[it] }
        }
        bytes[position] = 'X'.code.toByte()
        zip.writeBytes(bytes)
        reject(zip)
    }

    private fun reject(zip: File, kind: String = "app") {
        val destination = temporary.newFolder()
        assertThrows(IllegalArgumentException::class.java) { RuntimeV2Archive.extract(zip, destination, kind) }
        assertTrue(destination.list()?.isEmpty() == true)
    }

    private fun archive(entries: List<Entry>, zip64Mode: Zip64Mode = Zip64Mode.Never, stored: Boolean = false): File =
        temporary.newFile().also { file ->
            ZipArchiveOutputStream(file).use { output ->
                output.setUseZip64(zip64Mode)
                if (stored) output.setMethod(ZipEntry.STORED)
                for (entry in entries) {
                    val archiveEntry = ZipArchiveEntry(entry.path)
                    if (entry.unixMode != null) archiveEntry.unixMode = entry.unixMode
                    output.putArchiveEntry(archiveEntry)
                    output.write(entry.body.toByteArray())
                    output.closeArchiveEntry()
                }
            }
        }

    private data class Entry(val path: String, val body: String, val unixMode: Int? = null)

    private val validEntries = listOf(
        Entry("manifest.json", """
            {"schemaVersion":3,"runtimeProtocol":2,"kind":"app","id":"tool-demo","name":"测试工具",
            "description":"验收用例","version":"1","category":"tool","icon":"art/icon.svg",
            "cover":"art/cover.svg","input":["touch"]}
        """.trimIndent()),
        Entry("index.html", "<!doctype html><title>test</title>"),
        Entry("art/icon.svg", "<svg/>"),
        Entry("art/cover.svg", "<svg/>"),
    )
}
