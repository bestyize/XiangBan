package xyz.xiangban.xiaoban.common.link

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import kotlinx.serialization.json.Json

private const val ANDROID_KEYSTORE = "AndroidKeyStore"
private const val KEY_ALIAS = "xiaoban_link_master_v1"
private const val PREFS_NAME = "xiaoban_link_storage"
private const val KEY_ENCRYPTED_STATE = "encrypted_state"
private const val GCM_TAG_LENGTH = 128
private const val GCM_IV_LENGTH = 12

private val StorageJson = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
    encodeDefaults = true
}

/**
 * Android 平台上的 [LinkStorage] 实现：
 * 采用 Android Keystore 保护的 AES-256-GCM 密钥，将包含软件 EC 私钥与配对密钥 PRK 的
 * [LinkState] 加密后持久化于应用私有 SharedPreferences 中。
 * 兼容 Android 7.0+ (minSdk 24)。
 */
class AndroidKeystoreLinkStorage(context: Context) : LinkStorage {
    private val prefs = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    private fun getOrCreateMasterKey(): SecretKey {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        val existing = keyStore.getKey(KEY_ALIAS, null) as? SecretKey
        if (existing != null) return existing

        val keyGenerator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        val spec = KeyGenParameterSpec.Builder(
            KEY_ALIAS,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .build()
        keyGenerator.init(spec)
        return keyGenerator.generateKey()
    }

    override fun load(): LinkState {
        val encoded = prefs.getString(KEY_ENCRYPTED_STATE, null) ?: return LinkState()
        return try {
            val bytes = B64.decode(encoded)
            if (bytes.size < GCM_IV_LENGTH + 16) return LinkState()
            val iv = bytes.copyOfRange(0, GCM_IV_LENGTH)
            val ciphertext = bytes.copyOfRange(GCM_IV_LENGTH, bytes.size)
            val key = getOrCreateMasterKey()
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(GCM_TAG_LENGTH, iv))
            val plaintext = cipher.doFinal(ciphertext)
            StorageJson.decodeFromString(LinkState.serializer(), plaintext.decodeToString())
        } catch (_: Exception) {
            LinkState()
        }
    }

    override fun save(state: LinkState) {
        try {
            val jsonText = StorageJson.encodeToString(LinkState.serializer(), state)
            val plaintext = jsonText.encodeToByteArray()
            val key = getOrCreateMasterKey()
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.ENCRYPT_MODE, key)
            val iv = cipher.iv ?: error("Cipher IV is null")
            val ciphertext = cipher.doFinal(plaintext)
            val combined = ByteArray(iv.size + ciphertext.size)
            System.arraycopy(iv, 0, combined, 0, iv.size)
            System.arraycopy(ciphertext, 0, combined, iv.size, ciphertext.size)
            val encoded = B64.encode(combined)
            prefs.edit().putString(KEY_ENCRYPTED_STATE, encoded).apply()
        } catch (e: Exception) {
            // 保存失败时记录但不崩溃，避免阻塞主流程
            e.printStackTrace()
        }
    }
}
