package xyz.xiaban.wallpaper

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import xyz.xiaban.wallpaper.ui.App
import xyz.xiaban.wallpaper.ui.XiaobanTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent { XiaobanTheme { App() } }
    }
}
