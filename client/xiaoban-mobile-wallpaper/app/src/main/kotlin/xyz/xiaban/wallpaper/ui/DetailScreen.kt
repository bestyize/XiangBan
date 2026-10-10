package xyz.xiaban.wallpaper.ui

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
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
import androidx.compose.ui.layout.layout
import androidx.compose.ui.text.style.TextAlign
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

private const val MIN_DETAIL_ASPECT = 0.6f
private const val MAX_DETAIL_ASPECT = 1.1f

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

    Box(Modifier.fillMaxSize().background(Palette.Card)) {
    Column(Modifier.fillMaxSize()) {
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState())) {
            Media(item, onOpen = { app.view(item.viewable()) })
            val overlap = 24.dp
            Column(
                Modifier
                    .layout { measurable, constraints ->
                        val placeable = measurable.measure(constraints)
                        val pull = overlap.roundToPx()
                        layout(placeable.width, placeable.height - pull) { placeable.place(0, -pull) }
                    }
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(topStart = overlap, topEnd = overlap))
                    .background(Palette.Card)
                    .padding(horizontal = 16.dp, vertical = 20.dp),
            ) {
                Row(verticalAlignment = Alignment.Top) {
                    Text(
                        item.title, Modifier.weight(1f),
                        color = Palette.Text, fontSize = 18.sp, fontWeight = FontWeight.Bold, lineHeight = 24.sp,
                    )
                    if (item.likes > 0) {
                        Spacer(Modifier.width(12.dp))
                        Row(Modifier.padding(top = 3.dp), verticalAlignment = Alignment.CenterVertically) {
                            Icon(Icons.Filled.Favorite, null, Modifier.size(14.dp), tint = Palette.Heart)
                            Spacer(Modifier.width(4.dp))
                            Text(likesText(item.likes), color = Palette.Sub, fontSize = 12.sp)
                        }
                    }
                }
                if (item.tags.isNotEmpty()) {
                    Spacer(Modifier.height(12.dp))
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        for (tag in item.tags) {
                            Text("#$tag", color = Palette.Tag, fontSize = 14.sp)
                        }
                    }
                }
                Spacer(Modifier.height(16.dp))
                InfoCard(item)
            }
        }
        Column(Modifier.fillMaxWidth().border(0.5.dp, Palette.Line).navigationBarsPadding().padding(horizontal = 16.dp, vertical = 10.dp)) {
            if (transfer is Transfer.Failed) {
                Text(transfer.message, Modifier.padding(bottom = 6.dp, start = 4.dp), color = Palette.Brand, fontSize = 12.sp)
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    Modifier.size(48.dp).clip(CircleShape)
                        .background(if (favorite) Palette.HeartSoft else Palette.Field)
                        .clickable { app.toggleFavorite(item) },
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        if (favorite) Icons.Filled.Favorite else Icons.Outlined.FavoriteBorder,
                        if (favorite) "取消收藏" else "收藏",
                        tint = if (favorite) Palette.Heart else Palette.Sub,
                    )
                }
                Spacer(Modifier.width(12.dp))
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
    Box(
        Modifier.statusBarsPadding().padding(12.dp).size(40.dp).clip(CircleShape).background(Color(0x66000000)).clickable { app.close() },
        contentAlignment = Alignment.Center,
    ) { Icon(Icons.AutoMirrored.Filled.ArrowBack, "返回", Modifier.size(22.dp), tint = Color.White) }
    }
}

@Composable
private fun InfoCard(item: SiteItem) {
    val rows = buildList {
        add("类型" to if (item.kind == MediaKind.VIDEO) "视频" else "图片")
        if (item.width > 0 && item.height > 0) add("尺寸" to "${item.width} × ${item.height}")
        add("来源" to item.author.ifBlank { "小伴壁纸" })
    }
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(Palette.Field).padding(horizontal = 16.dp, vertical = 6.dp)) {
        rows.forEachIndexed { index, (label, value) ->
            if (index > 0) Box(Modifier.fillMaxWidth().height(0.5.dp).background(Palette.Hint.copy(alpha = 0.3f)))
            Row(Modifier.fillMaxWidth().padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(label, color = Palette.Sub, fontSize = 13.sp)
                Text(value, Modifier.weight(1f), color = Palette.Text, fontSize = 13.sp, textAlign = TextAlign.End, maxLines = 1)
            }
        }
    }
}

@Composable
private fun SaveButton(
    transfer: Transfer?,
    saved: Downloaded?,
    onSave: () -> Unit,
    onCancel: () -> Unit,
    onView: () -> Unit,
    modifier: Modifier,
) {
    val shape = RoundedCornerShape(24.dp)
    when {
        transfer is Transfer.Active -> {
            val progress = if (transfer.stage == Stage.DOWNLOADING) transfer.progress else 0f
            val label = when (transfer.stage) {
                Stage.REQUESTING -> "正在向网站请求…"
                Stage.DOWNLOADING -> downloadLabel(progress, transfer.totalBytes)
                Stage.SAVING -> "正在写入相册…"
            }
            Box(modifier.height(48.dp).clip(shape).background(Palette.BrandSoft)) {
                Box(Modifier.fillMaxHeight().fillMaxWidth(progress.coerceIn(0f, 1f)).background(Palette.Brand.copy(alpha = 0.22f)))
                Row(Modifier.fillMaxSize().padding(horizontal = 18.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(label, Modifier.weight(1f), color = Palette.Brand, fontSize = 15.sp, fontWeight = FontWeight.Medium)
                    Text("取消", Modifier.clickable(onClick = onCancel).padding(6.dp), color = Palette.Brand, fontSize = 13.sp)
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
    val shape = RoundedCornerShape(24.dp)
    Box(
        modifier.height(48.dp).clip(shape)
            .then(if (filled) Modifier.background(Palette.Brand) else Modifier.background(Palette.Card).border(1.dp, Palette.Brand, shape))
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(text, color = if (filled) Color.White else Palette.Brand, fontSize = 15.sp, fontWeight = FontWeight.Medium)
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
        FallbackImage(item.coverUrl, item.fallbackCoverUrl, null, Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
        FallbackImage(item.detailUrl, item.fallbackDetailUrl, item.title, Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
        if (video) {
            Box(
                Modifier.size(56.dp).clip(CircleShape).background(Color(0x66000000)),
                contentAlignment = Alignment.Center,
            ) { Icon(Icons.Filled.PlayArrow, "播放", Modifier.size(34.dp), tint = Color.White) }
        }
        Text(
            if (video) (if (item.viewable().playable) "点击播放" else "暂不支持在线播放，保存后在相册观看") else "点击全屏查看",
            Modifier.align(Alignment.BottomEnd).padding(start = 10.dp, end = 10.dp, bottom = 34.dp).clip(RoundedCornerShape(12.dp)).background(Color(0x77000000)).padding(horizontal = 10.dp, vertical = 4.dp),
            color = Color.White, fontSize = 12.sp,
        )
    }
}
