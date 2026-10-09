package xyz.xiaban.wallpaper.ui

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
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
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.outlined.FavoriteBorder
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.crossfade
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.SiteItem
import xyz.xiaban.wallpaper.data.Stage

/** Opens a saved wallpaper in whichever gallery app the user prefers. */
fun openInGallery(context: Context, entry: Downloaded, onFail: () -> Unit) {
    val mime = if (entry.kind == MediaKind.IMAGE) "image/*" else "video/*"
    val view = Intent(Intent.ACTION_VIEW).setDataAndType(Uri.parse(entry.uri), mime).addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    try {
        context.startActivity(view)
    } catch (_: ActivityNotFoundException) {
        onFail()
    }
}

private const val MIN_DETAIL_ASPECT = 0.5f
private const val MAX_DETAIL_ASPECT = 1.78f

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun DetailScreen(item: SiteItem, app: AppViewModel) {
    val favorites by app.favorites.collectAsState()
    val saved by app.saved.collectAsState()
    val transfers by app.transfers.collectAsState()
    val favorite = favorites.any { it.item.id == item.id }
    val entry = saved.firstOrNull { it.id == item.id }
    val transfer = transfers[item.id]
    val context = LocalContext.current

    BackHandler { app.close() }

    Column(Modifier.fillMaxSize().background(Palette.Card).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(52.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(52.dp).clickable { app.close() }, contentAlignment = Alignment.Center) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "返回", tint = Palette.Text)
            }
            Text(item.author.ifBlank { "小伴壁纸" }, color = Palette.Text, fontSize = 15.sp, fontWeight = FontWeight.Medium, maxLines = 1)
        }
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState())) {
            Media(item, onOpen = { app.view(item.viewable()) })
            Column(Modifier.padding(16.dp)) {
                Text(item.title, color = Palette.Text, fontSize = 18.sp, fontWeight = FontWeight.Bold, lineHeight = 24.sp)
                Spacer(Modifier.height(8.dp))
                Text(metaLine(item), color = Palette.Sub, fontSize = 13.sp)
                if (item.tags.isNotEmpty()) {
                    Spacer(Modifier.height(12.dp))
                    FlowRow(horizontalArrangement = androidx.compose.foundation.layout.Arrangement.spacedBy(10.dp)) {
                        for (tag in item.tags) Text("#$tag", color = Color(0xFF13386C), fontSize = 14.sp, lineHeight = 26.sp)
                    }
                }
            }
        }
        Column(Modifier.fillMaxWidth().border(0.5.dp, Palette.Line).navigationBarsPadding().padding(horizontal = 12.dp, vertical = 8.dp)) {
            if (transfer is Transfer.Failed) {
                Text(transfer.message, Modifier.padding(bottom = 6.dp, start = 4.dp), color = Palette.Red, fontSize = 12.sp)
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(
                    Modifier.clip(RoundedCornerShape(8.dp)).clickable { app.toggleFavorite(item) }.padding(horizontal = 14.dp, vertical = 2.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Icon(
                        if (favorite) Icons.Filled.Favorite else Icons.Outlined.FavoriteBorder,
                        if (favorite) "取消收藏" else "收藏",
                        tint = if (favorite) Palette.Red else Palette.Sub,
                    )
                    Text("收藏", color = if (favorite) Palette.Red else Palette.Sub, fontSize = 11.sp)
                }
                Spacer(Modifier.width(8.dp))
                SaveButton(
                    transfer = transfer,
                    saved = entry,
                    onSave = { app.save(item) },
                    onCancel = { app.cancel(item.id) },
                    onView = { entry?.let { e -> openInGallery(context, e) { app.say("没有找到可以打开它的相册应用") } } },
                    modifier = Modifier.weight(1f),
                )
            }
        }
    }
}

private fun metaLine(item: SiteItem): String = buildList {
    if (item.width > 0 && item.height > 0) add("${item.width}×${item.height}")
    add(if (item.kind == MediaKind.VIDEO) "视频" else "图片")
    if (item.likes > 0) add("${likesText(item.likes)} 赞")
}.joinToString("  ·  ")

@Composable
private fun SaveButton(
    transfer: Transfer?,
    saved: Downloaded?,
    onSave: () -> Unit,
    onCancel: () -> Unit,
    onView: () -> Unit,
    modifier: Modifier,
) {
    val shape = RoundedCornerShape(22.dp)
    when {
        transfer is Transfer.Active -> {
            val progress = if (transfer.stage == Stage.DOWNLOADING) transfer.progress else 0f
            val label = when (transfer.stage) {
                Stage.REQUESTING -> "正在向网站请求…"
                Stage.DOWNLOADING -> downloadLabel(progress, transfer.totalBytes)
                Stage.SAVING -> "正在写入相册…"
            }
            Box(modifier.height(44.dp).clip(shape).background(Palette.RedSoft)) {
                Box(Modifier.fillMaxHeight().fillMaxWidth(progress.coerceIn(0f, 1f)).background(Color(0x55FF2442)))
                Row(Modifier.fillMaxSize().padding(horizontal = 18.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(label, Modifier.weight(1f), color = Palette.Red, fontSize = 15.sp, fontWeight = FontWeight.Medium)
                    Text("取消", Modifier.clickable(onClick = onCancel).padding(6.dp), color = Palette.Red, fontSize = 13.sp)
                }
            }
        }
        saved != null -> PillButton("已保存到相册 · 查看", modifier, filled = false, onClick = onView)
        transfer is Transfer.Failed -> PillButton("保存失败，点击重试", modifier, filled = false, onClick = onSave)
        else -> PillButton("保存到相册", modifier, filled = true, onClick = onSave)
    }
}

@Composable
private fun PillButton(text: String, modifier: Modifier, filled: Boolean, onClick: () -> Unit) {
    val shape = RoundedCornerShape(22.dp)
    Box(
        modifier.height(44.dp).clip(shape)
            .background(if (filled) Palette.Red else Palette.Card)
            .border(1.dp, Palette.Red, shape)
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(text, color = if (filled) Color.White else Palette.Red, fontSize = 15.sp, fontWeight = FontWeight.Medium)
    }
}

/** The big picture. Tapping it, or the play button of a video, opens the full-screen viewer. */
@Composable
private fun Media(item: SiteItem, onOpen: () -> Unit) {
    val context = LocalContext.current
    val aspect = if (item.width > 0 && item.height > 0) (item.width.toFloat() / item.height).coerceIn(MIN_DETAIL_ASPECT, MAX_DETAIL_ASPECT) else 0.75f
    val video = item.kind == MediaKind.VIDEO

    Box(
        Modifier.fillMaxWidth().aspectRatio(aspect)
            .background(if (item.color != 0) Color(item.color).copy(alpha = 0.35f) else Palette.Field)
            .clickable(onClick = onOpen),
        contentAlignment = Alignment.Center,
    ) {
        // The grid picture is already cached, so it shows at once and the sharper one fades in over it.
        FallbackImage(item.coverUrl, item.fallbackCoverUrl, null, Modifier.fillMaxSize())
        FallbackImage(item.detailUrl, item.fallbackDetailUrl, item.title, Modifier.fillMaxSize())
        if (video) {
            Box(
                Modifier.size(56.dp).clip(CircleShape).background(Color(0x66000000)),
                contentAlignment = Alignment.Center,
            ) { Icon(Icons.Filled.PlayArrow, "播放", Modifier.size(34.dp), tint = Color.White) }
        }
        Text(
            if (video) (if (item.viewable().playable) "点击播放" else "暂不支持在线播放，保存后在相册观看") else "点击全屏查看",
            Modifier.align(Alignment.BottomEnd).padding(10.dp).clip(RoundedCornerShape(12.dp)).background(Color(0x77000000)).padding(horizontal = 10.dp, vertical = 4.dp),
            color = Color.White, fontSize = 12.sp,
        )
    }
}
