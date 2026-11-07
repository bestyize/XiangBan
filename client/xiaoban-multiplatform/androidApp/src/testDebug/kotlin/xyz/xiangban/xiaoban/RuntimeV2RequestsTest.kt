package xyz.xiangban.xiaoban

import org.junit.Assert.*
import org.junit.Test

class RuntimeV2RequestsTest {
    @Test fun paths() {
        assertEquals("runtime-v2-spike/v1/index.html", RuntimeV2Requests.assetPath("https://spike1.app.xiangban.xyz/v1/index.html?q=1"))
        assertEquals("runtime-v2-spike/index.html", RuntimeV2Requests.assetPath("https://spike1.app.xiangban.xyz/"))
        listOf("https://app.xiangban.xyz/a", "http://spike1.app.xiangban.xyz/a", "https://spike1.app.xiangban.xyz/%2e%2e/a", "https://spike1.app.xiangban.xyz/a%2fb").forEach { assertNull(it, RuntimeV2Requests.assetPath(it)) }
    }
    @Test fun mimeTypes() {
        mapOf("a.mjs" to "text/javascript", "a.js" to "text/javascript", "a.wasm" to "application/wasm", "a.html" to "text/html", "x" to "application/octet-stream").forEach { (file, type) -> assertEquals(type, RuntimeV2Requests.contentType(file)) }
    }
    @Test fun localAddresses() {
        listOf("127.2.3.4", "10.255.1.1", "172.16.0.0", "172.31.255.255", "192.168.2.1", "169.254.0.1", "localhost", "a.local", "::1", "[::1]", "fc00::1", "fdff::1", "::ffff:127.0.0.1").forEach { assertTrue(it, RuntimeV2Requests.isLocal(it)) }
        listOf("8.8.8.8", "172.15.1.1", "172.32.1.1", "192.169.1.1", "example.com", "local.example.com", "fe00::1", "2001:db8::1").forEach { assertFalse(it, RuntimeV2Requests.isLocal(it)) }
    }
}
