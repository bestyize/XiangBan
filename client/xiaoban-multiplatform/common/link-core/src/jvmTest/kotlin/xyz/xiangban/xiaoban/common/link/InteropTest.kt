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
 * 跨语言互通测试：Kotlin 手机端 ⇄ 真实 server-link ⇄ Rust 桌面端（同一账号，无需配对）（libs/link-client 的 interop_desktop 示例）。
 * 只在设置了环境变量 LINK_INTEROP_BIN（示例的可执行文件路径）和 TEST_DATABASE_URL 时运行，平时自动跳过。
 */
class InteropTest {
    private val bin = System.getenv("LINK_INTEROP_BIN")?.takeIf { it.isNotBlank() }

    private class Desktop(val process: Process, val reader: BufferedReader, val base: String, val phoneToken: String, val desktopId: String)

    private fun startDesktop(): Desktop {
        val p = ProcessBuilder(bin!!).redirectErrorStream(false).start()
        val reader = p.inputStream.bufferedReader()
        val ready = generateSequence { reader.readLine() }.first { it.startsWith("READY ") }
        val json = Json.parseToJsonElement(ready.removePrefix("READY ")).jsonObject
        fun s(k: String) = json[k]!!.jsonPrimitive.content
        // 桌面端后续输出（事件、错误）转到测试日志，方便排查。
        Thread { reader.forEachLine { println("DESKTOP: $it") } }.apply { isDaemon = true }.start()
        Thread { p.errorStream.bufferedReader().forEachLine { println("DESKTOP-ERR: $it") } }.apply { isDaemon = true }.start()
        return Desktop(p, reader, s("base"), s("phone_token"), s("desktop_id"))
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

    private fun newLink(scope: CoroutineScope, client: HttpClient, desktop: Desktop, storage: LinkStorage) =
        PhoneLink(scope, client, storage, { desktop.phoneToken }, "互通测试手机", baseUrl = desktop.base, backoffInitialMs = 100, backoffMaxMs = 400)

    @Test
    fun kotlinPhoneSeesTheRustDesktopOfTheSameAccountAndTalksBothWays() {
        if (bin == null) { println("skipped: LINK_INTEROP_BIN is not set"); return }
        val desktop = startDesktop()
        val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
        val client = HttpClient(CIO) { install(WebSockets) }
        try {
            runBlocking {
                val storage = InMemoryLinkStorage()
                val link = newLink(scope, client, desktop, storage)
                val inbox = Channel<LinkEvent>(Channel.UNLIMITED)
                scope.launch { link.events.collect { inbox.send(it) } }
                link.start()

                // 1. 不需要配对：登录同一账号就能看到电脑。
                inbox.next("连接") { it is LinkEvent.Connected }
                val added = inbox.next("发现电脑") { it is LinkEvent.PeerAdded } as LinkEvent.PeerAdded
                assertEquals(desktop.desktopId, added.peer.id)

                // 2. 桌面 → 手机：电脑上线后主动问候。
                assertEquals("hello from desktop", text(inbox.next("问候") { it is LinkEvent.Message }))

                // 3. 手机 → 桌面 → 手机：往返，含中文和长消息。
                link.send(desktop.desktopId, "ping from kotlin".encodeToByteArray())
                assertEquals("ECHO:PING FROM KOTLIN", text(inbox.next("回显1") { it is LinkEvent.Message }))
                val long = "你好，小伴".repeat(100)
                link.send(desktop.desktopId, long.encodeToByteArray())
                assertEquals("ECHO:" + long.uppercase(), text(inbox.next("回显2") { it is LinkEvent.Message }))

                // 4. 重启手机端（同一份存储）：还是同一台设备，照常收发。
                link.stop()
                val deviceId = storage.load().device?.id
                assertTrue(deviceId != null)
                val link2 = newLink(scope, client, desktop, storage)
                val inbox2 = Channel<LinkEvent>(Channel.UNLIMITED)
                scope.launch { link2.events.collect { inbox2.send(it) } }
                link2.start()
                inbox2.next("重连") { it is LinkEvent.Connected }
                assertEquals(deviceId, storage.load().device?.id)
                inbox2.next("再次发现电脑") { it is LinkEvent.PeerAdded || (it is LinkEvent.PeerOnline && it.online) }
                link2.send(desktop.desktopId, "after restart".encodeToByteArray())
                assertEquals("ECHO:AFTER RESTART", text(inbox2.next("回显3") { it is LinkEvent.Message && it.data.decodeToString().startsWith("ECHO:") }))

                // 5. 从账号里移除这台电脑：手机不再能给它发消息。
                link2.removeDevice(desktop.desktopId)
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
