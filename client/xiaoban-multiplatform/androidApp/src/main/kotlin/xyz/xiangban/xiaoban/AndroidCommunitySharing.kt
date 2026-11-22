package xyz.xiangban.xiaoban

import android.content.ClipData
import android.content.ClipboardManager
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.DashPathEffect
import android.graphics.Paint
import android.graphics.RectF
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
import xyz.xiangban.xiaoban.common.share.ShareCardOptions
import xyz.xiangban.xiaoban.common.share.ShareCardStyle
import xyz.xiangban.xiaoban.common.share.ShareContent
import xyz.xiangban.xiaoban.common.share.shareDisplayUrl
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
    override val communityPosterCopySupported = true
    override val forumCoverSupported = true
    override suspend fun renderForumCover(spec: xyz.xiangban.xiaoban.business.communitynative.ForumCoverSpec, thumbnail: Boolean) = renderLocalForumCover(spec, thumbnail)
    override suspend fun previewForumCover(file: xyz.xiangban.xiaoban.common.community.ForumUploadFile) = previewLocalForumCover(file)
    override val avatarEditorSupported: Boolean get() = forumPhotoPickerSupported
    override suspend fun decodeAvatarImage(file: xyz.xiangban.xiaoban.common.community.ForumUploadFile) = decodeLocalAvatar(file)
    override suspend fun encodeAvatarWebp(image: ImageBitmap) = encodeLocalAvatar(image)
    private var waiting: CancellableContinuation<Uri?>? = null
    private val saveDocument = activity.registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        val current = waiting; waiting = null
        if (current?.isActive == true) current.resume(result.data?.data)
    }

    override suspend fun encodeCommunityQr(text: String): List<BooleanArray>? = withContext(Dispatchers.Default) {
        if (text.isBlank()) return@withContext null
        runCatching {
            val matrix = QRCodeWriter().encode(
                text,
                BarcodeFormat.QR_CODE,
                0,
                0,
                mapOf(
                    EncodeHintType.MARGIN to 0,
                    EncodeHintType.ERROR_CORRECTION to ErrorCorrectionLevel.M,
                    EncodeHintType.CHARACTER_SET to "UTF-8",
                ),
            )
            List(matrix.height) { y -> BooleanArray(matrix.width) { x -> matrix[x, y] } }
        }.getOrNull()
    }

    /** 按分享卡片设计稿（极简白 / 暖纸笺 / 墨夜）本地绘制高清 PNG 卡片。 */
    override suspend fun renderCommunityShareCard(content: ShareContent, options: ShareCardOptions): ImageBitmap = withContext(Dispatchers.Default) {
        val width = 840
        val padX = 64
        val contentWidth = width - padX * 2

        val bgColor: Int
        val inkColor: Int
        val mutedColor: Int
        val softBgColor: Int
        val lineColor: Int
        val quoteBarColor: Int
        val accentColor = Color.rgb(227, 32, 60)
        when (options.style) {
            ShareCardStyle.White -> {
                bgColor = Color.WHITE
                inkColor = Color.rgb(31, 32, 36)
                mutedColor = Color.rgb(110, 112, 120)
                softBgColor = Color.rgb(244, 244, 246)
                lineColor = Color.rgb(236, 236, 239)
                quoteBarColor = Color.rgb(245, 199, 205)
            }
            ShareCardStyle.Warm -> {
                bgColor = Color.rgb(248, 244, 236)
                inkColor = Color.rgb(43, 35, 28)
                mutedColor = Color.rgb(124, 111, 98)
                softBgColor = Color.rgb(239, 232, 220)
                lineColor = Color.rgb(229, 220, 205)
                quoteBarColor = Color.rgb(217, 195, 168)
            }
            ShareCardStyle.Ink -> {
                bgColor = Color.rgb(28, 29, 34)
                inkColor = Color.rgb(242, 242, 245)
                mutedColor = Color.rgb(152, 154, 164)
                softBgColor = Color.rgb(39, 40, 46)
                lineColor = Color.rgb(45, 46, 53)
                quoteBarColor = Color.argb(68, 255, 255, 255)
            }
        }

        val titleLayout = buildTextLayout(content.title, contentWidth, 44f, inkColor, 3, bold = true, lineSpacing = 14f)
        val cleanSummary = content.summary.replace(Regex("\\s+"), " ").trim()
        val excerptLayout = if (options.showExcerpt && cleanSummary.isNotBlank()) {
            buildTextLayout(cleanSummary, contentWidth - 28, 30f, mutedColor, 4, bold = false, lineSpacing = 12f)
        } else null
        val authorText = listOf(content.author, content.dateLabel).filter(String::isNotBlank).joinToString(" · ")
        val headerHeight = 62
        val titleTop = 64 + headerHeight + 36
        val excerptTop = titleTop + titleLayout.height + 28
        val authorTop = if (excerptLayout != null) excerptTop + excerptLayout.height + 32 else titleTop + titleLayout.height + 32
        val dividerTop = if (authorText.isNotBlank()) authorTop + 52 + 34 else authorTop + 8
        val footerHeight = if (options.showQr) 156 else 76
        val totalHeight = (dividerTop + 36 + footerHeight + 60).coerceAtLeast(620)

        val bitmap = Bitmap.createBitmap(width, totalHeight, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        canvas.drawColor(bgColor)

        val paint = Paint(Paint.ANTI_ALIAS_FLAG)

        // 顶部品牌徽章与分类胶囊
        paint.color = accentColor
        canvas.drawRoundRect(RectF(padX.toFloat(), 66f, (padX + 48).toFloat(), 114f), 14f, 14f, paint)
        drawText(canvas, "伴", padX + 11, 74, 30, 26f, Color.WHITE, 1, bold = true)
        drawText(canvas, "小伴社区", padX + 64, 73, 320, 30f, inkColor, 1, bold = true)

        if (content.kindLabel.isNotBlank()) {
            val tagPaint = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
                textSize = 24f
                color = mutedColor
                typeface = Typeface.create("sans-serif", Typeface.NORMAL)
            }
            val tagW = tagPaint.measureText(content.kindLabel) + 40f
            val tagRect = RectF(width - padX - tagW, 70f, (width - padX).toFloat(), 112f)
            paint.color = softBgColor
            canvas.drawRoundRect(tagRect, 21f, 21f, paint)
            canvas.drawText(content.kindLabel, tagRect.left + 20f, tagRect.top + 29f, tagPaint)
        }

        // 标题
        canvas.save(); canvas.translate(padX.toFloat(), titleTop.toFloat()); titleLayout.draw(canvas); canvas.restore()

        // 可选正文摘录（左竖线）
        if (excerptLayout != null) {
            paint.color = quoteBarColor
            canvas.drawRoundRect(RectF(padX.toFloat(), excerptTop.toFloat(), (padX + 6).toFloat(), (excerptTop + excerptLayout.height).toFloat()), 3f, 3f, paint)
            canvas.save(); canvas.translate((padX + 26).toFloat(), excerptTop.toFloat()); excerptLayout.draw(canvas); canvas.restore()
        }

        // 作者与日期
        if (authorText.isNotBlank()) {
            paint.color = softBgColor
            canvas.drawCircle((padX + 20).toFloat(), (authorTop + 20).toFloat(), 20f, paint)
            val initial = (content.author.firstOrNull() ?: content.title.firstOrNull() ?: '伴').uppercaseChar().toString()
            drawText(canvas, initial, padX + 11, authorTop + 5, 24, 22f, inkColor, 1, bold = true)
            drawText(canvas, authorText, padX + 54, authorTop + 6, contentWidth - 54, 26f, mutedColor, 1)
        }

        // 虚线分隔
        val dashPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
            color = lineColor
            strokeWidth = 2.5f
            style = Paint.Style.STROKE
            pathEffect = DashPathEffect(floatArrayOf(14f, 10f), 0f)
        }
        canvas.drawLine(padX.toFloat(), dividerTop.toFloat(), (width - padX).toFloat(), dividerTop.toFloat(), dashPaint)

        // 底部短链与可选二维码
        val footY = dividerTop + 36
        val copyWidth = if (options.showQr) contentWidth - 184 else contentWidth
        drawText(canvas, if (options.showQr) "扫码在浏览器查看" else "来自小伴社区", padX, footY + 26, copyWidth, 28f, inkColor, 1, bold = true)
        drawText(canvas, shareDisplayUrl(content.url), padX, footY + 72, copyWidth, 24f, mutedColor, 2)

        if (options.showQr) {
            val qrSize = 148
            val qrBoxX = width - padX - qrSize
            val qrBoxY = footY
            paint.color = Color.WHITE
            canvas.drawRoundRect(RectF(qrBoxX.toFloat(), qrBoxY.toFloat(), (qrBoxX + qrSize).toFloat(), (qrBoxY + qrSize).toFloat()), 18f, 18f, paint)
            val qr = QRCodeWriter().encode(
                content.url,
                BarcodeFormat.QR_CODE,
                qrSize - 24,
                qrSize - 24,
                mapOf(
                    EncodeHintType.MARGIN to 0,
                    EncodeHintType.ERROR_CORRECTION to ErrorCorrectionLevel.M,
                    EncodeHintType.CHARACTER_SET to "UTF-8",
                ),
            )
            paint.color = if (options.style == ShareCardStyle.Warm) Color.rgb(43, 35, 28) else Color.rgb(27, 28, 32)
            for (y in 0 until qr.height) {
                for (x in 0 until qr.width) {
                    if (qr[x, y]) {
                        canvas.drawRect((qrBoxX + 12 + x).toFloat(), (qrBoxY + 12 + y).toFloat(), (qrBoxX + 13 + x).toFloat(), (qrBoxY + 13 + y).toFloat(), paint)
                    }
                }
            }
        }

        bitmap.asImageBitmap()
    }

    /** Fully local, fixed-size PNG source: public text and a QR code for the canonical browser URL. */
    override suspend fun createCommunityPoster(info: CommunityShare): ImageBitmap {
        val tag = when (info.kind) { "pack" -> "社区作品"; "topic" -> "社区讨论"; "scheme" -> "社区方案"; "project" -> "开源项目"; else -> "社区周刊" }
        return renderCommunityShareCard(
            ShareContent(title = info.title, url = info.url, summary = info.summary, kindLabel = tag),
            ShareCardOptions(),
        )
    }

    override val supportsChatImageSave = true

    override suspend fun saveChatImage(bytes: ByteArray, mime: String): CommunityPosterResult {
        val extension = when (mime) {
            "image/png" -> "png"
            "image/jpeg" -> "jpg"
            "image/webp" -> "webp"
            "image/gif" -> "gif"
            else -> return CommunityPosterResult.Failed
        }
        return saveImage("xiaoban-image-${System.currentTimeMillis()}.$extension", mime) { it.write(bytes) }
    }

    override suspend fun saveCommunityPoster(image: ImageBitmap): CommunityPosterResult =
        saveImage("xiaoban-community-${System.currentTimeMillis()}.png", "image/png") {
            check(image.asAndroidBitmap().compress(Bitmap.CompressFormat.PNG, 100, it))
        }

    override val forumDownloadSupported = true

    override suspend fun saveForumDownload(filename: String, bytes: ByteArray): CommunityPosterResult =
        saveFile(filename, "application/octet-stream", download = true) { it.write(bytes) }

    private suspend fun saveImage(filename: String, mime: String, write: (java.io.OutputStream) -> Unit): CommunityPosterResult =
        saveFile(filename, mime, download = false, write = write)

    private suspend fun saveFile(filename: String, mime: String, download: Boolean, write: (java.io.OutputStream) -> Unit): CommunityPosterResult {
        var inserted: Uri? = null
        return try {
            if (Build.VERSION.SDK_INT >= 29) withContext(Dispatchers.IO) {
                val values = ContentValues().apply {
                    put(MediaStore.Images.Media.DISPLAY_NAME, filename)
                    put(MediaStore.Images.Media.MIME_TYPE, mime)
                    put(MediaStore.MediaColumns.RELATIVE_PATH, (if (download) Environment.DIRECTORY_DOWNLOADS else Environment.DIRECTORY_PICTURES) + "/Xiaoban")
                    put(MediaStore.Images.Media.IS_PENDING, 1)
                }
                val collection = if (download) MediaStore.Downloads.EXTERNAL_CONTENT_URI else MediaStore.Images.Media.EXTERNAL_CONTENT_URI
                val uri = activity.contentResolver.insert(collection, values) ?: return@withContext CommunityPosterResult.Failed
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

    private suspend fun writeCachedPoster(image: ImageBitmap): Uri {
        val file = withContext(Dispatchers.IO) {
            val directory = File(activity.cacheDir, "community-posters").apply { mkdirs() }
            directory.listFiles()?.filter { it.name.startsWith("poster-") && it.lastModified() < System.currentTimeMillis() - 86_400_000 }?.forEach { it.delete() }
            File.createTempFile("poster-", ".png", directory).apply {
                outputStream().use { check(image.asAndroidBitmap().compress(Bitmap.CompressFormat.PNG, 100, it)) }
            }
        }
        return FileProvider.getUriForFile(activity, activity.packageName + ".community-posters", file)
    }

    override suspend fun shareCommunityPoster(image: ImageBitmap): CommunityPosterResult = try {
        val uri = writeCachedPoster(image)
        withContext(Dispatchers.Main.immediate) {
            val send = Intent(Intent.ACTION_SEND).apply {
                type = "image/png"; putExtra(Intent.EXTRA_STREAM, uri)
                clipData = ClipData.newUri(activity.contentResolver, "社区分享卡片", uri)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            activity.startActivity(Intent.createChooser(send, "发送分享卡片"))
        }
        CommunityPosterResult.ChooserOpened
    } catch (e: CancellationException) { throw e }
      catch (_: SecurityException) { CommunityPosterResult.PermissionDenied }
      catch (_: Exception) { CommunityPosterResult.Failed }

    override suspend fun copyCommunityPoster(image: ImageBitmap): CommunityPosterResult = try {
        val uri = writeCachedPoster(image)
        withContext(Dispatchers.Main.immediate) {
            val manager = activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            manager.setPrimaryClip(ClipData.newUri(activity.contentResolver, "社区分享卡片", uri))
        }
        CommunityPosterResult.Saved
    } catch (e: CancellationException) { throw e }
      catch (_: SecurityException) { CommunityPosterResult.PermissionDenied }
      catch (_: Exception) { CommunityPosterResult.Failed }
}

private fun buildTextLayout(text: String, width: Int, size: Float, color: Int, lines: Int, bold: Boolean = false, lineSpacing: Float = 7f): StaticLayout {
    val clean = text.replace(Regex("\\s+"), " ").trim()
    val paint = TextPaint(Paint.ANTI_ALIAS_FLAG).apply {
        textSize = size; this.color = color
        typeface = Typeface.create("sans-serif", if (bold) Typeface.BOLD else Typeface.NORMAL)
    }
    return StaticLayout.Builder.obtain(clean, 0, clean.length, paint, width)
        .setAlignment(Layout.Alignment.ALIGN_NORMAL).setIncludePad(false).setMaxLines(lines)
        .setEllipsize(android.text.TextUtils.TruncateAt.END).setLineSpacing(lineSpacing, 1f).build()
}

private fun drawText(canvas: Canvas, text: String, x: Int, y: Int, width: Int, size: Float, color: Int, lines: Int, bold: Boolean = false) {
    val layout = buildTextLayout(text, width, size, color, lines, bold)
    canvas.save(); canvas.translate(x.toFloat(), y.toFloat()); layout.draw(canvas); canvas.restore()
}
