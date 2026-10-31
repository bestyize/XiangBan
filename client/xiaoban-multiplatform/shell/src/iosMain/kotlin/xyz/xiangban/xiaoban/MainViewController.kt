package xyz.xiangban.xiaoban

import androidx.compose.ui.window.ComposeUIViewController
import androidx.compose.runtime.remember

fun MainViewController() = ComposeUIViewController { App(platform = remember { IOSCommunitySharing() }) }
