package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.*

@Composable
internal fun ReplyContentCards(env: CommunityEnv, refs: List<ResolvedWorkReference>, open: ((Route) -> Unit)?) {
    refs.take(3).forEach { ref ->
        val card = safeReferencedContent(ref)
        WhiteCard(Modifier.padding(top = 8.dp).fillMaxWidth().clickable(enabled = card != null && open != null) { card?.let { open?.invoke(contentCardRoute(ref.kind, it)) } }, padding = 12.dp) {
            if (card != null && ref.kind == "pack" && card.kind == "game") {
                Row(verticalAlignment = androidx.compose.ui.Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Cover(card.coverUrl, card.id, Modifier.size(64.dp))
                    Column(Modifier.weight(1f)) {
                        Text(card.title, style = XType.Body, fontWeight = FontWeight.Medium)
                        GameTrialButton(env, card.id, ref.version, open, Modifier.padding(top = 6.dp))
                    }
                }
            } else {
                Text(if (card == null) "引用内容已不可用" else card.title, style = XType.Body, fontWeight = FontWeight.Medium)
                if (card != null) {
                    Text(if (ref.kind == "scheme") "社区方案" else "${kindLabel(card.kind.orEmpty())}${ref.version?.let { " · v$it" }.orEmpty()}", style = XType.Caption, color = LocalExtraColors.current.ink3)
                    if (card.summary.isNotBlank()) Text(card.summary, Modifier.padding(top = 4.dp), style = XType.Caption, maxLines = 2, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}

/** New-reply uploads are the only pending assets this composer may delete. */
@Composable
internal fun NativeReplyComposer(env: CommunityEnv, topicId: Long, onDismiss: () -> Unit, onSent: () -> Unit, panelModifier: Modifier, editingPost: Post? = null, dismissRequest: Int = 0) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val account = remember { session.accountId }
    val currentAccount by rememberUpdatedState(session.accountId)
    var original by remember { mutableStateOf<ReplySource?>(null) }
    var sourceError by remember { mutableStateOf<String?>(null) }
    var sourceAttempt by remember { mutableStateOf(0) }
    val retainedIds = original?.attachments.orEmpty().map { it.id }.toSet()
    val loadingSource = editingPost != null && original == null
    var text by remember { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var uploading by remember { mutableStateOf(false) }
    var resolving by remember { mutableStateOf(false) }
    var leaving by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var attachments by remember { mutableStateOf<List<ForumAttachment>>(emptyList()) }
    var selected by remember { mutableStateOf<List<SelectedWorkReference>>(emptyList()) }
    var link by remember { mutableStateOf("") }
    var showMine by remember { mutableStateOf(false) }
    LaunchedEffect(editingPost?.id, sourceAttempt) {
        if (editingPost == null) return@LaunchedEffect
        sourceError = null
        try {
            val source = env.forum.replySource(editingPost.id)
            if (currentAccount != account) throw ApiException(401, "账号已切换，请重新打开编辑")
            text = source.body
            attachments = source.attachments
            selected = replyEditSelection(source, editingPost.workRefs)
            original = source
        } catch (e: CancellationException) { throw e } catch (e: Exception) { sourceError = messageOf(e) }
    }
    LaunchedEffect(currentAccount) { if (currentAccount != account) onDismiss() }
    val refsApi = remember(env) { ReplyReferencesApi(env.client, env.config) }
    val locked = busy || uploading || resolving || leaving || loadingSource
    fun dismiss() {
        if (busy || uploading || resolving || leaving) { error = "请等待当前操作完成"; return }
        leaving = true
        val pending = attachments.filterNot { it.id in retainedIds }
        env.scope.launch { if (currentAccount == account) pending.forEach { file -> runCatching { env.forum.deletePendingAttachment(file.id) } } }
        onDismiss()
    }
    LaunchedEffect(dismissRequest) { if (dismissRequest > 0) dismiss() }
    fun addLink(value: String) {
        if (locked) return
        if (selected.size >= 3) { error = "最多选择 3 个不同的作品或方案"; return }
        resolving = true; error = null
        scope.launch {
            try {
                val next = refsApi.fromUrl(value)
                if (currentAccount != account) throw ApiException(401, "账号已切换，请重新撰写回复")
                val all = selected + next
                validateWorkReferences(all.map { it.reference })
                selected = all; link = ""; showMine = false
            } catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
            finally { resolving = false }
        }
    }
    BottomSheet(::dismiss, panelModifier) {
        Column(Modifier.fillMaxWidth().heightIn(max = 680.dp).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(if (editingPost == null) "回复" else "编辑回复", style = XType.Title, fontWeight = FontWeight.SemiBold)
            if (loadingSource) {
                Text(sourceError ?: "正在读取回复原文…", style = XType.Body, color = if (sourceError == null) MaterialTheme.colorScheme.onSurface else Danger)
                if (sourceError != null) GhostPillButton("重试读取", { sourceAttempt++ }, Modifier.fillMaxWidth())
                GhostPillButton("取消", ::dismiss, Modifier.fillMaxWidth())
            } else {
            AreaBox(text, { text = it }, placeholder = "写下你的回复（至少 2 个字）…", minHeight = 120.dp, enabled = !locked, filled = true)
            ForumUploadControls(env, enabled = !busy && !resolving && !leaving, onImage = { text += it }, onBusy = { uploading = it }, attachments = attachments,
                onAttachments = { next -> attachments = next.map { if (it.id in retainedIds) it else it.copy(price = 0) } }, retainedAttachmentIds = retainedIds)
            Text("引用公开作品或方案（最多 3 个）", style = XType.Secondary, fontWeight = FontWeight.Medium)
            AreaBox(link, { link = it }, placeholder = "粘贴社区作品或方案链接", minHeight = 55.dp, enabled = !locked)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                GhostPillButton(if (resolving) "正在读取…" else "添加引用", { addLink(link) }, Modifier.weight(1f), enabled = !locked && link.isNotBlank() && selected.size < 3)
                GhostPillButton("我的作品", { showMine = !showMine }, Modifier.weight(1f), enabled = !locked && selected.size < 3)
            }
            if (showMine) {
                val loader = rememberLoader(env, "reply-my-packs-${session.accountId}") { env.content.myPacks() }
                val state by loader.state.collectAsState()
                when (val mine = state) {
                    is Load.Ready -> {
                        val packs = ownedChallengePacks(mine.value.items, session.accountId)
                        if (packs.isEmpty()) Text("还没有可引用的已上架作品。方案可粘贴公开链接引用。", style = XType.Caption)
                        Column(Modifier.heightIn(max = 180.dp).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            packs.forEach { pack -> GhostPillButton(pack.title, { addLink(env.forum.packUrl(pack.id)) }, Modifier.fillMaxWidth(), enabled = !locked) }
                        }
                    }
                    is Load.Failed -> Banner(mine.message, warn = true, actionText = "重试", onAction = loader::reload)
                    else -> Text("正在读取我的作品…", style = XType.Caption)
                }
            }
            selected.forEach { chosen -> Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Column(Modifier.weight(1f)) {
                    Text(chosen.card?.title ?: "${if (chosen.reference.kind == "pack") "作品" else "方案"} #${chosen.reference.id}（当前不可用）", style = XType.Body, maxLines = 2)
                    Text(if (chosen.reference.kind == "scheme") "方案" else chosen.reference.version?.let { "固定版本 v$it" } ?: "发表时固定为当前版本", style = XType.Caption, color = LocalExtraColors.current.ink3)
                }
                GhostPillButton("移除", { selected = selected.filterNot { it.reference == chosen.reference } }, enabled = !locked)
            } }
            error?.let { Text(it, style = XType.Secondary, color = Danger) }
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                GhostPillButton("取消", ::dismiss, Modifier.weight(1f), enabled = !locked)
                PillButton(if (busy) "发送中…" else if (uploading) "上传中…" else if (editingPost == null) "发送" else "保存修改", {
                    if (locked) return@PillButton
                    busy = true; error = null
                    scope.launch {
                        try {
                            if (currentAccount != account) throw ApiException(401, "账号已切换，请重新撰写回复")
                            if (editingPost == null) env.forum.reply(topicId, text.trim(), workRefs = selected.map { it.reference }, attachments = attachments)
                            else env.forum.editReply(editingPost.id, text.trim(), workRefs = selected.map { it.reference }, attachments = attachments)
                            onSent()
                        } catch (e: CancellationException) { throw e } catch (e: Exception) { error = messageOf(e) }
                        finally { busy = false }
                    }
                }, Modifier.weight(1f), enabled = !locked && session.loggedIn && currentAccount == account && text.trim().length >= 2)
            }
            }
        }
    }
}
