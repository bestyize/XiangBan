package xyz.xiangban.xiaoban

import kotlinx.cinterop.BetaInteropApi
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.usePinned
import kotlinx.coroutines.CancellableContinuation
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlinx.coroutines.withContext
import platform.Foundation.NSData
import platform.Foundation.NSDataReadingMappedIfSafe
import platform.Foundation.NSFileManager
import platform.Foundation.NSFileSize
import platform.Foundation.NSNumber
import platform.Foundation.NSURL
import platform.Foundation.create
import platform.PhotosUI.PHPickerConfiguration
import platform.PhotosUI.PHPickerFilter
import platform.PhotosUI.PHPickerResult
import platform.PhotosUI.PHPickerViewController
import platform.PhotosUI.PHPickerViewControllerDelegateProtocol
import platform.UIKit.UIApplication
import platform.darwin.NSObject
import platform.posix.memcpy
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.FORUM_IMAGE_MAX_BYTES
import xyz.xiangban.xiaoban.common.community.ForumUploadFile
import xyz.xiangban.xiaoban.common.community.forumUploadName
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

private val ForumPhotoTypes = listOf(
    "public.jpeg" to "image/jpeg",
    "public.png" to "image/png",
    "com.compuserve.gif" to "image/gif",
    "org.webmproject.webp" to "image/webp",
)

/** PHPicker grants access to only the photo the user chose. */
@OptIn(ExperimentalForeignApi::class, BetaInteropApi::class)
internal class IOSForumPhotoPicker : NSObject(), PHPickerViewControllerDelegateProtocol {
    private var waiting: CancellableContinuation<PHPickerResult?>? = null

    suspend fun pick(maxBytes: Long): ForumUploadFile? {
        val limit = minOf(maxBytes, FORUM_IMAGE_MAX_BYTES)
        if (limit <= 0) throw ApiException(403, "当前账号暂不能插图")
        val result = withContext(Dispatchers.Main) {
            suspendCancellableCoroutine<PHPickerResult?> { continuation ->
                if (waiting != null) {
                    continuation.resumeWithException(ApiException(400, "已有图片选择正在进行"))
                    return@suspendCancellableCoroutine
                }
                var presenter = UIApplication.sharedApplication.keyWindow?.rootViewController
                while (presenter?.presentedViewController != null) presenter = presenter.presentedViewController
                if (presenter == null) {
                    continuation.resumeWithException(ApiException(400, "无法打开系统相册"))
                    return@suspendCancellableCoroutine
                }
                val configuration = PHPickerConfiguration().apply {
                    filter = PHPickerFilter.imagesFilter
                    selectionLimit = 1
                }
                val picker = PHPickerViewController(configuration)
                picker.delegate = this@IOSForumPhotoPicker
                waiting = continuation
                presenter.presentViewController(picker, animated = true, completion = null)
            }
        } ?: return null
        val provider = result.itemProvider
        val (type, mime) = ForumPhotoTypes.firstOrNull { provider.hasItemConformingToTypeIdentifier(it.first) }
            ?: throw ApiException(400, "请选择 JPEG、PNG、GIF 或 WebP 图片")
        return suspendCancellableCoroutine { continuation ->
            val progress = provider.loadFileRepresentationForTypeIdentifier(type) { url, _ ->
                if (continuation.isActive) {
                    try {
                        val bytes = readPhoto(url, limit)
                        val fallback = "image." + mime.substringAfter('/')
                        val name = forumUploadName(provider.suggestedName ?: url?.lastPathComponent ?: fallback)
                        continuation.resume(ForumUploadFile(name, mime, bytes))
                    } catch (e: Exception) {
                        continuation.resumeWithException(e)
                    }
                }
            }
            continuation.invokeOnCancellation { progress.cancel() }
        }
    }

    override fun picker(picker: PHPickerViewController, didFinishPicking: List<*>) {
        picker.dismissViewControllerAnimated(true, completion = null)
        val continuation = waiting
        waiting = null
        if (continuation?.isActive == true) continuation.resume(didFinishPicking.firstOrNull() as? PHPickerResult)
    }

    private fun readPhoto(url: NSURL?, limit: Long): ByteArray {
        val path = url?.path ?: throw ApiException(400, "无法读取这张图片，请重新选择")
        val attributes = NSFileManager.defaultManager.attributesOfItemAtPath(path, error = null)
        val size = (attributes?.get(NSFileSize) as? NSNumber)?.unsignedLongLongValue
            ?: throw ApiException(400, "无法读取图片大小，请重新选择")
        if (size > limit.toULong()) throw ApiException(400, "图片超过大小上限（${limit / 1024 / 1024} MB）")
        if (size == 0UL) throw ApiException(400, "图片是空的")
        val data = NSData.create(contentsOfFile = path, options = NSDataReadingMappedIfSafe, error = null)
            ?: throw ApiException(400, "无法读取这张图片，请重新选择")
        if (data.length != size) throw ApiException(400, "图片读取不完整，请重新选择")
        return ByteArray(data.length.toInt()).also { bytes ->
            bytes.usePinned { memcpy(it.addressOf(0), data.bytes, data.length) }
        }
    }
}
