package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridState
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridItemSpan
import androidx.compose.foundation.lazy.staggeredgrid.items as gridItems
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiaban.wallpaper.companion.Expression
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.GalleryNaming

/** Favourites: a snapshot of each card, stored only on this phone. */
@Composable
fun FavoritesScreen(app: AppViewModel, grid: LazyListState) {
    val favorites by app.favorites.collectAsState()
    Column(Modifier.fillMaxSize().background(Palette.Page)) {
        Column(Modifier.fillMaxWidth().statusBarsPadding().padding(start = 16.dp, end = 16.dp, top = 14.dp, bottom = 10.dp)) {
            Text("收藏", color = Palette.Text, fontSize = 22.sp, fontWeight = FontWeight.Bold)
            Spacer(Modifier.height(2.dp))
            Text("只保存在这台手机上，不会上传，卸载应用后会清除", color = Palette.Sub, fontSize = 12.sp)
        }
        if (favorites.isEmpty()) {
            Center(Expression.LOVE, "还没有收藏\n点卡片右下角的心形，喜欢的壁纸就会放在这里")
        } else {
            FeedList(grid) {
                items(favorites, key = { it.item.id }) { fav ->
                    OnlineCard(fav.item, favorite = true, onFavorite = { app.toggleFavorite(fav.item) }, onClick = { app.open(fav.item) })
                }
            }
        }
    }
}

/** My page: what was saved to the gallery, and the way into settings. */
@Composable
fun MineScreen(app: AppViewModel, grid: LazyStaggeredGridState, onOpenSettings: () -> Unit) {
    val saved by app.saved.collectAsState()
    val favorites by app.favorites.collectAsState()
    val update by app.update.collectAsState()
    var chosen by remember { mutableStateOf<Downloaded?>(null) }
    val context = LocalContext.current

    Column(Modifier.fillMaxSize().background(Palette.Page)) {
        Column(Modifier.fillMaxWidth().statusBarsPadding()) {
            Row(Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text("小伴壁纸", color = Palette.Text, fontSize = 22.sp, fontWeight = FontWeight.Bold)
                    Spacer(Modifier.height(4.dp))
                    Text("让每一张屏幕，都有温度", color = Palette.Sub, fontSize = 13.sp)
                }
                PetFace(Expression.HAPPY, Modifier.width(96.dp))
            }
            AccountEntry(app)
            Row(
                Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp)
                    .clip(RoundedCornerShape(16.dp)).background(Palette.Card).padding(vertical = 14.dp),
            ) {
                Stat("已下载", saved.size, Modifier.weight(1f))
                Box(Modifier.width(0.5.dp).height(32.dp).background(Palette.Line).align(Alignment.CenterVertically))
                Stat("收藏", favorites.size, Modifier.weight(1f))
            }
            SettingsEntry(onOpenSettings, newVersion = update?.version)
            Text("已下载", Modifier.padding(start = 20.dp, top = 18.dp, bottom = 8.dp), color = Palette.Text, fontSize = 16.sp, fontWeight = FontWeight.Bold)
        }
        when {
            saved.isEmpty() -> Center(Expression.SLEEPY, "还没有保存过壁纸\n在壁纸或视频页打开一张，点「保存到相册」")
            else -> Waterfall(grid) {
                gridItems(saved, key = { it.id }) { entry ->
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
                }) { Text("在相册中打开", color = Palette.Brand) }
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

/** Who is signed in, or an invitation to sign in. Signing out asks first. */
@Composable
private fun AccountEntry(app: AppViewModel) {
    val account by app.account.collectAsState()
    var confirmOut by remember { mutableStateOf(false) }
    val user = account?.user

    Row(
        Modifier.fillMaxWidth().padding(horizontal = 16.dp).clip(RoundedCornerShape(16.dp)).background(Palette.Card)
            .then(if (user == null) Modifier.clickable(onClick = app::openLogin) else Modifier)
            .padding(horizontal = 16.dp, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(44.dp).clip(CircleShape).background(if (user == null) Palette.Field else Palette.BrandSoft), contentAlignment = Alignment.Center) {
            if (user == null) GlyphIcon(Glyph.User, Palette.Sub, 24.dp)
            else Text(user.nickname.take(1), color = Palette.Brand, fontSize = 18.sp, fontWeight = FontWeight.Bold)
        }
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Text(user?.nickname ?: "登录 / 注册", color = Palette.Text, fontSize = 16.sp, fontWeight = FontWeight.Bold, maxLines = 1)
            Text(user?.email ?: "与桌面版共用同一账号", color = Palette.Sub, fontSize = 12.sp, maxLines = 1)
        }
        if (user == null) {
            Box(Modifier.clip(CircleShape).background(Palette.Brand).padding(horizontal = 16.dp, vertical = 7.dp)) {
                Text("登录", color = Color.White, fontSize = 13.sp, fontWeight = FontWeight.Bold)
            }
        } else {
            Text("退出", Modifier.clip(CircleShape).clickable { confirmOut = true }.padding(horizontal = 10.dp, vertical = 6.dp), color = Palette.Sub, fontSize = 13.sp)
        }
    }

    if (confirmOut) {
        AlertDialog(
            onDismissRequest = { confirmOut = false },
            title = { Text("退出登录") },
            text = { Text("退出后，这台手机上的登录状态会清除。收藏和已下载的壁纸不受影响。", color = Palette.Sub, fontSize = 14.sp) },
            confirmButton = {
                TextButton(onClick = {
                    confirmOut = false
                    app.logout()
                }) { Text("退出", color = Palette.Brand) }
            },
            dismissButton = { TextButton(onClick = { confirmOut = false }) { Text("取消", color = Palette.Sub) } },
        )
    }
}

@Composable
private fun Stat(label: String, count: Int, modifier: Modifier) {
    Column(modifier, horizontalAlignment = Alignment.CenterHorizontally) {
        Text(count.toString(), color = Palette.Text, fontSize = 20.sp, fontWeight = FontWeight.Bold)
        Text(label, color = Palette.Sub, fontSize = 12.sp)
    }
}

@Composable
private fun SettingsEntry(onClick: () -> Unit, newVersion: String?) {
    Row(
        Modifier.fillMaxWidth().padding(horizontal = 16.dp).clip(RoundedCornerShape(16.dp)).background(Palette.Card)
            .clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconTile(Icons.Outlined.Settings, Palette.Text, Palette.Field)
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("设置", color = Palette.Text, fontSize = 15.sp)
                if (newVersion != null) {
                    Box(Modifier.padding(start = 6.dp).size(8.dp).clip(CircleShape).background(Palette.Brand).semantics { contentDescription = "有新版本" })
                }
            }
            Text(if (newVersion != null) "发现新版本 $newVersion" else "深色模式、清除缓存、关于", color = if (newVersion != null) Palette.Brand else Palette.Sub, fontSize = 12.sp)
        }
        Icon(Icons.AutoMirrored.Filled.KeyboardArrowRight, null, tint = Palette.Hint)
    }
}
