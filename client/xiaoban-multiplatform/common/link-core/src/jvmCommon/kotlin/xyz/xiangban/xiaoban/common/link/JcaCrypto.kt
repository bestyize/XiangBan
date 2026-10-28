package xyz.xiangban.xiaoban.common.link

import java.math.BigInteger
import java.security.AlgorithmParameters
import java.security.KeyFactory
import java.security.KeyPairGenerator
import java.security.SecureRandom
import java.security.interfaces.ECPrivateKey
import java.security.interfaces.ECPublicKey
import java.security.spec.ECGenParameterSpec
import java.security.spec.ECParameterSpec
import java.security.spec.ECPoint
import java.security.spec.ECPrivateKeySpec
import java.security.spec.ECPublicKeySpec
import javax.crypto.AEADBadTagException
import javax.crypto.Cipher
import javax.crypto.KeyAgreement
import javax.crypto.Mac
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec

/** 用 java.security / javax.crypto 实现（Android 24+ 与桌面 JVM 通用）。 */
class JcaCrypto : PlatformCrypto {
    private val random = SecureRandom()
    private val curve: ECParameterSpec by lazy {
        AlgorithmParameters.getInstance("EC").apply { init(ECGenParameterSpec("secp256r1")) }.getParameterSpec(ECParameterSpec::class.java)
    }

    private fun fixed32(value: BigInteger): ByteArray {
        val raw = value.toByteArray().let { if (it.size > 1 && it[0] == 0.toByte()) it.copyOfRange(1, it.size) else it }
        require(raw.size <= 32) { "超过 32 字节" }
        return ByteArray(32 - raw.size) + raw
    }

    override fun generateKeyPair(): KeyPairBytes {
        val gen = KeyPairGenerator.getInstance("EC").apply { initialize(ECGenParameterSpec("secp256r1"), random) }
        val pair = gen.generateKeyPair()
        val priv = pair.private as ECPrivateKey
        val pub = (pair.public as ECPublicKey).w
        return KeyPairBytes(fixed32(priv.s), byteArrayOf(4) + fixed32(pub.affineX) + fixed32(pub.affineY))
    }

    override fun ecdh(privateKey: ByteArray, peerPublic: ByteArray): ByteArray {
        if (privateKey.size != 32) throw LinkException(LinkError.BadKey)
        if (peerPublic.size != 65 || peerPublic[0] != 4.toByte()) throw LinkException(LinkError.BadKey)
        try {
            val factory = KeyFactory.getInstance("EC")
            val priv = factory.generatePrivate(ECPrivateKeySpec(BigInteger(1, privateKey), curve))
            val point = ECPoint(BigInteger(1, peerPublic.copyOfRange(1, 33)), BigInteger(1, peerPublic.copyOfRange(33, 65)))
            val pub = factory.generatePublic(ECPublicKeySpec(point, curve))
            return KeyAgreement.getInstance("ECDH").run {
                init(priv)
                doPhase(pub, true)
                generateSecret()
            }
        } catch (e: LinkException) {
            throw e
        } catch (e: Exception) {
            // 点不在曲线上、私钥为 0 等。
            throw LinkException(LinkError.BadKey)
        }
    }

    override fun hmacSha256(key: ByteArray, data: ByteArray): ByteArray =
        Mac.getInstance("HmacSHA256").run {
            // 空密钥在 SecretKeySpec 里不允许，HMAC 里等价于全零密钥。
            init(SecretKeySpec(if (key.isEmpty()) ByteArray(32) else key, "HmacSHA256"))
            doFinal(data)
        }

    override fun aesGcmSeal(key: ByteArray, nonce: ByteArray, aad: ByteArray, plain: ByteArray): ByteArray =
        Cipher.getInstance("AES/GCM/NoPadding").run {
            init(Cipher.ENCRYPT_MODE, SecretKeySpec(key, "AES"), GCMParameterSpec(128, nonce))
            updateAAD(aad)
            doFinal(plain)
        }

    override fun aesGcmOpen(key: ByteArray, nonce: ByteArray, aad: ByteArray, sealed: ByteArray): ByteArray? =
        try {
            Cipher.getInstance("AES/GCM/NoPadding").run {
                init(Cipher.DECRYPT_MODE, SecretKeySpec(key, "AES"), GCMParameterSpec(128, nonce))
                updateAAD(aad)
                doFinal(sealed)
            }
        } catch (e: AEADBadTagException) {
            null
        } catch (e: javax.crypto.BadPaddingException) {
            null
        }

    override fun randomBytes(count: Int): ByteArray = ByteArray(count).also { random.nextBytes(it) }
}

actual fun platformCrypto(): PlatformCrypto = JcaCrypto()
