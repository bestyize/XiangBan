package xyz.xiangban.xiaoban.common.link

class KeyPairBytes(
    /** 32 字节私钥 */
    val privateKey: ByteArray,
    /** 65 字节 SEC1 未压缩公钥（首字节 0x04） */
    val publicKey: ByteArray,
)

/** 协议用到的几个密码学原语。Android 与 JVM 用 Java 的实现；iOS 暂未实现。 */
interface PlatformCrypto {
    fun generateKeyPair(): KeyPairBytes

    /** P-256 ECDH，返回共享点的 x 坐标 32 字节。公钥不合法时抛 [LinkException]。 */
    fun ecdh(privateKey: ByteArray, peerPublic: ByteArray): ByteArray

    fun hmacSha256(key: ByteArray, data: ByteArray): ByteArray

    /** AES-256-GCM，返回密文加 16 字节标签。 */
    fun aesGcmSeal(key: ByteArray, nonce: ByteArray, aad: ByteArray, plain: ByteArray): ByteArray

    /** 认证失败返回 null。 */
    fun aesGcmOpen(key: ByteArray, nonce: ByteArray, aad: ByteArray, sealed: ByteArray): ByteArray?

    fun randomBytes(count: Int): ByteArray
}

expect fun platformCrypto(): PlatformCrypto
