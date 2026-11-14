package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

/** 新帖与评论共用文件选择、上传和失败重试。 */
@Composable
internal fun ForumUploadControls(
    env: CommunityEnv,
    enabled: Boolean,
    onImage: (ForumImage, String) -> Unit,
    onBusy: (Boolean) -> Unit,
    attachments: List<ForumAttachment>? = null,
    onAttachments: (List<ForumAttachment>) -> Unit = {},
    /** Already-associated source assets may be unlinked, never deleted as pending uploads. */
    retainedAttachmentIds: Set<String> = emptySet(),
    compact: Boolean = false,
    trailing: (@Composable () -> Unit)? = null,
    requireCommentImages: Boolean = false,
    commentLayout: (@Composable (pickImage: (() -> Unit)?, addAttachment: (() -> Unit)?, details: @Composable () -> Unit) -> Unit)? = null,
) {
    var showLimits by remember { mutableStateOf(false) }
    val platform = env.platform
    val hasFiles = platform?.forumFilePickerSupported == true
    val hasImages = platform?.forumPhotoPickerSupported == true || (commentLayout == null && hasFiles)
    if (commentLayout == null && !hasImages) {
        Text(if (attachments == null) "当前设备请在网页版回复中插图。" else "当前设备请到网页版插图或添加附件。",
            Modifier.padding(vertical = 8.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
        if (compact) trailing?.invoke()
        return
    }
    val session by env.session.state.collectAsState()
    val currentAccount by rememberUpdatedState(session.accountId)
    val scope = rememberCoroutineScope()
    val loader = remember(env, session.accountId) {
        Loader(scope) { env.forum.uploadSession() }
    }
    LaunchedEffect(loader, commentLayout == null) {
        if (commentLayout == null) loader.reload()
    }
    val limitsState by loader.state.collectAsState()
    val loadedLimits = (limitsState as? Load.Ready)?.value?.limits
    var commentLimits by remember(env, session.accountId) { mutableStateOf<ForumUploadLimits?>(null) }
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var retryFile by remember(session.accountId) { mutableStateOf<ForumUploadFile?>(null) }
    var retryImage by remember { mutableStateOf(true) }

    fun start(image: Boolean, retry: ForumUploadFile? = null) {
        if (platform == null || busy || !enabled) return
        if (!image && attachments.orEmpty().size >= 5) { error = "一个帖子最多 5 个附件"; return }
        val account = session.accountId
        busy = true; onBusy(true); error = null
        scope.launch {
            var file: ForumUploadFile? = retry
            try {
                val limits = loadedLimits ?: commentLimits ?: env.forum.uploadSession().limits.also { commentLimits = it }
                if (currentAccount != account) throw ApiException(401, "账号已切换，请重新选择文件")
                if (image && requireCommentImages && !limits.commentImages) {
                    throw ApiException(400, "当前论坛服务尚未开放图片评论")
                }
                val limit = minOf(if (image) limits.imageMaxBytes else limits.uploadMaxBytes, FORUM_MEMORY_MAX_BYTES,
                    if (image) FORUM_IMAGE_MAX_BYTES else Long.MAX_VALUE)
                if (limit <= 0) throw ApiException(403, if (image) "当前账号暂不能上传图片" else "当前账号暂不能上传附件")
                file = file ?: (
                    if (image && (commentLayout != null || !hasFiles)) platform.pickForumPhoto(limit)
                    else platform.pickForumFile(image, limit)
                ) ?: return@launch
                if (currentAccount != account) throw ApiException(401, "账号已切换，请重新选择文件")
                if (image) {
                    val result = env.forum.uploadImage(file, limit)
                    if (currentAccount != account) throw ApiException(401, "账号已切换，请重新选择文件")
                    onImage(result, file.name)
                } else {
                    val result = env.forum.uploadAttachment(file, limit)
                    if (currentAccount != account) throw ApiException(401, "账号已切换，请重新选择文件")
                    onAttachments(attachments.orEmpty() + result)
                }
                retryFile = null
            } catch (e: CancellationException) { throw e }
            catch (e: Exception) {
                error = (e as? ApiException)?.message ?: "选择或上传失败，请重试"
                if (currentAccount == account) { retryFile = file; retryImage = image }
            } finally { busy = false; onBusy(false) }
        }
    }
    val details: @Composable () -> Unit = {
        attachments?.forEach { attachment ->
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Column(Modifier.weight(1f)) {
                    Text(attachment.name, style = XType.Secondary, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text("${formatSize(attachment.size)} · ${if (attachment.price == 0) "免费下载" else "${attachment.price} 积分"}", style = XType.Caption, color = LocalExtraColors.current.ink3)
                }
                GhostPillButton("移除", {
                    if (!busy && enabled) {
                        busy = true; onBusy(true); error = null
                        scope.launch {
                            try {
                                if (attachment.id !in retainedAttachmentIds) env.forum.deletePendingAttachment(attachment.id)
                                onAttachments(attachments.filterNot { it.id == attachment.id })
                            } catch (e: CancellationException) { throw e }
                            catch (e: Exception) { error = messageOf(e) }
                            finally { busy = false; onBusy(false) }
                        }
                    }
                }, enabled = !busy && enabled)
            }
        }
        error?.let { Text(it, style = XType.Secondary, color = Danger) }
        retryFile?.let { file ->
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GhostPillButton("重试上传", { start(retryImage, file) }, Modifier.weight(1f), enabled = enabled && !busy)
                GhostPillButton("取消上传", { retryFile = null; error = null }, Modifier.weight(1f), enabled = !busy)
            }
        }
    }
    val status: @Composable () -> Unit = {
        when (val state = limitsState) {
            Load.Loading -> Text("正在读取上传额度…", style = XType.Caption, color = LocalExtraColors.current.ink3)
            is Load.Failed -> Banner(state.message, warn = true, actionText = "重试", onAction = loader::reload)
            else -> Unit
        }
    }
    if (commentLayout != null) {
        val pickImage: (() -> Unit)? = if (hasImages) ({ start(true) }) else null
        val addFile: (() -> Unit)? = if (hasFiles) ({ start(false) }) else null
        commentLayout(pickImage, addFile, details)
    } else {
        Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
            status()
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                if (compact) {
                    EditorTool(XIcons.Image, if (busy) "上传中" else "图片", enabled && !busy && (loadedLimits?.imageMaxBytes ?: 0) > 0) { start(true) }
                    if (hasFiles && attachments != null) EditorTool(XIcons.File, "附件", enabled && !busy && loadedLimits != null && attachments.size < 5) { start(false) }
                    trailing?.invoke()
                    Spacer(Modifier.weight(1f))
                    IconTap(XIcons.Info, "上传说明", { showLimits = !showLimits }, tint = LocalExtraColors.current.ink3)
                } else {
                    GhostPillButton(if (busy) "处理中…" else "插图", { start(true) }, Modifier.weight(1f), enabled = enabled && !busy && (loadedLimits?.imageMaxBytes ?: 0) > 0)
                    if (hasFiles && attachments != null) GhostPillButton("添加附件", { start(false) }, Modifier.weight(1f), enabled = enabled && !busy && loadedLimits != null && attachments.size < 5)
                }
            }
            if (!compact || showLimits) loadedLimits?.let {
                Text(if (it.imageMaxBytes <= 0) "当前账号暂不能上传图片" else "支持 PNG、JPEG、GIF、WebP，图片不超过 ${minOf(it.imageMaxBytes, FORUM_IMAGE_MAX_BYTES) / 1024 / 1024} MB。",
                    style = XType.Caption, color = LocalExtraColors.current.ink3)
                if (hasFiles && attachments != null) Text("最多 5 个附件，每个不超过 ${minOf(it.uploadMaxBytes, FORUM_MEMORY_MAX_BYTES) / 1024 / 1024} MB。", style = XType.Caption, color = LocalExtraColors.current.ink3)
            }
            details()
        }
    }
}

/** Quiet inline actions for the writing toolbar. */
@Composable
internal fun EditorTool(icon: ImageVector, label: String, enabled: Boolean = true, selected: Boolean = false, onClick: () -> Unit) {
    val ink = if (selected) Brand else if (enabled) MaterialTheme.colorScheme.onSurface else LocalExtraColors.current.ink3
    Row(Modifier.clickable(enabled = enabled, onClick = onClick).heightIn(min = 44.dp).padding(horizontal = 6.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        XIcon(icon, size = 20.dp, tint = ink)
        Text(label, style = XType.Secondary, color = ink)
    }
}
