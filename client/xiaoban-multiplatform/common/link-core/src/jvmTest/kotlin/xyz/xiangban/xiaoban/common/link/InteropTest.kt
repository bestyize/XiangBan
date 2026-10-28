package xyz.xiangban.xiaoban.common.link

import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.websocket.WebSockets
import java.io.BufferedReader
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

/**
 * 跨语言互通测试：Kotlin 手机端 ⇄ 真实 server-link ⇄ Rust 桌面端（libs/link-client 的 interop_desktop 示例）。
 * 只在设置了环境变量 LINK_INTEROP_BIN（示例的可执行文件路径）和 TEST_DATABASE_URL 时运行，平时自动跳过。
 */
class InteropTest {
    private val bin = System.getenv("LINK_INTEROP_BIN")?.takeIf { it.isNotBlank() }

    private class Desktop(val process: Process, val reader: BufferedReader, val base: String, val phoneToken: String, val qr: String, val desktopId: String)

    private fun startDesktop(): Desktop {
        val p = ProcessBuilder(bin!!).redirectErrorStream(false).start()
        val reader = p.inputStream.bufferedReader()
        val ready = generateSequence { reader.readLine() }.first { it.startsWith("READY ") }
        val json = Json.parseToJsonElement(ready.removePrefix("READY ")).jsonObject
        fun s(k: String) = json[k]!!.jsonPrimitive.content
        // 桌面端后续输出（事件、错误）转到测试日志，方便排查。
        Thread { reader.forEachLine { println("DESKTOP: $it") } }.apply { isDaemon = true }.start()
        Thread { p.errorStream.bufferedReader().forEachLine { println("DESKTOP-ERR: $it") } }.apply { isDaemon = true }.start()
        return Desktop(p, reader, s("base"), s("phone_token"), s("qr"), s("desktop_id"))
    }

    private suspend fun Channel<LinkEvent>.next(label: String, pred: (LinkEvent) -> Boolean): LinkEvent =
        withTimeout(20_000) {
            while (true) {
                val e = receive()
                if (pred(e)) return@withTimeout e
            }
            @Suppress("UNREACHABLE_CODE") error(label)
        }

    private fun text(e: LinkEvent) = (e as LinkEvent.Message).data.decodeToString()

    @Test
    fun kotlinPhonePairsWithRustDesktopAndTalksBothWays() {
        if (bin == null) { println("skipped: LINK_INTEROP_BIN is not set"); return }
        val desktop = startDesktop()
        val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
        val client = HttpClient(CIO) { install(WebSockets) }
        try {
            runBlocking {
                val storage = InMemoryLinkStorage()
                val link = PhoneLink(scope, client, storage, { desktop.phoneToken }, "互通测试手机", backoffInitialMs = 100, backoffMaxMs = 400)
                val inbox = Channel<LinkEvent>(Channel.UNLIMITED)
                scope.launch { link.events.collect { inbox.send(it) } }
                link.start()

                // 1. 扫码配对：证明由 Kotlin 算，Rust 桌面验证并确认。
                val peer = link.pair(desktop.qr)
                assertEquals(desktop.desktopId, peer.id)
                inbox.next("连接") { it is LinkEvent.Connected }

                // 2. 桌面 → 手机：Rust 加密，Kotlin 解密。
                assertEquals("hello from desktop", text(inbox.next("问候") { it is LinkEvent.Message }))

                // 3. 手机 → 桌面 → 手机：往返，含中文和长消息（补齐）。
                link.send(desktop.desktopId, "ping from kotlin".encodeToByteArray())
                assertEquals("ECHO:PING FROM KOTLIN", text(inbox.next("回显1") { it is LinkEvent.Message }))
                val long = "你好，小伴".repeat(100)
                link.send(desktop.desktopId, long.encodeToByteArray())
                assertEquals("ECHO:" + long.uppercase(), text(inbox.next("回显2") { it is LinkEvent.Message }))

                // 4. 重启手机端（同一存储）：配对恢复、新会话被桌面接受。
                link.stop()
                val saved = storage.load()
                assertEquals(1, saved.startCounter)
                assertTrue(saved.peers.containsKey(desktop.desktopId))
                val link2 = PhoneLink(scope, client, storage, { desktop.phoneToken }, "互通测试手机", backoffInitialMs = 100, backoffMaxMs = 400)
                val inbox2 = Channel<LinkEvent>(Channel.UNLIMITED)
                scope.launch { link2.events.collect { inbox2.send(it) } }
                link2.start()
                assertEquals(2, storage.load().startCounter)
                inbox2.next("重连") { it is LinkEvent.Connected }
                assertEquals("hello from desktop", text(inbox2.next("再次问候") { it is LinkEvent.Message }))
                link2.send(desktop.desktopId, "after restart".encodeToByteArray())
                assertEquals("ECHO:AFTER RESTART", text(inbox2.next("回显3") { it is LinkEvent.Message }))

                // 5. 解除配对：之后手机发不出去。
                link2.unpair(desktop.desktopId)
                assertTrue(runCatching { link2.send(desktop.desktopId, "x".encodeToByteArray()) }.isFailure)
                assertTrue(link2.peers().isEmpty())
                link2.stop()
            }
        } finally {
            scope.cancel()
            client.close()
            desktop.process.destroy()
        }
    }
}
