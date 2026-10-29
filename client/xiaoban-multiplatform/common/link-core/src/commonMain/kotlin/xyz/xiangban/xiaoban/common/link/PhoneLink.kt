package xyz.xiangban.xiaoban.common.link

import io.ktor.client.HttpClient
import io.ktor.client.plugins.websocket.webSocketSession
import io.ktor.client.request.header
import io.ktor.http.HttpMethod
import io.ktor.websocket.Frame
import io.ktor.websocket.close
import io.ktor.websocket.readText
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeoutOrNull
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import kotlin.random.Random
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.JsonHttp

const val DEFAULT_LINK_BASE = "https://link.xiangban.xyz/api/link"

/** 单条消息的字节上限，与中转一致。 */
const val MAX_LINK_MESSAGE_BYTES = 256 * 1024

private const val INSTALL_ID_ALPHABET = "abcdefghijklmnopqrstuvwxyz0123456789"

private fun randomInstallId(): String = buildString { repeat(32) { append(INSTALL_ID_ALPHABET[Random.nextInt(INSTALL_ID_ALPHABET.length)]) } }

/**
 * 手机端的互联客户端：用账号令牌向中转登记这台手机，保持一条 WebSocket 连接（断线自动重连），
 * 自动看到同一账号下的电脑并收发文本消息。没有配对、没有密钥。
 * 和 Rust 的 `xiaoban-link-client` 是同一份协议的另一端；[httpClient] 需要安装 WebSockets 插件。
 */
class PhoneLink(
    private val scope: CoroutineScope,
    private val httpClient: HttpClient,
    private val storage: LinkStorage,
    /** 当前可用的账号访问令牌，取不到返回 null；刷新由调用方负责。 */
    private val token: suspend () -> String?,
    private val deviceName: String,
    private val baseUrl: String = DEFAULT_LINK_BASE,
    private val backoffInitialMs: Long = 1_000,
    private val backoffMaxMs: Long = 30_000,
) {
    private val base = baseUrl.trimEnd('/')
    private val lock = Mutex()
    private var state: LinkState = LinkState()
    private val peerMap = LinkedHashMap<String, PeerSummary>()
    private var out: Channel<String>? = null
    private var loop: Job? = null
    @Volatile private var stopped = false

    private val _events = MutableSharedFlow<LinkEvent>(extraBufferCapacity = 512)
    val events: SharedFlow<LinkEvent> = _events.asSharedFlow()

    internal suspend fun emitEventForTest(event: LinkEvent) {
        _events.emit(event)
    }

    /** 测试用：每次 `send` 先通知它，方便检查发出去的内容（没有连上中转时也会通知）。 */
    internal var sendObserverForTest: ((peer: String, data: ByteArray) -> Unit)? = null

    internal suspend fun seedPeersForTest(peers: List<PeerSummary>) = lock.withLock {
        peerMap.clear()
        peers.forEach { peerMap[it.id] = it }
    }

    /** 启动：读状态（没有安装编号就生成并保存），开始后台连接。没登录时会一直重试，登录后自动连上。 */
    suspend fun start() {
        lock.withLock {
            state = storage.load().let { if (it.installId.isEmpty()) it.copy(installId = randomInstallId()) else it }
            storage.save(state)
        }
        stopped = false
        ensureLoop()
    }

    suspend fun stop() {
        stopped = true
        loop?.cancel()
        loop = null
        out?.close()
    }

    /** 同账号下的电脑，按名称排序。 */
    suspend fun peers(): List<PeerSummary> = lock.withLock { peerMap.values.sortedBy { it.name } }

    val deviceId: String? get() = state.device?.id

    // ───────────── REST ─────────────

    private suspend fun call(method: HttpMethod, path: String, body: JsonObject? = null): JsonObject {
        val tk = token() ?: throw ApiException(401, "请先登录")
        val text = JsonHttp(httpClient, base).raw(method, path, body?.toString(), tk)
        return if (text.isBlank()) JsonObject(emptyMap()) else Json.parseToJsonElement(text).jsonObject
    }

    private suspend fun ensureIdentity(): DeviceIdentity {
        lock.withLock { state.device }?.let { return it }
        val installId = lock.withLock { state.installId }
        val resp = call(HttpMethod.Post, "/devices", buildJsonObject {
            put("kind", "phone"); put("name", deviceName); put("install_id", installId)
        })
        val id = resp["device_id"]?.jsonPrimitive?.contentOrNull ?: throw ApiException(200, "服务返回了无法识别的内容")
        val identity = DeviceIdentity(id, deviceName)
        lock.withLock {
            // 登记期间如果别处已经保存了身份（并发启动），以先到的为准。
            state.device?.let { return it }
            state = state.copy(device = identity); storage.save(state)
        }
        return identity
    }

    /** 设备在中转被移除或换了账号：本地身份作废，下次连接重新登记（安装编号换新，避免又拿回被移除的）。 */
    private suspend fun wipeIdentity() {
        lock.withLock {
            peerMap.clear()
            state = LinkState(installId = randomInstallId())
            storage.save(state)
        }
        _events.emit(LinkEvent.Revoked)
    }

    /** 把一台电脑从账号的设备列表里移除（它再启动会重新出现）。 */
    suspend fun removeDevice(peer: String) {
        try {
            call(HttpMethod.Delete, "/devices/$peer")
        } catch (e: ApiException) {
            if (e.status != 404) throw e
        }
        if (lock.withLock { peerMap.remove(peer) } != null) _events.emit(LinkEvent.PeerRemoved(peer))
    }

    /** 给对端发一条文本消息（UTF-8，通常是 JSON）。进入发送队列即返回；没送达会收到 [LinkEvent.Undelivered]。 */
    suspend fun send(peer: String, data: ByteArray, notify: Boolean = false) {
        if (data.size > MAX_LINK_MESSAGE_BYTES) throw LinkException("消息太大")
        if (lock.withLock { peer !in peerMap }) throw LinkException("没有这台电脑")
        sendObserverForTest?.invoke(peer, data)
        val channel = out ?: throw LinkException("没有连上中转服务")
        val frame = buildJsonObject {
            put("op", "send"); put("to", peer); put("data", data.decodeToString()); put("notify", notify)
        }.toString()
        if (channel.trySend(frame).isFailure) throw LinkException("发送队列已满")
    }

    // ───────────── 连接 ─────────────

    private fun ensureLoop() {
        if (loop?.isActive == true) return
        loop = scope.launch {
            var backoff = backoffInitialMs
            while (isActive && !stopped) {
                val result = session()
                if (result == SessionResult.Stop) return@launch
                if (result == SessionResult.Connected) backoff = backoffInitialMs
                delay(backoff + Random.nextLong(0, (backoff / 4).coerceAtLeast(1)))
                backoff = (backoff * 2).coerceAtMost(backoffMaxMs)
            }
        }
    }

    private enum class SessionResult { NotConnected, Connected, Stop }

    private fun wsUrl(): String = when {
        base.startsWith("https://") -> "wss://" + base.removePrefix("https://")
        base.startsWith("http://") -> "ws://" + base.removePrefix("http://")
        else -> base
    }

    private fun parsePeer(v: JsonElement?): PeerSummary? {
        val o = v as? JsonObject ?: return null
        val id = o["id"]?.jsonPrimitive?.contentOrNull ?: return null
        return PeerSummary(id, o["name"]?.jsonPrimitive?.contentOrNull.orEmpty(), o["online"]?.jsonPrimitive?.booleanOrNull ?: false)
    }

    /** 用一份完整的对端列表替换本地的，并按差别发出出现 / 消失 / 上下线事件。 */
    private suspend fun applyPeers(fresh: Map<String, PeerSummary>) {
        val (added, removed, changed) = lock.withLock {
            val removed = peerMap.keys.filter { it !in fresh }
            val added = fresh.values.filter { it.id !in peerMap }
            val changed = fresh.values.filter { n -> peerMap[n.id]?.let { it.online != n.online } == true }
            peerMap.clear(); peerMap.putAll(fresh)
            Triple(added, removed, changed)
        }
        removed.forEach { _events.emit(LinkEvent.PeerRemoved(it)) }
        added.forEach { _events.emit(LinkEvent.PeerAdded(it)) }
        changed.forEach { _events.emit(LinkEvent.PeerOnline(it.id, it.online)) }
    }

    /** 主动向中转要一次对端列表（“重新查找”）。还没登记设备或没登录时什么也不做。 */
    suspend fun reloadPeers() {
        val device = lock.withLock { state.device } ?: return
        val resp = call(HttpMethod.Get, "/peers?device_id=${device.id}")
        val fresh = resp["items"]?.jsonArray?.mapNotNull { parsePeer(it) }?.associateBy { it.id } ?: return
        // 连接上时在线状态以连接为准（presence 帧更新更及时），这里只补充名单。
        applyPeers(if (out != null) fresh.mapValues { (id, p) -> peerMap[id]?.let { old -> p.copy(online = old.online || p.online) } ?: p } else fresh)
    }

    private suspend fun session(): SessionResult {
        val tk = token() ?: return SessionResult.NotConnected
        val device = try {
            ensureIdentity()
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (e: Exception) {
            return SessionResult.NotConnected
        }
        // 先确认中转还认这台设备：被移除或换了账号时，连接升级只会得到一个笼统的 404。
        try {
            call(HttpMethod.Get, "/peers?device_id=${device.id}")
        } catch (e: ApiException) {
            if (e.status == 404) wipeIdentity()
            return SessionResult.NotConnected
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (e: Exception) {
            return SessionResult.NotConnected
        }
        val ws = try {
            httpClient.webSocketSession("${wsUrl()}/ws?device_id=${device.id}") {
                header("Authorization", "Bearer $tk")
                header("Sec-WebSocket-Protocol", "xb-link")
            }
        } catch (e: Exception) {
            if (e is kotlinx.coroutines.CancellationException) throw e
            return SessionResult.NotConnected
        }
        val channel = Channel<String>(1024)
        out = channel
        val writer = scope.launch { for (frame in channel) ws.send(Frame.Text(frame)) }
        val pinger = scope.launch { while (isActive) { delay(25_000); channel.trySend("""{"op":"ping"}""") } }
        _events.emit(LinkEvent.Connected)
        var result = SessionResult.Connected
        try {
            while (true) {
                val frame = withTimeoutOrNull(90_000) { ws.incoming.receiveCatching() }?.getOrNull() ?: break
                if (frame is Frame.Text) {
                    val stop = handleFrame(frame.readText())
                    if (stop != null) { if (stop) result = SessionResult.Stop; break }
                } else if (frame is Frame.Close) break
            }
        } finally {
            out = null
            pinger.cancel()
            channel.close()
            writer.cancel()
            runCatching { ws.close() }
            val gone = lock.withLock {
                peerMap.values.filter { it.online }.map { it.id }.also { ids -> ids.forEach { peerMap[it] = peerMap.getValue(it).copy(online = false) } }
            }
            gone.forEach { _events.emit(LinkEvent.PeerOnline(it, false)) }
            _events.emit(LinkEvent.Disconnected)
        }
        return result
    }

    /** 返回 true 表示停止重连，false 表示结束本次连接后重试，null 继续。 */
    private suspend fun handleFrame(text: String): Boolean? {
        val v = runCatching { Json.parseToJsonElement(text).jsonObject }.getOrNull() ?: return null
        when (v["op"]?.jsonPrimitive?.contentOrNull) {
            "peers" -> {
                val fresh = v["items"]?.jsonArray?.mapNotNull { parsePeer(it) }?.associateBy { it.id } ?: return null
                applyPeers(fresh)
            }
            "peer.added" -> {
                val incoming = parsePeer(v["device"]) ?: return null
                val (merged, isNew, renamed, onlineChanged) = lock.withLock {
                    val old = peerMap[incoming.id]
                    val m = if (old != null) incoming.copy(online = old.online || incoming.online) else incoming
                    peerMap[m.id] = m
                    Quad(m, old == null, old != null && old.name != m.name, old != null && old.online != m.online)
                }
                if (isNew || renamed) _events.emit(LinkEvent.PeerAdded(merged))
                else if (onlineChanged) _events.emit(LinkEvent.PeerOnline(merged.id, merged.online))
            }
            "peer.removed" -> {
                val peer = v["device"]?.jsonPrimitive?.contentOrNull ?: return null
                if (lock.withLock { peerMap.remove(peer) } != null) _events.emit(LinkEvent.PeerRemoved(peer))
            }
            "presence" -> {
                val peer = v["device"]?.jsonPrimitive?.contentOrNull ?: return null
                val online = v["online"]?.jsonPrimitive?.booleanOrNull ?: return null
                lock.withLock { peerMap[peer]?.let { peerMap[peer] = it.copy(online = online) } }
                _events.emit(LinkEvent.PeerOnline(peer, online))
            }
            "deliver" -> {
                val from = v["from"]?.jsonPrimitive?.contentOrNull ?: return null
                val data = v["data"]?.jsonPrimitive?.contentOrNull ?: return null
                _events.emit(LinkEvent.Message(from, data.encodeToByteArray()))
            }
            "undelivered" -> _events.emit(LinkEvent.Undelivered(v["to"]?.jsonPrimitive?.contentOrNull.orEmpty()))
            "error" -> when (val code = v["code"]?.jsonPrimitive?.contentOrNull.orEmpty()) {
                "replaced" -> { _events.emit(LinkEvent.Replaced); return true }
                "device_revoked" -> { wipeIdentity(); return false }
                else -> _events.emit(LinkEvent.ServerError(code))
            }
        }
        return null
    }

    private data class Quad<A, B, C, D>(val a: A, val b: B, val c: C, val d: D)
}
