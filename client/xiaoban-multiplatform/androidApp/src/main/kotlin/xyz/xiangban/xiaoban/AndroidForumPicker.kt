package xyz.xiangban.xiaoban

import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions
import xyz.xiangban.xiaoban.common.community.FORUM_MEMORY_MAX_BYTES
import xyz.xiangban.xiaoban.common.community.ForumUploadFile
import xyz.xiangban.xiaoban.common.community.forumUploadName
import java.io.ByteArrayOutputStream
import xyz.xiangban.xiaoban.common.community.ApiException
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

/** Storage Access Framework，只获得用户点选的单个 content URI，不申请存储权限。 */
class AndroidForumPicker(private val activity: ComponentActivity) {
    private var waiting: CancellableContinuation<Uri?>? = null
    private val launcher = activity.registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        val continuation = waiting
        waiting = null
        if (continuation?.isActive == true) continuation.resume(uri)
    }

    suspend fun pick(image: Boolean, maxBytes: Long): ForumUploadFile? {
        val limit = minOf(maxBytes, FORUM_MEMORY_MAX_BYTES)
        if (limit <= 0) throw ApiException(400, "当前账号暂不能上传文件")
        val uri = withContext(Dispatchers.Main.immediate) {
            suspendCancellableCoroutine<Uri?> { continuation ->
                if (waiting != null) {
                    continuation.resumeWithException(ApiException(400, "已有文件选择正在进行"))
                    return@suspendCancellableCoroutine
                }
                waiting = continuation
                continuation.invokeOnCancellation { activity.runOnUiThread { if (waiting === continuation) waiting = null } }
                try {
                    launcher.launch(if (image) arrayOf("image/png", "image/jpeg", "image/gif", "image/webp") else arrayOf("*/*"))
                } catch (e: Exception) {
                    waiting = null
                    if (continuation.isActive) continuation.resumeWithException(ApiException(400, "无法打开系统文件选择器"))
                }
            }
        } ?: return null
        return withContext(Dispatchers.IO) {
            if (uri.scheme != "content") throw ApiException(400, "请选择系统文件选择器中的文件")
            val resolver = activity.contentResolver
            var name = if (image) "image" else "file"
            resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (nameIndex >= 0 && !cursor.isNull(nameIndex)) name = cursor.getString(nameIndex)
                    val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                    if (sizeIndex >= 0 && !cursor.isNull(sizeIndex) && cursor.getLong(sizeIndex) > limit)
                        throw ApiException(400, "文件超过上传上限（${limit / 1024 / 1024} MB）")
                }
            }
            val mime = resolver.getType(uri) ?: "application/octet-stream"
            val output = ByteArrayOutputStream()
            resolver.openInputStream(uri)?.use { input ->
                val buffer = ByteArray(8192)
                var total = 0L
                while (true) {
                    val size = input.read(buffer, 0, minOf(buffer.size.toLong(), limit - total + 1).toInt())
                    if (size < 0) break
                    total += size
                    if (total > limit) throw ApiException(400, "文件超过上传上限（${limit / 1024 / 1024} MB）")
                    output.write(buffer, 0, size)
                }
            } ?: throw ApiException(400, "无法读取这个文件，请重新选择")
            if (output.size() == 0) throw ApiException(400, "文件是空的")
            ForumUploadFile(forumUploadName(name), mime, output.toByteArray())
        }
    }
}

/** 通过委托接文件选择能力，不改既有 AndroidPlatform 的语音和遥控能力。 */
class ForumPickerPlatform(private val delegate: PlatformActions, private val picker: AndroidForumPicker) : PlatformActions by delegate {
    override val forumFilePickerSupported = true
    override suspend fun pickForumFile(image: Boolean, maxBytes: Long) = picker.pick(image, maxBytes)
}
