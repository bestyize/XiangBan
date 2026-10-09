package xyz.xiaban.wallpaper.data

import android.content.ContentResolver
import android.content.ContentValues
import android.graphics.BitmapFactory
import android.media.MediaMetadataRetriever
import android.net.Uri
import android.provider.BaseColumns
import android.provider.MediaStore
import java.io.File

/** The name a saved file gets in the gallery: readable title, a short content hash so repeats never collide. */
object GalleryNaming {
    private const val MAX_TITLE = 40

    fun fileName(title: String, sha256: String, extension: String): String {
        val cleaned = title.filter { it.isLetterOrDigit() || it == ' ' || it == '-' || it == '_' }.trim().take(MAX_TITLE).trim()
        return "${cleaned.ifEmpty { "xiaoban" }}_${sha256.take(8)}.$extension"
    }

    fun folder(kind: MediaKind) = if (kind == MediaKind.IMAGE) "Pictures/$ALBUM" else "Movies/$ALBUM"

    const val ALBUM = "小伴壁纸"
}

/** Where a finished download goes; the real one is the system gallery, tests use a stand-in. */
interface GallerySink {
    /** Returns the address of the new gallery entry. */
    fun save(file: File, kind: MediaKind, mime: String, displayName: String): String
}

/** Writes files into the system gallery through MediaStore. Needs no storage permission on Android 10 and later. */
class GalleryWriter(private val resolver: ContentResolver) : GallerySink {
    override fun save(file: File, kind: MediaKind, mime: String, displayName: String): String {
        val collection = if (kind == MediaKind.IMAGE) {
            MediaStore.Images.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        } else {
            MediaStore.Video.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        }
        val values = ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, displayName)
            put(MediaStore.MediaColumns.MIME_TYPE, mime)
            put(MediaStore.MediaColumns.RELATIVE_PATH, GalleryNaming.folder(kind))
            put(MediaStore.MediaColumns.IS_PENDING, 1)
        }
        val uri = resolver.insert(collection, values) ?: throw SiteException("无法写入相册")
        try {
            val out = resolver.openOutputStream(uri) ?: throw SiteException("无法写入相册")
            out.use { sink -> file.inputStream().use { it.copyTo(sink) } }
            resolver.update(uri, ContentValues().apply { put(MediaStore.MediaColumns.IS_PENDING, 0) }, null, null)
        } catch (e: Throwable) {
            resolver.delete(uri, null, null)
            throw if (e is SiteException) e else SiteException("写入相册失败：${e.message ?: "存储空间不足？"}")
        }
        return uri.toString()
    }

    fun exists(uri: String): Boolean = try {
        resolver.query(Uri.parse(uri), arrayOf(BaseColumns._ID), null, null, null)?.use { it.moveToFirst() } ?: false
    } catch (_: Exception) {
        false
    }

    fun delete(uri: String): Boolean = try {
        resolver.delete(Uri.parse(uri), null, null) > 0
    } catch (_: Exception) {
        false
    }
}

/** Checks that the phone itself can decode a file before it goes into the gallery. */
object MediaProbe {
    class Size(val width: Int, val height: Int)

    fun check(file: File, kind: MediaKind): Size = when (kind) {
        MediaKind.IMAGE -> {
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeFile(file.path, bounds)
            if (bounds.outWidth <= 0 || bounds.outHeight <= 0) throw SiteException("系统无法解码这张图片")
            Size(bounds.outWidth, bounds.outHeight)
        }
        MediaKind.VIDEO -> {
            val retriever = MediaMetadataRetriever()
            try {
                retriever.setDataSource(file.path)
                val w = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_WIDTH)?.toIntOrNull()
                val h = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_VIDEO_HEIGHT)?.toIntOrNull()
                if (w == null || h == null || w <= 0 || h <= 0) throw SiteException("系统无法解码这个视频")
                Size(w, h)
            } catch (e: RuntimeException) {
                throw SiteException("系统无法解码这个视频")
            } finally {
                retriever.release()
            }
        }
    }
}
