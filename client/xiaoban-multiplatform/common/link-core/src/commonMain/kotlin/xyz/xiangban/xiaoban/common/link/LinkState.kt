package xyz.xiangban.xiaoban.common.link

import kotlinx.serialization.Serializable

/** 同账号手机与电脑互联用到的错误（本地可预期的失败，不涉及密钥）。 */
class LinkException(message: String) : Exception(message)

@Serializable
data class DeviceIdentity(
    /** 中转分配的设备编号 */
    val id: String,
    val name: String,
)

/**
 * 需要长期保存的状态：没有任何密钥，丢了只是重新登记成一台新设备。
 * 在中转登记设备时带上 [installId]，重复登记会拿回同一个设备编号。
 */
@Serializable
data class LinkState(
    val installId: String = "",
    val device: DeviceIdentity? = null,
)

/** 存哪里由平台决定：Android 用应用私有存储，测试用内存。 */
interface LinkStorage {
    fun load(): LinkState
    fun save(state: LinkState)
}

class InMemoryLinkStorage(private var state: LinkState = LinkState()) : LinkStorage {
    override fun load() = state
    override fun save(state: LinkState) { this.state = state }
}

/** 同账号下的一台对端设备（手机的对端是电脑）。 */
data class PeerSummary(val id: String, val name: String, val online: Boolean)

sealed interface LinkEvent {
    data object Connected : LinkEvent
    data object Disconnected : LinkEvent
    data class PeerOnline(val peer: String, val online: Boolean) : LinkEvent
    /** 同账号下多了一台电脑（或它改了名字）。 */
    data class PeerAdded(val peer: PeerSummary) : LinkEvent
    data class PeerRemoved(val peer: String) : LinkEvent
    /** 对端发来的文本消息。 */
    class Message(val peer: String, val data: ByteArray) : LinkEvent
    /** 中转告知没送达（对端离线且没标通知）。 */
    data class Undelivered(val peer: String) : LinkEvent
    data class ServerError(val code: String) : LinkEvent
    /** 同一设备在别处连接，本连接被顶替，不再自动重连。 */
    data object Replaced : LinkEvent
    /** 设备在中转被移除（或换了账号），本地身份已清除，会自动重新登记成新设备。 */
    data object Revoked : LinkEvent
}
