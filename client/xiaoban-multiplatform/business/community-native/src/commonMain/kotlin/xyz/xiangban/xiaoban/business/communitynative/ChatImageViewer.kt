package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

internal data class ChatImagePreview(val data: String, val label: String)

@Composable
private fun chatImageBytes(data: String): ByteArray? {
    val bytes by produceState<ByteArray?>(null, data) {
        value = withContext(Dispatchers.Default) {
            runCatching { Base64.decode(data.substringAfter(";base64,")) }.getOrNull()
        }
    }
    return bytes
}

@Composable
internal fun ChatImageThumbnail(image: ChatImagePreview, onPreview: (ChatImagePreview) -> Unit) {
    val bytes = chatImageBytes(image.data)
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
internal fun ChatImageViewer(image: ChatImagePreview, platform: PlatformActions?, onClose: () -> Unit) {
    val bytes = chatImageBytes(image.data)
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
            AsyncImage(
                model = bytes,
                imageLoader = rememberImageLoader(),
                contentDescription = image.label,
                contentScale = ContentScale.Fit,
                onError = { failed = bytes != null },
                onSuccess = { failed = false },
                modifier = Modifier.fillMaxSize()
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
                    },
            )
            if (failed) Text("无法读取图片", Modifier.align(Alignment.Center), color = Color.White)
            Row(
                Modifier.align(Alignment.TopEnd).statusBarsPadding().padding(8.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically,
            ) {
                val source = bytes
                if (platform != null && platform.supportsChatImageSave && source != null && !failed) {
                    GhostPillButton("保存图片", {
                        saving = true
                        scope.launch {
                            try {
                                notice = when (platform.saveChatImage(source, image.data.substringAfter("data:").substringBefore(";"))) {
                                    CommunityPosterResult.Saved -> "图片已保存"
                                    CommunityPosterResult.Cancelled -> null
                                    else -> "无法保存图片"
                                }
                            } finally { saving = false }
                        }
                    }, enabled = !saving)
                }
                IconTap(XIcons.Close, "关闭", onClose, tint = Color.White)
            }
            Toast(notice) { notice = null }
        }
    }
}

@Composable
internal fun ImageGenerationSheet(
    initialPrompt: String,
    sizes: List<String>,
    onDismiss: () -> Unit,
    onGenerate: (String, Int, String?) -> Unit,
) {
    var prompt by remember { mutableStateOf(initialPrompt) }
    var count by remember { mutableStateOf(1) }
    var size by remember(sizes) { mutableStateOf(sizes.firstOrNull()) }
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        BottomSheet(onDismiss, Modifier.imePadding()) {
            Text("生成图片", style = XType.Title)
            OutlinedTextField(
                value = prompt, onValueChange = { prompt = it },
                label = { Text("描述你想生成的画面") },
                modifier = Modifier.fillMaxWidth().padding(top = 12.dp), minLines = 2, maxLines = 5,
                isError = prompt.trim().length > 4000,
            )
            Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                (1..4).forEach { n -> ChipPill("${n} 张", selected = count == n, onClick = { count = n }) }
            }
            if (sizes.size > 1) {
                Row(Modifier.horizontalScroll(rememberScrollState()).padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    sizes.forEach { option -> ChipPill(option, selected = size == option, onClick = { size = option }) }
                }
            }
            Text("使用小伴 AI 额度，按张计费", Modifier.padding(top = 10.dp), style = XType.Caption)
            GhostPillButton("生成", { onGenerate(prompt, count, size) }, Modifier.fillMaxWidth().padding(top = 10.dp),
                enabled = prompt.trim().length in 1..4000)
        }
    }
}
