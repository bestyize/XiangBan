package xyz.xiangban.xiaoban

import java.net.Inet6Address
import java.net.InetAddress
import java.net.Socket
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.TimeUnit
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class RuntimeV2ProxyTest {
    @Test fun filtersSpecialUseAndTranslationAddresses() {
        val nonpublic = listOf(
            "0.1.2.3", "10.1.2.3", "100.127.0.1", "127.0.0.2", "169.254.169.254", "172.31.1.2",
            "192.0.0.1", "192.0.2.1", "192.168.1.1", "198.19.1.2", "198.51.100.1", "203.0.113.1",
            "224.1.1.1", "240.0.0.1", "255.255.255.255", "::", "::1", "::ffff:192.168.1.1",
            "64:ff9b::7f00:1", "64:ff9b:1::808:808", "100::1", "2001:0::1", "2001:db8::1",
            "2002:7f00:1::1", "3fff::1", "fc00::1", "fdff::1", "fe80::1", "ff02::1",
        )
        nonpublic.forEach { assertFalse(it, isPublicRuntimeAddress(InetAddress.getByName(it))) }
        val mapped = ByteArray(16).apply {
            this[10] = -1
            this[11] = -1
            this[12] = 8
            this[13] = 8
            this[14] = 8
            this[15] = 8
        }
        assertFalse(isPublicRuntimeAddress(Inet6Address.getByAddress(null, mapped, 0)))
        listOf("8.8.8.8", "172.32.0.1", "2606:4700::1111", "64:ff9b::808:808").forEach {
            assertTrue(it, isPublicRuntimeAddress(InetAddress.getByName(it)))
        }
    }

    @Test fun deniesVirtualDomainsAndLegacyAddressSyntaxBeforeResolution() {
        listOf(null, "LOCALHOST.", "a.localhost", "router.local", "home.arpa", "router.home.arpa",
            "single-label", "app.xiangban.xyz", "a.APP.XIANGBAN.XYZ.", "127.1", "0x7f000001", "0177.0.0.1",
            "[::1]", "[::ffff:8.8.8.8]", "[fe80::1%wlan0]").forEach {
            assertTrue(it, RuntimeV2Proxy.isBlockedHost(it))
        }
        listOf("example.com", "chat.xiangban.xyz", "8.8.8.8", "[2606:4700::1111]").forEach {
            assertFalse(it, RuntimeV2Proxy.isBlockedHost(it))
        }
    }

    @Test fun loopbackListenerRejectsDisabledCleartextAndPrivateRequests() {
        val rejections = LinkedBlockingQueue<String>()
        RuntimeV2Proxy.onRejected = { rejections.add(it) }
        try {
            RuntimeV2Proxy.setNetworkEnabled(false)
            assertEquals("HTTP/1.1 403 Forbidden", status())
            assertEquals("disabled", rejections.poll(2, TimeUnit.SECONDS))
            RuntimeV2Proxy.setNetworkEnabled(true)
            assertEquals("HTTP/1.1 400 Bad Request", status("GET http://example.com/ HTTP/1.1\r\n\r\n"))
            assertEquals("invalid-request", rejections.poll(2, TimeUnit.SECONDS))
            assertEquals("HTTP/1.1 400 Bad Request", status("CONNECT example.com:8443 HTTP/1.1\r\nX: ${"x".repeat(8_192)}\r\n\r\n"))
            assertEquals("invalid-request", rejections.poll(2, TimeUnit.SECONDS))
            assertEquals("HTTP/1.1 403 Forbidden", status("CONNECT [::1]:8443 HTTP/1.1\r\nHost: [::1]:8443\r\n\r\n"))
            assertEquals("blocked-host", rejections.poll(2, TimeUnit.SECONDS))
        } finally {
            RuntimeV2Proxy.setNetworkEnabled(false)
            RuntimeV2Proxy.onRejected = null
        }
    }

    private fun status(request: String? = null): String? = Socket("127.0.0.1", RuntimeV2Proxy.port).use { socket ->
        socket.soTimeout = 2_000
        if (request != null) socket.getOutputStream().write(request.toByteArray(Charsets.US_ASCII))
        socket.getInputStream().bufferedReader(Charsets.US_ASCII).readLine()
    }
}
