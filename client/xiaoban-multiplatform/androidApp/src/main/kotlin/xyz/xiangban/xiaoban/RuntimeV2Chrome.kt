package xyz.xiangban.xiaoban

import android.graphics.Rect
import android.view.Gravity
import android.view.View
import android.widget.FrameLayout
import androidx.activity.ComponentActivity
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.StartOffset
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.ComposeView
import androidx.compose.ui.platform.ViewCompositionStrategy
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.paneTitle
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.business.communitynative.CenterDialog
import xyz.xiangban.xiaoban.business.communitynative.Danger
import xyz.xiangban.xiaoban.business.communitynative.DangerSoft
import xyz.xiangban.xiaoban.business.communitynative.GhostPillButton
import xyz.xiangban.xiaoban.business.communitynative.LocalExtraColors
import xyz.xiangban.xiaoban.business.communitynative.XIcon
import xyz.xiangban.xiaoban.business.communitynative.XIcons
import xyz.xiangban.xiaoban.business.communitynative.XType
import xyz.xiangban.xiaoban.business.communitynative.XiaobanTheme
import xyz.xiangban.xiaoban.common.community.DesignTokens
import kotlin.math.roundToInt

/** The launcher, capsule, and menu from the app plaza's b7 and b8 runtime artboards. */
internal class RuntimeV2Chrome(
    activity: ComponentActivity,
    private val root: FrameLayout,
    private val title: String,
    private val kindLabel: String,
    private val icon: ImageBitmap?,
    private val official: Boolean,
    private val author: String?,
    private val onRestart: () -> Unit,
    private val onExit: () -> Unit,
    private val onFavorite: (() -> Unit)?,
    private val onShare: (() -> Unit)?,
    private val onPinShortcut: (() -> Unit)?,
    private val onMenuChanged: (Boolean) -> Unit = {},
    private val landscapeLaunch: Boolean = false,
) {
    private sealed interface Overlay {
        data object Loading : Overlay
        data class Error(val message: String, val retry: () -> Unit) : Overlay
    }

    private class MediaConsent(val message: String, val onDecision: (Boolean) -> Unit)

    private var overlay by mutableStateOf<Overlay?>(Overlay.Loading)
    private var mediaConsent by mutableStateOf<MediaConsent?>(null)
    private var usingMedia by mutableStateOf(false)
    private var landscape by mutableStateOf(false)
    private var menuHorizontalInset by mutableStateOf(0.dp)
    private var menuBottomPadding by mutableStateOf(30.dp)
    private var menuVerticalInset by mutableStateOf(0.dp)
    private val displayMetrics = activity.resources.displayMetrics

    var menuOpen by mutableStateOf(false)
        private set
    val consentOpen: Boolean get() = mediaConsent != null

    var favorited by mutableStateOf(false)
    var favoriteBusy by mutableStateOf(false)
    var shareBusy by mutableStateOf(false)
    var shortcutBusy by mutableStateOf(false)

    // Only the capsule's small view receives touches while the work is running.
    private val capsuleView = ComposeView(activity).apply {
        setViewCompositionStrategy(ViewCompositionStrategy.DisposeOnViewTreeLifecycleDestroyed)
        setContent { XiaobanTheme { Capsule() } }
    }
    private val overlayView = ComposeView(activity).apply {
        setViewCompositionStrategy(ViewCompositionStrategy.DisposeOnViewTreeLifecycleDestroyed)
        setContent {
            XiaobanTheme {
                Box(Modifier.fillMaxSize()) {
                    when (val state = this@RuntimeV2Chrome.overlay) {
                        Overlay.Loading -> LaunchCard()
                        is Overlay.Error -> LaunchCard(state)
                        null -> if (menuOpen) RunMenu()
                    }
                    mediaConsent?.let { MediaConsentDialog(it) }
                }
            }
        }
    }

    init {
        root.addView(capsuleView, FrameLayout.LayoutParams(
            pixels(48f), pixels(48f), Gravity.TOP or Gravity.RIGHT,
        ))
        root.addView(overlayView, FrameLayout.LayoutParams(FrameLayout.LayoutParams.MATCH_PARENT, FrameLayout.LayoutParams.MATCH_PARENT))
        updateInsets(0, 0, 0, 0, landscape = false)
        syncVisibility()
    }

    /** Physical pixels relative to the full-screen WebView, before conversion to CSS pixels. */
    val capsuleBounds: Rect get() {
        val layout = capsuleView.layoutParams as FrameLayout.LayoutParams
        val right = root.width - layout.rightMargin
        return Rect(right - layout.width, layout.topMargin, right, layout.topMargin + layout.height)
    }
    val capsuleRightInset: Int get() = (capsuleView.layoutParams as FrameLayout.LayoutParams).rightMargin

    fun updateInsets(left: Int, top: Int, right: Int, bottom: Int, landscape: Boolean) {
        this.landscape = landscape
        val layout = capsuleView.layoutParams as FrameLayout.LayoutParams
        val topMargin = top
        val rightMargin = right + pixels(6f)
        if (layout.topMargin != topMargin || layout.rightMargin != rightMargin) {
            layout.topMargin = topMargin
            layout.rightMargin = rightMargin
            capsuleView.layoutParams = layout
        }
        menuHorizontalInset = (maxOf(left, right) / displayMetrics.density).dp
        menuVerticalInset = (maxOf(top, bottom) / displayMetrics.density).dp
        menuBottomPadding = maxOf(30.dp, (bottom / displayMetrics.density + 6).dp)
    }

    private fun openMenu() {
        if (overlay != null || consentOpen || menuOpen) return
        menuOpen = true
        syncVisibility()
        onMenuChanged(true)
    }

    fun closeMenu() {
        if (!menuOpen) return
        menuOpen = false
        syncVisibility()
        onMenuChanged(false)
    }

    fun loading() {
        overlay = Overlay.Loading
        closeMenu()
        syncVisibility()
    }

    fun loaded() {
        overlay = null
        syncVisibility()
    }

    fun error(message: String, retry: () -> Unit) {
        overlay = Overlay.Error(message, retry)
        closeMenu()
        syncVisibility()
    }

    fun showMediaConsent(message: String, onDecision: (Boolean) -> Unit) {
        if (consentOpen) {
            onDecision(false)
            return
        }
        mediaConsent = MediaConsent(message, onDecision)
        syncVisibility()
    }

    /** Clears a canceled WebView request without making a decision for it. */
    fun dismissMediaConsent() {
        if (!consentOpen) return
        mediaConsent = null
        syncVisibility()
    }

    fun setMediaActive(active: Boolean) { usingMedia = active }

    private fun resolveMediaConsent(request: MediaConsent, allow: Boolean) {
        if (mediaConsent !== request) return
        dismissMediaConsent()
        request.onDecision(allow)
    }

    private fun pixels(dp: Float) = (dp * displayMetrics.density).roundToInt()

    private fun syncVisibility() {
        capsuleView.visibility = if (overlay == null) View.VISIBLE else View.GONE
        capsuleView.importantForAccessibility = if (menuOpen || consentOpen) View.IMPORTANT_FOR_ACCESSIBILITY_NO_HIDE_DESCENDANTS else View.IMPORTANT_FOR_ACCESSIBILITY_AUTO
        overlayView.visibility = if (overlay != null || menuOpen || consentOpen) View.VISIBLE else View.GONE
    }

    @Composable
    private fun Capsule() {
        Box(
            Modifier.fillMaxSize().chromeClick(onClick = ::openMenu)
                .semantics { if (usingMedia) stateDescription = "正在使用摄像头或麦克风" },
            contentAlignment = Alignment.Center,
        ) {
            Box(
                Modifier.size(32.dp).background(Color(0xFF20232A), CircleShape)
                    .border(1.dp, Color.White.copy(alpha = .35f), CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                XIcon(XIcons.More, Modifier.rotate(90f), tint = Color.White, size = 20.dp, description = "应用菜单")
            }
            if (usingMedia) Box(
                Modifier.align(Alignment.TopEnd).padding(top = 5.dp, end = 8.dp)
                    .size(6.dp).background(Color(DesignTokens.Danger), CircleShape),
            )
        }
    }

    @Composable
    private fun MediaConsentDialog(request: MediaConsent) {
        val deny = { resolveMediaConsent(request, allow = false) }
        CenterDialog(onDismiss = deny) {
            Text(request.message, style = XType.Body, color = MaterialTheme.colorScheme.onSurface)
            Row(Modifier.fillMaxWidth().padding(top = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GhostPillButton(
                    "不允许", deny, Modifier.weight(1f), height = 44.dp,
                    textColor = MaterialTheme.colorScheme.onSurfaceVariant, onPage = true,
                )
                Box(
                    Modifier.weight(1f).height(44.dp).clip(CircleShape).background(DangerSoft)
                        .chromeClick { resolveMediaConsent(request, allow = true) },
                    contentAlignment = Alignment.Center,
                ) {
                    Text("允许", style = XType.Body, color = Danger, fontWeight = FontWeight.SemiBold)
                }
            }
        }
    }

    @Composable
    private fun RunMenu() {
        val scheme = MaterialTheme.colorScheme
        val scrim = remember { Animatable(0f) }
        val panelProgress = remember { Animatable(0f) }
        LaunchedEffect(Unit) { scrim.animateTo(.45f, tween(200)) }
        LaunchedEffect(landscape) {
            panelProgress.snapTo(0f)
            panelProgress.animateTo(1f, if (landscape) tween(200) else tween(260, easing = CubicBezierEasing(.3f, .7f, .2f, 1f)))
        }
        val panelShape = if (landscape) RoundedCornerShape(16.dp) else RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp)
        val panelWidth = if (landscape) Modifier.widthIn(max = 440.dp).fillMaxWidth() else Modifier.fillMaxWidth()
        Box(Modifier.fillMaxSize()) {
            Box(
                Modifier.fillMaxSize().graphicsLayer { alpha = scrim.value }
                    .background(Color.Black).chromeClick(onClick = ::closeMenu)
                    .semantics { contentDescription = "关闭菜单" },
            )
            Box(
                Modifier.fillMaxSize().padding(
                    horizontal = if (landscape) menuHorizontalInset else 0.dp,
                    vertical = if (landscape) menuVerticalInset else 0.dp,
                ),
                contentAlignment = if (landscape) Alignment.Center else Alignment.BottomCenter,
            ) {
                Column(
                    panelWidth.graphicsLayer {
                        if (landscape) alpha = panelProgress.value
                        else translationY = size.height * (1f - panelProgress.value)
                    }.clip(panelShape).background(scheme.surface)
                        .pointerInput(Unit) { detectTapGestures {} }
                        .semantics { paneTitle = "应用菜单" }
                        .verticalScroll(rememberScrollState())
                        .padding(
                            start = 16.dp + if (landscape) 0.dp else menuHorizontalInset,
                            end = 16.dp + if (landscape) 0.dp else menuHorizontalInset,
                            top = 16.dp,
                            bottom = if (landscape) 16.dp else menuBottomPadding,
                        ),
                ) {
                    MenuHeader()
                    Box(Modifier.fillMaxWidth().height(.5.dp).background(scheme.outline))
                    Row(Modifier.fillMaxWidth().padding(top = 14.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                        MenuAction(XIcons.Refresh, "重新开始", Modifier.weight(1f), onRestart)
                        onFavorite?.let {
                            MenuAction(XIcons.Star, if (favorited) "取消收藏" else "收藏", Modifier.weight(1f), it,
                                enabled = !favoriteBusy, selected = favorited)
                        }
                        onShare?.let { MenuAction(XIcons.Share, "分享", Modifier.weight(1f), it, enabled = !shareBusy) }
                        onPinShortcut?.let { MenuAction(XIcons.Pin, "添加到桌面", Modifier.weight(1f), it, enabled = !shortcutBusy) }
                    }
                    MenuPill("退出", onExit)
                }
            }
        }
    }

    @Composable
    private fun MenuHeader() {
        val scheme = MaterialTheme.colorScheme
        Row(
            Modifier.fillMaxWidth().padding(bottom = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            WorkIcon(44.dp, 12.dp)
            Column {
                Text(title, style = XType.Body, fontWeight = FontWeight.SemiBold, color = scheme.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Row(
                    Modifier.padding(top = 2.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    if (official) Text(
                        "官方",
                        Modifier.background(LocalExtraColors.current.fill, RoundedCornerShape(4.dp)).padding(horizontal = 6.dp, vertical = 1.dp),
                        style = XType.Caption, color = scheme.onSurfaceVariant,
                    )
                    Text(if (!official && !author.isNullOrBlank()) "$author · $kindLabel" else kindLabel,
                        style = XType.Caption, color = scheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            }
        }
    }

    @Composable
    private fun MenuAction(image: ImageVector, label: String, modifier: Modifier, action: () -> Unit,
        enabled: Boolean = true, selected: Boolean = false) {
        val tint = if (selected) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurface
        Column(
            modifier.clickable(interactionSource = null, indication = null, enabled = enabled, role = Role.Button, onClick = action),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Box(
                Modifier.size(48.dp).background(LocalExtraColors.current.fill, RoundedCornerShape(16.dp)),
                contentAlignment = Alignment.Center,
            ) { XIcon(image, tint = tint, size = 22.dp) }
            Text(label, style = XType.Caption, color = tint, textAlign = TextAlign.Center)
        }
    }

    @Composable
    private fun MenuPill(label: String, action: () -> Unit) {
        val extra = LocalExtraColors.current
        Row(
            Modifier.padding(top = 16.dp).fillMaxWidth().height(44.dp)
                .clip(CircleShape).background(extra.fill).chromeClick(onClick = action),
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(label, style = XType.Body, color = MaterialTheme.colorScheme.onSurface)
        }
    }

    @Composable
    private fun LaunchCard(error: Overlay.Error? = null) {
        val scheme = MaterialTheme.colorScheme
        val extra = LocalExtraColors.current
        val appear = remember { Animatable(0f) }
        LaunchedEffect(Unit) { appear.animateTo(1f, tween(200)) }
        Box(Modifier.fillMaxSize().background(scheme.background).pointerInput(Unit) { detectTapGestures {} }, contentAlignment = Alignment.Center) {
            Column(Modifier.graphicsLayer { alpha = appear.value }.padding(horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                WorkIcon(76.dp, 18.dp)
                Text(title, Modifier.padding(top = 16.dp), style = XType.Section, fontWeight = FontWeight.SemiBold, color = scheme.onBackground, textAlign = TextAlign.Center)
                when {
                    error != null -> {
                        Text(error.message, Modifier.padding(top = 12.dp), style = XType.Secondary, color = scheme.onSurfaceVariant, textAlign = TextAlign.Center)
                        MenuPill("重新打开", error.retry)
                    }
                    landscapeLaunch || landscape -> {
                        val rotation = remember { Animatable(0f) }
                        LaunchedEffect(Unit) { rotation.animateTo(-90f, tween(900, delayMillis = 250, easing = CubicBezierEasing(.5f, 0f, .3f, 1f))) }
                        XIcon(rotateIcon, Modifier.padding(top = 26.dp).graphicsLayer { rotationZ = rotation.value }, tint = extra.ink3, size = 40.dp)
                        Text("横过手机", Modifier.padding(top = 12.dp), style = XType.Secondary, color = scheme.onSurfaceVariant)
                    }
                    else -> LoadingDots(Modifier.padding(top = 22.dp), extra.ink3)
                }
            }
        }
    }

    @Composable
    private fun WorkIcon(size: Dp, radius: Dp) {
        val shape = RoundedCornerShape(radius)
        val scheme = MaterialTheme.colorScheme
        Box(Modifier.size(size).clip(shape).background(LocalExtraColors.current.fill).border(.5.dp, scheme.outline, shape), contentAlignment = Alignment.Center) {
            if (icon != null) Image(icon, contentDescription = null, Modifier.fillMaxSize(), contentScale = ContentScale.Crop)
            else XIcon(if (kindLabel == "游戏") XIcons.Gamepad else XIcons.Grid, tint = scheme.onSurfaceVariant, size = size * .5f)
        }
    }
}

@Composable
private fun LoadingDots(modifier: Modifier, color: Color) {
    val transition = rememberInfiniteTransition(label = "launch dots")
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
        repeat(3) { dot ->
            val opacity by transition.animateFloat(
                initialValue = .3f,
                targetValue = 1f,
                animationSpec = infiniteRepeatable(tween(500), RepeatMode.Reverse, StartOffset(dot * 200)),
                label = "launch dot $dot",
            )
            Box(Modifier.size(5.dp).graphicsLayer { alpha = opacity }.background(color, CircleShape))
        }
    }
}

private fun Modifier.chromeClick(onClick: () -> Unit): Modifier =
    clickable(interactionSource = null, indication = null, role = Role.Button, onClick = onClick)

private val rotateIcon by lazy {
    ImageVector.Builder(defaultWidth = 24.dp, defaultHeight = 24.dp, viewportWidth = 24f, viewportHeight = 24f)
        .addPath(pathData = addPathNodes("M8 2.5h8a2 2 0 0 1 2 2v15a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2v-15a2 2 0 0 1 2-2zM11 18.5h2"),
            stroke = SolidColor(Color.Black), strokeLineWidth = 1.5f, strokeLineCap = StrokeCap.Round, strokeLineJoin = StrokeJoin.Round)
        .build()
}
