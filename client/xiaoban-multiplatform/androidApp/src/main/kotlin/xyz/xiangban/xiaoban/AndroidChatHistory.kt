package xyz.xiangban.xiaoban

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import java.io.ByteArrayOutputStream
import java.io.File
import xyz.xiangban.xiaoban.common.community.ChatHistoryFiles

/** 小伴AI 的对话历史存在应用私有目录里（`files/chats/…`）；先写临时文件再换名，写一半断电也不会坏掉原来的。 */
class AndroidChatHistoryFiles(context: Context) : ChatHistoryFiles {
    private val root = context.applicationContext.filesDir
    private val mediaLimit = 128L * 1024 * 1024

    private fun directoryBytes(directory: File): Long =
        directory.listFiles().orEmpty().filter(File::isFile).sumOf(File::length)

    override val backgroundReads = true

    override fun read(name: String): String? = File(root, name).takeIf { it.isFile }?.readText()

    override fun readMedia(name: String): ByteArray? =
        File(root, name).takeIf { it.isFile && it.length() in 1..16L * 1024 * 1024 }?.readBytes()

    override fun hasMedia(name: String): Boolean = File(root, name).isFile

    override fun write(name: String, text: String) = writeBytes(File(root, name), text.toByteArray(Charsets.UTF_8))

    override fun writeMedia(name: String, bytes: ByteArray) {
        val file = File(root, name)
        val parent = requireNotNull(file.parentFile)
        check(directoryBytes(parent) + bytes.size <= mediaLimit) { "此对话的图片存储已达 128 MiB，请使用新对话。" }
        writeBytes(file, bytes)
    }

    override fun readMediaThumbnail(name: String): ByteArray? {
        val original = File(root, name)
        if (!original.isFile || original.length() !in 1..16L * 1024 * 1024) return null
        val thumbnail = File(root, "$name.thumb.jpg")
        if (thumbnail.isFile && thumbnail.length() in 1..256L * 1024) return thumbnail.readBytes()

        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        BitmapFactory.decodeFile(original.path, bounds)
        val edge = maxOf(bounds.outWidth, bounds.outHeight)
        if (edge <= 0) return null
        var sample = 1
        while (edge / sample > 512) sample *= 2
        val bitmap = BitmapFactory.decodeFile(original.path, BitmapFactory.Options().apply { inSampleSize = sample }) ?: return null
        try {
            val output = ByteArrayOutputStream()
            val flattened = Bitmap.createBitmap(bitmap.width, bitmap.height, Bitmap.Config.RGB_565)
            try {
                Canvas(flattened).apply { drawColor(Color.WHITE); drawBitmap(bitmap, 0f, 0f, null) }
                if (!flattened.compress(Bitmap.CompressFormat.JPEG, 82, output)) return null
            } finally { flattened.recycle() }
            val bytes = output.toByteArray()
            if (directoryBytes(requireNotNull(thumbnail.parentFile)) + bytes.size <= mediaLimit) {
                runCatching { writeBytes(thumbnail, bytes) }
            }
            return bytes
        } finally { bitmap.recycle() }
    }

    private fun writeBytes(file: File, bytes: ByteArray) {
        val parent = requireNotNull(file.parentFile)
        check(parent.isDirectory || parent.mkdirs()) { "无法创建对话目录" }
        val temp = File(file.parentFile, file.name + ".tmp")
        try {
            temp.outputStream().use { stream -> stream.write(bytes); stream.fd.sync() }
            // rename is atomic on Android; never delete the only good copy on failure.
            check(temp.renameTo(file)) { "无法保存对话，原记录已保留" }
        } finally { temp.delete() }
    }

    override fun delete(name: String) {
        val file = File(root, name)
        check(!file.exists() || if (name.startsWith("agent/tool-output/") || name.startsWith("agent/attachments/")) file.deleteRecursively() else file.delete()) { "无法删除对话记录" }
    }

    override fun deleteTree(name: String) {
        check(File(root, name).deleteRecursively()) { "无法删除对话图片" }
    }

    /** 智能体内核把对话存在 `files/agent/s-<编号>.jsonl`（见 MainActivity 里的 agentDir）。 */
    override fun agentCopy(id: String): String = "agent/s-${escapedId(id)}.jsonl"
    override fun agentProgressCopy(id: String): String = "agent/progress/s-${escapedId(id)}.jsonl"
    override fun agentSidecars(id: String): List<String> =
        listOf(agentCopy(id), agentProgressCopy(id), "agent/tool-output/${escapedId(id)}", "agent/attachments/${escapedId(id)}", "agent/trust/$id")

    override fun deletionMarker(folder: String, id: String): String = "agent/deleted/$id"

    override fun deletedIds(folder: String): Set<String> =
        File(root, "agent/deleted").listFiles().orEmpty()
            .filter { it.isFile && it.length() <= 4096 && it.readText() == folder }
            .map { it.name }.toSet()

    private fun escapedId(id: String): String =
        id.map { if (it in 'A'..'Z') "%${it.code.toString(16).uppercase()}" else it }.joinToString("")
}
