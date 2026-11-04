package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.layout.*
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ForumAttachment
import xyz.xiangban.xiaoban.common.community.formatDownloadSize
import xyz.xiangban.xiaoban.common.community.messageOf

@Composable
internal fun ForumAttachments(env: CommunityEnv, attachments: List<ForumAttachment>, open: ((Route) -> Unit)?) {
    if (attachments.isEmpty()) return
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val platform = env.platform
    var pending by remember(session.accountId) { mutableStateOf<ForumAttachment?>(null) }
    var busy by remember { mutableStateOf<String?>(null) }
    var unlocked by remember(session.accountId) { mutableStateOf(emptySet<String>()) }
    var message by remember { mutableStateOf<String?>(null) }

    fun download(file: ForumAttachment, purchase: Boolean) {
        if (busy != null || platform?.forumDownloadSupported != true) return
        busy = file.id
        scope.launch {
            try {
                if (purchase) {
                    val result = env.forum.purchaseAttachment(file)
                    check(result.purchased) { "附件未解锁" }
                    unlocked = unlocked + file.id
                }
                val bytes = env.forum.downloadAttachment(file)
                val filename = file.name.map { if (it in "/\\:*?\"<>|" || it.isISOControl()) '_' else it }
                    .joinToString("").trim().takeUnless { it.isBlank() || it == "." || it == ".." } ?: "附件"
                message = when (platform.saveForumDownload(filename, bytes)) {
                    CommunityPosterResult.Saved -> "已保存附件"
                    CommunityPosterResult.ChooserOpened -> "请在系统菜单中保存附件"
                    CommunityPosterResult.Cancelled -> if (purchase) "附件已解锁，可再次下载" else null
                    else -> "保存失败，请重试"
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                message = messageOf(e)
            } finally {
                busy = null
            }
        }
    }

    Column(Modifier.padding(top = 10.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text("附件", style = XType.Section)
        attachments.forEach { file ->
            val needsPurchase = file.price > 0 && !file.purchased && file.id !in unlocked
            Column(Modifier.fillMaxWidth().outlinedCard(10.dp).padding(10.dp)) {
                Text(file.name, style = XType.Body)
                Text(formatDownloadSize(file.size), style = XType.Caption)
                val label = when {
                    busy == file.id -> "下载中…"
                    platform?.forumDownloadSupported != true -> "当前平台无法保存附件"
                    needsPurchase -> "${file.price} 积分 · 购买并下载"
                    file.price > 0 -> "已解锁 · 下载"
                    else -> "免费下载"
                }
                GhostPillButton(label, {
                    if (file.price > 0 && !session.loggedIn) open?.invoke(Route.Login)
                    else if (needsPurchase) pending = file
                    else download(file, purchase = false)
                }, enabled = busy == null && platform?.forumDownloadSupported == true)
            }
        }
        message?.let { Text(it, style = XType.Caption) }
    }
    pending?.let { file ->
        AlertDialog(
            onDismissRequest = { pending = null },
            title = { Text("购买附件") },
            text = { Text("购买「${file.name}」需要 ${file.price} 积分，重复下载免费。") },
            confirmButton = {
                TextButton(onClick = { pending = null; download(file, purchase = true) }) { Text("购买并下载") }
            },
            dismissButton = { TextButton(onClick = { pending = null }) { Text("取消") } },
        )
    }
}
