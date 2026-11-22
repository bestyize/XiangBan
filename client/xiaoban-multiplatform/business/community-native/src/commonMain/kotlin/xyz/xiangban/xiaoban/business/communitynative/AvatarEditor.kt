package xyz.xiangban.xiaoban.business.communitynative

// 更换头像：底部选图弹层 + 全屏裁剪页。网页窄屏用同一套交互。
// 设计稿：client/xiaoban-multiplatform/docs/design-specs/prototypes/avatar-editor.html

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.gestures.detectTransformGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.Paint
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import kotlin.math.roundToInt
import xyz.xiangban.xiaoban.common.community.ForumImage
import androidx.compose.ui.graphics.Canvas as BitmapCanvas

/** 导出头像的边长（像素）。 */
const val AVATAR_EXPORT_PX = 512
/** 本机图片的大小上限。 */
const val AVATAR_MAX_BYTES = 10L * 1024 * 1024
/** 解码后的原图长边上限，避免大图占满内存。 */
const val AVATAR_SOURCE_MAX_SIDE = 2048
private const val AVATAR_SMALL_PX = 256
private const val AVATAR_MAX_ZOOM = 4f

/** 圆心在画面中心；tx/ty 是图片中心相对圆心的偏移，和直径同一单位（像素）。 */
data class AvatarCrop(val iw: Int, val ih: Int, val zoom: Float = 1f, val tx: Float = 0f, val ty: Float = 0f) {
    /** zoom = 1 时图片短边正好等于圆的直径。 */
    fun scale(d: Float): Float = d / minOf(iw, ih) * zoom

    /** 让图片始终盖满圆。 */
    fun clamped(d: Float): AvatarCrop {
        val s = scale(d)
        val mx = maxOf(0f, (iw * s - d) / 2)
        val my = maxOf(0f, (ih * s - d) / 2)
        return copy(tx = tx.coerceIn(-mx, mx), ty = ty.coerceIn(-my, my))
    }

    fun panned(d: Float, dx: Float, dy: Float): AvatarCrop = copy(tx = tx + dx, ty = ty + dy).clamped(d)

    /** 以 (px, py)（相对圆心）为不动点缩放。 */
    fun zoomed(d: Float, z: Float, px: Float = 0f, py: Float = 0f): AvatarCrop {
        val next = z.coerceIn(1f, AVATAR_MAX_ZOOM)
        val k = next / zoom
        return copy(zoom = next, tx = px - (px - tx) * k, ty = py - (py - ty) * k).clamped(d)
    }

    /** 原图顺时针转 90° 后的状态。 */
    fun rotated(d: Float): AvatarCrop = AvatarCrop(ih, iw, zoom, -ty, tx).clamped(d)

    /** 画面尺寸变了（如横竖屏）时按比例换算偏移。 */
    fun rescaled(from: Float, to: Float): AvatarCrop = if (from <= 0f) this else copy(tx = tx * to / from, ty = ty * to / from)

    /** 圆的外接正方形在原图里的左、上和边长。 */
    fun source(d: Float): Triple<Float, Float, Float> {
        val s = scale(d)
        return Triple(iw / 2f - (d / 2 + tx) / s, ih / 2f - (d / 2 + ty) / s, d / s)
    }

    /** 圆内对应原图不足 256 像素时提示可能模糊。 */
    val small: Boolean get() = minOf(iw, ih) / zoom < AVATAR_SMALL_PX
}

/** 双击：放大到 2.2 倍，已放大时还原。 */
fun avatarDoubleTapZoom(zoom: Float): Float = if (zoom > 1.4f) 1f else 2.2f

private fun sourceRect(image: ImageBitmap, crop: AvatarCrop, d: Float): Pair<IntOffset, IntSize> {
    val (sx, sy, size) = crop.source(d)
    val side = size.roundToInt().coerceIn(1, minOf(image.width, image.height))
    return IntOffset(sx.roundToInt().coerceIn(0, image.width - side), sy.roundToInt().coerceIn(0, image.height - side)) to IntSize(side, side)
}

internal fun ImageBitmap.rotatedClockwise(): ImageBitmap {
    val out = ImageBitmap(height, width)
    BitmapCanvas(out).apply {
        translate(height.toFloat(), 0f)
        rotate(90f)
        drawImage(this@rotatedClockwise, Offset.Zero, Paint())
    }
    return out
}

/** 把圆的外接正方形画成 [px]×[px] 白底方图，显示时由界面裁成圆。 */
internal fun renderAvatar(image: ImageBitmap, crop: AvatarCrop, d: Float, px: Int = AVATAR_EXPORT_PX): ImageBitmap {
    val out = ImageBitmap(px, px)
    val canvas = BitmapCanvas(out)
    canvas.drawRect(0f, 0f, px.toFloat(), px.toFloat(), Paint().apply { color = Color.White })
    val (offset, size) = sourceRect(image, crop, d)
    canvas.drawImageRect(image, offset, size, IntOffset.Zero, IntSize(px, px), Paint().apply { filterQuality = FilterQuality.High })
    return out
}

internal enum class AvatarStatus { Idle, Cropping, Saving, Failed }

/** 放在设置页里：关掉裁剪页后，保存失败的那张图还在，再打开直接回到裁剪。 */
@Stable
internal class AvatarEditorState {
    var image by mutableStateOf<ImageBitmap?>(null)
    var crop by mutableStateOf<AvatarCrop?>(null)
    var diameter by mutableStateOf(0f)
    var sheet by mutableStateOf(false)
    var cropOpen by mutableStateOf(false)
    var picking by mutableStateOf(false)
    var status by mutableStateOf(AvatarStatus.Idle)
    var error by mutableStateOf<String?>(null)
    var pending by mutableStateOf(false)
    private var imageVersion = 0
    /** 已上传的 (裁剪签名, 图片地址)：同一张图同一位置重试时不再上传。 */
    var uploaded: Pair<String, ForumImage>? = null

    val saving: Boolean get() = status == AvatarStatus.Saving
    val failed: Boolean get() = status == AvatarStatus.Failed

    fun signature(): String {
        val c = crop ?: return ""
        val d = diameter.takeIf { it > 0f } ?: 1f
        return listOf(imageVersion, c.iw, c.ih, c.zoom, c.tx / d, c.ty / d).joinToString("|")
    }

    fun open() {
        if (pending) { pending = false; cropOpen = true } else sheet = true
    }

    fun use(bitmap: ImageBitmap) {
        image = bitmap
        imageVersion++
        crop = AvatarCrop(bitmap.width, bitmap.height)
        uploaded = null
        error = null
        pending = false
        status = AvatarStatus.Cropping
        sheet = false
        cropOpen = true
    }

    fun rotate() {
        val current = image ?: return
        val c = crop ?: return
        if (saving) return
        image = current.rotatedClockwise()
        imageVersion++
        crop = c.rotated(diameter)
    }

    fun close() {
        if (saving) return
        if (failed && image != null) pending = true else clear()
        cropOpen = false
        sheet = false
    }

    fun clear() {
        image = null
        crop = null
        uploaded = null
        error = null
        pending = false
        status = AvatarStatus.Idle
        cropOpen = false
    }
}

/** 账号头像：有图显示图，没有时品牌色底加名字首字。 */
@Composable
internal fun AccountAvatar(url: String?, name: String?, size: Dp, modifier: Modifier = Modifier) {
    Box(modifier.size(size).clip(CircleShape).background(if (url.isNullOrBlank()) Brand else LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
        if (!url.isNullOrBlank()) {
            AsyncImage(url, null, imageLoader = rememberImageLoader(), contentScale = ContentScale.Crop, modifier = Modifier.fillMaxSize())
        } else {
            Text(name?.take(1)?.uppercase() ?: "?", color = Color.White, fontSize = (size.value * 0.4f).sp, fontWeight = FontWeight.SemiBold)
        }
    }
}

/** 设置里的头像行：右侧当前头像和箭头。 */
@Composable
internal fun AvatarRow(url: String?, name: String?, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("头像", Modifier.weight(1f), style = XType.Body)
        AccountAvatar(url, name, 40.dp)
        XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
    }
}

/** 保存失败后离开裁剪页：留一行提示，可继续或放弃。 */
@Composable
internal fun AvatarPendingRow(onDiscard: () -> Unit, onResume: () -> Unit) {
    val extra = LocalExtraColors.current
    Row(
        Modifier.fillMaxWidth().background(extra.brandSoft).padding(start = 16.dp, end = 12.dp, top = 6.dp, bottom = 6.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        Text("有一张裁好的头像还没保存", Modifier.weight(1f), style = XType.Secondary, color = extra.brandInk)
        Text("放弃", Modifier.clip(CircleShape).clickable(onClick = onDiscard).padding(horizontal = 10.dp, vertical = 7.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text("继续", Modifier.clip(CircleShape).background(MaterialTheme.colorScheme.surface).clickable(onClick = onResume).padding(horizontal = 12.dp, vertical = 7.dp),
            style = XType.Secondary, color = extra.brandInk, fontWeight = FontWeight.SemiBold)
    }
}

/** 选图弹层：从相册选择 + 取消。不提供拍照。 */
@Composable
internal fun AvatarPickSheet(state: AvatarEditorState, onPick: () -> Unit) {
    BottomSheet(onDismiss = { if (!state.picking) state.sheet = false }, flush = true, cornerRadius = 16.dp) {
        Text("更换头像", Modifier.padding(start = 16.dp, end = 16.dp, bottom = 8.dp), style = XType.Title, fontWeight = FontWeight.SemiBold)
        Row(
            Modifier.fillMaxWidth().clickable(enabled = !state.picking, onClick = onPick).padding(horizontal = 16.dp, vertical = 15.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            XIcon(XIcons.Image, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 20.dp)
            Text("从相册选择", Modifier.weight(1f), style = XType.Body)
            if (state.picking) Text("正在读取…", style = XType.Secondary, color = LocalExtraColors.current.ink3)
        }
        Box(Modifier.fillMaxWidth().height(8.dp).background(MaterialTheme.colorScheme.background))
        Text("取消", Modifier.fillMaxWidth().clickable(enabled = !state.picking) { state.sheet = false }.padding(vertical = 15.dp),
            style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
    }
}

@Composable
private fun CropIcon(icon: ImageVector, description: String, enabled: Boolean, onClick: () -> Unit) {
    Box(Modifier.size(48.dp).clip(RoundedCornerShape(12.dp)).alpha(if (enabled) 1f else 0.35f).clickable(enabled = enabled, onClick = onClick), contentAlignment = Alignment.Center) {
        XIcon(icon, size = 20.dp, description = description)
    }
}

/** 全屏裁剪页：390 宽方形画面，圆占 300/390；下方工具、提示、预览和保存。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
internal fun AvatarCropPage(state: AvatarEditorState, name: String?, currentUrl: String?, onSave: () -> Unit) {
    val extra = LocalExtraColors.current
    val saving = state.saving
    BackHandler { state.close() }
    Column(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).clickable(indication = null, interactionSource = null) {}) {
        ScreenBar("裁剪头像", onBack = { state.close() })
        if (state.failed) {
            Row(
                Modifier.padding(start = 12.dp, end = 12.dp, bottom = 8.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(Danger.copy(alpha = 0.08f)).padding(horizontal = 12.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                XIcon(XIcons.Info, tint = Danger, size = 16.dp)
                Text("保存失败，${(state.error ?: "网络连接中断").trimEnd('。', '.')}。头像没有改动。", style = XType.Secondary, color = Danger)
            }
        }
        BoxWithConstraints(Modifier.fillMaxWidth().background(extra.fill), contentAlignment = Alignment.Center) {
            val side = if (maxWidth < 480.dp) maxWidth else 480.dp
            val d = with(LocalDensity.current) { side.toPx() } * 300f / 390f
            LaunchedEffect(d) {
                if (state.diameter != d) {
                    state.crop = state.crop?.rescaled(state.diameter, d)?.clamped(d)
                    state.diameter = d
                }
            }
            CropStage(state, side, d)
        }
        val crop = state.crop
        Row(Modifier.fillMaxWidth().padding(start = 8.dp, end = 8.dp, top = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            CropIcon(XIcons.RotateCw, "旋转 90 度", !saving) { state.rotate() }
            XIcon(XIcons.ZoomOut, tint = extra.ink3, size = 16.dp)
            Slider(
                value = crop?.zoom ?: 1f,
                onValueChange = { z -> state.crop = state.crop?.zoomed(state.diameter, z) },
                valueRange = 1f..AVATAR_MAX_ZOOM,
                enabled = !saving && crop != null,
                modifier = Modifier.weight(1f).padding(horizontal = 4.dp),
                colors = SliderDefaults.colors(thumbColor = Color.White, activeTrackColor = Brand, inactiveTrackColor = extra.line),
            )
            XIcon(XIcons.ZoomIn, tint = extra.ink3, size = 20.dp)
            CropIcon(XIcons.ResetCrop, "还原位置", !saving) { state.crop = crop?.let { AvatarCrop(it.iw, it.ih) } }
        }
        val small = crop?.small == true
        Text(
            if (small) "图片较小，再放大可能会模糊" else "单指拖动 · 双指缩放 · 双击放大",
            Modifier.fillMaxWidth().padding(start = 16.dp, end = 16.dp, top = 2.dp, bottom = 10.dp),
            style = XType.Caption, color = if (small) Color(0xFFB26A00) else extra.ink3, textAlign = TextAlign.Center,
        )
        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    AccountAvatar(currentUrl, name, 48.dp)
                    Text(if (state.failed) "当前 · 未改变" else "当前", style = XType.Caption, color = extra.ink3)
                }
                XIcon(XIcons.ArrowRight, tint = extra.line, size = 16.dp, modifier = Modifier.padding(bottom = 18.dp))
                Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    CropPreview(state, 56.dp)
                    Text("新头像", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            Column(Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    CropPreview(state, 24.dp)
                    Text(name ?: "", style = XType.Secondary, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false))
                    Text("· 刚刚", style = XType.Caption, color = extra.ink3)
                }
                Text("在帖子和评论里，你的头像这样显示。", Modifier.padding(top = 4.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2)
            }
        }
        Spacer(Modifier.weight(1f))
        Row(Modifier.fillMaxWidth().navigationBarsPadding().padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            GhostPillButton("重新选择", { state.sheet = true }, Modifier.weight(1f), enabled = !saving)
            Row(
                Modifier.weight(1f).height(48.dp).alpha(if (saving || crop == null) 0.75f else 1f).clip(CircleShape).background(extra.brandSoft)
                    .clickable(enabled = !saving && crop != null, onClick = onSave),
                horizontalArrangement = Arrangement.Center, verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(if (saving) "保存中…" else if (state.failed) "重试" else "保存", style = XType.Section, fontWeight = FontWeight.SemiBold, color = extra.brandInk)
            }
        }
    }
}

@Composable
private fun CropStage(state: AvatarEditorState, side: Dp, d: Float) {
    val image = state.image ?: return
    var dragging by remember { mutableStateOf(false) }
    val mask = MaterialTheme.colorScheme.surface.copy(alpha = 0.74f)
    val locked = state.saving
    Canvas(
        Modifier.size(side)
            .pointerInput(locked) {
                if (locked) return@pointerInput
                awaitEachGesture {
                    awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                    dragging = true
                    do { val event = awaitPointerEvent(PointerEventPass.Initial) } while (event.changes.any { it.pressed })
                    dragging = false
                }
            }
            .pointerInput(locked, d) {
                if (locked) return@pointerInput
                detectTapGestures(onDoubleTap = { p ->
                    state.crop = state.crop?.let { it.zoomed(d, avatarDoubleTapZoom(it.zoom), p.x - size.width / 2f, p.y - size.height / 2f) }
                })
            }
            .pointerInput(locked, d) {
                if (locked) return@pointerInput
                detectTransformGestures { centroid, pan, zoom, _ ->
                    val c = state.crop ?: return@detectTransformGestures
                    var next = c.panned(d, pan.x, pan.y)
                    if (zoom != 1f) next = next.zoomed(d, next.zoom * zoom, centroid.x - size.width / 2f, centroid.y - size.height / 2f)
                    state.crop = next
                }
            },
    ) {
        val c = state.crop ?: return@Canvas
        val s = c.scale(d)
        val w = c.iw * s
        val h = c.ih * s
        val cx = size.width / 2
        val cy = size.height / 2
        drawImage(
            image, IntOffset.Zero, IntSize(image.width, image.height),
            dstOffset = IntOffset((cx + c.tx - w / 2).roundToInt(), (cy + c.ty - h / 2).roundToInt()),
            dstSize = IntSize(w.roundToInt(), h.roundToInt()), filterQuality = FilterQuality.Medium,
        )
        val r = d / 2
        val circle = Rect(Offset(cx, cy), r)
        drawPath(Path().apply { fillType = PathFillType.EvenOdd; addRect(Rect(Offset.Zero, size)); addOval(circle) }, mask)
        if (dragging) {
            clipPath(Path().apply { addOval(circle) }) {
                for (f in listOf(1f / 3, 2f / 3)) {
                    drawLine(Color.White.copy(alpha = 0.75f), Offset(cx - r + d * f, cy - r), Offset(cx - r + d * f, cy + r), 1.dp.toPx())
                    drawLine(Color.White.copy(alpha = 0.75f), Offset(cx - r, cy - r + d * f), Offset(cx + r, cy - r + d * f), 1.dp.toPx())
                }
            }
        }
        drawCircle(Color.Black.copy(alpha = 0.10f), r + 1.5.dp.toPx(), Offset(cx, cy), style = Stroke(1.dp.toPx()))
        drawCircle(Color.White.copy(alpha = 0.95f), r, Offset(cx, cy), style = Stroke(2.dp.toPx()))
    }
}

@Composable
private fun CropPreview(state: AvatarEditorState, size: Dp) {
    val image = state.image
    val crop = state.crop
    val d = state.diameter
    Canvas(Modifier.size(size).clip(CircleShape).background(LocalExtraColors.current.fill)) {
        if (image == null || crop == null || d <= 0f) return@Canvas
        val (offset, src) = sourceRect(image, crop, d)
        drawImage(image, offset, src, dstSize = IntSize(this.size.width.roundToInt(), this.size.height.roundToInt()), filterQuality = FilterQuality.High)
    }
}
