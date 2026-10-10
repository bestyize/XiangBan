package xyz.xiaban.wallpaper.ui

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import xyz.xiaban.wallpaper.data.UpdateRules

/**
 * Hands an address to the browser. Only HTTPS addresses are opened, because these come from the website and a
 * notice must not be able to start other apps through an intent: or custom-scheme address.
 */
fun openLink(context: Context, url: String, onFail: () -> Unit) {
    if (!UpdateRules.isJumpLink(url)) return onFail()
    try {
        context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)).addCategory(Intent.CATEGORY_BROWSABLE))
    } catch (_: ActivityNotFoundException) {
        onFail()
    }
}
