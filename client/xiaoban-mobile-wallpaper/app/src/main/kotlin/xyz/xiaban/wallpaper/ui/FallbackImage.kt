package xyz.xiaban.wallpaper.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.request.crossfade
import coil3.size.Size

/**
 * A picture from the provider's own link; if that fails and the site holds a copy, the copy is shown instead.
 * Only when both fail is [onError] called.
 */
@Composable
fun FallbackImage(
    primary: String,
    fallback: String,
    contentDescription: String?,
    modifier: Modifier = Modifier,
    contentScale: ContentScale = ContentScale.Fit,
    fullResolution: Boolean = false,
    onLoading: () -> Unit = {},
    onSuccess: () -> Unit = {},
    onError: (Throwable?) -> Unit = {},
) {
    var fellBack by remember(primary, fallback) { mutableStateOf(false) }
    val usingFallback = fellBack && fallback.isNotEmpty()
    val request = ImageRequest.Builder(LocalContext.current)
        .data(if (usingFallback) fallback else primary)
        .crossfade(true)
        .apply { if (fullResolution) size(Size.ORIGINAL) }
        .build()
    AsyncImage(
        model = request,
        contentDescription = contentDescription,
        contentScale = contentScale,
        modifier = modifier,
        onLoading = { onLoading() },
        onSuccess = { onSuccess() },
        onError = {
            if (!usingFallback && fallback.isNotEmpty()) fellBack = true else onError(it.result.throwable)
        },
    )
}
