package xyz.xiangban.xiaoban.common.community

import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.test.runTest
import kotlin.test.*

class NativeDownloadTest {
    @Test fun purchaseAndDownloadUseForumAuth() = runTest {
        val server = FakeServer()
        server.on("/api/attachments/test/purchase", body = """{"purchased":true,"charged":3,"balance":7}""")
        server.on({ it.url.encodedPath == "/api/attachments/test" }, body = "file")
        val api = ForumApi(server.client, CommunityConfig()) { "forum-token" }
        val attachment = ForumAttachment("test", "test.txt", 4, price = 3)
        assertEquals(3, api.purchaseAttachment(attachment).charged)
        assertContentEquals("file".encodeToByteArray(), api.downloadAttachment(attachment))
        assertEquals(listOf(HttpMethod.Post, HttpMethod.Get), server.requests.map { it.method })
        assertTrue(server.requests.all { it.headers["Authorization"] == "Bearer forum-token" })
    }

    @Test fun rejectsTruncatedDownloadsAndSurfacesServerErrors() = runTest {
        val server = FakeServer()
        server.on("/api/attachments/short", body = "a")
        server.on("/api/attachments/locked", HttpStatusCode.Forbidden, """{"message":"无权下载"}""")
        val api = ForumApi(server.client, CommunityConfig()) { null }
        assertFailsWith<ApiException> { api.downloadAttachment(ForumAttachment("short", "a.txt", 2)) }
        assertEquals("无权下载", assertFailsWith<ApiException> {
            api.downloadAttachment(ForumAttachment("locked", "a.txt", 2))
        }.message)
    }

    @Test fun directorySortsEntriesWithMissingStatistics() {
        val a = SoftwareEntry("owner/a", "tools", emptyList(), "owner/a", "", stars = null)
        val b = a.copy(project = "owner/b", stars = 12)
        val c = a.copy(project = "owner/c", stars = 8, updatedAt = "2026-11-03T00:00:00Z")
        val snapshots = mapOf(a.project to SoftwareSummary(starGain = 3))
        val entries = listOf(a, b, c)
        assertEquals(listOf(b, c, a), entries.sortedWith(softwareDirectoryOrder(snapshots, "stars")))
        assertEquals(listOf(c, b, a), entries.sortedWith(softwareDirectoryOrder(snapshots, "updated")))
        assertEquals(listOf(a, c, b), entries.sortedWith(softwareDirectoryOrder(snapshots, "hot")))
    }

    @Test fun onlyOffersPackagesFromConfiguredReleaseRepository() {
        val entry = SoftwareEntry("owner/app", "tools", listOf("Android"), "owner/releases", "v1")
        val base = "https://github.com/owner/releases/releases/download/v1"
        val release = SoftwareRelease("v1", assets = listOf(
            SoftwareAsset("app-arm64.apk", "$base/app-arm64.apk"),
            SoftwareAsset("app-debug.apk", "$base/app-debug.apk"),
            SoftwareAsset("other.apk", "https://github.com/owner/other/releases/download/v1/other.apk"),
            SoftwareAsset("../app.apk", "$base/app.apk"),
        ))
        val result = release.packages(entry).single()
        assertEquals("Android", result.platform)
        assertEquals("ARM64", result.architecture)
    }
}
