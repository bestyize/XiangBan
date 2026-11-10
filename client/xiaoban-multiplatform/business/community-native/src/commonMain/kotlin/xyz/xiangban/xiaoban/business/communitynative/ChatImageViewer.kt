package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import coil3.compose.AsyncImage
import kotlin.io.encoding.Base64
import kotlin.math.roundToLong
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import com.multiplatform.webview.web.WebContent
import com.multiplatform.webview.web.WebView
import com.multiplatform.webview.web.WebViewState

internal data class ChatImagePreview(val data: String, val label: String, val generated: Boolean = false, val prompt: String? = null)

private data class ResolvedImage(val data: String? = null, val loading: Boolean = true)

@Composable
private fun resolvedChatImage(image: ChatImagePreview, resolve: suspend (String) -> String?): ResolvedImage =
    key(image, resolve) {
        val source by produceState(ResolvedImage()) { value = ResolvedImage(resolve(image.data), loading = false) }
        source
    }

/** The native WebView renders the original GIF on Android and iOS, without scripts or network access. */
@Composable
private fun AnimatedChatImage(data: String, modifier: Modifier) {
    val state = remember(data) {
        val encoded = data.removePrefix("data:image/gif;base64,")
        val safe = data.startsWith("data:image/gif;base64,") && encoded.length <= 23_000_000 &&
            encoded.all { it in 'A'..'Z' || it in 'a'..'z' || it in '0'..'9' || it in "+/=" }
        val source = if (safe) data else ""
        WebViewState(WebContent.Data("""<!doctype html><html><head><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'"><style>html,body{margin:0;height:100%;background:transparent;overflow:hidden}img{width:100%;height:100%;object-fit:contain}</style></head><body><img src="$source" alt="GIF 动图"></body></html>""")).apply {
            webSettings.isJavaScriptEnabled = false
            webSettings.backgroundColor = Color.Transparent
            webSettings.androidWebSettings.domStorageEnabled = false
        }
    }
    WebView(state = state, modifier = modifier)
}

@Composable
private fun chatImageBytes(data: String?): ByteArray? = key(data) {
    val bytes by produceState<ByteArray?>(null) {
        value = data?.let { encoded -> withContext(Dispatchers.Default) {
            runCatching { Base64.decode(encoded.substringAfter(";base64,")) }.getOrNull()
        } }
    }
    bytes
}

@Composable
internal fun ChatImageThumbnail(image: ChatImagePreview, resolve: suspend (String) -> String?, onPreview: (ChatImagePreview) -> Unit) {
    val data = resolvedChatImage(image, resolve).data
    if (data?.startsWith("data:image/gif;base64,") == true) {
        Box(Modifier.fillMaxWidth().heightIn(min = 180.dp, max = 260.dp).clip(RoundedCornerShape(12.dp))) {
            AnimatedChatImage(data, Modifier.matchParentSize())
            Box(Modifier.matchParentSize().clickable(onClickLabel = "查看 GIF 动图") { onPreview(image) })
        }
        return
    }
    val bytes = chatImageBytes(data)
    AsyncImage(
        model = bytes,
        imageLoader = rememberImageLoader(),
        contentDescription = image.label,
        contentScale = ContentScale.Fit,
        modifier = Modifier.fillMaxWidth().heightIn(min = 120.dp, max = 260.dp)
            .clip(RoundedCornerShape(12.dp)).background(LocalExtraColors.current.fill)
            .clickable(onClickLabel = "查看大图") { onPreview(image) },
    )
}

@Composable
internal fun ChatImageViewer(
    image: ChatImagePreview,
    platform: PlatformActions?,
    resolve: suspend (String) -> String?,
    onReference: (() -> Unit)? = null,
    onReview: (() -> Unit)? = null,
    referencing: Boolean = false,
    onClose: () -> Unit,
) {
    val sourceImage = resolvedChatImage(image, resolve)
    val data = sourceImage.data
    val bytes = chatImageBytes(data)
    val scope = rememberCoroutineScope()
    var zoom by remember(image) { mutableStateOf(1f) }
    var offset by remember(image) { mutableStateOf(Offset.Zero) }
    var failed by remember(image) { mutableStateOf(false) }
    var saving by remember(image) { mutableStateOf(false) }
    var notice by remember(image) { mutableStateOf<String?>(null) }
    Dialog(onDismissRequest = onClose, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        BoxWithConstraints(Modifier.fillMaxSize().background(Color.Black).clipToBounds()) {
            val density = LocalDensity.current
            val width = with(density) { maxWidth.toPx() }
            val height = with(density) { maxHeight.toPx() }
            val imageModifier = Modifier.fillMaxSize()
                    .pointerInput(image, width, height) {
                        detectTransformGestures { centroid, pan, scale, _ ->
                            val next = (zoom * scale).coerceIn(1f, 5f)
                            val center = Offset(width / 2f, height / 2f)
                            val moved = centroid - center - (centroid - center - offset) * (next / zoom) + pan
                            val limitX = width * (next - 1f) / 2f
                            val limitY = height * (next - 1f) / 2f
                            offset = Offset(moved.x.coerceIn(-limitX, limitX), moved.y.coerceIn(-limitY, limitY))
                            zoom = next
                        }
                    }
                    .graphicsLayer {
                        scaleX = zoom
                        scaleY = zoom
                        translationX = offset.x
                        translationY = offset.y
                    }
            if (data?.startsWith("data:image/gif;base64,") == true) AnimatedChatImage(data, imageModifier)
            else AsyncImage(
                model = bytes, imageLoader = rememberImageLoader(), contentDescription = image.label,
                contentScale = ContentScale.Fit, onError = { failed = bytes != null }, onSuccess = { failed = false },
                modifier = imageModifier,
            )
            if (failed || !sourceImage.loading && data == null) Text("无法读取图片", Modifier.align(Alignment.Center), color = Color.White)
            Row(
                Modifier.align(Alignment.TopEnd).statusBarsPadding().padding(8.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically,
            ) {
                if (onReference != null) GhostPillButton(
                    if (referencing) "正在准备图片…" else "引用此图继续聊",
                    onReference, enabled = data != null && !failed && !referencing,
                )
                val source = bytes
                if (platform != null && platform.supportsChatImageSave && source != null && data != null && !failed) {
                    GhostPillButton("保存图片", {
                        saving = true
                        scope.launch {
                            try {
                                notice = when (platform.saveChatImage(source, data.substringAfter("data:").substringBefore(";"))) {
                                    CommunityPosterResult.Saved -> "图片已保存"
                                    CommunityPosterResult.Cancelled -> null
                                    CommunityPosterResult.ChooserOpened -> "请选择保存位置"
                                    else -> "无法保存图片"
                                }
                            } finally { saving = false }
                        }
                    }, enabled = !saving)
                }
                IconTap(XIcons.Close, "关闭", onClose, tint = Color.White)
            }
            if (image.generated && data != null && !failed) {
                val dimensions = remember(bytes, data) { originalImageSize(bytes, data) }
                Row(
                    Modifier.align(Alignment.BottomCenter).navigationBarsPadding().padding(12.dp)
                        .clip(RoundedCornerShape(16.dp)).background(Color.Black.copy(alpha = 0.8f)).padding(horizontal = 12.dp, vertical = 6.dp),
                    verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    dimensions?.let { Text(it, style = XType.Caption, color = Color.White) }
                    if (onReview != null) GhostPillButton("检查画面和文字", onReview, enabled = !referencing, height = 36.dp)
                }
            }
            Toast(notice) { notice = null }
        }
    }
}

private fun originalImageSize(bytes: ByteArray?, data: String): String? {
    if (bytes == null) return null
    fun byte(offset: Int) = bytes[offset].toInt() and 255
    fun int(offset: Int) = (byte(offset) shl 24) or (byte(offset + 1) shl 16) or (byte(offset + 2) shl 8) or byte(offset + 3)
    val (width, height) = when {
        data.startsWith("data:image/png;") && bytes.size >= 24 -> int(16) to int(20)
        data.startsWith("data:image/gif;") && bytes.size >= 10 -> (byte(6) or (byte(7) shl 8)) to (byte(8) or (byte(9) shl 8))
        else -> return null
    }
    return if (width > 0 && height > 0) "$width × $height" else null
}

private fun imageSizeLabel(size: String): String = when (size) {
    "auto" -> "自动 · 由服务决定尺寸"
    "1024x1024" -> "1:1 · 1024 × 1024"
    "1024x1536" -> "2:3 · 1024 × 1536"
    "1536x1024" -> "3:2 · 1536 × 1024"
    "1024x1792" -> "4:7 · 1024 × 1792"
    "1792x1024" -> "7:4 · 1792 × 1024"
    else -> size.replace("x", " × ")
}

@Composable
internal fun ImageGenerationSheet(
    initialPrompt: String,
    sizes: List<String>,
    formats: List<String> = listOf("png"),
    quota: String,
    basePointsPerImage: Double?,
    onDismiss: () -> Unit,
    onGenerate: (String, Int, String?, String, Int, Int) -> Unit,
) {
    var prompt by remember { mutableStateOf(initialPrompt) }
    var count by remember { mutableStateOf(1) }
    var size by remember(sizes) { mutableStateOf(sizes.firstOrNull()) }
    var gif by remember { mutableStateOf(false) }
    var frames by remember { mutableStateOf("9") }
    var duration by remember { mutableStateOf("200") }
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        BottomSheet(onDismiss) {
            Text("生成图片", style = XType.Title)
            if ("gif" in formats) Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                ChipPill("静态图片", selected = !gif, onClick = { gif = false })
                ChipPill("GIF 动图", selected = gif, onClick = { gif = true })
            }
            OutlinedTextField(
                value = prompt, onValueChange = { prompt = it },
                label = { Text("描述你想生成的画面") },
                modifier = Modifier.fillMaxWidth().padding(top = 12.dp), minLines = 2, maxLines = 5,
                isError = prompt.trim().length > 4000,
            )
            if (!gif) Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                (1..4).forEach { n -> ChipPill("${n} 张", selected = count == n, onClick = { count = n }) }
            }
            if (gif) Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(frames, { frames = it.filter(Char::isDigit).take(2) }, label = { Text("帧数 2–16") }, modifier = Modifier.weight(1f), singleLine = true)
                OutlinedTextField(duration, { duration = it.filter(Char::isDigit).take(4) }, label = { Text("每帧毫秒 20–2000") }, modifier = Modifier.weight(1f), singleLine = true)
            }
            if (sizes.size > 1) {
                Row(Modifier.horizontalScroll(rememberScrollState()).padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    sizes.forEach { option -> ChipPill(imageSizeLabel(option), selected = size == option, onClick = { size = option }) }
                }
            } else {
                Text("画幅 · ${imageSizeLabel(sizes.firstOrNull() ?: "auto")}", Modifier.padding(top = 10.dp), style = XType.Caption)
            }
            val estimate = basePointsPerImage?.takeIf { !gif && it.isFinite() && it > 0 }?.let {
                val cents = (it * count * 100).roundToLong()
                if (cents == 0L) "不足 0.01" else "${cents / 100}.${(cents % 100).toString().padStart(2, '0')}"
            }
            Text(
                if (estimate == null) "$quota · 按实际用量扣费，暂无点数预估"
                else "$quota · 基础参考 $estimate 点，按实际用量扣费",
                Modifier.padding(top = 10.dp), style = XType.Caption,
            )
            GhostPillButton("生成", { onGenerate(prompt, if (gif) 1 else count, size, if (gif) "gif" else "png", frames.toIntOrNull() ?: 9, duration.toIntOrNull() ?: 200) }, Modifier.fillMaxWidth().padding(top = 10.dp),
                enabled = prompt.trim().length in 1..4000 && (!gif || (frames.toIntOrNull() in 2..16 && duration.toIntOrNull() in 20..2000)))
        }
    }
}
