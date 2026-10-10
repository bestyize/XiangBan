package xyz.xiaban.wallpaper.ui

import android.net.Uri
import android.util.Log
import android.util.Size
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.ui.unit.Dp
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import xyz.xiaban.wallpaper.data.Downloaded
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.SiteItem
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/** Room kept at the bottom of every scrolling page so its last row can be lifted clear of the floating tab bar. */
@Composable
fun tabBarClearance(): Dp = TAB_BAR_SPACE + WindowInsets.navigationBars.asPaddingValues().calculateBottomPadding()

private val TAB_BAR_SPACE = 92.dp
private val CardShape = RoundedCornerShape(16.dp)
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
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 16.dp + tabBarClearance()),
        verticalItemSpacing = 12.dp,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        content = content,
    )
}

/** Shared card shell: the whole picture in a rounded frame, with a play badge for videos and a footer laid over its bottom edge. */
@Composable
private fun CardShell(
    aspect: Float,
    placeholder: Color,
    isVideo: Boolean,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    picture: @Composable () -> Unit,
    footerStart: @Composable RowScope.() -> Unit,
    footerEnd: @Composable RowScope.() -> Unit = {},
) {
    Box(
        Modifier
            .fillMaxWidth()
            .aspectRatio(aspect)
            .clip(CardShape)
            .background(placeholder)
            .combinedClickable(onClick = onClick, onLongClick = onLongClick),
    ) {
        picture()
        if (isVideo) {
            Box(
                Modifier.align(Alignment.Center).size(40.dp).clip(CircleShape).background(Color(0x66000000)),
                contentAlignment = Alignment.Center,
            ) { Icon(Icons.Filled.PlayArrow, null, Modifier.size(26.dp), tint = Color.White) }
        }
        Row(
            Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .background(Brush.verticalGradient(listOf(Color.Transparent, Color(0x66000000))))
                .padding(start = 10.dp, end = 10.dp, top = 18.dp, bottom = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            footerStart()
            Spacer(Modifier.weight(1f))
            footerEnd()
        }
    }
}

private const val FEED_MIN_ASPECT = 0.8f
private const val FEED_MAX_ASPECT = 1.5f

fun feedAspect(width: Int, height: Int): Float = coverAspect(width, height).coerceIn(FEED_MIN_ASPECT, FEED_MAX_ASPECT)

/** The single-column list used by the online feeds and favourites: one large picture per row. */
@Composable
fun FeedList(state: LazyListState, modifier: Modifier = Modifier, content: LazyListScope.() -> Unit) {
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        state = state,
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 20.dp + tabBarClearance()),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        content = content,
    )
}

@Composable
fun OnlineCard(item: SiteItem, favorite: Boolean, onFavorite: () -> Unit, onClick: () -> Unit) {
    val video = item.kind == MediaKind.VIDEO
    val title = item.title.ifBlank { item.tags.firstOrNull().orEmpty() }
    val meta = buildList {
        if (item.width > 0 && item.height > 0) add("${item.width}×${item.height}")
        addAll(item.tags.take(2))
    }.joinToString("   ")
    var failed by remember(item.detailUrl) { mutableStateOf(false) }

    Box(
        Modifier
            .fillMaxWidth()
            .aspectRatio(feedAspect(item.width, item.height))
            .clip(RoundedCornerShape(20.dp))
            .background(if (item.color != 0) Color(item.color).copy(alpha = 0.35f) else Palette.Field)
            .clickable(onClick = onClick),
    ) {
        FallbackImage(
            primary = item.detailUrl,
            fallback = item.fallbackDetailUrl,
            contentDescription = item.title,
            modifier = Modifier.fillMaxSize(),
            contentScale = ContentScale.Crop,
            onSuccess = { failed = false },
            onError = {
                failed = true
                Log.w("Cover", "${item.detailUrl.take(60)} failed: $it")
            },
        )
        if (failed) {
            Text("封面加载失败\n点开可看大图", Modifier.align(Alignment.Center), color = Palette.Hint, fontSize = 12.sp, textAlign = TextAlign.Center)
        }
        Box(
            Modifier.align(Alignment.BottomCenter).fillMaxWidth()
                .background(Brush.verticalGradient(listOf(Color.Transparent, Color(0x99000000))))
                .padding(start = 16.dp, end = 16.dp, top = 48.dp, bottom = 14.dp),
        ) {
            Column {
                if (title.isNotBlank()) {
                    Text(title, color = Color.White, fontSize = 17.sp, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                if (meta.isNotEmpty()) {
                    Spacer(Modifier.height(2.dp))
                    Text(meta, color = Color(0xCCFFFFFF), fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            }
        }
        if (video) {
            Box(
                Modifier.align(Alignment.Center).size(52.dp).clip(CircleShape).background(Color(0x66000000)),
                contentAlignment = Alignment.Center,
            ) { Icon(Icons.Filled.PlayArrow, null, Modifier.size(32.dp), tint = Color.White) }
        }
        Row(
            Modifier.align(Alignment.TopEnd).padding(12.dp)
                .clip(CircleShape).background(Color(0x59000000))
                .clickable(onClick = onFavorite)
                .padding(horizontal = 10.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                if (favorite) Icons.Filled.Favorite else Icons.Outlined.FavoriteBorder,
                contentDescription = if (favorite) "取消收藏" else "收藏",
                modifier = Modifier.size(18.dp),
                tint = if (favorite) Palette.Heart else Color.White,
            )
            if (item.likes > 0) {
                Spacer(Modifier.width(4.dp))
                Text(likesText(item.likes), color = Color.White, fontSize = 12.sp)
            }
        }
    }
}

@Composable
fun SavedCard(entry: Downloaded, onClick: () -> Unit, onLongClick: () -> Unit) {
    CardShell(
        aspect = coverAspect(entry.width, entry.height),
        placeholder = Palette.Field,
        isVideo = entry.kind == MediaKind.VIDEO,
        onClick = onClick,
        onLongClick = onLongClick,
        picture = { GalleryThumb(entry.uri, Modifier.fillMaxSize()) },
        footerStart = {
            Text(
                SimpleDateFormat("M月d日 保存", Locale.CHINA).format(Date(entry.savedAt)),
                color = Color.White, fontSize = 11.sp, maxLines = 1,
            )
        },
    )
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
