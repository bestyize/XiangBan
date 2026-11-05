package xyz.xiangban.xiaoban

import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.usePinned
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import platform.Foundation.NSData
import platform.Foundation.NSFileManager
import platform.Foundation.NSTemporaryDirectory
import platform.Foundation.NSUUID
import platform.Foundation.create
import platform.Foundation.NSURL
import platform.Foundation.writeToFile
import platform.UIKit.UIActivityViewController
import platform.UIKit.UIApplication
import platform.UIKit.UIPasteboard
import platform.UIKit.popoverPresentationController
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions
import xyz.xiangban.xiaoban.business.communitynative.CommunityPosterResult

/** Link sharing and clipboard support. Community poster export is unavailable on iOS. */
@OptIn(ExperimentalForeignApi::class)
class IOSCommunitySharing : PlatformActions {
    override val communitySystemShareSupported = false
    override fun copy(text: String) { UIPasteboard.generalPasteboard.string = text }
    override fun openUrl(url: String) {
        NSURL.URLWithString(url)?.let { UIApplication.sharedApplication.openURL(it, emptyMap<Any?, Any>(), null) }
    }
    override val forumDownloadSupported = true
    override val supportsChatImageSave = true

    override suspend fun saveChatImage(bytes: ByteArray, mime: String): CommunityPosterResult {
        val extension = when (mime) {
            "image/png" -> "png"
            "image/jpeg" -> "jpg"
            "image/webp" -> "webp"
            "image/gif" -> "gif"
            else -> return CommunityPosterResult.Failed
        }
        return saveForumDownload("xiaoban-image-${NSUUID().UUIDString}.$extension", bytes)
    }

    override suspend fun saveForumDownload(filename: String, bytes: ByteArray): CommunityPosterResult =
        withContext(Dispatchers.Main) {
            val directory = NSTemporaryDirectory() + "forum-" + NSUUID().UUIDString
            val fileManager = NSFileManager.defaultManager
            if (!fileManager.createDirectoryAtPath(directory, withIntermediateDirectories = true, attributes = null, error = null)) {
                return@withContext CommunityPosterResult.Failed
            }
            val path = "$directory/$filename"
            val data = if (bytes.isEmpty()) NSData() else bytes.usePinned {
                NSData.create(bytes = it.addressOf(0), length = bytes.size.toULong())
            }
            if (!data.writeToFile(path, atomically = true) ||
                !presentShare(NSURL.fileURLWithPath(path)) { fileManager.removeItemAtPath(directory, error = null) }
            ) {
                fileManager.removeItemAtPath(directory, error = null)
                return@withContext CommunityPosterResult.Failed
            }
            CommunityPosterResult.ChooserOpened
        }

    override fun share(title: String, url: String) {
        NSURL.URLWithString(url)?.let { presentShare(it) }
    }

    private fun presentShare(item: NSURL, cleanup: (() -> Unit)? = null): Boolean {
        var presenter = UIApplication.sharedApplication.keyWindow?.rootViewController ?: return false
        while (true) {
            presenter = presenter.presentedViewController ?: break
        }
        val sheet = UIActivityViewController(activityItems = listOf(item), applicationActivities = null)
        if (cleanup != null) sheet.completionWithItemsHandler = { _, _, _, _ -> cleanup() }
        sheet.popoverPresentationController?.sourceView = presenter.view
        sheet.popoverPresentationController?.sourceRect = presenter.view.bounds
        presenter.presentViewController(sheet, animated = true, completion = null)
        return true
    }
}
