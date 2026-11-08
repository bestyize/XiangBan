package xyz.xiangban.xiaoban

import android.content.ClipData
import android.content.ContentValues
import android.content.Intent
import android.graphics.BitmapFactory
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.util.Base64
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.FileProvider
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.IOException

/** The MBTI image comes from the official bundle; the user always chooses the destination. */
internal class RuntimeV2MbtiShare(private val activity: ComponentActivity) {
    private var pending: Pair<ByteArray, (String?, String?) -> Unit>? = null
    private val saveDocument = activity.registerForActivityResult(ActivityResultContracts.CreateDocument("image/png")) { uri ->
        val request = pending
        pending = null
        if (request != null) {
            if (uri == null) request.second("cancelled", null)
            else {
                try {
                    activity.contentResolver.openOutputStream(uri)?.use { it.write(request.first) } ?: throw IOException("No output stream")
                    request.second("saved", null)
                } catch (_: Exception) { request.second(null, "busy") }
            }
        }
    }

    suspend fun perform(action: String, image: String, url: String, reply: (String?, String?) -> Unit) {
        if (pending != null) { reply(null, "busy"); return }
        if (action == "link") {
            val link = validatedLink(url) ?: run { reply(null, "invalid"); return }
            try {
                val send = Intent(Intent.ACTION_SEND).apply { type = "text/plain"; putExtra(Intent.EXTRA_TEXT, link) }
                activity.startActivity(Intent.createChooser(send, "分享结果链接"))
                reply("opened", null)
            } catch (_: Exception) { reply(null, "unsupported") }
            return
        }
        if (action != "save" && action != "image") { reply(null, "invalid"); return }
        val bytes = withContext(Dispatchers.Default) { decodePng(image) }
        if (bytes == null) { reply(null, "invalid"); return }
        if (action == "save") {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                try {
                    withContext(Dispatchers.IO) { saveToGallery(bytes) }
                    reply("saved", null)
                } catch (e: CancellationException) { throw e }
                catch (_: Exception) { reply(null, "busy") }
                return
            }
            pending = bytes to reply
            try { saveDocument.launch("小伴人格画像.png") }
            catch (_: Exception) { pending = null; reply(null, "unsupported") }
            return
        }
        try {
            val file = withContext(Dispatchers.IO) {
                val folder = File(activity.cacheDir, "community-posters").apply { mkdirs() }
                folder.listFiles()?.filter { it.name.startsWith("mbti-") && it.lastModified() < System.currentTimeMillis() - 86_400_000 }?.forEach { it.delete() }
                File.createTempFile("mbti-", ".png", folder).apply { writeBytes(bytes) }
            }
            val uri = FileProvider.getUriForFile(activity, activity.packageName + ".community-posters", file)
            val send = Intent(Intent.ACTION_SEND).apply {
                type = "image/png"; putExtra(Intent.EXTRA_STREAM, uri)
                clipData = ClipData.newUri(activity.contentResolver, "人格海报", uri)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            activity.startActivity(Intent.createChooser(send, "分享人格海报"))
            reply("opened", null)
        } catch (e: CancellationException) { throw e }
        catch (_: Exception) { reply(null, "busy") }
    }

    private fun saveToGallery(bytes: ByteArray) {
        val resolver = activity.contentResolver
        val values = ContentValues().apply {
            put(MediaStore.Images.Media.DISPLAY_NAME, "小伴人格画像-${System.currentTimeMillis()}.png")
            put(MediaStore.Images.Media.MIME_TYPE, "image/png")
            put(MediaStore.Images.Media.RELATIVE_PATH, "${Environment.DIRECTORY_PICTURES}/小伴")
            put(MediaStore.Images.Media.IS_PENDING, 1)
        }
        val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values) ?: throw IOException("Cannot create image")
        try {
            resolver.openOutputStream(uri)?.use { it.write(bytes) } ?: throw IOException("No output stream")
            if (resolver.update(uri, ContentValues().apply { put(MediaStore.Images.Media.IS_PENDING, 0) }, null, null) != 1) throw IOException("Cannot publish image")
        } catch (error: Exception) {
            resolver.delete(uri, null, null)
            throw error
        }
    }

    companion object {
        private const val PREFIX = "data:image/png;base64,"
        internal fun decodePng(value: String): ByteArray? {
            if (!value.startsWith(PREFIX) || value.length > 2_800_000) return null
            val encoded = value.substring(PREFIX.length)
            if (encoded.any { it !in 'A'..'Z' && it !in 'a'..'z' && it !in '0'..'9' && it != '+' && it != '/' && it != '=' }) return null
            val bytes = try { Base64.decode(encoded, Base64.NO_WRAP) } catch (_: IllegalArgumentException) { return null }
            if (bytes.size > 2 * 1024 * 1024 || bytes.size < 24 || !bytes.copyOfRange(0, 8).contentEquals(byteArrayOf(-119,80,78,71,13,10,26,10))) return null
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
            return bytes.takeIf { bounds.outMimeType == "image/png" && bounds.outWidth == 1080 && bounds.outHeight == 1440 }
        }

        internal fun validatedLink(value: String): String? {
            if (value.length > 512) return null
            val uri = runCatching { java.net.URI(value) }.getOrNull() ?: return null
            if (uri.scheme != "https" || uri.rawAuthority != "chat.xiangban.xyz" ||
                uri.rawPath != "/builtin-games/mbti/share.html" || uri.rawFragment != null) return null
            val params = linkedMapOf<String, String>()
            for (piece in uri.rawQuery?.split('&') ?: return null) {
                val parts = piece.split('=', limit = 2)
                if (parts.size != 2 || parts[0] !in setOf("t", "p", "n", "d") || params.containsKey(parts[0])) return null
                params[parts[0]] = runCatching { java.net.URLDecoder.decode(parts[1], "UTF-8") }.getOrNull() ?: return null
            }
            if (params.keys != setOf("t", "p", "n", "d")) return null
            val type = params["t"].orEmpty()
            if (!Regex("[EI][SN][TF][JP]").matches(type)) return null
            if (!Regex("\\d{1,3}(,\\d{1,3}){3}").matches(params["p"].orEmpty()) ||
                !Regex("\\d{1,2}(,\\d{1,2}){3}").matches(params["n"].orEmpty())) return null
            val nums = params["p"].orEmpty().split(',').mapNotNull { it.toIntOrNull() }
            val totals = params["n"].orEmpty().split(',').mapNotNull { it.toIntOrNull() }
            if (nums.size != 4 || nums.any { it !in 0..100 } || totals.size != 4 || totals.any { it !in 1..12 }) return null
            if (nums.indices.any { type[it] != listOf("EI", "SN", "TF", "JP")[it][if (nums[it] >= 50) 0 else 1] }) return null
            if (!Regex("\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}\\.\\d{3}Z").matches(params["d"].orEmpty())) return null
            return value
        }
    }
}
