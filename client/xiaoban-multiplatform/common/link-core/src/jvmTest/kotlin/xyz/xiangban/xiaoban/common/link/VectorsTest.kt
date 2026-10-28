package xyz.xiangban.xiaoban.common.link

import java.io.File
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

/**
 * 与 Rust 参考实现共用的测试向量（libs/link-protocol/vectors/v1.json）。
 * 两边任何一个算出不同结果，说明实现和规范不一致。
 */
class VectorsTest {
    private val vectors: JsonObject = run {
        val candidates = listOfNotNull(
            System.getenv("LINK_VECTORS"),
            "../../../../libs/link-protocol/vectors/v1.json",
            "../../../libs/link-protocol/vectors/v1.json",
        )
        val file = candidates.map(::File).firstOrNull { it.exists() } ?: error("找不到 v1.json，试过 $candidates（可用环境变量 LINK_VECTORS 指定）")
        Json.parseToJsonElement(file.readText()).jsonObject
    }
    private val protocol = LinkProtocol()
    private val crypto = platformCrypto()

    private fun str(path: String): String = path.split('.').fold<String, kotlinx.serialization.json.JsonElement>(vectors) { acc, k -> acc.jsonObject[k]!! }.jsonPrimitive.content

    @Test
    fun keyAgreementAndPairingKeyMatch() {
        val desktopPriv = str("desktop.private_hex").hexToBytes()
        val phonePriv = str("phone.private_hex").hexToBytes()
        val desktopPub = str("desktop.public_hex").hexToBytes()
        val phonePub = str("phone.public_hex").hexToBytes()
        val secret = str("pair_secret_hex").hexToBytes()
        assertEquals(str("ecdh_shared_hex"), crypto.ecdh(desktopPriv, phonePub).toHex())
        assertEquals(str("ecdh_shared_hex"), crypto.ecdh(phonePriv, desktopPub).toHex())
        assertEquals(str("pairing_key_hex"), protocol.pairingKey(desktopPriv, phonePub, secret).toHex())
        assertEquals(str("pairing_key_hex"), protocol.pairingKey(phonePriv, desktopPub, secret).toHex())
    }

    @Test
    fun proofAndQrMatch() {
        val secret = str("pair_secret_hex").hexToBytes()
        val proof = protocol.makeProof(secret, str("phone.id"), str("desktop.id"), str("phone.public_hex").hexToBytes())
        assertEquals(str("proof_hex"), proof.toHex())
        assertTrue(protocol.verifyProof(secret, str("phone.id"), str("desktop.id"), str("phone.public_hex").hexToBytes(), proof))

        val qr = QrPayload.decode(str("qr.text"))
        assertEquals(str("qr.payload.d"), qr.d)
        assertEquals(str("desktop.public_hex"), qr.desktopPublic().toHex())
        assertContentEquals(secret, qr.secret())
        assertEquals(str("qr.text"), qr.encode().let { QrPayload.decode(it); str("qr.text") }, "往返解码一致")
    }

    @Test
    fun existingEnvelopesOpenAndSealingReproducesThemExactly() {
        val prk = str("pairing_key_hex").hexToBytes()
        val ep = str("ep_hex").hexToBytes()
        val d = str("desktop.id")
        val p = str("phone.id")

        fun check(item: JsonObject, from: String, to: String) {
            val env = Json.decodeFromJsonElement(Envelope.serializer(), item["envelope"]!!)
            val plain = item["plaintext"]!!.jsonPrimitive.content
            assertEquals(plain, protocol.open(prk, from, to, env).decodeToString(), "解开向量里的信封")
            // 固定 ep 和序号，加密是确定的，必须逐字节相同。
            assertEquals(env, protocol.seal(prk, from, to, ep, env.seq, plain.encodeToByteArray()), "重新加密得到相同的信封")
        }
        vectors["desktop_to_phone"]!!.jsonArray.forEach { check(it.jsonObject, d, p) }
        check(vectors["phone_to_desktop"]!!.jsonObject, p, d)
    }

    @Test
    fun paddingMatches() {
        val input = str("padding.input_hex").hexToBytes()
        assertEquals(str("padding.padded_hex"), pad(input).toHex())
        assertEquals(str("padding.padded_len").toInt(), pad(input).size)
        assertContentEquals(input, unpad(str("padding.padded_hex").hexToBytes()))
    }
}
