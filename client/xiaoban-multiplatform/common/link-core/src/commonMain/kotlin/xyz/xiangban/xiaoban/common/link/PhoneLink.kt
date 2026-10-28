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
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.longOrNull
import kotlinx.serialization.json.put
import kotlin.random.Random
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.JsonHttp

private val LinkJson = Json { ignoreUnknownKeys = true; explicitNulls = false }

/**
 * 手机端的互联客户端：扫码配对、保持到中转的 WebSocket 连接（断线自动重连）、加解密收发、重放防护。
 * 和 Rust 的 `xiaoban-link-client` 是同一份协议的另一端；[httpClient] 需要安装 WebSockets 插件。
 */
class PhoneLink(
    private val scope: CoroutineScope,
    private val httpClient: HttpClient,
    private val storage: LinkStorage,
    /** 当前可用的账号访问令牌，取不到返回 null；刷新由调用方负责。 */
    private val token: suspend () -> String?,
    private val deviceName: String,
    private val crypto: PlatformCrypto = platformCrypto(),
    private val backoffInitialMs: Long = 1_000,
    private val backoffMaxMs: Long = 30_000,
) {
    private val protocol = LinkProtocol(crypto)
    private val lock = Mutex()
    private var state: LinkState = LinkState()
    private val senders = HashMap<String, LinkSender>()
    private val guards = HashMap<String, ReplayGuard>()
    private val online = HashSet<String>()
    private var dirty = false
    private var out: Channel<String>? = null
    private var loop: Job? = null
    private var flusher: Job? = null
    private var stopped = false

    private val _events = MutableSharedFlow<LinkEvent>(extraBufferCapacity = 512)
    val events: SharedFlow<LinkEvent> = _events.asSharedFlow()

    /** 启动：读状态，启动计数加一并落盘，已配对过的话开始后台连接。 */
    suspend fun start() {
        lock.withLock {
            state = storage.load().let { it.copy(startCounter = it.startCounter + 1) }
            storage.save(state)
            state.peers.forEach { (peer, rec) ->
                guards[peer] = ReplayGuard.restore(rec.replayEp?.let { ReplayGuard.State(rec.replayCounter ?: 0, it, rec.replayLastSeq ?: -1) })
            }
        }
        stopped = false
        flusher = scope.launch { while (isActive) { delay(1_000); flush() } }
        if (hasPairing()) ensureLoop()
    }

    suspend fun stop() {
        stopped = true
        loop?.cancel()
        flusher?.cancel()
        out?.close()
        flush()
    }

    suspend fun peers(): List<PeerSummary> = lock.withLock {
        state.peers.values.map { PeerSummary(it.peerId, it.peerName, it.peerId in online) }.sortedBy { it.name }
    }

    private suspend fun hasPairing() = lock.withLock { state.device != null && state.relayBase != null && state.peers.isNotEmpty() }

    private suspend fun flush() = lock.withLock {
        if (dirty) {
            dirty = false
            state = state.copy(peers = state.peers.mapValues { (peer, rec) ->
                val g = guards[peer]?.state()
                if (g == null) rec else rec.copy(replayCounter = g.counter, replayEp = g.ep, replayLastSeq = g.lastSeq)
            })
            storage.save(state)
        }
    }

    // ───────────── REST ─────────────

    private suspend fun call(base: String, method: HttpMethod, path: String, body: JsonObject? = null): JsonObject {
        val tk = token() ?: throw ApiException(401, "请先登录")
        val text = JsonHttp(httpClient, base).raw(method, path, body?.toString(), tk)
        return if (text.isBlank()) JsonObject(emptyMap()) else Json.parseToJsonElement(text).jsonObject
    }

    private suspend fun ensureIdentity(base: String): DeviceIdentity {
        lock.withLock { state.device }?.let { return it }
        val pair = crypto.generateKeyPair()
        val resp = call(base, HttpMethod.Post, "/devices", buildJsonObject {
            put("kind", "phone"); put("name", deviceName); put("public_key", B64.encode(pair.publicKey))
        })
        val id = resp["device_id"]?.jsonPrimitive?.contentOrNull ?: throw ApiException(200, "服务返回了无法识别的内容")
        val identity = DeviceIdentity(id, deviceName, B64.encode(pair.privateKey), B64.encode(pair.publicKey))
        lock.withLock { state = state.copy(device = identity); storage.save(state) }
        return identity
    }

    /**
     * 扫码配对：核对桌面公钥 → 用二维码里的一次性密钥做证明 → 兑换 → 等桌面确认。
     * 桌面确认后建立配对并开始连接；超时或被拒会抛 [ApiException]。
     */
    suspend fun pair(qrText: String, confirmTimeoutMs: Long = 60_000): PeerSummary {
        val qr = QrPayload.decode(qrText)
        val base = qr.r.trimEnd('/')
        lock.withLock { if (state.relayBase != base) { state = state.copy(relayBase = base); storage.save(state) } }
        val me = ensureIdentity(base)
        val secret = qr.secret()
        val proof = protocol.makeProof(secret, me.id, qr.d, B64.decode(me.publicKey))
        val resp = call(base, HttpMethod.Post, "/pair", buildJsonObject {
            put("code", qr.c); put("device_id", me.id); put("proof", B64.encode(proof))
        })
        val pairingId = resp["pairing_id"]?.jsonPrimitive?.contentOrNull ?: throw ApiException(200, "服务返回了无法识别的内容")
        val desktop = resp["desktop"]?.jsonObject ?: throw ApiException(200, "服务返回了无法识别的内容")
        // 中转给的桌面公钥必须与二维码里的一致，否则中转可能在做手脚。
        if (!constantTimeEquals(B64.decode(desktop["public_key"]!!.jsonPrimitive.content), qr.desktopPublic())) throw LinkException(LinkError.BadKey, "桌面公钥与二维码不一致")
        val prk = protocol.pairingKey(B64.decode(me.privateKey), qr.desktopPublic(), secret)
        val name = desktop["name"]?.jsonPrimitive?.contentOrNull ?: "电脑"

        // 等桌面验证证明并确认：轮询配对列表，出现这条就是已激活。
        val deadline = confirmTimeoutMs
        var waited = 0L
        while (true) {
            val list = call(base, HttpMethod.Get, "/pairings?device_id=${me.id}&status=active")["items"]?.jsonArray.orEmpty()
            if (list.any { it.jsonObject["pairing_id"]?.jsonPrimitive?.contentOrNull == pairingId }) break
            if (waited >= deadline) throw ApiException(408, "桌面没有确认配对，请在电脑上重新生成二维码", "pairing_timeout")
            delay(500); waited += 500
        }
        lock.withLock {
            state = state.copy(peers = state.peers + (qr.d to PeerRecord(pairingId, qr.d, name, qr.k, B64.encode(prk))))
            storage.save(state)
            senders.remove(qr.d)
            guards[qr.d] = ReplayGuard()
        }
        ensureLoop()
        val summary = PeerSummary(qr.d, name, false)
        _events.emit(LinkEvent.Paired(summary))
        return summary
    }

    /** 解除配对：通知中转（对端会收到 unpaired），并清除本地密钥。 */
    suspend fun unpair(peer: String) {
        val (base, rec) = lock.withLock { state.relayBase to state.peers[peer] }
        if (base == null || rec == null) throw LinkException(LinkError.BadKey, "没有这个已配对的设备")
        try {
            call(base, HttpMethod.Delete, "/pairings/${rec.pairingId}")
        } catch (e: ApiException) {
            if (e.status != 404) throw e
        }
        forget(peer)
    }

    private suspend fun forget(peer: String) = lock.withLock {
        senders.remove(peer); guards.remove(peer); online.remove(peer)
        state = state.copy(peers = state.peers - peer)
        storage.save(state)
    }

    /** 给已配对的对端发一条明文（本类加密）。进入发送队列即返回；没送达会收到 [LinkEvent.Undelivered]。 */
    suspend fun send(peer: String, data: ByteArray, notify: Boolean = false) {
        val frame = lock.withLock {
            val me = state.device?.id ?: throw LinkException(LinkError.BadKey, "尚未登记设备")
            val rec = state.peers[peer] ?: throw LinkException(LinkError.BadKey, "没有这个已配对的设备")
            val sender = senders.getOrPut(peer) { LinkSender(protocol, B64.decode(rec.prk), me, peer, state.startCounter) }
            val env = sender.seal(data)
            buildJsonObject {
                put("op", "send"); put("env", LinkJson.encodeToJsonElement(Envelope.serializer(), env)); put("notify", notify)
            }.toString()
        }
        val channel = out ?: throw LinkException(LinkError.BadKey, "没有连上中转服务")
        if (channel.trySend(frame).isFailure) throw LinkException(LinkError.TooLarge, "发送队列已满")
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

    private fun wsUrl(base: String): String = when {
        base.startsWith("https://") -> "wss://" + base.removePrefix("https://")
        base.startsWith("http://") -> "ws://" + base.removePrefix("http://")
        else -> base
    }

    private suspend fun session(): SessionResult {
        val (base, device) = lock.withLock { state.relayBase to state.device }
        if (base == null || device == null) return SessionResult.NotConnected
        val tk = token() ?: return SessionResult.NotConnected
        val ws = try {
            httpClient.webSocketSession("${wsUrl(base)}/ws?device_id=${device.id}") {
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
            flush()
            val gone = lock.withLock { online.toList().also { online.clear() } }
            gone.forEach { _events.emit(LinkEvent.PeerOnline(it, false)) }
            _events.emit(LinkEvent.Disconnected)
        }
        return result
    }

    /** 返回 true 表示停止重连，false 表示结束本次连接后重试，null 继续。 */
    private suspend fun handleFrame(text: String): Boolean? {
        val v = runCatching { Json.parseToJsonElement(text).jsonObject }.getOrNull() ?: return null
        when (v["op"]?.jsonPrimitive?.contentOrNull) {
            "presence" -> {
                val peer = v["device"]?.jsonPrimitive?.contentOrNull ?: return null
                val on = v["online"]?.jsonPrimitive?.booleanOrNull ?: return null
                lock.withLock { if (on) online.add(peer) else online.remove(peer) }
                _events.emit(LinkEvent.PeerOnline(peer, on))
            }
            "deliver" -> {
                val env = runCatching { LinkJson.decodeFromJsonElement(Envelope.serializer(), v["env"]!!) }.getOrNull() ?: return null
                onDeliver(env)
            }
            "undelivered" -> _events.emit(LinkEvent.Undelivered(v["to"]?.jsonPrimitive?.contentOrNull ?: "", v["seq"]?.jsonPrimitive?.longOrNull ?: 0))
            "unpaired" -> {
                val peer = v["device"]?.jsonPrimitive?.contentOrNull ?: return null
                if (lock.withLock { peer in state.peers }) { forget(peer); _events.emit(LinkEvent.Unpaired(peer)) }
            }
            "error" -> when (val code = v["code"]?.jsonPrimitive?.contentOrNull ?: "") {
                "replaced" -> { _events.emit(LinkEvent.Replaced); return true }
                "device_revoked" -> {
                    lock.withLock { senders.clear(); guards.clear(); online.clear(); state = LinkState(relayBase = state.relayBase, startCounter = state.startCounter); storage.save(state) }
                    _events.emit(LinkEvent.Revoked)
                    return true
                }
                else -> _events.emit(LinkEvent.ServerError(code))
            }
        }
        return null
    }

    private suspend fun onDeliver(env: Envelope) {
        val (me, rec) = lock.withLock { state.device?.id to state.peers[env.from] }
        if (me == null || rec == null || env.to != me) return
        val plain = try {
            protocol.open(B64.decode(rec.prk), env.from, env.to, env)
        } catch (e: LinkException) {
            _events.emit(LinkEvent.Dropped(env.from, "decrypt:${e.error}")); return
        }
        try {
            lock.withLock { guards.getOrPut(env.from) { ReplayGuard() }.check(env); dirty = true }
        } catch (e: LinkException) {
            _events.emit(LinkEvent.Dropped(env.from, "replay:${e.error}")); return
        }
        _events.emit(LinkEvent.Message(env.from, plain))
    }
}
