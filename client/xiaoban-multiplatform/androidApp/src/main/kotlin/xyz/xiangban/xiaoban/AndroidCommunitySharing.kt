package xyz.xiangban.xiaoban

import android.content.ClipData
import android.content.ContentValues
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.Shader
import android.graphics.Typeface
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.text.Layout
import android.text.StaticLayout
import android.text.TextPaint
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asAndroidBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.core.content.FileProvider
import com.google.zxing.BarcodeFormat
import com.google.zxing.EncodeHintType
import com.google.zxing.qrcode.QRCodeWriter
import com.google.zxing.qrcode.decoder.ErrorCorrectionLevel
import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.business.communitynative.CommunityPosterResult
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions
import xyz.xiangban.xiaoban.common.community.CommunityShare
import java.io.File
import kotlin.coroutines.resume

/** A separate adapter so sharing never changes chat, game, microphone or remote-control behavior. */
class CommunitySharingPlatform(private val delegate: PlatformActions, private val activity: ComponentActivity) : PlatformActions by delegate {
    private val links = MutableStateFlow<String?>(null)
    override val communityLinks = links.asStateFlow()
    override fun consumeCommunityLink() { links.value = null }
    fun accept(intent: Intent?) {
        if (intent?.action == Intent.ACTION_VIEW) {
            val raw = intent.dataString
            // Core performs the actual origin/path/parameter allowlist before requesting public metadata.
            if (raw != null && raw.length <= 2048) links.value = raw
        }
    }
    override val communityPosterSupported = true
    override val forumCoverSupported = true
    override suspend fun renderForumCover(spec: xyz.xiangban.xiaoban.business.communitynative.ForumCoverSpec, thumbnail: Boolean) = renderLocalForumCover(spec, thumbnail)
    override suspend fun previewForumCover(file: xyz.xiangban.xiaoban.common.community.ForumUploadFile) = previewLocalForumCover(file)
    private var waiting: CancellableContinuation<Uri?>? = null
    private val saveDocument = activity.registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        val current = waiting; waiting = null
        if (current?.isActive == true) current.resume(result.data?.data)
    }

    /** Fully local, fixed-size PNG source: public text and a QR code for the canonical browser URL. */
    override suspend fun createCommunityPoster(info: CommunityShare): ImageBitmap = withContext(Dispatchers.Default) {
        val bitmap = Bitmap.createBitmap(750, 1180, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        canvas.drawColor(Color.WHITE)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG)
        paint.shader = LinearGradient(0f, 0f, 750f, 440f, Color.rgb(35, 37, 44), Color.rgb(95, 98, 112), Shader.TileMode.CLAMP)
        canvas.drawRect(0f, 0f, 750f, 360f, paint); paint.shader = null
        paint.color = Color.argb(24, 255, 255, 255)
        canvas.drawCircle(650f, 40f, 210f, paint); canvas.drawCircle(100f, 360f, 130f, paint)
        drawText(canvas, "超级小伴 · 社区", 56, 70, 638, 30f, Color.WHITE, 1, true)
        val tag = when (info.kind) { "pack" -> "社区作品"; "topic" -> "社区讨论"; "scheme" -> "社区方案"; "project" -> "开源项目"; else -> "社区周刊" }
        drawText(canvas, tag, 56, 264, 638, 48f, Color.WHITE, 1, true)
        drawText(canvas, info.title, 56, 410, 638, 42f, Color.rgb(30, 31, 35), 3, true)
        drawText(canvas, info.summary.ifBlank { "打开公开页面查看完整内容" }, 56, 610, 638, 27f, Color.rgb(100, 103, 112), 5)
        paint.color = Color.rgb(230, 231, 235); canvas.drawRect(56f, 918f, 694f, 920f, paint)
        val qr = QRCodeWriter().encode(info.url, BarcodeFormat.QR_CODE, 190, 190,
            mapOf(EncodeHintType.MARGIN to 4, EncodeHintType.ERROR_CORRECTION to ErrorCorrectionLevel.M, EncodeHintType.CHARACTER_SET to "UTF-8"))
        paint.color = Color.BLACK
        for (y in 0 until qr.height) for (x in 0 until qr.width) if (qr[x, y]) canvas.drawRect((504 + x).toFloat(), (950 + y).toFloat(), (505 + x).toFloat(), (951 + y).toFloat(), paint)
        drawText(canvas, "超级小伴", 56, 957, 420, 36f, Color.rgb(255, 36, 66), 1, true)
        drawText(canvas, "扫码查看公开内容", 56, 1020, 420, 24f, Color.DKGRAY, 1)
        drawText(canvas, Uri.parse(info.url).host.orEmpty(), 56, 1068, 420, 21f, Color.GRAY, 1)
        drawText(canvas, info.url.substringAfter("://"), 56, 1110, 420, 18f, Color.GRAY, 2)
        bitmap.asImageBitmap()
    }

    override val supportsChatImageSave = true

    override suspend fun saveChatImage(bytes: ByteArray, mime: String): CommunityPosterResult {
        val extension = when (mime) {
            "image/png" -> "png"
            "image/jpeg" -> "jpg"
            "image/webp" -> "webp"
            else -> return CommunityPosterResult.Failed
        }
        return saveImage("xiaoban-image-${System.currentTimeMillis()}.$extension", mime) { it.write(bytes) }
    }

    override suspend fun saveCommunityPoster(image: ImageBitmap): CommunityPosterResult =
        saveImage("xiaoban-community-${System.currentTimeMillis()}.png", "image/png") {
            check(image.asAndroidBitmap().compress(Bitmap.CompressFormat.PNG, 100, it))
        }

    private suspend fun saveImage(filename: String, mime: String, write: (java.io.OutputStream) -> Unit): CommunityPosterResult {
        var inserted: Uri? = null
        return try {
            if (Build.VERSION.SDK_INT >= 29) withContext(Dispatchers.IO) {
                val values = ContentValues().apply {
                    put(MediaStore.Images.Media.DISPLAY_NAME, filename)
                    put(MediaStore.Images.Media.MIME_TYPE, mime)
                    put(MediaStore.Images.Media.RELATIVE_PATH, Environment.DIRECTORY_PICTURES + "/Xiaoban")
                    put(MediaStore.Images.Media.IS_PENDING, 1)
                }
                val uri = activity.contentResolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values) ?: return@withContext CommunityPosterResult.Failed
                inserted = uri
                activity.contentResolver.openOutputStream(uri)?.use(write) ?: error("No output")
                check(activity.contentResolver.update(uri, ContentValues().apply { put(MediaStore.Images.Media.IS_PENDING, 0) }, null, null) > 0)
                inserted = null
                CommunityPosterResult.Saved
            } else {
                // Android 7–9: system save dialog grants one-document access; no broad storage permission.
                val uri = withContext(Dispatchers.Main.immediate) {
                    suspendCancellableCoroutine<Uri?> { continuation ->
                        if (waiting != null) { continuation.resume(null); return@suspendCancellableCoroutine }
                        waiting = continuation
                        continuation.invokeOnCancellation { activity.runOnUiThread { if (waiting === continuation) waiting = null } }
                        try {
                            saveDocument.launch(Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                                type = mime
                                addCategory(Intent.CATEGORY_OPENABLE)
                                putExtra(Intent.EXTRA_TITLE, filename)
                            })
                        }
                        catch (_: Exception) { waiting = null; continuation.resume(null) }
                    }
                } ?: return CommunityPosterResult.Cancelled
                withContext(Dispatchers.IO) {
                    activity.contentResolver.openOutputStream(uri)?.use(write) ?: error("No output")
                }
                CommunityPosterResult.Saved
            }
        } catch (e: CancellationException) { throw e }
          catch (_: SecurityException) { CommunityPosterResult.PermissionDenied }
          catch (_: Exception) { CommunityPosterResult.Failed }
        finally {
            inserted?.let { uri -> withContext(kotlinx.coroutines.NonCancellable + Dispatchers.IO) { runCatching { activity.contentResolver.delete(uri, null, null) } } }
        }
    }

    override suspend fun shareCommunityPoster(image: ImageBitmap): CommunityPosterResult = try {
        val file = withContext(Dispatchers.IO) {
            val directory = File(activity.cacheDir, "community-posters").apply { mkdirs() }
            directory.listFiles()?.filter { it.name.startsWith("poster-") && it.lastModified() < System.currentTimeMillis() - 86_400_000 }?.forEach { it.delete() }
            File.createTempFile("poster-", ".png", directory).apply {
                outputStream().use { check(image.asAndroidBitmap().compress(Bitmap.CompressFormat.PNG, 100, it)) }
            }
        }
        val uri = FileProvider.getUriForFile(activity, activity.packageName + ".community-posters", file)
        withContext(Dispatchers.Main.immediate) {
            val send = Intent(Intent.ACTION_SEND).apply {
                type = "image/png"; putExtra(Intent.EXTRA_STREAM, uri)
                clipData = ClipData.newUri(activity.contentResolver, "社区海报", uri)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            activity.startActivity(Intent.createChooser(send, "分享社区海报"))
        }
        CommunityPosterResult.ChooserOpened
    } catch (e: CancellationException) { throw e }
      catch (_: SecurityException) { CommunityPosterResult.PermissionDenied }
      catch (_: Exception) { CommunityPosterResult.Failed }
}

private fun drawText(canvas: Canvas, text: String, x: Int, y: Int, width: Int, size: Float, color: Int, lines: Int, bold: Boolean = false) {
    val paint = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
        textSize = size; this.color = color
        typeface = Typeface.create("sans-serif", if (bold) Typeface.BOLD else Typeface.NORMAL)
    }
    val layout = StaticLayout.Builder.obtain(text.replace(Regex("\\s+"), " ").trim(), 0, text.replace(Regex("\\s+"), " ").trim().length, paint, width)
        .setAlignment(Layout.Alignment.ALIGN_NORMAL).setIncludePad(false).setMaxLines(lines)
        .setEllipsize(android.text.TextUtils.TruncateAt.END).setLineSpacing(7f, 1f).build()
    canvas.save(); canvas.translate(x.toFloat(), y.toFloat()); layout.draw(canvas); canvas.restore()
}
