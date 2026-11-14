package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.compose.AsyncImage
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ForumImage
import xyz.xiangban.xiaoban.common.community.NewsComment
import xyz.xiangban.xiaoban.common.community.messageOf

private enum class CommentPanel { Keyboard, Emoji, Gallery, More, Closed }

private val CommentEmojis = listOf(
    "😊" to "微笑", "😂" to "笑哭", "👍" to "点赞", "❤️" to "爱心",
    "🙌" to "举手", "🤔" to "思考", "🎉" to "庆祝",
    "😄" to "大笑", "🥹" to "感动", "🤣" to "笑翻", "😍" to "喜欢", "😅" to "尴尬", "😭" to "大哭", "😮" to "惊讶",
    "🤩" to "星星眼", "😎" to "墨镜", "🫡" to "敬礼", "👏" to "鼓掌", "🙏" to "感谢", "💪" to "加油", "🙈" to "捂眼",
    "🔥" to "火", "✨" to "闪光", "👀" to "看看", "💡" to "灵感", "🌿" to "绿叶", "☕" to "咖啡", "💯" to "满分",
)

/** 计数与服务端 Rust 的 chars().count() 一致，含代理对的表情按一个 Unicode 码点计。 */
internal fun commentLength(text: String): Int = text.indices.count {
    !(text[it].isLowSurrogate() && it > 0 && text[it - 1].isHighSurrogate())
}

private fun TextFieldValue.insert(text: String): TextFieldValue {
    val start = selection.min
    return TextFieldValue(this.text.replaceRange(start, selection.max, text), TextRange(start + text.length))
}

@Composable
internal fun CommentComposer(
    value: TextFieldValue,
    onValueChange: (TextFieldValue) -> Unit,
    onDismiss: () -> Unit,
    onSubmit: () -> Unit,
    limit: Int,
    replyTo: String? = null,
    title: String = "写评论",
    submitLabel: String = "发布",
    busyLabel: String? = null,
    length: Int = commentLength(value.text.trim()),
    hasImages: Boolean = false,
    locked: Boolean = false,
    editorReady: Boolean = true,
    error: String? = null,
    canMention: Boolean = false,
    onPickImage: (() -> Unit)? = null,
    preview: (@Composable () -> Unit)? = null,
    more: (@Composable () -> Unit)? = null,
    loading: @Composable () -> Unit = {},
) {
    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val focus = remember { FocusRequester() }
    val focusManager = LocalFocusManager.current
    val keyboard = LocalSoftwareKeyboardController.current
    val imeVisible = WindowInsets.ime.getBottom(LocalDensity.current) > 0
    var panel by remember { mutableStateOf(CommentPanel.Keyboard) }
    var expanded by remember { mutableStateOf(false) }

    fun switch(next: CommentPanel) {
        if (next != CommentPanel.Keyboard) {
            keyboard?.hide()
            focusManager.clearFocus()
        }
        panel = next
    }

    fun dismiss() {
        if (!locked || !editorReady) keyboard?.hide()
        onDismiss()
    }

    LaunchedEffect(panel, editorReady) {
        if (editorReady && panel == CommentPanel.Keyboard) {
            withFrameNanos { }
            focus.requestFocus()
            keyboard?.show()
        }
    }

    BottomSheet(
        onDismiss = ::dismiss,
        flush = true,
        cornerRadius = 16.dp,
        onBack = {
            when (panel) {
                CommentPanel.Emoji, CommentPanel.Gallery, CommentPanel.More -> switch(CommentPanel.Closed)
                else -> dismiss()
            }
        },
    ) {
        Column(Modifier.fillMaxWidth().heightIn(max = 640.dp)) {
            Row(
                Modifier.fillMaxWidth().padding(start = 16.dp, end = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    replyTo?.let { "回复 $it" } ?: title, Modifier.weight(1f), style = XType.Section,
                    fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis,
                )
                if (editorReady) CommentTool(
                    if (expanded) XIcons.Collapse else XIcons.Expand,
                    if (expanded) "收起编辑区" else "展开编辑区",
                    onClick = { expanded = !expanded },
                )
                CommentTool(XIcons.Close, "关闭评论编辑", onClick = ::dismiss)
            }

            Column(
                Modifier.weight(1f, fill = false).fillMaxWidth().verticalScroll(rememberScrollState())
                    .padding(start = 16.dp, end = 16.dp, bottom = 4.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                if (editorReady) {
                    BasicTextField(
                        value = value,
                        onValueChange = onValueChange,
                        modifier = Modifier.fillMaxWidth().semantics { contentDescription = "评论内容" }.focusRequester(focus)
                            .onFocusChanged { if (it.isFocused) panel = CommentPanel.Keyboard },
                        readOnly = locked,
                        textStyle = XType.Body.copy(color = colors.onSurface),
                        cursorBrush = SolidColor(Brand),
                        minLines = if (expanded) 7 else 2,
                        maxLines = if (expanded) 12 else 4,
                        decorationBox = { input ->
                            Box(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(extra.fill).padding(12.dp)) {
                                if (value.text.isEmpty()) Text("说点什么…", style = XType.Body, color = colors.onSurfaceVariant)
                                input()
                            }
                        },
                    )
                    preview?.invoke()
                    if (panel == CommentPanel.More) more?.invoke()
                } else {
                    loading()
                }
                error?.let { Text(it, style = XType.Secondary, color = Danger) }
                if (length > limit || (editorReady && expanded)) {
                    Text("$length / $limit", Modifier.align(Alignment.End), style = XType.Caption,
                        color = if (length > limit) Danger else colors.onSurfaceVariant)
                }
            }
            if (!editorReady) {
                Spacer(Modifier.height(12.dp))
                return@BottomSheet
            }
            Row(
                Modifier.fillMaxWidth().padding(start = 8.dp, end = 16.dp, top = 4.dp, bottom = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                val emojiOpen = panel == CommentPanel.Emoji
                CommentTool(
                    if (emojiOpen) XIcons.Keyboard else XIcons.Smile,
                    if (emojiOpen) "切换到键盘" else "打开表情",
                    enabled = !locked,
                    active = emojiOpen,
                ) { switch(if (emojiOpen) CommentPanel.Keyboard else CommentPanel.Emoji) }
                if (canMention) CommentTool(XIcons.AtSign, "插入 @", enabled = !locked) {
                    val start = value.selection.min
                    val needsSpace = value.text.getOrNull(start - 1)?.let { it.isLetterOrDigit() || it == '_' } == true
                    onValueChange(value.insert(if (needsSpace) " @" else "@"))
                    switch(CommentPanel.Keyboard)
                }
                if (onPickImage != null) {
                    val galleryOpen = panel == CommentPanel.Gallery
                    CommentTool(
                        if (galleryOpen) XIcons.Keyboard else XIcons.Image,
                        if (galleryOpen) "切换到键盘" else "打开相册",
                        enabled = !locked,
                        active = galleryOpen,
                    ) {
                        switch(if (galleryOpen) CommentPanel.Keyboard else CommentPanel.Gallery)
                    }
                }
                if (more != null) CommentTool(XIcons.Plus, "更多评论选项", enabled = !locked, active = panel == CommentPanel.More) {
                    switch(if (panel == CommentPanel.More) CommentPanel.Keyboard else CommentPanel.More)
                }
                Spacer(Modifier.weight(1f))
                val ready = !locked && length <= limit && (length > 0 || hasImages)
                Box(
                    Modifier.heightIn(min = 44.dp).clip(CircleShape)
                        .background(if (ready) extra.brandSoft else extra.fill)
                        .clickable(enabled = ready, role = Role.Button, onClick = onSubmit)
                        .padding(horizontal = 16.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(busyLabel ?: submitLabel, style = XType.Secondary, fontWeight = FontWeight.SemiBold,
                        color = if (ready) extra.brandInk else colors.onSurfaceVariant, maxLines = 1)
                }
            }
            when {
                panel == CommentPanel.Emoji && !imeVisible -> {
                    Hairline()
                    Column(Modifier.fillMaxWidth().background(colors.background).heightIn(max = 184.dp)
                        .verticalScroll(rememberScrollState()).padding(horizontal = 8.dp, vertical = 4.dp)) {
                        Text("常用表情", Modifier.padding(start = 8.dp, top = 4.dp), style = XType.Caption,
                            color = colors.onSurfaceVariant)
                        CommentEmojis.chunked(7).forEach { row ->
                            EmojiRow(row, enabled = !locked) { onValueChange(value.insert(it)) }
                        }
                    }
                }
                panel == CommentPanel.Gallery && onPickImage != null && !imeVisible -> {
                    Hairline()
                    Box(Modifier.fillMaxWidth().background(colors.background).padding(16.dp)) {
                        GhostPillButton("从相册选择", onPickImage, Modifier.fillMaxWidth(), enabled = !locked, height = 44.dp)
                    }
                }
            }
        }
    }
}

@Composable
internal fun PlainCommentComposer(
    env: CommunityEnv,
    visible: Boolean,
    replyTo: NewsComment?,
    onDismiss: () -> Unit,
    onPublished: () -> Unit,
    publish: suspend (String, Long?, List<String>) -> Unit,
) {
    var value by rememberSaveable(replyTo?.id, stateSaver = TextFieldValue.Saver) { mutableStateOf(TextFieldValue()) }
    var images by remember(replyTo?.id) { mutableStateOf<List<CommentDraftImage>>(emptyList()) }
    var busy by remember { mutableStateOf(false) }
    var uploading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    if (!visible) return
    val locked = busy || uploading

    val editor: @Composable ((() -> Unit)?, @Composable () -> Unit) -> Unit = { pickImage, uploadDetails ->
        CommentComposer(
            value, { value = it; error = null },
            onDismiss = { if (!locked) { error = null; onDismiss() } },
            onSubmit = submit@{
                if (locked) return@submit
                val content = value.text.trim()
                val imageIds = images.map { it.image.id }
                busy = true
                error = null
                scope.launch {
                    try {
                        publish(content, replyTo?.id, imageIds)
                        value = TextFieldValue()
                        images = emptyList()
                        onPublished()
                    } catch (e: CancellationException) {
                        throw e
                    } catch (e: Exception) {
                        error = messageOf(e)
                    } finally {
                        busy = false
                    }
                }
            },
            limit = 2000,
            replyTo = replyTo?.author?.nickname,
            locked = locked,
            busyLabel = when {
                busy -> "发布中…"
                uploading -> "上传中…"
                else -> null
            },
            error = error,
            hasImages = images.isNotEmpty(),
            onPickImage = pickImage,
            preview = {
                CommentDraftImages(env, images, if (locked) null else { id -> images = images.filterNot { it.image.id == id } })
                uploadDetails()
            },
        )
    }
    if (env.platform?.forumPhotoPickerSupported == true) {
        ForumUploadControls(
            env, enabled = !busy,
            onImage = { image, name ->
                if (images.none { it.image.id == image.id }) images += CommentDraftImage(image, name)
                error = null
            },
            onBusy = { uploading = it },
            requireCommentImages = true,
            commentLayout = { pickImage, _, details -> editor(pickImage, details) },
        )
    } else editor(null) {}
}

internal data class CommentDraftImage(val image: ForumImage, val name: String)

@Composable
internal fun CommentDraftImages(env: CommunityEnv, images: List<CommentDraftImage>, remove: ((String) -> Unit)?) {
    if (images.isEmpty()) return
    LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(images, key = { it.image.id }) { item ->
            Box(Modifier.size(64.dp).clip(RoundedCornerShape(8.dp))) {
                AsyncImage(
                    env.forum.imageUrl(item.image.url), item.name, imageLoader = rememberImageLoader(),
                    modifier = Modifier.fillMaxSize(), contentScale = ContentScale.Crop,
                )
                if (remove != null) CircleOverlayButton(XIcons.Close, "移除${item.name}", { remove(item.image.id) },
                    Modifier.align(Alignment.TopEnd))
            }
        }
    }
}

@Composable
private fun CommentTool(
    icon: ImageVector,
    label: String,
    enabled: Boolean = true,
    active: Boolean = false,
    onClick: () -> Unit,
) {
    Box(
        Modifier.size(44.dp).clip(CircleShape)
            .semantics { if (active) selected = true }
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        XIcon(icon, size = 20.dp, description = label,
            tint = if (active) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun EmojiRow(emojis: List<Pair<String, String>>, enabled: Boolean, insert: (String) -> Unit) {
    Row(Modifier.fillMaxWidth()) {
        emojis.forEach { (emoji, name) ->
            Box(
                Modifier.weight(1f).height(44.dp).clip(RoundedCornerShape(8.dp))
                    .semantics { contentDescription = name }
                    .clickable(enabled = enabled, role = Role.Button) { insert(emoji) },
                contentAlignment = Alignment.Center,
            ) {
                Text(emoji, fontSize = 22.sp, maxLines = 1)
            }
        }
    }
}
