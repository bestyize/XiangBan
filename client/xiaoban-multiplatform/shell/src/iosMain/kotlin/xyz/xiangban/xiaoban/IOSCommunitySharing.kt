package xyz.xiangban.xiaoban

import kotlinx.cinterop.ExperimentalForeignApi
import platform.Foundation.NSURL
import platform.UIKit.UIActivityViewController
import platform.UIKit.UIApplication
import platform.UIKit.UIPasteboard
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions

/** Link sharing and clipboard support. Community poster export is unavailable on iOS. */
@OptIn(ExperimentalForeignApi::class)
class IOSCommunitySharing : PlatformActions {
    override val communitySystemShareSupported = false
    override fun copy(text: String) { UIPasteboard.generalPasteboard.string = text }
    override fun openUrl(url: String) {
        NSURL.URLWithString(url)?.let { UIApplication.sharedApplication.openURL(it, emptyMap<Any?, Any>(), null) }
    }
    override fun share(title: String, url: String) {
        val link = NSURL.URLWithString(url) ?: return
        var presenter = UIApplication.sharedApplication.keyWindow?.rootViewController ?: return
        while (true) {
            presenter = presenter.presentedViewController ?: break
        }
        val sheet = UIActivityViewController(activityItems = listOf(link), applicationActivities = null)
        sheet.popoverPresentationController?.sourceView = presenter.view
        sheet.popoverPresentationController?.sourceRect = presenter.view.bounds
        presenter.presentViewController(sheet, animated = true, completion = null)
    }
}
