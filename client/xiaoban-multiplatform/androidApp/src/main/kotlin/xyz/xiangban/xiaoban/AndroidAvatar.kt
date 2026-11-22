package xyz.xiangban.xiaoban

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Matrix
import android.media.ExifInterface
import android.os.Build
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asAndroidBitmap
import androidx.compose.ui.graphics.asImageBitmap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.business.communitynative.AVATAR_MAX_BYTES
import xyz.xiangban.xiaoban.business.communitynative.AVATAR_SOURCE_MAX_SIDE
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.ForumUploadFile
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream

/** 更换头像的原图：只在本机解码，按 EXIF 摆正，长边缩到 [AVATAR_SOURCE_MAX_SIDE] 以内。 */
internal suspend fun decodeLocalAvatar(file: ForumUploadFile): ImageBitmap = withContext(Dispatchers.Default) {
    if (!file.mime.startsWith("image/")) throw ApiException(400, "请选择图片文件")
    if (file.bytes.size > AVATAR_MAX_BYTES) throw ApiException(400, "图片超过 10 MB，换一张试试")
    val options = BitmapFactory.Options().apply { inJustDecodeBounds = true }
    BitmapFactory.decodeByteArray(file.bytes, 0, file.bytes.size, options)
    if (options.outWidth <= 0 || options.outHeight <= 0) throw ApiException(400, "这张图片打不开，换一张试试")
    options.inSampleSize = 1
    while (maxOf(options.outWidth, options.outHeight) / options.inSampleSize > AVATAR_SOURCE_MAX_SIDE * 2) options.inSampleSize *= 2
    options.inJustDecodeBounds = false
    val decoded = BitmapFactory.decodeByteArray(file.bytes, 0, file.bytes.size, options) ?: throw ApiException(400, "这张图片打不开，换一张试试")
    val matrix = Matrix()
    val orientation = runCatching { ExifInterface(ByteArrayInputStream(file.bytes)).getAttributeInt(ExifInterface.TAG_ORIENTATION, ExifInterface.ORIENTATION_NORMAL) }
        .getOrDefault(ExifInterface.ORIENTATION_NORMAL)
    when (orientation) {
        ExifInterface.ORIENTATION_FLIP_HORIZONTAL -> matrix.setScale(-1f, 1f)
        ExifInterface.ORIENTATION_ROTATE_180 -> matrix.setRotate(180f)
        ExifInterface.ORIENTATION_FLIP_VERTICAL -> matrix.setScale(1f, -1f)
        ExifInterface.ORIENTATION_TRANSPOSE -> { matrix.setRotate(90f); matrix.postScale(-1f, 1f) }
        ExifInterface.ORIENTATION_ROTATE_90 -> matrix.setRotate(90f)
        ExifInterface.ORIENTATION_TRANSVERSE -> { matrix.setRotate(-90f); matrix.postScale(-1f, 1f) }
        ExifInterface.ORIENTATION_ROTATE_270 -> matrix.setRotate(-90f)
    }
    val longSide = maxOf(decoded.width, decoded.height)
    if (longSide > AVATAR_SOURCE_MAX_SIDE) {
        val k = AVATAR_SOURCE_MAX_SIDE.toFloat() / longSide
        matrix.postScale(k, k)
    }
    val upright = if (matrix.isIdentity) decoded else Bitmap.createBitmap(decoded, 0, 0, decoded.width, decoded.height, matrix, true).also { if (it !== decoded) decoded.recycle() }
    upright.asImageBitmap()
}

internal suspend fun encodeLocalAvatar(image: ImageBitmap): ByteArray = withContext(Dispatchers.Default) {
    val output = ByteArrayOutputStream()
    val format = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
        Bitmap.CompressFormat.WEBP_LOSSY
    } else {
        @Suppress("DEPRECATION")
        Bitmap.CompressFormat.WEBP
    }
    if (!image.asAndroidBitmap().compress(format, 88, output)) throw ApiException(0, "生成头像失败，请重试")
    output.toByteArray()
}
