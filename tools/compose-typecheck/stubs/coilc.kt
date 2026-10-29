package coil3.compose
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
val LocalPlatformContext = androidx.compose.runtime.compositionLocalOf<Any?> { null }
@Composable fun AsyncImage(model: Any?, contentDescription: String?, imageLoader: coil3.ImageLoader, modifier: Modifier = Modifier, contentScale: ContentScale = ContentScale.Crop) {}
