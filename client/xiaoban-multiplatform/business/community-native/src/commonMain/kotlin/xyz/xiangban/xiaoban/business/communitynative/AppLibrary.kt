package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import coil3.ImageLoader
import coil3.compose.LocalPlatformContext
import coil3.network.ktor3.KtorNetworkFetcherFactory
import coil3.svg.SvgDecoder
import kotlinx.coroutines.CancellationException
import xiaoban.business.community_native.generated.resources.Res
import xyz.xiangban.xiaoban.common.community.*

private const val WancisouPackId = "58"

/** Only official/local favorites live here; public works use the existing account bookmark API. */
internal class AppLibraryState(private val prefs: AppPrefs) {
    var favorites by mutableStateOf(AppFavorites.decode(prefs.get("app-favorites")))
        private set
    fun toggle(item: AppShortcut) {
        val next = AppFavorites.toggle(favorites, item)
        prefs.put("app-favorites", AppFavorites.encode(next))
        favorites = next
    }
}

@Composable
internal fun AppActionButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier, primary: Boolean = false,
    enabled: Boolean = true, height: Dp = 48.dp, icon: ImageVector? = null) {
    val extra = LocalExtraColors.current
    val tint = if (primary) extra.brandInk else MaterialTheme.colorScheme.onSurface
    Row(modifier.height(height).clip(CircleShape).background(if (primary) extra.brandSoft else extra.fill)
        .clickable(enabled = enabled, onClick = onClick).padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally)) {
        icon?.let { XIcon(it, tint = tint.copy(alpha = if (enabled) 1f else .4f), size = 20.dp) }
        Text(text, style = if (height <= 34.dp) XType.Secondary else XType.Body, fontWeight = FontWeight.SemiBold, color = tint.copy(alpha = if (enabled) 1f else .4f), maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
internal fun AppShortcutIcon(env: CommunityEnv, item: AppShortcut, modifier: Modifier = Modifier.size(56.dp)) {
    val glyph = if (item.source == "builtin") BuiltinMobileGames.glyph(item.id) else null
    AppArtworkIcon(env, item, modifier, glyph)
}

@Composable
private fun AppGlyphIcon(glyph: String, modifier: Modifier) {
    val tint = if (ThemeState.dark) Color(0xFFE0A274) else Color(0xFFB06A34)
    val shape = RoundedCornerShape(16.dp)
    BoxWithConstraints(modifier.clip(shape).background(tint.copy(alpha = if (ThemeState.dark) .18f else .12f)).border(0.5.dp, tint.copy(alpha = .3f), shape), contentAlignment = Alignment.Center) {
        Text(glyph, fontFamily = FontFamily.Serif, fontWeight = FontWeight.Bold, fontSize = (maxWidth.value * .46f).sp, color = tint)
    }
}

@Composable
private fun AppArtworkIcon(env: CommunityEnv, item: AppShortcut, modifier: Modifier, glyph: String?) {
    val revision = if (item.source == "local") env.games?.revision?.collectAsState()?.value else null
    var bytes by remember(item.key, item.slug, revision) { mutableStateOf<ByteArray?>(null) }
    LaunchedEffect(env, item.key, item.slug, revision) {
        try {
            bytes = when (item.source) {
                "builtin" -> env.platform?.builtinGameIcon(item.id)
                "local" -> env.games?.let { store -> store.readFileBytes(item.id, store.read(item.id).icon) }
                "community" -> {
                    // Recent and bookmarked works can arrive with a pack ID and no slug.
                    val isWancisou = item.slug == "wancisou" || (
                        item.slug.isNullOrEmpty() && item.id == WancisouPackId &&
                            env.config.forumBase.trimEnd('/') == CommunityConfig().forumBase
                        )
                    if (isWancisou) Res.readBytes("files/wancisou-icon.webp") else null
                }
                else -> null
            }
        } catch (e: CancellationException) { throw e } catch (_: Exception) { /* neutral game icon */ }
    }
    Box(modifier.clip(RoundedCornerShape(16.dp)).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
        if (glyph != null) AppGlyphIcon(glyph, Modifier.fillMaxSize())
        else XIcon(XIcons.Gamepad, size = 24.dp, tint = LocalExtraColors.current.brandInk)
        val model: Any? = bytes ?: item.coverUrl?.let(env.forum::imageUrl)
        if (model != null) AsyncImage(model, null, imageLoader = rememberAppImageLoader(), contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
    }
}

@Composable
internal fun rememberAppImageLoader(): ImageLoader {
    val context = LocalPlatformContext.current
    val loader = remember(context) { ImageLoader.Builder(context).components { add(SvgDecoder.Factory()); add(KtorNetworkFetcherFactory()) }.build() }
    DisposableEffect(loader) { onDispose { loader.shutdown() } }
    return loader
}

@Composable
internal fun AppShortcutTile(env: CommunityEnv, item: AppShortcut, modifier: Modifier = Modifier,
    onDetails: (() -> Unit)? = null, subtitle: String? = null, onClick: () -> Unit) {
    Column(modifier.clip(RoundedCornerShape(8.dp))
        .combinedClickable(onLongClickLabel = "查看作品详情", onLongClick = onDetails, onClick = onClick)
        .padding(vertical = 4.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        AppShortcutIcon(env, item)
        Text(item.title, Modifier.padding(top = 8.dp), style = XType.Caption, maxLines = 1, overflow = TextOverflow.Ellipsis)
        subtitle?.let { Text(it, Modifier.padding(top = 2.dp), style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 1) }
    }
}

@Composable
internal fun CommunityShortcutTile(env: CommunityEnv, item: AppShortcut, modifier: Modifier, showGameDetails: Boolean = false, onDetails: () -> Unit) {
    CommunityWorkLaunch(env, item.id.toLong(), openGameDetails = onDetails.takeIf { showGameDetails }) { loading, _, error, launch ->
        Column(modifier, horizontalAlignment = Alignment.CenterHorizontally) {
            AppShortcutTile(env, item, Modifier.fillMaxWidth(), onDetails = onDetails, onClick = launch)
            if (loading) Text("正在打开…", style = XType.Caption, color = LocalExtraColors.current.ink3)
            error?.let { Text(it, style = XType.Caption, color = Danger) }
        }
    }
}
