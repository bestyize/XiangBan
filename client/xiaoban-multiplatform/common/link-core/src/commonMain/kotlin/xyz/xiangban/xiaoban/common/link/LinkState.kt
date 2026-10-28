package xyz.xiangban.xiaoban.common.link

import kotlinx.serialization.Serializable

@Serializable
data class DeviceIdentity(
    val id: String,
    val name: String,
    /** 32 字节私钥，base64url。正式版应由 Keystore 保护的密钥加密后再存。 */
    val privateKey: String,
    val publicKey: String,
)

@Serializable
data class PeerRecord(
    val pairingId: String,
    val peerId: String,
    val peerName: String,
    val peerPublic: String,
    /** 配对密钥 PRK，base64url */
    val prk: String,
    /** 重放防护进度 */
    val replayCounter: Long? = null,
    val replayEp: String? = null,
    val replayLastSeq: Long? = null,
)

/** 手机端需要长期保存的互联状态。 */
@Serializable
data class LinkState(
    /** 中转服务的 HTTP 地址（来自二维码） */
    val relayBase: String? = null,
    val device: DeviceIdentity? = null,
    /** 每次启动加一后落盘，用于生成 `ep`（协议 §2.3）。 */
    val startCounter: Long = 0,
    val peers: Map<String, PeerRecord> = emptyMap(),
)

/** 存哪里由平台决定：Android 用应用私有存储并用 Keystore 保护私钥与配对密钥，测试用内存。 */
interface LinkStorage {
    fun load(): LinkState
    fun save(state: LinkState)
}

class InMemoryLinkStorage(private var state: LinkState = LinkState()) : LinkStorage {
    override fun load() = state
    override fun save(state: LinkState) { this.state = state }
}

data class PeerSummary(val id: String, val name: String, val online: Boolean)

sealed interface LinkEvent {
    data object Connected : LinkEvent
    data object Disconnected : LinkEvent
    data class PeerOnline(val peer: String, val online: Boolean) : LinkEvent
    data class Paired(val peer: PeerSummary) : LinkEvent
    data class Unpaired(val peer: String) : LinkEvent
    /** 对端发来的明文（已解密、已通过重放检查）。 */
    class Message(val peer: String, val data: ByteArray) : LinkEvent
    data class Undelivered(val peer: String, val seq: Long) : LinkEvent
    /** 收到的信封被丢弃：解密失败或重放。 */
    data class Dropped(val peer: String, val reason: String) : LinkEvent
    data class ServerError(val code: String) : LinkEvent
    /** 同一设备在别处连接，本连接被顶替，不再自动重连。 */
    data object Replaced : LinkEvent
    /** 设备在中转被撤销，本地配对已清除。 */
    data object Revoked : LinkEvent
}
