package xyz.xiaban.wallpaper.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Place
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.SingletonImageLoader
import xyz.xiaban.wallpaper.data.GalleryNaming
import xyz.xiaban.wallpaper.data.VideoCache

/** Settings page, opened from the 我的 tab: appearance, storage and about. */
@Composable
fun SettingsScreen(app: AppViewModel, themeMode: ThemeMode, onThemeMode: (ThemeMode) -> Unit, onClose: () -> Unit) {
    val context = LocalContext.current
    val loader = SingletonImageLoader.get(context)
    var cacheBytes by remember { mutableLongStateOf((loader.diskCache?.size ?: 0L) + VideoCache.sizeBytes(context)) }
    val version = remember { context.packageManager.getPackageInfo(context.packageName, 0).versionName.orEmpty() }

    BackHandler(onBack = onClose)

    Column(Modifier.fillMaxSize().background(Palette.Page).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(56.dp).padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(44.dp).clip(CircleShape).clickable(onClick = onClose), contentAlignment = Alignment.Center) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "返回", tint = Palette.Text)
            }
            Text("设置", Modifier.padding(start = 4.dp), color = Palette.Text, fontSize = 18.sp, fontWeight = FontWeight.Bold)
        }
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).navigationBarsPadding().padding(16.dp)) {
            SettingsGroup("外观") { ThemeRow(themeMode, onThemeMode) }
            Spacer(Modifier.height(20.dp))
            SettingsGroup("存储") {
                SettingRow(Icons.Outlined.Delete, Palette.Heart, Palette.HeartSoft, "清除图片和视频缓存", sizeText(cacheBytes)) {
                    loader.memoryCache?.clear()
                    loader.diskCache?.clear()
                    VideoCache.clear(context)
                    cacheBytes = (loader.diskCache?.size ?: 0L) + VideoCache.sizeBytes(context)
                    app.say("已清除")
                }
                SettingRow(Icons.Outlined.Place, Palette.Text, Palette.Field, "保存位置", "相册 › ${GalleryNaming.ALBUM}", last = true)
            }
            Spacer(Modifier.height(20.dp))
            SettingsGroup("关于") {
                SettingRow(Icons.Outlined.Home, Palette.Sky, Palette.SkySoft, "壁纸来源", "wallpaper.xiangban.xyz")
                SettingRow(Icons.Outlined.Info, Palette.Sub, Palette.Field, "版本", version, last = true)
            }
        }
    }
}

@Composable
private fun SettingsGroup(title: String, content: @Composable ColumnScope.() -> Unit) {
    Text(title, Modifier.padding(start = 4.dp, bottom = 8.dp), color = Palette.Sub, fontSize = 13.sp)
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(Palette.Card), content = content)
}

@Composable
private fun ThemeRow(selected: ThemeMode, onSelect: (ThemeMode) -> Unit) {
    Column(Modifier.fillMaxWidth().padding(16.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(36.dp).clip(RoundedCornerShape(11.dp)).background(Palette.Field), contentAlignment = Alignment.Center) {
                GlyphIcon(Glyph.Moon, Palette.Text, 20.dp)
            }
            Spacer(Modifier.width(12.dp))
            Text("深色模式", Modifier.weight(1f), color = Palette.Text, fontSize = 15.sp)
            Text(selected.label, color = Palette.Sub, fontSize = 13.sp)
        }
        Spacer(Modifier.height(14.dp))
        Row(
            Modifier.fillMaxWidth().clip(RoundedCornerShape(14.dp)).background(Palette.Field).padding(3.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            for (mode in ThemeMode.entries) {
                val on = mode == selected
                Box(
                    Modifier.weight(1f).height(38.dp).clip(RoundedCornerShape(11.dp))
                        .then(if (on) Modifier.background(Palette.Brand) else Modifier)
                        .clickable { onSelect(mode) },
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        mode.label,
                        color = if (on) Color.White else Palette.Sub,
                        fontSize = 14.sp,
                        fontWeight = if (on) FontWeight.Bold else FontWeight.Normal,
                    )
                }
            }
        }
    }
}

@Composable
fun IconTile(icon: ImageVector, tint: Color, soft: Color) {
    Box(Modifier.size(36.dp).clip(RoundedCornerShape(11.dp)).background(soft), contentAlignment = Alignment.Center) {
        Icon(icon, null, Modifier.size(20.dp), tint = tint)
    }
}

@Composable
private fun SettingRow(
    icon: ImageVector,
    tint: Color,
    soft: Color,
    title: String,
    value: String,
    last: Boolean = false,
    onClick: (() -> Unit)? = null,
) {
    Row(
        Modifier.fillMaxWidth()
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
            .padding(horizontal = 16.dp, vertical = 16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconTile(icon, tint, soft)
        Spacer(Modifier.width(12.dp))
        Text(title, Modifier.weight(1f), color = Palette.Text, fontSize = 15.sp)
        Text(value, color = Palette.Sub, fontSize = 13.sp)
    }
    if (!last) Box(Modifier.fillMaxWidth().padding(start = 64.dp).height(0.5.dp).background(Palette.Line))
}
