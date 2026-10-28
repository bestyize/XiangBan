package xyz.xiangban.xiaoban

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import xyz.xiangban.xiaoban.common.link.AndroidKeystoreLinkStorage

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

        val storage = SharedPrefsTokenStorage(this)
        val platform = AndroidPlatformActions(this)
        val linkStorage = AndroidKeystoreLinkStorage(this)
        setContent {
            App(storage, platform, linkStorage)
        }
    }
}

@Preview
@Composable
fun AppAndroidPreview() {
    App()
}