package xyz.xiangban.xiaoban.common.link

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

/** 协议规则的测试，只用 [LinkProtocol] 自己的收发，不依赖固定向量（向量在 jvmTest）。 */
class ProtocolRulesTest {
    private val protocol = LinkProtocol()
    private val crypto = platformCrypto()
    private val d = "dsk_a"
    private val p = "phn_a"

    private fun pairedKey(): ByteArray {
        val desktop = crypto.generateKeyPair()
        val phone = crypto.generateKeyPair()
        val secret = crypto.randomBytes(32)
        val a = protocol.pairingKey(desktop.privateKey, phone.publicKey, secret)
        val b = protocol.pairingKey(phone.privateKey, desktop.publicKey, secret)
        assertContentEquals(a, b, "两端算出同一个配对密钥")
        return a
    }

    private fun err(block: () -> Unit): LinkError = assertFailsWith<LinkException> { block() }.error

    @Test
    fun roundtripAndDirectionBinding() {
        val prk = pairedKey()
        val env = LinkSender(protocol, prk, d, p, 1).seal("你好，小伴".encodeToByteArray())
        assertEquals("你好，小伴", protocol.open(prk, d, p, env).decodeToString())
        val back = LinkSender(protocol, prk, p, d, 1).seal("hi".encodeToByteArray())
        assertEquals(LinkError.Decrypt, err { protocol.open(prk, d, p, back) })
        assertEquals("hi", protocol.open(prk, p, d, back).decodeToString())
    }

    @Test
    fun tamperingWithAnyVisibleFieldFails() {
        val prk = pairedKey()
        val env = LinkSender(protocol, prk, d, p, 1).seal("secret".encodeToByteArray())
        listOf(
            env.copy(seq = env.seq + 1),
            env.copy(from = "dsk_evil"),
            env.copy(to = "phn_evil"),
            env.copy(ep = B64.encode(B64.decode(env.ep).also { it[15] = (it[15].toInt() xor 1).toByte() })),
            env.copy(ct = B64.encode(B64.decode(env.ct).also { it[0] = (it[0].toInt() xor 1).toByte() })),
        ).forEach { assertTrue(runCatching { protocol.open(prk, d, p, it) }.isFailure) }
        assertEquals(LinkError.UnsupportedVersion, err { protocol.open(prk, d, p, env.copy(v = 2)) })
    }

    @Test
    fun paddingHidesLength() {
        val prk = pairedKey()
        val tx = LinkSender(protocol, prk, d, p, 1)
        val short = tx.seal("a".encodeToByteArray())
        val mid = tx.seal(ByteArray(200) { 'b'.code.toByte() })
        val big = tx.seal(ByteArray(300) { 'c'.code.toByte() })
        assertEquals(short.ct.length, mid.ct.length)
        assertTrue(big.ct.length > short.ct.length)
        assertEquals(300, protocol.open(prk, d, p, big).size)
        assertContentEquals(ByteArray(0), unpad(pad(ByteArray(0))))
        assertEquals(LinkError.BadPadding, err { unpad(byteArrayOf(0, 0)) })
        assertEquals(LinkError.TooLarge, err { LinkSender(protocol, prk, d, p, 1).seal(ByteArray(300 * 1024)) })
    }

    @Test
    fun proofBindsIdentityAndKey() {
        val phone = crypto.generateKeyPair()
        val secret = crypto.randomBytes(32)
        val proof = protocol.makeProof(secret, p, d, phone.publicKey)
        assertTrue(protocol.verifyProof(secret, p, d, phone.publicKey, proof))
        assertTrue(!protocol.verifyProof(secret, p, d, crypto.generateKeyPair().publicKey, proof), "换成别人的公钥")
        assertTrue(!protocol.verifyProof(secret, "phn_other", d, phone.publicKey, proof))
        assertTrue(!protocol.verifyProof(ByteArray(32), p, d, phone.publicKey, proof))
    }

    @Test
    fun badPublicKeysAreRejected() {
        val mine = crypto.generateKeyPair()
        assertEquals(LinkError.BadKey, err { crypto.ecdh(mine.privateKey, ByteArray(65)) })
        assertEquals(LinkError.BadKey, err { crypto.ecdh(mine.privateKey, ByteArray(10) { 4 }) })
        // 不在曲线上的点
        assertEquals(LinkError.BadKey, err { crypto.ecdh(mine.privateKey, byteArrayOf(4) + ByteArray(64) { 1 }) })
    }

    @Test
    fun qrRoundtripAndValidation() {
        val desktop = crypto.generateKeyPair()
        val qr = QrPayload("https://x/api/link", d, "CODE", B64.encode(desktop.publicKey), B64.encode(crypto.randomBytes(32)))
        val text = qr.encode()
        assertTrue(text.startsWith("xbl1:"))
        assertEquals(qr, QrPayload.decode(text))
        assertEquals(LinkError.BadEncoding, err { QrPayload.decode("xbl2:abc") })
        assertEquals(LinkError.BadEncoding, err { QrPayload.decode(qr.copy(s = B64.encode(ByteArray(8))).encode()) })
        assertEquals(LinkError.BadEncoding, err { QrPayload.decode("xbl1:@@@") })
    }

    @Test
    fun replayGuardRules() {
        val prk = pairedKey()
        val tx = LinkSender(protocol, prk, d, p, 5)
        val e0 = tx.seal(byteArrayOf(0)); val e1 = tx.seal(byteArrayOf(1)); val e2 = tx.seal(byteArrayOf(2))
        val g = ReplayGuard()
        g.check(e0)
        g.check(e2)
        assertEquals(LinkError.Replay, err { g.check(e1) }, "回头的序号")
        assertEquals(LinkError.Replay, err { g.check(e2) }, "同一条重放")

        val n0 = LinkSender(protocol, prk, d, p, 6).seal(byteArrayOf(9))
        g.check(n0)
        assertEquals(LinkError.Replay, err { g.check(e2) }, "旧会话进不来")

        val g2 = ReplayGuard.restore(g.state())
        assertEquals(LinkError.Replay, err { g2.check(e0) })
        assertEquals(LinkError.Replay, err { g2.check(n0) })
        val clash = LinkSender(protocol, prk, d, p, 6).seal(byteArrayOf(7))
        assertNotEquals(n0.ep, clash.ep)
        assertEquals(LinkError.Replay, err { g2.check(clash) }, "启动计数相同而 ep 不同")
    }
}
