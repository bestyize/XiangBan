package xyz.xiangban.xiaoban.common.link

import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/*
 * 设备互联协议 v1 的 Kotlin 实现。规范：docs/agent/设备互联协议.md；参考实现：libs/link-protocol（Rust）。
 * 必须通过 libs/link-protocol/vectors/v1.json，改任何一行算法都要先改规范和向量。
 */

const val PROTOCOL_VERSION = 1
const val PAD_BLOCK = 256
const val MAX_CIPHERTEXT = 256 * 1024
const val QR_PREFIX = "xbl1:"

enum class LinkError { BadKey, BadEncoding, Decrypt, BadPadding, TooLarge, UnsupportedVersion, Replay, BadProof }

class LinkException(val error: LinkError, message: String = error.name) : Exception(message)

@OptIn(ExperimentalEncodingApi::class)
object B64 {
    private val codec = Base64.UrlSafe.withPadding(Base64.PaddingOption.ABSENT)
    fun encode(bytes: ByteArray): String = codec.encode(bytes)
    fun decode(text: String): ByteArray = try {
        codec.decode(text)
    } catch (e: IllegalArgumentException) {
        throw LinkException(LinkError.BadEncoding)
    }
}

fun ByteArray.toHex(): String = joinToString("") { (it.toInt() and 0xff).toString(16).padStart(2, '0') }

fun String.hexToBytes(): ByteArray {
    require(length % 2 == 0) { "十六进制长度必须是偶数" }
    return ByteArray(length / 2) { substring(it * 2, it * 2 + 2).toInt(16).toByte() }
}

/** 常量时间比较。 */
fun constantTimeEquals(a: ByteArray, b: ByteArray): Boolean {
    if (a.size != b.size) return false
    var diff = 0
    for (i in a.indices) diff = diff or (a[i].toInt() xor b[i].toInt())
    return diff == 0
}

@Serializable
data class Envelope(val v: Int, val from: String, val to: String, val ep: String, val seq: Long, val ct: String)

/** 二维码载荷：桌面显示，手机扫描。 */
@Serializable
data class QrPayload(
    /** 中转服务的 HTTP 地址（含路径前缀） */
    val r: String,
    /** 桌面设备编号 */
    val d: String,
    /** 一次性配对码 */
    val c: String,
    /** 桌面公钥 base64url */
    val k: String,
    /** 一次性密钥 base64url */
    val s: String,
) {
    fun desktopPublic(): ByteArray = B64.decode(k)
    fun secret(): ByteArray = B64.decode(s)

    fun encode(): String = QR_PREFIX + B64.encode(Json.encodeToString(serializer(), this).encodeToByteArray())

    companion object {
        fun decode(text: String): QrPayload {
            if (!text.startsWith(QR_PREFIX)) throw LinkException(LinkError.BadEncoding)
            val payload = try {
                Json { ignoreUnknownKeys = true }.decodeFromString(serializer(), B64.decode(text.removePrefix(QR_PREFIX)).decodeToString())
            } catch (e: LinkException) {
                throw e
            } catch (e: Exception) {
                throw LinkException(LinkError.BadEncoding)
            }
            if (payload.desktopPublic().size != 65 || payload.secret().size != 32) throw LinkException(LinkError.BadEncoding)
            return payload
        }
    }
}

/** ISO/IEC 7816-4 补齐到 [PAD_BLOCK] 的整数倍，隐藏真实长度。 */
fun pad(plain: ByteArray): ByteArray {
    var size = plain.size + 1
    if (size % PAD_BLOCK != 0) size += PAD_BLOCK - size % PAD_BLOCK
    return ByteArray(size).also {
        plain.copyInto(it)
        it[plain.size] = 0x80.toByte()
    }
}

fun unpad(padded: ByteArray): ByteArray {
    var end = padded.size
    while (end > 0 && padded[end - 1] == 0.toByte()) end--
    if (end == 0 || padded[end - 1] != 0x80.toByte()) throw LinkException(LinkError.BadPadding)
    return padded.copyOfRange(0, end - 1)
}

/** `ep` 前 8 字节是发送方持久化的启动计数（大端）。 */
fun epCounter(ep: ByteArray): Long {
    if (ep.size < 8) throw LinkException(LinkError.BadEncoding)
    var v = 0L
    for (i in 0 until 8) v = (v shl 8) or (ep[i].toLong() and 0xff)
    return v
}

class LinkProtocol(private val crypto: PlatformCrypto = platformCrypto()) {
    private fun hkdfExtract(salt: ByteArray, ikm: ByteArray) = crypto.hmacSha256(salt, ikm)

    private fun hkdfExpand(prk: ByteArray, info: ByteArray, length: Int): ByteArray {
        var previous = ByteArray(0)
        val out = ArrayList<Byte>()
        var counter = 1
        while (out.size < length) {
            previous = crypto.hmacSha256(prk, previous + info + byteArrayOf(counter.toByte()))
            out.addAll(previous.toList())
            counter++
        }
        return out.take(length).toByteArray()
    }

    /** 配对密钥 PRK = HKDF-Extract(salt = 一次性密钥, ikm = ECDH 结果)。 */
    fun pairingKey(ownPrivate: ByteArray, peerPublic: ByteArray, secret: ByteArray): ByteArray =
        hkdfExtract(secret, crypto.ecdh(ownPrivate, peerPublic))

    private fun proofInput(phoneId: String, desktopId: String, phonePublic: ByteArray): ByteArray =
        "xb-link/1 pair-proof".encodeToByteArray() + 0 + phoneId.encodeToByteArray() + 0 + desktopId.encodeToByteArray() + 0 + phonePublic

    /** 手机用二维码里的一次性密钥证明“这个公钥是扫了码的人给的”。 */
    fun makeProof(secret: ByteArray, phoneId: String, desktopId: String, phonePublic: ByteArray): ByteArray =
        crypto.hmacSha256(secret, proofInput(phoneId, desktopId, phonePublic))

    fun verifyProof(secret: ByteArray, phoneId: String, desktopId: String, phonePublic: ByteArray, proof: ByteArray): Boolean =
        constantTimeEquals(makeProof(secret, phoneId, desktopId, phonePublic), proof)

    private fun messageKey(prk: ByteArray, ep: ByteArray, from: String, to: String): ByteArray {
        val prkE = hkdfExtract(ep, prk)
        return hkdfExpand(prkE, "xb-link/1 key $from>$to".encodeToByteArray(), 32)
    }

    private fun nonce(seq: Long): ByteArray = ByteArray(12).also { n ->
        for (i in 0 until 8) n[11 - i] = (seq ushr (8 * i)).toByte()
    }

    private fun aad(from: String, to: String, ep: ByteArray, seq: Long): ByteArray =
        "xb-link/1".encodeToByteArray() + 0 + from.encodeToByteArray() + 0 + to.encodeToByteArray() + 0 + ep + nonce(seq).copyOfRange(4, 12)

    /** 新的会话标识：持久化的启动计数（已加一并落盘）加 8 字节随机数。 */
    fun makeEp(startCounter: Long): ByteArray {
        val counter = ByteArray(8) { (startCounter ushr (8 * (7 - it))).toByte() }
        return counter + crypto.randomBytes(8)
    }

    fun seal(prk: ByteArray, from: String, to: String, ep: ByteArray, seq: Long, plain: ByteArray): Envelope {
        val padded = pad(plain)
        if (padded.size + 16 > MAX_CIPHERTEXT) throw LinkException(LinkError.TooLarge)
        val sealed = crypto.aesGcmSeal(messageKey(prk, ep, from, to), nonce(seq), aad(from, to, ep, seq), padded)
        return Envelope(PROTOCOL_VERSION, from, to, B64.encode(ep), seq, B64.encode(sealed))
    }

    /** 解开信封；[expectedFrom]/[expectedTo] 是接收方认定的配对双方，信封里写的必须一致。 */
    fun open(prk: ByteArray, expectedFrom: String, expectedTo: String, env: Envelope): ByteArray {
        if (env.v != PROTOCOL_VERSION) throw LinkException(LinkError.UnsupportedVersion, "version ${env.v}")
        if (env.from != expectedFrom || env.to != expectedTo) throw LinkException(LinkError.Decrypt)
        val ep = B64.decode(env.ep)
        if (ep.size != 16) throw LinkException(LinkError.BadEncoding)
        val sealed = B64.decode(env.ct)
        if (sealed.size > MAX_CIPHERTEXT) throw LinkException(LinkError.TooLarge)
        val padded = crypto.aesGcmOpen(messageKey(prk, ep, env.from, env.to), nonce(env.seq), aad(env.from, env.to, ep, env.seq), sealed)
            ?: throw LinkException(LinkError.Decrypt)
        return unpad(padded)
    }
}

/** 发送方：每次启动用已加一并落盘的启动计数创建，序号从 0 递增。 */
class LinkSender(private val protocol: LinkProtocol, private val prk: ByteArray, private val from: String, private val to: String, startCounter: Long, ep: ByteArray? = null) {
    private val epBytes = ep ?: protocol.makeEp(startCounter)
    private var next = 0L

    @Synchronized
    fun seal(plain: ByteArray): Envelope = protocol.seal(prk, from, to, epBytes, next, plain).also { next++ }
}

/**
 * 接收端重放防护。同一会话内序号必须严格递增（允许跳号）；启动计数更大的是对方重启后的新会话；更小的是旧会话，拒绝。
 * [state] 必须持久化，重启后用 [restore] 恢复。
 */
class ReplayGuard private constructor(private var state: State?) {
    data class State(val counter: Long, val ep: String, val lastSeq: Long)

    constructor() : this(null)

    fun state(): State? = state

    /** 在信封**解密成功之后**调用。通过则记下；拒绝时抛 [LinkError.Replay]，状态不变。 */
    @Synchronized
    fun check(env: Envelope) {
        val counter = epCounter(B64.decode(env.ep))
        val cur = state
        when {
            cur == null || counter > cur.counter -> {}
            counter == cur.counter -> if (cur.ep != env.ep || env.seq <= cur.lastSeq) throw LinkException(LinkError.Replay)
            else -> throw LinkException(LinkError.Replay)
        }
        state = State(counter, env.ep, env.seq)
    }

    companion object {
        fun restore(state: State?) = ReplayGuard(state)
    }
}
