package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.messageOf

internal val LocalCommunityDetail = staticCompositionLocalOf { false }
private val LocalDetailMessage = staticCompositionLocalOf<(String) -> Unit> { {} }

@Composable
internal fun CommunityDetail(content: @Composable () -> Unit) {
    var message by remember { mutableStateOf<String?>(null) }
    val showMessage: (String) -> Unit = remember { { message = it } }
    val dark = ThemeState.dark
    val surface = if (dark) Color(0xFF191F2C) else Color.White
    val ink = if (dark) Color(0xFFF8FAFC) else Color(0xFF111827)
    val ink2 = if (dark) Color(0xFFCBD5E1) else Color(0xFF475569)
    val line = if (dark) Color.White.copy(alpha = .075f) else Color(0xFF0F172A).copy(alpha = .055f)
    val extra = LocalExtraColors.current.copy(
        ink3 = if (dark) Color(0xFF8895AA) else Color(0xFF838E9F),
        fill = if (dark) Color(0xFF232A3B) else Color(0xFFEBEFF7),
        hairline = line,
        line = if (dark) Color.White.copy(alpha = .15f) else Color(0xFF0F172A).copy(alpha = .12f),
    )
    val scheme = MaterialTheme.colorScheme.copy(
        background = surface, onBackground = ink, surface = surface, onSurface = ink,
        surfaceVariant = extra.fill, onSurfaceVariant = ink2, outline = line, outlineVariant = line,
    )
    CompositionLocalProvider(
        LocalCommunityDetail provides true,
        LocalDetailMessage provides showMessage,
        LocalExtraColors provides extra,
        LocalContentColor provides ink,
    ) {
        MaterialTheme(colorScheme = scheme) {
            Box(Modifier.fillMaxSize().background(surface).windowInsetsPadding(WindowInsets.safeDrawing.only(WindowInsetsSides.Horizontal))) {
                content()
                Toast(message) { message = null }
            }
        }
    }
}

@Composable
internal fun DetailAuthorBar(
    author: String,
    back: () -> Unit,
    onShare: (() -> Unit)? = null,
    onAuthor: (() -> Unit)? = null,
    avatar: @Composable () -> Unit = { DetailInitial(author) },
    actions: @Composable RowScope.() -> Unit = {},
) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 48.dp).padding(horizontal = 8.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            DetailBarIcon(XIcons.ChevronLeft, "返回", back, 19)
            Row(
                Modifier.weight(1f).then(if (onAuthor != null) Modifier.clickable(role = Role.Button, onClick = onAuthor) else Modifier),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                avatar()
                Text(author, style = XType.DetailAuthor, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            actions()
            onShare?.let { DetailBarIcon(XIcons.Share, "分享", it, 18) }
        }
        Spacer(Modifier.height(1.dp))
    }
}

@Composable
internal fun DetailAuthorFollow(env: CommunityEnv, uid: String, open: ((Route) -> Unit)?) {
    val session by env.session.state.collectAsState()
    if (uid.isBlank() || uid == session.accountId || open == null) return
    val loader = rememberLoader(env, listOf("detail-author", uid, session.accountId)) { env.social.profile(uid) }
    val state by loader.state.collectAsState()
    val profile = (state as? Load.Ready)?.value
    if (profile?.blocked == true || (profile?.banned == true && !profile.following)) return
    val scope = rememberCoroutineScope()
    val showMessage = LocalDetailMessage.current
    var pending by remember(uid, session.accountId) { mutableStateOf(false) }
    val following = profile?.following == true
    val extra = LocalExtraColors.current
    val tint = if (following) extra.ink3 else extra.brandInk
    Box(
        Modifier.height(44.dp).clickable(enabled = !pending && (!session.loggedIn || profile != null), role = Role.Button) {
            if (!session.loggedIn) open(Route.Login)
            else {
                pending = true
                scope.launch {
                    try {
                        env.social.follow(uid, !following)
                        loader.reload()
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: Exception) {
                        showMessage(messageOf(e))
                    } finally {
                        pending = false
                    }
                }
            }
        },
        contentAlignment = Alignment.Center,
    ) {
        Text(
            if (following) "已关注" else "关注",
            Modifier.clip(CircleShape).background(if (following) extra.fill else Color.Transparent)
                .border(1.dp, if (following) extra.hairline else tint, CircleShape).padding(horizontal = 14.dp, vertical = 5.dp),
            style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = tint,
        )
    }
}

@Composable
private fun DetailBarIcon(icon: ImageVector, label: String, onClick: () -> Unit, size: Int) {
    Box(
        Modifier.width(34.dp).height(48.dp).clickable(role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        XIcon(icon, size = size.dp, description = label)
    }
}

@Composable
internal fun DetailInitial(name: String, size: Int = 30, background: Color = Brand) {
    Box(Modifier.size(size.dp).background(background, CircleShape), contentAlignment = Alignment.Center) {
        Text(name.trimStart('@').firstOrNull()?.uppercase() ?: "伴", style = XType.Caption, color = Color.White, fontWeight = FontWeight.SemiBold)
    }
}

@Composable
internal fun DetailGallery(images: List<String>, title: String) {
    if (images.isEmpty()) return
    val loader = rememberImageLoader()
    val pager = rememberPagerState { images.size }
    val scope = rememberCoroutineScope()
    var ratios by remember(images) { mutableStateOf(emptyMap<String, Float>()) }
    val active = pager.currentPage.coerceIn(images.indices)
    val fill = LocalExtraColors.current.fill
    Column(Modifier.fillMaxWidth()) {
        BoxWithConstraints(Modifier.fillMaxWidth()) {
            val height = (maxWidth / (ratios[images[active]] ?: (3f / 2f))).coerceAtMost(420.dp)
            HorizontalPager(
                state = pager,
                modifier = Modifier.fillMaxWidth().height(height).background(fill),
                key = { images[it] },
                beyondViewportPageCount = if (images.size > 1) 1 else 0,
                userScrollEnabled = images.size > 1,
            ) { page ->
                val src = images[page]
                val input = if (images.size > 1) Modifier.clickable(role = Role.Button, onClickLabel = "下一张图片") {
                    scope.launch { pager.animateScrollToPage((page + 1) % images.size) }
                } else Modifier
                AsyncImage(
                    model = src,
                    contentDescription = if (images.size > 1) "$title，第 ${page + 1} 张" else "$title 封面",
                    imageLoader = loader,
                    contentScale = ContentScale.Fit,
                    modifier = Modifier.fillMaxSize().then(input),
                    onSuccess = { result ->
                        val image = result.result.image
                        if (image.width > 0 && image.height > 0) {
                            val ratio = image.width.toFloat() / image.height
                            if (ratios[src] != ratio) ratios = ratios + (src to ratio)
                        }
                    },
                )
            }
        }
        if (images.size > 1) {
            Row(
                Modifier.fillMaxWidth().height(20.dp).padding(top = 6.dp),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                images.indices.forEach { index ->
                    Box(
                        Modifier.size(12.dp, 20.dp).semantics {
                            contentDescription = "第 ${index + 1} 张"
                            selected = index == active
                        }.clickable(role = Role.Button) { scope.launch { pager.animateScrollToPage(index) } },
                        contentAlignment = Alignment.Center,
                    ) {
                        Box(Modifier.size(6.dp).background(if (index == active) Brand else LocalExtraColors.current.line, CircleShape))
                    }
                }
            }
        }
    }
}

@Composable
internal fun DetailBody(
    hasDots: Boolean = false,
    horizontalPadding: Dp = 16.dp,
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(Modifier.fillMaxWidth().padding(start = horizontalPadding, end = horizontalPadding, top = if (hasDots) 8.dp else 14.dp, bottom = 12.dp), content = content)
}

@Composable
internal fun DetailHeading(title: String) {
    if (title.isNotBlank()) Text(title, Modifier.padding(bottom = 8.dp), style = XType.DetailTitle, fontWeight = FontWeight.Bold)
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun DetailTags(tags: List<String>, onFirstTag: (() -> Unit)? = null) {
    val labels = tags.filter(String::isNotBlank).distinct()
    if (labels.isEmpty()) return
    val tint = if (ThemeState.dark) Color(0xFF7FB2FF) else Color(0xFF2554A3)
    FlowRow(Modifier.fillMaxWidth().padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        labels.forEachIndexed { index, tag ->
            val action = if (index == 0 && onFirstTag != null) Modifier.clickable(role = Role.Button, onClick = onFirstTag) else Modifier
            Text("#$tag", action, style = XType.DetailAuthor, color = tint)
        }
    }
}

@Composable
internal fun DetailInfoStrip(
    label: String,
    text: String,
    icon: ImageVector? = null,
    onClick: (() -> Unit)? = null,
    expanded: Boolean = false,
) {
    val shape = RoundedCornerShape(10.dp)
    Row(
        Modifier.padding(top = 10.dp).fillMaxWidth().clip(shape).background(LocalExtraColors.current.fill)
            .then(if (onClick == null) Modifier else Modifier.clickable(role = Role.Button, onClick = onClick))
            .heightIn(min = 36.dp).padding(horizontal = 11.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        icon?.let { XIcon(it, size = 12.dp) }
        Text(label, style = XType.Secondary, fontWeight = FontWeight.Bold,
            color = if (icon == null) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurface)
        Text(text, Modifier.weight(1f), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (onClick != null) XIcon(if (expanded) XIcons.ChevronDown else XIcons.ChevronRight, size = 13.dp)
    }
}

@Composable
internal fun DetailMetadata(text: String) {
    if (text.isNotBlank()) Text(text, Modifier.padding(top = 12.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
}
