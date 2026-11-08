package xyz.xiangban.xiaoban

import java.io.BufferedInputStream
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Proxy
import java.net.ServerSocket
import java.net.Socket
import java.net.SocketTimeoutException
import java.net.UnknownHostException
import java.util.Locale
import java.util.concurrent.ExecutionException
import java.util.concurrent.RejectedExecutionException
import java.util.concurrent.SynchronousQueue
import java.util.concurrent.ThreadPoolExecutor
import java.util.concurrent.TimeUnit
import java.util.concurrent.TimeoutException

/** A CONNECT-only loopback proxy for the isolated Android runtime process. */
internal object RuntimeV2Proxy {
    private const val MAX_CONNECTIONS = 32
    private const val HEADER_TIMEOUT_MS = 4_000
    private const val DNS_TIMEOUT_MS = 4_000L
    private const val CONNECT_TIMEOUT_MS = 8_000L
    private const val TLS_TIMEOUT_MS = 8_000
    private const val REAP_INTERVAL_MS = 15_000
    private val idleNanos = TimeUnit.MINUTES.toNanos(5)
    private val lock = Any()
    private var networkEnabled = false
    private val active = mutableSetOf<Tunnel>()
    private val workers = threadPool(MAX_CONNECTIONS, "runtime-v2-proxy")
    private val dnsWorkers = threadPool(4, "runtime-v2-dns")
    private val server by lazy {
        ServerSocket().apply {
            try {
                soTimeout = REAP_INTERVAL_MS
                bind(InetSocketAddress(InetAddress.getByAddress(byteArrayOf(127, 0, 0, 1)), 0), MAX_CONNECTIONS)
                daemon("runtime-v2-proxy-listener") { listen(this) }.start()
            } catch (e: IOException) {
                runCatching { close() }
                throw e
            }
        }
    }

    val port: Int get() = server.localPort

    @Volatile var onRejected: ((String) -> Unit)? = null

    fun setNetworkEnabled(enabled: Boolean) = synchronized(lock) {
        networkEnabled = enabled
        if (!enabled) {
            active.forEach(Tunnel::close)
            active.clear()
        }
    }

    fun isBlockedHost(host: String?): Boolean {
        val name = normalizedHost(host) ?: return true
        if (name == "app.xiangban.xyz" || name.endsWith(".app.xiangban.xyz") ||
            name == "home.arpa" || name.endsWith(".home.arpa") || name.endsWith(".localhost") ||
            name.endsWith(".local") || name.endsWith(".internal") || name.endsWith(".localdomain")) return true
        if (':' in name) {
            val ip = literalAddress(name) ?: return true
            return ip.address.size != 16 || !isPublicRuntimeAddress(ip)
        }
        if (name.substringAfterLast('.').matches(legacyIpv4Tail)) {
            return literalAddress(name)?.let { !isPublicRuntimeAddress(it) } ?: true
        }
        return '.' !in name || name.length > 253 || name.split('.').any { !it.matches(dnsLabel) }
    }

    private fun listen(listener: ServerSocket) {
        var lastReap = System.nanoTime()
        while (!listener.isClosed) {
            val now = System.nanoTime()
            if (now - lastReap >= TimeUnit.MILLISECONDS.toNanos(REAP_INTERVAL_MS.toLong())) {
                reapIdle(now)
                lastReap = now
            }
            val client = try {
                listener.accept()
            } catch (_: SocketTimeoutException) {
                continue
            } catch (_: IOException) {
                // This process should fail closed if its proxy listener stops working.
                runCatching { listener.close() }
                setNetworkEnabled(false)
                return
            }
            val tunnel = Tunnel(client)
            val denied = synchronized(lock) {
                when {
                    !networkEnabled -> "disabled"
                    active.size >= MAX_CONNECTIONS -> "busy"
                    else -> { active.add(tunnel); null }
                }
            }
            if (denied != null) {
                reject(client, denied, if (denied == "disabled") "403 Forbidden" else "503 Service Unavailable")
                tunnel.close()
                continue
            }
            try {
                workers.execute { handle(tunnel) }
            } catch (_: RejectedExecutionException) {
                reject(client, "busy", "503 Service Unavailable")
                close(tunnel)
            }
        }
    }

    private fun handle(tunnel: Tunnel) {
        val client = tunnel.client
        try {
            client.soTimeout = HEADER_TIMEOUT_MS
            val input = client.getInputStream().buffered()
            val target = try {
                readConnectTarget(input, deadline(HEADER_TIMEOUT_MS.toLong()))
            } catch (_: SocketTimeoutException) {
                null
            }
            if (target == null) return reject(client, "invalid-request", "400 Bad Request")
            if (isBlockedHost(target.host)) return reject(client, "blocked-host", "403 Forbidden")
            val host = checkNotNull(normalizedHost(target.host))
            val addresses = resolve(host) ?: return reject(client, "upstream-failure", "502 Bad Gateway")
            if (addresses.any { !isPublicRuntimeAddress(it) }) return reject(client, "blocked-address", "403 Forbidden")
            val upstream = dial(tunnel, addresses, target.port)
                ?: return reject(client, "upstream-failure", "502 Bad Gateway")
            client.getOutputStream().write(connectedResponse)
            client.soTimeout = TLS_TIMEOUT_MS
            val tls = try { hasClientHello(input, deadline(TLS_TIMEOUT_MS.toLong())) } catch (_: SocketTimeoutException) { false }
            if (!tls) {
                report("not-tls")
                return
            }
            client.soTimeout = 0
            relayBoth(tunnel, input, upstream)
        } catch (_: IOException) {
            // Socket closes and connection aborts are ordinary tunnel shutdown.
        } catch (_: InterruptedException) {
            Thread.currentThread().interrupt()
        } finally {
            close(tunnel)
        }
    }

    private fun resolve(host: String): Array<InetAddress>? {
        literalAddress(host)?.let { return arrayOf(it) }
        val lookup = try {
            // Absolute DNS name avoids treating an external host as a LAN search-domain alias.
            dnsWorkers.submit<Array<InetAddress>> { InetAddress.getAllByName("$host.") }
        } catch (_: RejectedExecutionException) {
            return null
        }
        return try {
            lookup.get(DNS_TIMEOUT_MS, TimeUnit.MILLISECONDS)
        } catch (_: ExecutionException) {
            null
        } catch (_: TimeoutException) {
            null
        } finally {
            lookup.cancel(true)
        }
    }

    private fun dial(tunnel: Tunnel, addresses: Array<InetAddress>, port: Int): Socket? {
        val expires = deadline(CONNECT_TIMEOUT_MS)
        for (address in addresses) {
            val remaining = TimeUnit.NANOSECONDS.toMillis(expires - System.nanoTime())
            if (remaining <= 0) break
            val upstream = Socket(Proxy.NO_PROXY)
            synchronized(lock) {
                if (tunnel !in active) {
                    runCatching { upstream.close() }
                    return null
                }
                tunnel.upstream = upstream
            }
            try {
                // This endpoint contains the validated address, so connect never resolves DNS again.
                upstream.connect(InetSocketAddress(address, port), remaining.coerceAtMost(2_500L).toInt())
                return upstream
            } catch (_: IOException) {
                runCatching { upstream.close() }
            }
        }
        return null
    }

    private fun relayBoth(tunnel: Tunnel, input: BufferedInputStream, upstream: Socket) {
        val client = tunnel.client
        val reverse = daemon("runtime-v2-proxy-reverse") {
            try {
                tunnel.relay(upstream.getInputStream(), client.getOutputStream())
                client.shutdownOutput()
            } catch (_: IOException) {
                close(tunnel)
            }
        }
        reverse.start()
        try {
            tunnel.relay(input, upstream.getOutputStream())
            upstream.shutdownOutput()
        } finally {
            if (!upstream.isOutputShutdown) close(tunnel)
            reverse.join()
        }
    }

    private fun reject(client: Socket, code: String, response: String) {
        if (client.isClosed) return
        report(code)
        runCatching { client.getOutputStream().write("HTTP/1.1 $response\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".toByteArray(Charsets.US_ASCII)) }
    }

    private fun report(code: String) { runCatching { onRejected?.invoke(code) } }

    private fun close(tunnel: Tunnel) = synchronized(lock) {
        active.remove(tunnel)
        tunnel.close()
    }

    private fun reapIdle(now: Long) = synchronized(lock) {
        active.removeAll { tunnel ->
            val idle = now - tunnel.lastActivity >= idleNanos
            if (idle) tunnel.close()
            idle
        }
    }

    private class Tunnel(val client: Socket) {
        var upstream: Socket? = null // Guarded by lock.
        @Volatile var lastActivity = System.nanoTime()
            private set

        fun relay(input: InputStream, output: OutputStream) {
            val buffer = ByteArray(16 * 1024)
            while (true) {
                val read = input.read(buffer)
                if (read == -1) return
                output.write(buffer, 0, read)
                lastActivity = System.nanoTime()
            }
        }

        fun close() {
            runCatching { client.close() }
            runCatching { upstream?.close() }
        }
    }
}

private val connectedResponse = "HTTP/1.1 200 Connection Established\r\n\r\n".toByteArray(Charsets.US_ASCII)
private val connectLine = Regex("CONNECT (\\[[0-9A-Fa-f:.]+]|[^\\s:]+):([0-9]{1,5}) HTTP/1\\.[01]")
private val headerName = Regex("[!#$%&'*+.^_`|~0-9A-Za-z-]+")
private val dnsLabel = Regex("[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?")
private val legacyIpv4Tail = Regex("[0-9]+|0x[0-9a-f]*")
private val ipv6Literal = Regex("[0-9a-f:.]+")
private const val MAX_HEADER_BYTES = 8192

private data class ConnectTarget(val host: String, val port: Int)

private fun readConnectTarget(input: BufferedInputStream, expires: Long): ConnectTarget? {
    var remaining = MAX_HEADER_BYTES
    fun readByte(): Int {
        if (remaining-- <= 0 || System.nanoTime() - expires >= 0) return -1
        return input.read()
    }
    fun readLine(): String? {
        val line = StringBuilder()
        while (true) {
            val byte = readByte()
            if (byte == 13) return line.toString().takeIf { readByte() == 10 }
            if (byte != 9 && byte !in 32..126) return null
            line.append(byte.toChar())
        }
    }
    val match = connectLine.matchEntire(readLine() ?: return null) ?: return null
    val port = match.groupValues[2].toInt().takeIf { it in 1..65_535 } ?: return null
    while (true) {
        val header = readLine() ?: return null
        if (header.isEmpty()) return ConnectTarget(match.groupValues[1], port)
        val name = header.substringBefore(':', "")
        if (!name.matches(headerName) || name.equals("Transfer-Encoding", ignoreCase = true)) return null
        if (name.equals("Content-Length", ignoreCase = true) && header.substringAfter(':').trim() != "0") return null
    }
}

private fun hasClientHello(input: BufferedInputStream, expires: Long): Boolean {
    val prefix = IntArray(6)
    input.mark(prefix.size + 1)
    for (index in prefix.indices) {
        if (System.nanoTime() - expires >= 0) return false
        prefix[index] = input.read()
        if (prefix[index] == -1) return false
    }
    input.reset()
    val length = (prefix[3] shl 8) or prefix[4]
    return prefix[0] == 22 && prefix[1] == 3 && prefix[2] in 1..3 && length in 1..16_384 && prefix[5] == 1
}

private fun normalizedHost(host: String?): String? = host?.lowercase(Locale.ROOT)?.removeSuffix(".")?.let {
    if (it.startsWith('[') && it.endsWith(']')) it.substring(1, it.length - 1) else it
}

private fun literalAddress(host: String): InetAddress? {
    if (':' in host) {
        if (!host.matches(ipv6Literal)) return null
        return try { InetAddress.getByName("[$host]") } catch (_: UnknownHostException) { null }
    }
    val parts = host.split('.', limit = 5)
    if (parts.size != 4) return null
    val bytes = ByteArray(4)
    for ((index, part) in parts.withIndex()) {
        val value = part.toIntOrNull() ?: return null
        if (value !in 0..255 || value.toString() != part) return null
        bytes[index] = value.toByte()
    }
    return InetAddress.getByAddress(bytes)
}

internal fun isPublicRuntimeAddress(address: InetAddress): Boolean {
    val bytes = address.address
    if (bytes.size == 4) return isPublicIpv4(bytes, 0)
    if (bytes.size != 16 || address.isAnyLocalAddress || address.isLoopbackAddress || address.isLinkLocalAddress) return false
    val octets = bytes.map { it.toInt() and 255 }
    val first = octets[0] shl 8 or octets[1]
    val second = octets[2] shl 8 or octets[3]
    val third = octets[4] shl 8 or octets[5]
    // RFC 6052's well-known prefix is safe only when its embedded IPv4 address is public.
    if (first == 0x0064 && second == 0xff9b && (4..11).all { octets[it] == 0 }) return isPublicIpv4(bytes, 12)
    if (first !in 0x2000..0x3fff) return false
    return when (first) {
        0x2001 -> second !in 0x0000..0x01ff && second != 0x0db8 // Includes Teredo and documentation.
        0x2002 -> false // 6to4 can embed nonpublic IPv4.
        0x2620 -> second != 0x004f || third != 0x8000
        0x3fff -> second !in 0x0000..0x0fff
        else -> true
    }
}

private fun isPublicIpv4(bytes: ByteArray, offset: Int): Boolean {
    val a = bytes[offset].toInt() and 255
    val b = bytes[offset + 1].toInt() and 255
    val c = bytes[offset + 2].toInt() and 255
    return when (a) {
        0, 10, 127, in 224..255 -> false
        100 -> b !in 64..127
        169 -> b != 254
        172 -> b !in 16..31
        192 -> when (b) {
            0 -> c != 0 && c != 2
            31 -> c != 196
            52 -> c != 193
            88 -> c != 99
            168 -> false
            175 -> c != 48
            else -> true
        }
        198 -> b != 18 && b != 19 && (b != 51 || c != 100)
        203 -> b != 0 || c != 113
        else -> true
    }
}

private fun deadline(ms: Long): Long = System.nanoTime() + TimeUnit.MILLISECONDS.toNanos(ms)

private fun daemon(name: String, run: () -> Unit) = Thread(run, name).apply { isDaemon = true }

private fun threadPool(max: Int, name: String) = ThreadPoolExecutor(
    0, max, 30, TimeUnit.SECONDS, SynchronousQueue(), { runnable -> daemon(name) { runnable.run() } },
)
