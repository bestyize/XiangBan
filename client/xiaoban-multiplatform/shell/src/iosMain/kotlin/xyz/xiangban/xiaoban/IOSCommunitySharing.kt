package xyz.xiangban.xiaoban

import kotlinx.cinterop.ExperimentalForeignApi
import platform.Foundation.NSURL
import platform.UIKit.UIApplication
import platform.UIKit.UIPasteboard
import xyz.xiangban.xiaoban.business.communitynative.PlatformActions

/** Link-only fallback. Poster export and system sharing are explicitly unavailable on iOS. */
@OptIn(ExperimentalForeignApi::class)
class IOSCommunitySharing : PlatformActions {
    override val communitySystemShareSupported = false
    override fun copy(text: String) { UIPasteboard.generalPasteboard.string = text }
    override fun openUrl(url: String) {
        NSURL.URLWithString(url)?.let { UIApplication.sharedApplication.openURL(it, emptyMap<Any?, Any>(), null) }
    }
    override fun share(title: String, url: String) { copy(url) }
}
