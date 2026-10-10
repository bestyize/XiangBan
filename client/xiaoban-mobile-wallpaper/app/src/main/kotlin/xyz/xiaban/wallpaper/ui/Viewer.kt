package xyz.xiaban.wallpaper.ui

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import android.content.pm.ActivityInfo
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.LifecycleEventObserver
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.exoplayer.DefaultLoadControl
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.ui.AspectRatioFrameLayout
import androidx.media3.ui.PlayerView
import xyz.xiaban.wallpaper.data.MediaKind
import xyz.xiaban.wallpaper.data.VideoCache

private fun Context.findActivity(): Activity? = when (this) {
    is Activity -> this
    is ContextWrapper -> baseContext.findActivity()
    else -> null
}

/** Hides the status and navigation bars while a picture or video fills the screen; a swipe from the edge shows them briefly. */
@Composable
private fun ImmersiveBars() {
    val view = LocalView.current
    DisposableEffect(view) {
        val window = view.context.findActivity()?.window
        val controller = window?.let { WindowCompat.getInsetsController(it, view) }
        controller?.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        controller?.hide(WindowInsetsCompat.Type.systemBars())
        onDispose { controller?.show(WindowInsetsCompat.Type.systemBars()) }
    }
}

/** Full-screen viewer: pictures can be pinched, dragged and double-tapped; videos play with a seek bar. */
@Composable
fun Viewer(item: Viewable, onClose: () -> Unit) {
    ImmersiveBars()
    BackHandler(onBack = onClose)
    if (item.kind == MediaKind.VIDEO) VideoViewer(item, onClose) else ImageViewer(item, onClose)
}

@Composable
private fun CloseButton(onClose: () -> Unit, modifier: Modifier = Modifier) {
    Box(
        modifier.padding(8.dp).size(44.dp).clip(CircleShape).background(Color(0x66000000)).clickable(onClick = onClose),
        contentAlignment = Alignment.Center,
    ) { Icon(Icons.Filled.Close, "关闭", Modifier.size(24.dp), tint = Color.White) }
}

@Composable
private fun ImageViewer(item: Viewable, onClose: () -> Unit) {
    val context = LocalContext.current
    val state = rememberZoomState()
    var chrome by remember { mutableStateOf(true) }
    var loading by remember(item.key) { mutableStateOf(true) }
    var failed by remember(item.key) { mutableStateOf(false) }

    Box(Modifier.fillMaxSize()) {
        ZoomableBox(aspect = item.aspect, state = state, onTap = { chrome = !chrome }, onDismiss = onClose) {
            // The picture from the detail page is already cached, so something sharp-ish shows at once.
            FallbackImage(item.poster, item.fallbackPoster, null, Modifier.fillMaxSize())
            FallbackImage(
                // Original size so a zoomed-in picture shows real detail instead of a blown-up screen-sized copy.
                primary = item.image, fallback = item.fallbackImage, contentDescription = item.title,
                modifier = Modifier.fillMaxSize(), fullResolution = true,
                onLoading = { loading = true },
                onSuccess = {
                    loading = false
                    failed = false
                },
                onError = {
                    loading = false
                    failed = true
                },
            )
        }
        AnimatedVisibility(chrome, Modifier.align(Alignment.TopStart).statusBarsPadding(), enter = fadeIn(), exit = fadeOut()) {
            CloseButton(onClose)
        }
        if (loading) {
            CircularProgressIndicator(Modifier.align(Alignment.BottomCenter).padding(bottom = 48.dp).size(22.dp), color = Color.White, strokeWidth = 2.dp)
        }
        if (failed) {
            Text(
                "大图加载失败，正在显示较小的版本",
                Modifier.align(Alignment.BottomCenter).padding(bottom = 40.dp).clip(RoundedCornerShape(16.dp)).background(Color(0x99000000)).padding(horizontal = 14.dp, vertical = 8.dp),
                color = Color.White, fontSize = 12.sp,
            )
        }
    }
}

private const val MIN_BUFFER_MS = 15_000
private const val MAX_BUFFER_MS = 30_000
private const val START_BUFFER_MS = 2_500
private const val REBUFFER_MS = 6_000
private const val BUFFER_BYTES = 48 * 1024 * 1024

@Composable
private fun VideoViewer(item: Viewable, onClose: () -> Unit) {
    val context = LocalContext.current
    var error by remember(item.key) { mutableStateOf<String?>(null) }
    var buffering by remember(item.key) { mutableStateOf(true) }
    var controls by remember { mutableStateOf(true) }
    var landscape by remember { mutableStateOf(false) }
    var triedFallback by remember(item.key) { mutableStateOf(false) }

    val player = remember(item.key) {
        // Videos here are 4K files of 100 MB or more: keep the in-memory buffer small and let the disk cache hold the rest.
        val load = DefaultLoadControl.Builder()
            .setBufferDurationsMs(MIN_BUFFER_MS, MAX_BUFFER_MS, START_BUFFER_MS, REBUFFER_MS)
            .setTargetBufferBytes(BUFFER_BYTES)
            .setPrioritizeTimeOverSizeThresholds(false)
            .build()
        ExoPlayer.Builder(context)
            .setLoadControl(load)
            .setMediaSourceFactory(DefaultMediaSourceFactory(VideoCache.dataSourceFactory(context, remote = !item.local)))
            .build().apply {
                setMediaItem(MediaItem.fromUri(item.video))
                repeatMode = Player.REPEAT_MODE_ONE
                playWhenReady = true
                prepare()
            }
    }

    DisposableEffect(player) {
        val listener = object : Player.Listener {
            override fun onPlaybackStateChanged(state: Int) {
                buffering = state == Player.STATE_BUFFERING
            }

            override fun onPlayerError(e: PlaybackException) {
                // The provider's link failed: try the site's own copy once before giving up.
                if (!triedFallback && item.fallbackVideo.isNotEmpty()) {
                    triedFallback = true
                    player.setMediaItem(MediaItem.fromUri(item.fallbackVideo))
                    player.prepare()
                    player.play()
                    return
                }
                error = when (e.errorCode) {
                    PlaybackException.ERROR_CODE_IO_NETWORK_CONNECTION_FAILED,
                    PlaybackException.ERROR_CODE_IO_NETWORK_CONNECTION_TIMEOUT -> "网络连接不上，请检查网络后重试"
                    PlaybackException.ERROR_CODE_DECODING_FAILED,
                    PlaybackException.ERROR_CODE_DECODER_INIT_FAILED,
                    PlaybackException.ERROR_CODE_DECODING_FORMAT_UNSUPPORTED -> "这台手机无法解码这个视频"
                    else -> "视频无法播放（${e.errorCodeName}）"
                }
            }
        }
        player.addListener(listener)
        onDispose {
            player.removeListener(listener)
            player.release()
        }
    }

    // Pause when the app goes to the background instead of streaming a 4K video unseen.
    val lifecycle = LocalLifecycleOwner.current.lifecycle
    DisposableEffect(lifecycle, player) {
        val observer = LifecycleEventObserver { _, event -> if (event == Lifecycle.Event.ON_STOP) player.pause() }
        lifecycle.addObserver(observer)
        onDispose { lifecycle.removeObserver(observer) }
    }

    // Landscape is offered by the player's own full-screen button; leaving puts the screen back to normal.
    DisposableEffect(landscape) {
        val activity = context.findActivity()
        activity?.requestedOrientation = if (landscape) ActivityInfo.SCREEN_ORIENTATION_SENSOR_LANDSCAPE else ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
        onDispose { }
    }
    DisposableEffect(Unit) {
        onDispose { context.findActivity()?.requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED }
    }

    Box(Modifier.fillMaxSize().background(Color.Black)) {
        // The poster stays visible until the first frame is drawn over it.
        FallbackImage(item.poster, item.fallbackPoster, null, Modifier.fillMaxSize())
        AndroidView(
            factory = { ctx ->
                PlayerView(ctx).apply {
                    this.player = player
                    useController = true
                    resizeMode = AspectRatioFrameLayout.RESIZE_MODE_FIT
                    setShutterBackgroundColor(android.graphics.Color.TRANSPARENT)
                    setShowBuffering(PlayerView.SHOW_BUFFERING_ALWAYS)
                    controllerShowTimeoutMs = CONTROLLER_TIMEOUT_MS
                    setControllerVisibilityListener(PlayerView.ControllerVisibilityListener { controls = it == android.view.View.VISIBLE })
                    setFullscreenButtonClickListener { toLandscape -> landscape = toLandscape }
                }
            },
            update = { it.player = player },
            onRelease = { it.player = null },
            modifier = Modifier.fillMaxSize(),
        )
        AnimatedVisibility(controls, Modifier.align(Alignment.TopStart).statusBarsPadding(), enter = fadeIn(), exit = fadeOut()) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                CloseButton(onClose)
                Text(item.title, color = Color.White, fontSize = 14.sp, maxLines = 1, modifier = Modifier.padding(end = 16.dp))
            }
        }
        error?.let { message ->
            Column(
                Modifier.align(Alignment.Center).padding(32.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(message, color = Color.White, fontSize = 15.sp, textAlign = TextAlign.Center)
                Spacer(Modifier.height(16.dp))
                Text(
                    "重试",
                    Modifier.clip(RoundedCornerShape(18.dp)).background(Palette.Brand).clickable {
                        error = null
                        player.prepare()
                        player.play()
                    }.padding(horizontal = 28.dp, vertical = 9.dp),
                    color = Color.White, fontSize = 15.sp,
                )
            }
        }
        if (buffering && error == null) {
            Text(
                "缓冲中…视频较大，网络慢时会等一会儿",
                Modifier.align(Alignment.BottomCenter).padding(bottom = 96.dp).clip(RoundedCornerShape(14.dp)).background(Color(0x88000000)).padding(horizontal = 12.dp, vertical = 6.dp),
                color = Color.White, fontSize = 11.sp,
            )
        }
    }
}

private const val CONTROLLER_TIMEOUT_MS = 3_000
