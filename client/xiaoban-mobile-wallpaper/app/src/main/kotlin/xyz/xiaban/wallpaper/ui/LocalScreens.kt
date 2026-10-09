package xyz.xiaban.wallpaper.ui

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridState
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridItemSpan
import androidx.compose.foundation.lazy.staggeredgrid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.SingletonImageLoader
import xyz.xiaban.wallpaper.companion.Expression
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.GalleryNaming
import xyz.xiaban.wallpaper.data.VideoCache

/** Favourites: a snapshot of each card, stored only on this phone. */
@Composable
fun FavoritesScreen(app: AppViewModel, grid: LazyStaggeredGridState) {
    val favorites by app.favorites.collectAsState()
    Column(Modifier.fillMaxSize().background(Palette.Page)) {
        Column(Modifier.fillMaxWidth().background(Palette.Card).statusBarsPadding().padding(horizontal = 16.dp, vertical = 10.dp)) {
            Text("收藏", color = Palette.Text, fontSize = 20.sp, fontWeight = FontWeight.Bold)
            Text("只保存在这台手机上，不会上传，卸载应用后会清除", color = Palette.Sub, fontSize = 12.sp)
        }
        if (favorites.isEmpty()) {
            Center(Expression.LOVE, "还没有收藏\n点卡片右下角的心形，喜欢的壁纸就会放在这里")
        } else {
            Waterfall(grid) {
                items(favorites, key = { it.item.id }) { fav ->
                    OnlineCard(fav.item, favorite = true, onFavorite = { app.toggleFavorite(fav.item) }, onClick = { app.open(fav.item) })
                }
            }
        }
    }
}

/** My page: what was saved to the gallery, and a few settings. */
@Composable
fun MineScreen(app: AppViewModel, grid: LazyStaggeredGridState) {
    val saved by app.saved.collectAsState()
    val favorites by app.favorites.collectAsState()
    var section by rememberSaveable { mutableStateOf(0) }
    var chosen by remember { mutableStateOf<Downloaded?>(null) }
    val context = LocalContext.current

    Column(Modifier.fillMaxSize().background(Palette.Page)) {
        Column(Modifier.fillMaxWidth().background(Palette.Card).statusBarsPadding()) {
            Row(Modifier.padding(horizontal = 16.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                PetFace(Expression.HAPPY, Modifier.width(84.dp))
                Spacer(Modifier.width(12.dp))
                Column {
                    Text("小伴壁纸", color = Palette.Text, fontSize = 20.sp, fontWeight = FontWeight.Bold)
                    Spacer(Modifier.height(2.dp))
                    Text("下载的壁纸在相册里，收藏只在这台手机上", color = Palette.Sub, fontSize = 12.sp)
                    Spacer(Modifier.height(6.dp))
                    Text("已下载 ${saved.size}   收藏 ${favorites.size}", color = Palette.Text, fontSize = 13.sp)
                }
            }
            Row(Modifier.padding(horizontal = 8.dp)) {
                SectionTab("已下载", section == 0) { section = 0 }
                SectionTab("设置", section == 1) { section = 1 }
            }
        }
        when {
            section == 1 -> SettingsList(app, context)
            saved.isEmpty() -> Center(Expression.SLEEPY, "还没有保存过壁纸\n在壁纸或视频页打开一张，点「保存到相册」")
            else -> Waterfall(grid) {
                items(saved, key = { it.id }) { entry ->
                    SavedCard(entry, onClick = { app.view(entry.viewable()) }, onLongClick = { chosen = entry })
                }
                item(key = "end", span = StaggeredGridItemSpan.FullLine) {
                    Box(Modifier.fillMaxWidth().padding(vertical = 14.dp), contentAlignment = Alignment.Center) {
                        Text("都在这里了", color = Palette.Hint, fontSize = 12.sp)
                    }
                }
            }
        }
    }

    chosen?.let { entry ->
        AlertDialog(
            onDismissRequest = { chosen = null },
            title = { Text(entry.title, maxLines = 2) },
            text = { Text("保存在相册「${GalleryNaming.ALBUM}」里。点一下卡片可以在应用里全屏查看。", color = Palette.Sub, fontSize = 14.sp) },
            confirmButton = {
                TextButton(onClick = {
                    chosen = null
                    openInGallery(context, entry) { app.say("没有找到可以打开它的相册应用") }
                }) { Text("在相册中打开", color = Palette.Red) }
            },
            dismissButton = {
                TextButton(onClick = {
                    chosen = null
                    app.deleteSaved(entry)
                }) { Text("删除", color = Palette.Sub) }
            },
        )
    }
}

@Composable
private fun SectionTab(label: String, selected: Boolean, onClick: () -> Unit) {
    Column(Modifier.clickable(onClick = onClick).padding(horizontal = 12.dp, vertical = 8.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Text(
            label,
            color = if (selected) Palette.Text else Palette.Sub,
            fontSize = 15.sp,
            fontWeight = if (selected) FontWeight.Bold else FontWeight.Normal,
        )
        Spacer(Modifier.height(3.dp))
        Box(
            Modifier.width(16.dp).height(3.dp).clip(RoundedCornerShape(2.dp))
                .background(if (selected) Palette.Red else androidx.compose.ui.graphics.Color.Transparent),
        )
    }
}

@Composable
private fun SettingsList(app: AppViewModel, context: Context) {
    val loader = SingletonImageLoader.get(context)
    var cacheBytes by remember { mutableLongStateOf((loader.diskCache?.size ?: 0L) + VideoCache.sizeBytes(context)) }
    val version = remember { context.packageManager.getPackageInfo(context.packageName, 0).versionName.orEmpty() }
    Column(Modifier.fillMaxSize().padding(top = 8.dp), verticalArrangement = Arrangement.Top) {
        SettingRow("清除图片和视频缓存", sizeText(cacheBytes)) {
            loader.memoryCache?.clear()
            loader.diskCache?.clear()
            VideoCache.clear(context)
            cacheBytes = (loader.diskCache?.size ?: 0L) + VideoCache.sizeBytes(context)
            app.say("已清除")
        }
        SettingRow("保存位置", "相册 › ${GalleryNaming.ALBUM}")
        SettingRow("壁纸来源", "wallpaper.xiangban.xyz")
        SettingRow("版本", version)
    }
}

@Composable
private fun SettingRow(title: String, value: String, onClick: (() -> Unit)? = null) {
    Row(
        Modifier.fillMaxWidth().background(Palette.Card)
            .then(if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier)
            .padding(horizontal = 16.dp, vertical = 15.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(title, Modifier.weight(1f), color = Palette.Text, fontSize = 15.sp)
        Text(value, color = Palette.Sub, fontSize = 14.sp)
    }
    Box(Modifier.fillMaxWidth().height(0.5.dp).background(Palette.Line))
}
