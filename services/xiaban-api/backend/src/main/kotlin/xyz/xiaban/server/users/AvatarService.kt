package xyz.xiaban.server.users

import io.ktor.http.HttpStatusCode
import xyz.xiaban.server.db.Db
import xyz.xiaban.server.db.execute
import xyz.xiaban.server.db.queryOne
import xyz.xiaban.server.http.ApiException
import xyz.xiaban.server.http.badRequest
import java.io.ByteArrayInputStream
import java.security.SecureRandom
import java.util.Base64
import javax.imageio.ImageIO

class AvatarService(private val db: Db) {
    suspend fun save(userId: Long, bytes: ByteArray) {
        if (bytes.size > MAX_BYTES) throw ApiException(HttpStatusCode.PayloadTooLarge, "avatar_too_large", "头像压缩后不能超过 64KB")
        if (bytes.isEmpty()) badRequest("请选择头像图片")
        val valid = runCatching {
            ImageIO.createImageInputStream(ByteArrayInputStream(bytes)).use { stream ->
                val readers = ImageIO.getImageReaders(stream)
                if (!readers.hasNext()) return@use false
                val reader = readers.next()
                try {
                    reader.input = stream
                    if (reader.formatName.lowercase() !in setOf("jpeg", "jpg")) return@use false
                    if (reader.getWidth(0) !in 1..512 || reader.getHeight(0) !in 1..512) return@use false
                    reader.read(0) != null
                } finally { reader.dispose() }
            }
        }.getOrDefault(false)
        if (!valid) badRequest("头像需为已压缩的 JPEG 图片，尺寸不能超过 512×512", "invalid_avatar")
        val key = Base64.getUrlEncoder().withoutPadding().encodeToString(ByteArray(32).also(SecureRandom()::nextBytes))
        db.read { c -> c.execute("insert into user_avatars(user_id,asset_key,image) values (?,?,?) on conflict(user_id) do update set asset_key=excluded.asset_key,image=excluded.image,updated_at=now()", userId, key, bytes) }
    }
    suspend fun remove(userId: Long) = db.read { c -> c.execute("delete from user_avatars where user_id=?", userId) }
    suspend fun image(key: String): ByteArray? {
        if (!Regex("^[A-Za-z0-9_-]{43}$").matches(key)) return null
        return db.read { c -> c.queryOne("select image from user_avatars where asset_key=?", key) { it.getBytes(1) } }
    }
    companion object { const val MAX_BYTES = 64 * 1024 }
}
