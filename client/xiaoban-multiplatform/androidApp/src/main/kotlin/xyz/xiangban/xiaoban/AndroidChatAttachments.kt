package xyz.xiangban.xiaoban

import android.content.ContentResolver
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Matrix
import android.media.ExifInterface
import android.net.Uri
import android.provider.OpenableColumns
import android.util.Base64
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.nio.ByteBuffer
import java.nio.charset.CharacterCodingException
import java.nio.charset.CodingErrorAction
import java.util.Locale
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException
import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.common.community.ChatAttachment
import xyz.xiangban.xiaoban.common.community.validateChatAttachments

/** Construct once in onCreate, before the activity starts. Reads only the URI the user selected. */
class AndroidChatAttachments(activity: ComponentActivity) : DefaultLifecycleObserver {
    private class Selection(val image: Boolean, val continuation: CancellableContinuation<ChatAttachment?>) {
        var loading: Job? = null
    }

    private val resolver: ContentResolver = activity.applicationContext.contentResolver
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private val lock = Any()
    private var waiting: Selection? = null
    private var pickerOpen = false
    private var destroyed = false
    private val launcher = activity.registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        selected(uri)
    }

    init {
        activity.lifecycle.addObserver(this)
    }

    suspend fun pick(image: Boolean): ChatAttachment? = withContext(Dispatchers.Main.immediate) {
        suspendCancellableCoroutine { continuation ->
            val selection = Selection(image, continuation)
            val failure = synchronized(lock) {
                when {
                    destroyed -> IllegalStateException("附件选择页面已关闭")
                    pickerOpen || waiting != null -> IllegalStateException("请先完成当前附件选择")
                    else -> { waiting = selection; pickerOpen = true; null }
                }
            }
            if (failure != null) {
                continuation.resumeWithException(failure)
                return@suspendCancellableCoroutine
            }
            continuation.invokeOnCancellation {
                val loading = synchronized(lock) {
                    if (waiting === selection) waiting = null
                    selection.loading
                }
                // The system picker may still return: keep pickerOpen until its callback arrives.
                loading?.cancel()
            }
            if (!continuation.isActive) {
                synchronized(lock) { pickerOpen = false }
                return@suspendCancellableCoroutine
            }
            try {
                launcher.launch(if (image) ImageTypes else TextTypes)
            } catch (e: Exception) {
                synchronized(lock) {
                    pickerOpen = false
                    if (waiting === selection) waiting = null
                }
                if (continuation.isActive) continuation.resumeWithException(e)
            }
        }
    }

    private fun selected(uri: Uri?) {
        val selection = synchronized(lock) {
            pickerOpen = false
            waiting
        } ?: return
        if (uri == null) {
            complete(selection, Result.success(null))
            return
        }
        val loading = scope.launch(Dispatchers.IO, start = CoroutineStart.LAZY) {
            try {
                val attachment = if (selection.image) readImage(uri) else readText(uri)
                validateChatAttachments(listOf(attachment))
                currentCoroutineContext().ensureActive()
                complete(selection, Result.success(attachment))
            } catch (e: CancellationException) {
                selection.continuation.cancel(e)
            } catch (e: Exception) {
                complete(selection, Result.failure(e))
            }
        }
        val active = synchronized(lock) {
            if (destroyed || waiting !== selection) false
            else { selection.loading = loading; true }
        }
        if (active) loading.start() else loading.cancel()
    }

    private fun complete(selection: Selection, result: Result<ChatAttachment?>) {
        val continuation = synchronized(lock) {
            if (waiting !== selection) null
            else { waiting = null; selection.continuation }
        } ?: return
        if (continuation.isActive) result.fold({ continuation.resume(it) }, { continuation.resumeWithException(it) })
    }

    override fun onDestroy(owner: LifecycleOwner) {
        val selection = synchronized(lock) {
            destroyed = true
            pickerOpen = false
            waiting.also { waiting = null }
        }
        selection?.continuation?.cancel(CancellationException("附件选择页面已关闭"))
        scope.cancel()
        launcher.unregister()
        owner.lifecycle.removeObserver(this)
    }

    private fun displayName(uri: Uri, fallback: String): String {
        val name = resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
            if (cursor.moveToFirst()) cursor.getString(0) else null
        } ?: uri.lastPathSegment ?: fallback
        return name.substringAfterLast('/').substringAfterLast('\\')
            .filterNot { it.isISOControl() }.trim().takeLast(160).ifBlank { fallback }
    }

    private suspend fun readLimited(uri: Uri, limit: Int, tooLarge: String): ByteArray {
        require(uri.scheme == ContentResolver.SCHEME_CONTENT) { "请选择手机中的本地文件" }
        return requireNotNull(resolver.openInputStream(uri)) { "无法打开所选附件" }.use { input ->
            val output = ByteArrayOutputStream(minOf(limit, 16 * 1024))
            val buffer = ByteArray(8 * 1024)
            while (true) {
                currentCoroutineContext().ensureActive()
                val count = input.read(buffer)
                if (count < 0) break
                require(output.size() + count <= limit) { tooLarge }
                output.write(buffer, 0, count)
            }
            output.toByteArray()
        }
    }

    private suspend fun readText(uri: Uri): ChatAttachment {
        val name = displayName(uri, "附件")
        val extension = name.substringAfterLast('.', "").lowercase(Locale.ROOT)
        require(extension in TextExtensions) { "请选择 TXT、Markdown、JSON 或其他受支持的文本源码文件" }
        val bytes = readLimited(uri, 256 * 1024, "文本附件不能超过 256 KiB")
        val text = try {
            Charsets.UTF_8.newDecoder().onMalformedInput(CodingErrorAction.REPORT)
                .onUnmappableCharacter(CodingErrorAction.REPORT).decode(ByteBuffer.wrap(bytes)).toString().removePrefix("\uFEFF")
        } catch (e: CharacterCodingException) {
            throw IllegalArgumentException("文本附件必须使用 UTF-8 编码", e)
        }
        require(text.none { it.isISOControl() && it !in "\n\r\t\u000C" }) { "所选文件包含二进制内容，请选择纯文本文件" }
        return ChatAttachment(name = name, text = text)
    }

    private suspend fun readImage(uri: Uri): ChatAttachment {
        val name = displayName(uri, "图片.jpg")
        val bytes = readLimited(uri, 16 * 1024 * 1024, "原图不能超过 16 MiB，请先缩小图片")
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
        require(bounds.outWidth > 0 && bounds.outHeight > 0 && bounds.outMimeType in ImageTypes) { "仅支持有效的 PNG、JPEG、WebP 图片" }
        var sample = 1
        while (maxOf(bounds.outWidth, bounds.outHeight) / sample > MaxImageEdge) sample *= 2
        currentCoroutineContext().ensureActive()
        var bitmap = requireNotNull(BitmapFactory.decodeByteArray(bytes, 0, bytes.size, BitmapFactory.Options().apply {
            inSampleSize = sample
            inScaled = false
            inPreferredConfig = Bitmap.Config.ARGB_8888
        })) { "无法解码所选图片" }
        try {
            if (bounds.outMimeType == "image/jpeg") {
                val matrix = jpegOrientation(bytes)
                if (!matrix.isIdentity) {
                    val rotated = Bitmap.createBitmap(bitmap, 0, 0, bitmap.width, bitmap.height, matrix, true)
                    if (rotated !== bitmap) { bitmap.recycle(); bitmap = rotated }
                }
            }
            val edge = maxOf(bitmap.width, bitmap.height)
            if (edge > MaxImageEdge) {
                val scale = MaxImageEdge.toFloat() / edge
                val resized = Bitmap.createScaledBitmap(bitmap, maxOf(1, (bitmap.width * scale).toInt()), maxOf(1, (bitmap.height * scale).toInt()), true)
                if (resized !== bitmap) { bitmap.recycle(); bitmap = resized }
            }
            if (bitmap.hasAlpha()) {
                val flattened = Bitmap.createBitmap(bitmap.width, bitmap.height, Bitmap.Config.RGB_565)
                Canvas(flattened).apply { drawColor(Color.WHITE); drawBitmap(bitmap, 0f, 0f, null) }
                bitmap.recycle()
                bitmap = flattened
            }
            for (quality in intArrayOf(90, 80, 65, 50)) {
                currentCoroutineContext().ensureActive()
                val output = ByteArrayOutputStream()
                check(bitmap.compress(Bitmap.CompressFormat.JPEG, quality, output)) { "无法压缩所选图片" }
                if (output.size() <= MaxJpegBytes) {
                    return ChatAttachment(name = name, imageDataUrl = JpegPrefix + Base64.encodeToString(output.toByteArray(), Base64.NO_WRAP))
                }
            }
            throw IllegalArgumentException("图片压缩后仍然过大，请选择更小的图片")
        } finally {
            bitmap.recycle()
        }
    }

    private fun jpegOrientation(bytes: ByteArray): Matrix = Matrix().apply {
        val orientation = ExifInterface(ByteArrayInputStream(bytes)).getAttributeInt(ExifInterface.TAG_ORIENTATION, ExifInterface.ORIENTATION_NORMAL)
        when (orientation) {
            ExifInterface.ORIENTATION_FLIP_HORIZONTAL -> setScale(-1f, 1f)
            ExifInterface.ORIENTATION_ROTATE_180 -> setRotate(180f)
            ExifInterface.ORIENTATION_FLIP_VERTICAL -> setScale(1f, -1f)
            ExifInterface.ORIENTATION_TRANSPOSE -> { setRotate(90f); postScale(-1f, 1f) }
            ExifInterface.ORIENTATION_ROTATE_90 -> setRotate(90f)
            ExifInterface.ORIENTATION_TRANSVERSE -> { setRotate(-90f); postScale(-1f, 1f) }
            ExifInterface.ORIENTATION_ROTATE_270 -> setRotate(-90f)
        }
    }

    private companion object {
        const val MaxImageEdge = 1600
        const val JpegPrefix = "data:image/jpeg;base64,"
        val MaxJpegBytes = (2_800_000 - JpegPrefix.length) / 4 * 3
        val ImageTypes = arrayOf("image/png", "image/jpeg", "image/webp")
        // The extension and strict UTF-8 checks remain authoritative when a provider ignores MIME filters.
        val TextTypes = arrayOf("text/*", "application/json", "application/x-ndjson", "application/xml", "application/javascript", "application/yaml", "application/toml", "application/octet-stream")
        val TextExtensions = setOf(
            "txt", "md", "markdown", "json", "jsonl", "csv", "tsv", "yaml", "yml", "xml", "html", "htm", "css", "svg",
            "js", "mjs", "cjs", "ts", "tsx", "jsx", "kt", "kts", "java", "rs", "py", "rb", "go", "c", "h", "cpp", "hpp",
            "cs", "swift", "sh", "bash", "zsh", "toml", "ini", "conf", "log", "sql",
        )
    }
}
