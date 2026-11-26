package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.ImageLoader
import coil3.compose.AsyncImage
import coil3.compose.LocalPlatformContext
import coil3.network.ktor3.KtorNetworkFetcherFactory
import androidx.compose.runtime.remember
import xyz.xiangban.xiaoban.common.community.DesignTokens as Tokens
import xyz.xiangban.xiaoban.common.community.Load

private val CardShape = RoundedCornerShape(Tokens.RadiusCard.dp)

@Composable
fun Hairline(modifier: Modifier = Modifier) = xyz.xiangban.xiaoban.common.ui.controls.Hairline(modifier)

@Composable
fun CenterMessage(text: String, action: String? = null, onAction: (() -> Unit)? = null, modifier: Modifier = Modifier.fillMaxSize()) =
    xyz.xiangban.xiaoban.common.ui.controls.CenterMessage(text, action, onAction, modifier)

@Composable
fun Spinner(modifier: Modifier = Modifier.fillMaxSize()) = xyz.xiangban.xiaoban.common.ui.controls.Spinner(modifier)

/** 详情页的统一加载外壳：加载中、失败（可重试）、成功三态。 */
@Composable
fun <T> LoadBox(state: Load<T>, onRetry: () -> Unit, content: @Composable (T) -> Unit) {
    when (state) {
        Load.Loading -> Spinner()
        is Load.Failed -> CenterMessage(if (state.notFound) "内容不存在或已下架" else state.message, if (state.notFound) null else "重试", onRetry)
        is Load.Ready -> content(state.value)
    }
}

internal val LocalCommunityImageLoader = staticCompositionLocalOf<ImageLoader?> { null }

@Composable
fun rememberImageLoader(): ImageLoader {
    LocalCommunityImageLoader.current?.let { return it }
    val context = LocalPlatformContext.current
    val loader = remember(context) { ImageLoader.Builder(context).components { add(KtorNetworkFetcherFactory()) }.build() }
    DisposableEffect(loader) { onDispose(loader::shutdown) }
    return loader
}

/** 封面：有图就显示图，没有（或没加载出来）时垫一块按编号轮换的渐变，保证列表整齐。 */
@Composable
fun Cover(url: String?, id: Long, modifier: Modifier = Modifier, contentScale: ContentScale = ContentScale.Crop) {
    val (from, to) = Tokens.coverGradient(id)
    Box(modifier.clip(CardShape).background(Brush.linearGradient(listOf(Color(from), Color(to))))) {
        if (!url.isNullOrBlank()) {
            AsyncImage(model = url, contentDescription = null, imageLoader = rememberImageLoader(), contentScale = contentScale, modifier = Modifier.fillMaxSize())
        }
    }
}
