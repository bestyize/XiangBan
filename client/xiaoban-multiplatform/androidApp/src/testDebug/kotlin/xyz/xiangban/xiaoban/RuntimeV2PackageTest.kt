package xyz.xiangban.xiaoban

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertThrows
import org.junit.Test

class RuntimeV2PackageTest {
    @Test fun onlyProtocol2ManifestsAreAccepted() {
        val base = """{"schemaVersion":3,"runtimeProtocol":RUNTIME,"kind":"app","id":"demo-app","name":"测试作品",
            "description":"测试说明","version":"1.0","category":"tool","icon":"icon.svg","cover":"cover.svg","input":["touch"]}"""
        assertEquals("demo-app", RuntimeV2Manifest.read(base.replace("RUNTIME", "2")).id)
        for (value in listOf("1", "3", "null", "true", "\"2\"", "2.0")) {
            assertThrows(IllegalArgumentException::class.java) { RuntimeV2Manifest.read(base.replace("RUNTIME", value)) }
        }
        assertThrows(IllegalArgumentException::class.java) { RuntimeV2Manifest.read(base.replace(""","runtimeProtocol":RUNTIME""", "")) }
    }

    @Test fun validatesTheProtocol2ManifestBeforeOpeningAssets() {
        val manifest = """
            {"schemaVersion":3,"runtimeProtocol":2,"kind":"app","id":"demo-app","name":"测试作品",
            "description":"测试说明","version":"1.0","category":"tool","icon":"icon.svg",
            "cover":"cover.svg","input":["touch"],"capabilities":["camera"]}
        """.trimIndent()
        val work = RuntimeV2Manifest.read(manifest)
        assertEquals("index.html", work.entry)
        assertEquals("portrait", work.orientation)
        assertEquals(setOf("camera"), work.capabilities)
        assertThrows(IllegalArgumentException::class.java) { RuntimeV2Manifest.read(manifest.replace("cover.svg", "../cover.svg")) }
        assertThrows(IllegalArgumentException::class.java) { RuntimeV2Manifest.read(manifest.replace("\"schemaVersion\":3", "\"schemaVersion\":2")) }
    }

    @Test fun routesOnlyApprovedFilesUnderTheCurrentVersionAndOrigin() {
        val host = "local-demo.app.xiangban.xyz"
        fun asset(url: String) = runtimeV2AssetPath(url, host, "1.0")
        assertEquals("lib/@scope/code.mjs", asset("https://$host/v1.0/lib/%40scope/code.mjs?q=1#anchor"))
        assertEquals("index.html", asset("https://$host:443/v1.0/index.html"))
        listOf(
            "http://$host/v1.0/index.html",
            "https://$host:444/v1.0/index.html",
            "https://name@$host/v1.0/index.html",
            "https://$host.example.org/v1.0/index.html",
            "https://another.app.xiangban.xyz/v1.0/index.html",
            "https://$host/v2.0/index.html",
            "https://$host/v1.0/%2e%2e/index.html",
            "https://$host/v1.0/lib%2Fcode.mjs",
            "https://$host/v1.0/lib%5Ccode.mjs",
            "https://$host/v1.0/lib%255Ccode.mjs",
            "https://$host/v1.0/lib//code.mjs",
            "https://$host/v1.0/a/b/c/d/e/f/g/h/code.mjs",
            "https://$host/v1.0/code.exe",
        ).forEach { url -> assertNull(url, asset(url)) }
    }
}
