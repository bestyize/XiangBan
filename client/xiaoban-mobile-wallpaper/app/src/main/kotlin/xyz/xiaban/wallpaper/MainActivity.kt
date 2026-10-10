package xyz.xiaban.wallpaper

import android.graphics.Color
import android.graphics.drawable.ColorDrawable
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import xyz.xiaban.wallpaper.ui.App
import xyz.xiaban.wallpaper.ui.ThemeMode
import xyz.xiaban.wallpaper.ui.XiaobanTheme
import xyz.xiaban.wallpaper.ui.isDark

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val prefs = getSharedPreferences("ui", MODE_PRIVATE)
        setContent {
            var mode by remember { mutableStateOf(ThemeMode.of(prefs.getString("theme", null))) }
            val dark = mode.isDark()
            DisposableEffect(dark) {
                enableEdgeToEdge(
                    statusBarStyle = SystemBarStyle.auto(Color.TRANSPARENT, Color.TRANSPARENT) { dark },
                    navigationBarStyle = SystemBarStyle.auto(
                        Color.argb(0xE6, 0xFF, 0xFF, 0xFF),
                        Color.argb(0x80, 0x1B, 0x1B, 0x1B),
                    ) { dark },
                )
                window.setBackgroundDrawable(ColorDrawable(if (dark) 0xFF0F0F0F.toInt() else 0xFFF7F7F7.toInt()))
                onDispose {}
            }
            XiaobanTheme(dark) {
                App(mode) {
                    mode = it
                    prefs.edit().putString("theme", it.key).apply()
                }
            }
        }
    }
}
