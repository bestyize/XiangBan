package xyz.xiangban.xiaoban

import android.content.ClipData
import android.content.Context
import android.content.Intent
import android.graphics.*
import androidx.core.content.FileProvider
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.common.community.AndroidMobileWorkspace
import xyz.xiangban.xiaoban.common.community.ForumUploadFile
import xyz.xiangban.xiaoban.common.community.MobileGameStore
import java.io.ByteArrayOutputStream
import java.io.File
import java.util.UUID

/** Only a validated game snapshot or an explicitly rendered public preview can leave the app. */
internal class AndroidCreationSharing(private val context: Context) {
    suspend fun gamePackage(id: String): ForumUploadFile = withContext(Dispatchers.IO) {
        val store = MobileGameStore(AndroidMobileWorkspace(File(context.filesDir, "workspace")))
        ForumUploadFile("$id.zip", "application/zip", RuntimeV2Archive.encode(store.exportFiles(id)))
    }

    suspend fun saveGame(id: String): Boolean {
        val file = gamePackage(id)
        return withContext(Dispatchers.IO) {
            if (android.os.Build.VERSION.SDK_INT >= 29) {
                val resolver = context.contentResolver
                val values = android.content.ContentValues().apply {
                    put(android.provider.MediaStore.MediaColumns.DISPLAY_NAME, file.name)
                    put(android.provider.MediaStore.MediaColumns.MIME_TYPE, "application/zip")
                    put(android.provider.MediaStore.MediaColumns.RELATIVE_PATH, android.os.Environment.DIRECTORY_DOWNLOADS + "/Xiaoban")
                    put(android.provider.MediaStore.MediaColumns.IS_PENDING, 1)
                }
                val uri = resolver.insert(android.provider.MediaStore.Downloads.EXTERNAL_CONTENT_URI, values) ?: return@withContext false
                try {
                    requireNotNull(resolver.openOutputStream(uri)).use { it.write(file.bytes) }
                    resolver.update(uri, android.content.ContentValues().apply { put(android.provider.MediaStore.MediaColumns.IS_PENDING, 0) }, null, null) > 0
                } catch (e: Exception) { runCatching { resolver.delete(uri, null, null) }; false }
            } else false
        }
    }

    suspend fun shareGame(id: String): Boolean {
        val file = gamePackage(id)
        return shareFile(file.name, file.mime, file.bytes)
    }

    suspend fun cover(id: String): ByteArray? = withContext(Dispatchers.IO) {
        val file = when (id) {
            "sunflower" -> "sunflower.webp"
            "soft-hills" -> "soft-hills.svg"
            "daylight-clock" -> "clock.svg"
            "fireflies" -> "fireflies.svg"
            else -> return@withContext null
        }
        context.assets.open("creation-previews/$file").use { it.readBytes() }
    }

    suspend fun sharePreview(title: String, description: String, cover: ByteArray?): Boolean {
        val png = withContext(Dispatchers.Default) {
            val bitmap = Bitmap.createBitmap(720, 900, Bitmap.Config.ARGB_8888)
            try {
                val canvas = Canvas(bitmap)
                canvas.drawColor(Color.rgb(249, 248, 246))
                val paint = Paint(Paint.ANTI_ALIAS_FLAG)
                paint.shader = LinearGradient(0f, 0f, 720f, 480f, Color.rgb(244, 224, 226), Color.rgb(225, 232, 218), Shader.TileMode.CLAMP)
                canvas.drawRoundRect(32f, 32f, 688f, 490f, 32f, 32f, paint)
                paint.shader = null
                // SVG covers remain text-only here; no untrusted HTML is executed to take a screenshot.
                val image = cover?.let { BitmapFactory.decodeByteArray(it, 0, it.size) }
                if (image != null) {
                    val scale = minOf(560f / image.width, 390f / image.height)
                    val w = image.width * scale; val h = image.height * scale
                    canvas.drawBitmap(image, null, RectF(360f-w/2, 261f-h/2, 360f+w/2, 261f+h/2), paint)
                    image.recycle()
                } else {
                    paint.color = Color.rgb(149, 119, 123); paint.textSize = 110f; paint.textAlign = Paint.Align.CENTER
                    canvas.drawText(title.take(2), 360f, 290f, paint); paint.textAlign = Paint.Align.LEFT
                }
                paint.color = Color.rgb(39, 41, 45); paint.textSize = 38f; paint.typeface = Typeface.create(Typeface.DEFAULT, Typeface.BOLD)
                canvas.drawText(title.take(16), 48f, 563f, paint)
                paint.typeface = Typeface.DEFAULT; paint.textSize = 25f; paint.color = Color.rgb(109, 112, 119)
                var text = description.replace('\n', ' ').take(160)
                repeat(4) { row ->
                    val count = paint.breakText(text, true, 624f, null)
                    canvas.drawText(text.take(count), 48f, 620f + row * 38f, paint)
                    text = text.drop(count)
                }
                paint.color = Color.rgb(220, 63, 84); paint.textSize = 26f
                canvas.drawText("小伴 · 创造", 48f, 840f, paint)
                ByteArrayOutputStream().use { out -> bitmap.compress(Bitmap.CompressFormat.PNG, 100, out); out.toByteArray() }
            } finally { bitmap.recycle() }
        }
        return shareFile("xiaoban-creation.png", "image/png", png)
    }

    private suspend fun shareFile(name: String, mime: String, bytes: ByteArray): Boolean {
        val file = withContext(Dispatchers.IO) {
            val directory = File(context.cacheDir, "creation-share").apply { mkdirs() }
            // Retain recent chooser grants; clean only stale files, never overwrite another share.
            directory.listFiles()?.filter { System.currentTimeMillis() - it.lastModified() > 7L * 24 * 60 * 60 * 1000 }?.forEach { if (it.isDirectory) it.deleteRecursively() else it.delete() }
            val requestDirectory = File(directory, UUID.randomUUID().toString()).apply { mkdirs() }
            File(requestDirectory, name).apply { writeBytes(bytes) }
        }
        return withContext(Dispatchers.Main) {
            val uri = FileProvider.getUriForFile(context, context.packageName + ".community-posters", file)
            val intent = Intent(Intent.ACTION_SEND).apply {
                type = mime
                putExtra(Intent.EXTRA_STREAM, uri)
                clipData = ClipData.newRawUri(name, uri)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            context.startActivity(Intent.createChooser(intent, "分享作品").addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
            true
        }
    }
}
