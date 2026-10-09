package xyz.xiaban.wallpaper.ui

import android.net.Uri
import android.util.Log
import android.util.Size
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridScope
import androidx.compose.foundation.lazy.staggeredgrid.LazyStaggeredGridState
import androidx.compose.foundation.lazy.staggeredgrid.LazyVerticalStaggeredGrid
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridCells
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.outlined.FavoriteBorder
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.crossfade
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.SiteItem
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

private val CardShape = RoundedCornerShape(8.dp)
private const val THUMB_BOX = 640

/** The two-column waterfall used by every tab. Headers and footers span both columns. */
@Composable
fun Waterfall(
    state: LazyStaggeredGridState,
    modifier: Modifier = Modifier,
    content: LazyStaggeredGridScope.() -> Unit,
) {
    LazyVerticalStaggeredGrid(
        columns = StaggeredGridCells.Fixed(2),
        modifier = modifier.fillMaxSize(),
        state = state,
        contentPadding = PaddingValues(6.dp),
        verticalItemSpacing = 6.dp,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        content = content,
    )
}

/** Shared card shell: a picture of the right shape, up to two lines of title, then a footer row. */
@Composable
private fun CardShell(
    aspect: Float,
    placeholder: Color,
    isVideo: Boolean,
    title: String,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    picture: @Composable () -> Unit,
    footer: @Composable RowScope.() -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .clip(CardShape)
            .background(Palette.Card)
            .combinedClickable(onClick = onClick, onLongClick = onLongClick),
    ) {
        Box(Modifier.fillMaxWidth().aspectRatio(aspect).background(placeholder)) {
            picture()
            if (isVideo) {
                Box(
                    Modifier.align(Alignment.TopEnd).padding(6.dp).size(24.dp).clip(CircleShape).background(Color(0x66000000)),
                    contentAlignment = Alignment.Center,
                ) { Icon(Icons.Filled.PlayArrow, null, Modifier.size(16.dp), tint = Color.White) }
            }
        }
        if (title.isNotBlank()) {
            Text(
                title,
                Modifier.padding(start = 8.dp, end = 8.dp, top = 8.dp),
                color = Palette.Text, fontSize = 14.sp, lineHeight = 19.sp, maxLines = 2, overflow = TextOverflow.Ellipsis,
            )
        }
        Row(Modifier.fillMaxWidth().padding(8.dp), verticalAlignment = Alignment.CenterVertically, content = footer)
    }
}

@Composable
fun OnlineCard(item: SiteItem, favorite: Boolean, onFavorite: () -> Unit, onClick: () -> Unit) {
    val context = LocalContext.current
    CardShell(
        aspect = coverAspect(item.width, item.height),
        placeholder = if (item.color != 0) Color(item.color).copy(alpha = 0.35f) else Palette.Field,
        isVideo = item.kind == MediaKind.VIDEO,
        title = item.title,
        onClick = onClick,
        picture = {
            var failed by remember(item.coverUrl) { mutableStateOf(false) }
            Box(Modifier.fillMaxSize()) {
                FallbackImage(
                    primary = item.coverUrl,
                    fallback = item.fallbackCoverUrl,
                    contentDescription = item.title,
                    modifier = Modifier.fillMaxSize(),
                    onSuccess = { failed = false },
                    onError = {
                        failed = true
                        Log.w("Cover", "${item.coverUrl.take(60)} failed: $it")
                    },
                )
                if (failed) {
                    Text("封面加载失败\n点开可看大图", Modifier.align(Alignment.Center), color = Palette.Hint, fontSize = 12.sp, textAlign = TextAlign.Center)
                }
            }
        },
        footer = {
            Avatar(item.author, item.avatarUrl)
            Text(
                item.author.ifBlank { "小伴壁纸" },
                Modifier.weight(1f).padding(horizontal = 6.dp),
                color = Palette.Sub, fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis,
            )
            Row(Modifier.clip(CircleShape).clickable(onClick = onFavorite).padding(2.dp), verticalAlignment = Alignment.CenterVertically) {
                Icon(
                    if (favorite) Icons.Filled.Favorite else Icons.Outlined.FavoriteBorder,
                    contentDescription = if (favorite) "取消收藏" else "收藏",
                    modifier = Modifier.size(16.dp),
                    tint = if (favorite) Palette.Red else Palette.Sub,
                )
                Spacer(Modifier.width(3.dp))
                Text(likesText(item.likes), color = Palette.Sub, fontSize = 12.sp)
            }
        },
    )
}

@Composable
fun SavedCard(entry: Downloaded, onClick: () -> Unit, onLongClick: () -> Unit) {
    CardShell(
        aspect = coverAspect(entry.width, entry.height),
        placeholder = Palette.Field,
        isVideo = entry.kind == MediaKind.VIDEO,
        title = entry.title,
        onClick = onClick,
        onLongClick = onLongClick,
        picture = { GalleryThumb(entry.uri, Modifier.fillMaxSize()) },
        footer = {
            Text(
                SimpleDateFormat("M月d日 保存", Locale.CHINA).format(Date(entry.savedAt)),
                color = Palette.Sub, fontSize = 12.sp, maxLines = 1,
            )
        },
    )
}

@Composable
private fun Avatar(name: String, url: String) {
    val context = LocalContext.current
    Box(Modifier.size(20.dp).clip(CircleShape).background(Palette.Field), contentAlignment = Alignment.Center) {
        Text(name.take(1).uppercase(), color = Palette.Sub, fontSize = 10.sp)
        if (url.isNotEmpty()) {
            AsyncImage(
                model = ImageRequest.Builder(context).data(url).crossfade(true).build(),
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

/** A thumbnail of a gallery entry, made by the system so videos work as well as pictures. It keeps the picture's own shape. */
@Composable
fun GalleryThumb(uri: String, modifier: Modifier = Modifier) {
    val resolver = LocalContext.current.contentResolver
    val bitmap by produceState<ImageBitmap?>(null, uri) {
        value = withContext(Dispatchers.IO) {
            runCatching { resolver.loadThumbnail(Uri.parse(uri), Size(THUMB_BOX, THUMB_BOX), null).asImageBitmap() }.getOrNull()
        }
    }
    bitmap?.let { Image(it, contentDescription = null, contentScale = ContentScale.Fit, modifier = modifier) }
}
