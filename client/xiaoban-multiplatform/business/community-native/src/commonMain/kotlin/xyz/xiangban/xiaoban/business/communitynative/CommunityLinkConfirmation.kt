package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.RoundRect
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.dialog
import androidx.compose.ui.semantics.hideFromAccessibility
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import coil3.ImageLoader
import coil3.compose.AsyncImagePainter
import coil3.compose.LocalPlatformContext
import coil3.compose.rememberAsyncImagePainter
import coil3.compose.rememberConstraintsSizeResolver
import coil3.network.ktor3.KtorNetworkFetcherFactory
import coil3.request.ImageRequest
import xyz.xiangban.xiaoban.common.community.ApiException
import xyz.xiangban.xiaoban.common.community.CapabilityPackKinds
import xyz.xiangban.xiaoban.common.community.CommunityShare
import xyz.xiangban.xiaoban.common.community.CommunitySharingApi
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.communityShareTarget

@OptIn(ExperimentalComposeUiApi::class)
@Composable
internal fun CommunityLinkConfirmation(env: CommunityEnv, open: (Route) -> Unit, modifier: Modifier, content: @Composable BoxScope.() -> Unit) {
    val platform = env.platform
    val incoming by (platform?.communityLinks ?: EmptyCommunityLinks).collectAsState()
    val pending = incoming
    Box(Modifier.fillMaxSize()) {
        val backdrop = if (pending == null) Modifier else Modifier.blur(2.dp).clearAndSetSemantics { hideFromAccessibility() }
        Box(backdrop.then(modifier).fillMaxSize(), content = content)
        if (platform != null && pending != null) {
            val target = remember(pending, env.config) { communityShareTarget(pending, env.config) }
            val api = remember(env) { CommunitySharingApi(env.client, env.config) }
            val loader = rememberLoader(env, pending) {
                api.publicInfo(target ?: throw ApiException(400, "此链接不是可打开的公开社区内容"), includeCover = true)
            }
            val state by loader.state.collectAsState()
            val close: () -> Unit = { if (platform.communityLinks.value == pending) platform.consumeCommunityLink() }
            LinkPreviewDialog(env, state, target != null, target?.requestsApply == true, close, loader::reload) { info ->
                if (target != null && platform.communityLinks.value == pending) {
                    platform.consumeCommunityLink()
                    open(when (target.kind) {
                        "pack" -> if (info.packKind in CapabilityPackKinds) Route.Ability(target.id.toLong()) else Route.Pack(target.id.toLong())
                        "topic" -> Route.Topic(target.id.toLong())
                        "scheme" -> Route.Scheme(target.id.toLong())
                        "project" -> Route.Project(target.id)
                        else -> Route.Weekly(target.id)
                    })
                }
            }
        }
    }
}

// Values from the centered card in docs/design-specs/prototypes/share-link-landing.html.
private data class LinkColors(
    val surface: Color, val fill: Color, val ink: Color, val muted: Color, val faint: Color,
    val line: Color, val scrim: Color, val amber: Color, val amberFill: Color,
)

private val LightLink = LinkColors(
    Color.White, Color(0xFFF2F2F4), Color(0xFF1B1C1F), Color(0xFF5D6068), Color(0xFF94979E),
    Color.Black.copy(alpha = .07f), Color(0xFF0C0C0E).copy(alpha = .42f), Color(0xFFA86A00), Color(0xFFFFF5E0),
)
private val DarkLink = LinkColors(
    Color(0xFF1C1C1F), Color(0xFF26262A), Color(0xFFF1F1F3), Color(0xFFA6A8B0), Color(0xFF6F7178),
    Color.White.copy(alpha = .08f), Color.Black.copy(alpha = .6f), Color(0xFFF0B44C), Color(0xFFF0B44C).copy(alpha = .12f),
)

@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun LinkPreviewDialog(
    env: CommunityEnv,
    state: Load<CommunityShare>,
    validLink: Boolean,
    requestsApply: Boolean,
    onDismiss: () -> Unit,
    onRetry: () -> Unit,
    onOpen: (CommunityShare) -> Unit,
) {
    val colors = if (ThemeState.dark) DarkLink else LightLink
    val info = (state as? Load.Ready)?.value
    val loading = state is Load.Loading
    var coverFailed by remember(info?.coverUrl) { mutableStateOf(false) }
    val cover = info?.coverUrl?.takeUnless { coverFailed }
    val hasTopImage = loading || cover != null
    val enter = remember { Animatable(0f) }
    val focus = remember { FocusRequester() }
    val pendingDismiss by rememberUpdatedState(onDismiss)

    BackHandler(onBack = onDismiss)
    LaunchedEffect(Unit) {
        focus.requestFocus()
        enter.animateTo(1f, tween(260, easing = CubicBezierEasing(.2f, .9f, .3f, 1.2f)))
    }
    Box(Modifier.fillMaxSize()) {
        Box(Modifier.matchParentSize().background(colors.scrim).pointerInput(Unit) { detectTapGestures { pendingDismiss() } })
        Box(Modifier.fillMaxSize().safeDrawingPadding().padding(horizontal = 24.dp, vertical = 14.dp), contentAlignment = Alignment.Center) {
            Box(
                Modifier.widthIn(max = 342.dp).fillMaxWidth()
                    .graphicsLayer { alpha = enter.value.coerceIn(0f, 1f); scaleX = .94f + .06f * enter.value; scaleY = scaleX }
                    .dialogCardSurface(colors.surface).pointerInput(Unit) { detectTapGestures {} }
                    .semantics { dialog() }.focusRequester(focus).focusable(),
            ) {
                Column(Modifier.fillMaxWidth().verticalScroll(rememberScrollState())) {
                    when {
                        loading -> LinkCoverSkeleton(colors)
                        info != null && cover != null -> LinkCover(env, info, cover, requestsApply, colors) { coverFailed = true }
                    }
                    Column(Modifier.fillMaxWidth().padding(start = 18.dp, end = 18.dp, top = if (hasTopImage) 16.dp else 22.dp, bottom = 18.dp)) {
                        when (state) {
                            Load.Loading -> {
                                LinkTextSkeleton(colors)
                                LinkReadyActions(onDismiss, onOpen = {}, colors = colors, loading = true)
                            }
                            is Load.Ready -> {
                                if (cover != null) LinkTitle(state.value, colors, compact = false)
                                else Row(horizontalArrangement = Arrangement.spacedBy(14.dp)) {
                                    Box(Modifier.size(56.dp).clip(RoundedCornerShape(14.dp)).background(Color(0xFF24292F)), contentAlignment = Alignment.Center) {
                                        XIcon(linkKindIcon(state.value), tint = Color.White, size = 26.dp)
                                    }
                                    Column(Modifier.weight(1f)) {
                                        LinkKindChip(state.value, requestsApply, colors, false)
                                        Spacer(Modifier.height(6.dp))
                                        LinkTitle(state.value, colors, compact = true)
                                    }
                                }
                                if (requestsApply) {
                                    Row(
                                        Modifier.padding(top = 12.dp).fillMaxWidth().clip(RoundedCornerShape(12.dp))
                                            .background(colors.amberFill).padding(horizontal = 12.dp, vertical = 10.dp),
                                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                                    ) {
                                        XIcon(LinkInfo, Modifier.padding(top = 1.5.dp), tint = colors.amber, size = 16.dp)
                                        Text("这是电脑端内容，手机只能查看详情，不会自动下载或应用。", color = colors.amber,
                                            style = XType.Secondary.copy(fontSize = 12.5.sp, lineHeight = 19.375.sp))
                                    }
                                }
                                LinkReadyActions(onDismiss, { onOpen(state.value) }, colors)
                            }
                            is Load.Failed -> LinkFailure(state, validLink, colors, onDismiss, onRetry)
                        }
                    }
                }
                DialogCardClose(onDismiss, enabled = true, fill = if (hasTopImage) Color.Black.copy(alpha = .35f) else colors.fill,
                    ink = if (hasTopImage) Color.White else colors.faint)
            }
        }
    }
}

@Composable
private fun LinkCover(env: CommunityEnv, info: CommunityShare, url: String, requestsApply: Boolean, colors: LinkColors, onError: () -> Unit) {
    val context = LocalPlatformContext.current
    val client = remember(env.client, context) { env.client.config { followRedirects = false } }
    val loader = remember(context, client) {
        ImageLoader.Builder(context).components { add(KtorNetworkFetcherFactory(httpClient = { client })) }.build()
    }
    DisposableEffect(loader) { onDispose { loader.shutdown(); client.close() } }
    val imageSize = rememberConstraintsSizeResolver()
    val request = remember(context, url, imageSize) { ImageRequest.Builder(context).data(url).size(imageSize).build() }
    val cover = rememberAsyncImagePainter(request, imageLoader = loader, contentScale = ContentScale.Crop, onState = {
        if (it is AsyncImagePainter.State.Error) onError()
    })
    var chipSize by remember { mutableStateOf(IntSize.Zero) }
    Box(Modifier.fillMaxWidth().height(168.dp).background(colors.fill)) {
        Image(cover, null, Modifier.matchParentSize().then(imageSize), contentScale = ContentScale.Crop)
        // The draft blurs the cover only where the label and close control sit above it.
        Image(cover, null, Modifier.matchParentSize().drawWithCache {
            val inset = 10.dp.toPx()
            val close = 32.dp.toPx()
            val mask = Path().apply {
                addRoundRect(RoundRect(
                    Rect(inset, inset, inset + chipSize.width, inset + chipSize.height), CornerRadius(chipSize.height / 2f),
                ))
                addRoundRect(RoundRect(
                    Rect(size.width - inset - close, inset, size.width - inset, inset + close), CornerRadius(close / 2f),
                ))
            }
            onDrawWithContent { clipPath(mask) { this@onDrawWithContent.drawContent() } }
        }.blur(8.dp), contentScale = ContentScale.Crop)
        LinkKindChip(info, requestsApply, colors, true, Modifier.padding(10.dp).onSizeChanged { chipSize = it })
    }
}

@Composable
private fun LinkKindChip(info: CommunityShare, requestsApply: Boolean, colors: LinkColors, onCover: Boolean, modifier: Modifier = Modifier) {
    val label = when (info.kind) {
        "topic" -> "帖子"
        "pack" -> if (requestsApply) "桌面装扮" else "作品"
        "scheme" -> "方案"
        "project" -> "开源精选"
        "weekly" -> "社区周刊"
        else -> "社区内容"
    }
    val textColor = if (onCover) Color.White else colors.muted
    Row(
        modifier.height(22.dp).clip(CircleShape).background(if (onCover) Color.Black.copy(alpha = .5f) else colors.surface)
            .then(if (onCover) Modifier else Modifier.border(1.dp, colors.line, CircleShape)).padding(horizontal = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically,
    ) {
        XIcon(linkKindIcon(info), tint = textColor, size = 12.dp)
        Text(label, color = textColor, style = XType.Caption.copy(fontSize = 11.sp), fontWeight = FontWeight.SemiBold)
    }
}

@Composable
private fun LinkTitle(info: CommunityShare, colors: LinkColors, compact: Boolean) {
    Text(info.title, color = colors.ink, style = XType.Section.copy(
        fontSize = if (compact) 15.5.sp else 17.sp, lineHeight = if (compact) 21.7.sp else 23.8.sp, letterSpacing = (-.01f).em,
    ), fontWeight = FontWeight.Bold, maxLines = 2, overflow = TextOverflow.Ellipsis)
    if (info.summary.isNotBlank()) {
        Text(info.summary, Modifier.padding(top = if (compact) 4.dp else 6.dp), color = colors.muted,
            style = XType.Secondary.copy(fontSize = if (compact) 12.5.sp else 13.sp, lineHeight = if (compact) 20.sp else 20.8.sp),
            maxLines = 2, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun LinkReadyActions(onDismiss: () -> Unit, onOpen: () -> Unit, colors: LinkColors, loading: Boolean = false) {
    Column(Modifier.padding(top = 16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        LinkButton(if (loading) "正在读取" else "查看详情", onOpen, Modifier.fillMaxWidth(), primary = true, loading = loading, arrow = !loading)
        Box(Modifier.fillMaxWidth().height(42.dp).clip(CircleShape)
            .clickable(interactionSource = null, indication = null, role = Role.Button, onClick = onDismiss), contentAlignment = Alignment.Center) {
            Text("暂不查看", color = colors.muted, style = XType.Secondary.copy(fontSize = 14.sp, lineHeight = 21.sp))
        }
    }
}

@Composable
private fun LinkFailure(state: Load.Failed, validLink: Boolean, colors: LinkColors, onDismiss: () -> Unit, onRetry: () -> Unit) {
    val unavailable = state.notFound || !validLink
    Column(Modifier.fillMaxWidth().padding(start = 8.dp, end = 8.dp, top = 14.dp, bottom = 4.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.size(64.dp).clip(RoundedCornerShape(20.dp)).background(colors.fill), contentAlignment = Alignment.Center) {
            XIcon(if (unavailable) LinkGhost else LinkWifi, tint = colors.faint, size = 28.dp)
        }
        Text(if (state.notFound) "内容已下架或未公开" else "暂时打不开这个链接", Modifier.padding(top = 14.dp), color = colors.ink,
            style = XType.Section.copy(fontSize = 17.sp, lineHeight = 25.5.sp), fontWeight = FontWeight.Bold, textAlign = TextAlign.Center)
        Text(when {
            state.notFound -> "分享者可能已删除或设为私密"
            !validLink -> state.message
            else -> "网络不太稳定，检查后再试一次"
        }, Modifier.padding(top = 6.dp), color = colors.muted, style = XType.Secondary.copy(lineHeight = 20.8.sp), textAlign = TextAlign.Center)
    }
    Row(Modifier.fillMaxWidth().padding(top = 16.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        if (unavailable) LinkButton("知道了", onDismiss, Modifier.fillMaxWidth(), colors.fill, colors.ink)
        else {
            LinkButton("取消", onDismiss, Modifier.weight(1f), colors.fill, colors.ink)
            LinkButton("重试", onRetry, Modifier.weight(1f), primary = true)
        }
    }
}

@Composable
private fun LinkButton(
    label: String, onClick: () -> Unit, modifier: Modifier, background: Color = Brand, color: Color = Color.White,
    primary: Boolean = false, loading: Boolean = false, arrow: Boolean = false,
) {
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val scale by animateFloatAsState(if (primary && pressed) .98f else 1f, tween(120), label = "分享链接按钮")
    Row(
        modifier.height(48.dp).graphicsLayer { alpha = if (loading) .45f else 1f; scaleX = scale; scaleY = scale }
            .clip(CircleShape).background(background)
            .clickable(interaction, indication = null, enabled = !loading, role = Role.Button, onClick = onClick),
        horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically,
    ) {
        if (loading) CircularProgressIndicator(Modifier.size(16.dp), color = Color.White, trackColor = Color.White.copy(alpha = .5f), strokeWidth = 2.dp)
        Text(label, color = color, style = XType.Body.copy(fontSize = if (primary) 15.5.sp else 15.sp),
            fontWeight = if (primary) FontWeight(650) else FontWeight.SemiBold)
        if (arrow) XIcon(LinkArrow, tint = color, size = 17.dp)
    }
}

@Composable
private fun LinkCoverSkeleton(colors: LinkColors) {
    LinkSkeleton(colors, Modifier.fillMaxWidth().height(168.dp), corner = 0)
}

@Composable
private fun LinkTextSkeleton(colors: LinkColors) {
    LinkSkeleton(colors, Modifier.size(width = 60.dp, height = 20.dp), corner = 10)
    LinkSkeleton(colors, Modifier.padding(top = 12.dp).fillMaxWidth().height(18.dp))
    LinkSkeleton(colors, Modifier.padding(top = 8.dp).fillMaxWidth(.7f).height(18.dp))
    LinkSkeleton(colors, Modifier.padding(top = 12.dp).fillMaxWidth().height(12.dp))
}

@Composable
private fun LinkSkeleton(colors: LinkColors, modifier: Modifier, corner: Int = 8) {
    val transition = rememberInfiniteTransition(label = "分享链接骨架")
    val progress by transition.animateFloat(0f, 1f, infiniteRepeatable(tween(1300, easing = LinearEasing)), label = "骨架扫光")
    Box(modifier.clip(RoundedCornerShape(corner.dp)).background(Brush.horizontalGradient(
        colorStops = arrayOf(0f to colors.line, .5f to Color(0xFF7F7F7F).copy(alpha = .18f), 1f to colors.line),
        startX = progress * 700f - 350f, endX = progress * 700f + 350f,
    )))
}

private fun linkKindIcon(info: CommunityShare): ImageVector = when (info.kind) {
    "pack", "scheme" -> XIcons.Monitor
    "project" -> LinkCode
    "weekly" -> LinkNews
    else -> LinkPost
}

private val LinkPost by lazy { linkIcon("帖子", "M5 5h14v10H9l-4 4z") }
private val LinkCode by lazy { linkIcon("开源精选", "m8 8-4 4 4 4M16 8l4 4-4 4M14 5l-4 14") }
private val LinkNews by lazy { linkIcon("周刊", "M5 4h11v16H6a1 1 0 0 1-1-1zM16 8h3v10a2 2 0 0 1-2 2M8 8h5M8 12h5M8 16h3") }
private val LinkInfo by lazy { linkIcon("提示", "M3 12a9 9 0 1 0 18 0a9 9 0 1 0-18 0M12 8h.01M11 12h1v5h1") }
private val LinkGhost by lazy { linkIcon("不可用", "M6 20V10a6 6 0 0 1 12 0v10l-2-1.5-2 1.5-2-1.5-2 1.5-2-1.5zM10 10h.01M14 10h.01") }
private val LinkWifi by lazy { linkIcon("网络失败", "M2 2l20 20M8.5 16.5a5 5 0 0 1 7 0M5 12.9a10 10 0 0 1 5.2-2.8M19 12.9a10 10 0 0 0-2.3-1.6M2 9a15 15 0 0 1 4.2-2.7M22 9a15 15 0 0 0-9-3.9M12 20h.01") }
private val LinkArrow by lazy { linkIcon("查看详情", "M5 12h14M13 6l6 6-6 6", 2.2f) }

private fun linkIcon(name: String, path: String, stroke: Float = 1.8f) =
    ImageVector.Builder(name, 24.dp, 24.dp, 24f, 24f).addPath(addPathNodes(path),
        stroke = SolidColor(Color.Black), strokeLineWidth = stroke, strokeLineCap = StrokeCap.Round, strokeLineJoin = StrokeJoin.Round).build()
