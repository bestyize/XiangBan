package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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

/** 新帖和回复共用上传状态；只在上传成功后一次性回填站内 Markdown。 */
@Composable
internal fun ForumUploadControls(
    env: CommunityEnv,
    enabled: Boolean,
    onImage: (String) -> Unit,
    onBusy: (Boolean) -> Unit,
    attachments: List<ForumAttachment>? = null,
    onAttachments: (List<ForumAttachment>) -> Unit = {},
) {
    val platform = env.platform
    if (platform?.forumFilePickerSupported != true) {
        Text(if (attachments == null) "当前设备请在网页版回复中插图。" else "当前设备请到网页版插图或添加附件。",
            Modifier.padding(vertical = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        return
    }
    val session by env.session.state.collectAsState()
    val currentAccount by rememberUpdatedState(session.accountId)
    val loader = rememberLoader(env, "upload-limits-${session.accountId}") { env.forum.uploadSession() }
    val limitsState by loader.state.collectAsState()
    val limits = (limitsState as? Load.Ready)?.value?.limits
    val scope = rememberCoroutineScope()
    var busy by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var retryFile by remember(session.accountId) { mutableStateOf<ForumUploadFile?>(null) }
    var retryImage by remember { mutableStateOf(true) }

    fun start(image: Boolean, retry: ForumUploadFile? = null) {
        if (busy || !enabled || limits == null) return
        if (!image && attachments.orEmpty().size >= 5) { error = "一个帖子最多 5 个附件"; return }
        val limit = minOf(if (image) limits.imageMaxBytes else limits.uploadMaxBytes, FORUM_MEMORY_MAX_BYTES,
            if (image) FORUM_IMAGE_MAX_BYTES else Long.MAX_VALUE)
        val account = session.accountId
        busy = true; onBusy(true); error = null
        scope.launch {
            var file: ForumUploadFile? = retry
            try {
                file = file ?: platform.pickForumFile(image, limit) ?: return@launch
                if (currentAccount != account) throw ApiException(401, "账号已切换，请重新选择文件")
                if (image) {
                    val result = env.forum.uploadImage(file, limit)
                    if (currentAccount != account) throw ApiException(401, "账号已切换，请重新选择文件")
                    onImage(forumImageMarkdown(result, file.name))
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
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        when (val state = limitsState) {
            Load.Loading -> Text("正在读取上传额度…", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            is Load.Failed -> Banner(state.message, warn = true, actionText = "重试", onAction = loader::reload)
            else -> Unit
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            GhostPillButton(if (busy) "处理中…" else "插图", { start(true) }, Modifier.weight(1f), enabled = enabled && !busy && (limits?.imageMaxBytes ?: 0) > 0)
            if (attachments != null) GhostPillButton("添加附件", { start(false) }, Modifier.weight(1f), enabled = enabled && !busy && limits != null && attachments.size < 5)
        }
        limits?.let {
            Text(if (it.imageMaxBytes <= 0) "升级为成员后可插图。" else "支持 PNG、JPEG、GIF、WebP，图片不超过 ${minOf(it.imageMaxBytes, FORUM_IMAGE_MAX_BYTES) / 1024 / 1024} MB。",
                fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
            if (attachments != null) Text("最多 5 个附件，每个不超过 ${minOf(it.uploadMaxBytes, FORUM_MEMORY_MAX_BYTES) / 1024 / 1024} MB。", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        }
        attachments?.forEach { attachment ->
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Column(Modifier.weight(1f)) {
                    Text(attachment.name, fontSize = 13.sp)
                    Text("${attachment.size} 字节 · 免费下载", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                }
                GhostPillButton("移除", {
                    if (!busy && enabled) {
                        busy = true; onBusy(true); error = null
                        scope.launch {
                            try {
                                env.forum.deletePendingAttachment(attachment.id)
                                onAttachments(attachments.filterNot { it.id == attachment.id })
                            } catch (e: CancellationException) { throw e }
                            catch (e: Exception) { error = messageOf(e) }
                            finally { busy = false; onBusy(false) }
                        }
                    }
                }, enabled = !busy && enabled)
            }
        }
        error?.let { Text(it, fontSize = 13.sp, color = Danger) }
        retryFile?.let { file ->
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GhostPillButton("重试上传", { start(retryImage, file) }, Modifier.weight(1f), enabled = enabled && !busy)
                GhostPillButton("取消上传", { retryFile = null; error = null }, Modifier.weight(1f), enabled = !busy)
            }
        }
    }
}
