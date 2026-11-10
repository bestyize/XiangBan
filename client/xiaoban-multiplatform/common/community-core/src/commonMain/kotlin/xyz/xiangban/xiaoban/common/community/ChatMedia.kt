package xyz.xiangban.xiaoban.common.community

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlin.io.encoding.Base64

internal fun isChatMediaReference(value: String): Boolean = CHAT_MEDIA_REFERENCE.matches(value)

private val CHAT_MEDIA_REFERENCE = Regex("media:[a-f0-9]{64}\\.(png|jpeg|webp|gif)")
private const val MAX_CHAT_MEDIA_BYTES = 16 * 1024 * 1024

/** A reference is relative to one account and conversation. Only data URLs leave this store. */
internal class ChatMedia(
    private val files: ChatHistoryFiles,
    private val io: Mutex,
    private val owner: String,
    private val id: String,
) {
    private fun path(reference: String) = "$owner/media/$id/" + reference.removePrefix("media:")

    private fun store(data: String): String {
        val mime = data.substringBefore(',').removePrefix("data:").removeSuffix(";base64")
        val format = mime.removePrefix("image/")
        if (format !in listOf("png", "jpeg", "webp", "gif") || !data.startsWith("data:$mime;base64,")) return data
        require(data.length <= 23_000_000) { "图片过大，无法保存" }
        val bytes = Base64.decode(data.substringAfter(','))
        require(bytes.size in 1..MAX_CHAT_MEDIA_BYTES) { "图片过大或内容无效" }
        val reference = "media:" + Sha256().apply { update(bytes) }.hex() + ".$format"
        val name = path(reference)
        if (!files.hasMedia(name)) files.writeMedia(name, bytes)
        return reference
    }

    fun compact(turn: ChatTurn): ChatTurn = turn.copy(
        attachments = turn.attachments.map { it.copy(imageDataUrl = it.imageDataUrl?.let(::store)) },
        generatedImages = turn.generatedImages.map(::store),
    )

    suspend fun save(images: List<String>): List<String> = withContext(Dispatchers.Default) {
        io.withLock {
            check(files.read(files.deletionMarker(owner, id)) == null) { "对话已删除" }
            images.map(::store)
        }
    }

    suspend fun resolve(data: String): String? {
        if (!isChatMediaReference(data)) return data.takeIf { it.startsWith("data:image/") }
        return withContext(Dispatchers.Default) {
            io.withLock {
                if (files.read(files.deletionMarker(owner, id)) != null) return@withLock null
                val bytes = files.readMedia(path(data))?.takeIf { it.size in 1..MAX_CHAT_MEDIA_BYTES } ?: return@withLock null
                "data:image/" + data.substringAfterLast('.') + ";base64," + Base64.encode(bytes)
            }
        }
    }
}
