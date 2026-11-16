package xyz.xiangban.xiaoban

import androidx.compose.ui.window.ComposeUIViewController
import androidx.compose.runtime.remember
import platform.Foundation.NSUserDefaults
import xyz.xiangban.xiaoban.business.communitynative.AppPrefs

private class IOSPrefs : AppPrefs {
    private val defaults = NSUserDefaults.standardUserDefaults
    override fun get(key: String): String? = defaults.stringForKey("xiaoban.$key")
    override fun put(key: String, value: String) { defaults.setObject(value, forKey = "xiaoban.$key") }
}

fun MainViewController() = ComposeUIViewController { App(platform = remember { IOSCommunitySharing() }, prefs = remember { IOSPrefs() }) }
