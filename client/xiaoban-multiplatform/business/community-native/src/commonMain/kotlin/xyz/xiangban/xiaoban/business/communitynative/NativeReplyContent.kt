package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
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

/** New-reply attachments are the only pending assets this composer may delete. */
@Composable
internal fun NativeReplyComposer(
    env: CommunityEnv,
    topicId: Long,
    onDismiss: () -> Unit,
    onSent: () -> Unit,
    parent: Post? = null,
    editingPost: Post? = null,
) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val account = remember { session.accountId }
    val currentAccount by rememberUpdatedState(session.accountId)
    var original by remember { mutableStateOf<ReplySource?>(null) }
    var sourceError by remember { mutableStateOf<String?>(null) }
    var sourceAttempt by remember { mutableStateOf(0) }
    val retainedIds = original?.attachments.orEmpty().map { it.id }.toSet()
    val loadingSource = editingPost != null && original == null
    var text by remember { mutableStateOf(TextFieldValue()) }
    var busy by remember { mutableStateOf(false) }
    var uploading by remember { mutableStateOf(false) }
    var resolving by remember { mutableStateOf(false) }
    var leaving by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var attachments by remember { mutableStateOf<List<ForumAttachment>>(emptyList()) }
    var selected by remember { mutableStateOf<List<SelectedWorkReference>>(emptyList()) }
    var images by remember { mutableStateOf<List<CommentDraftImage>>(emptyList()) }
    LaunchedEffect(editingPost?.id, sourceAttempt) {
        if (editingPost == null) return@LaunchedEffect
        sourceError = null
        try {
            val source = env.forum.replySource(editingPost.id)
            if (currentAccount != account) throw ApiException(401, "账号已切换，请重新打开编辑")
            text = TextFieldValue(source.body, TextRange(source.body.length))
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
    val body = remember(text.text, images) { (text.text + images.joinToString("") { forumImageMarkdown(it.image, it.name) }).trim() }
    val length = remember(body) { commentLength(body) }
    fun submit() {
        if (locked || !session.loggedIn || currentAccount != account || length !in 1..10_000) return
        busy = true
        error = null
        scope.launch {
            try {
                val refs = selected.map { it.reference }
                if (editingPost == null) env.forum.reply(topicId, body, replyTo = parent?.id, workRefs = refs, attachments = attachments)
                else env.forum.editReply(editingPost.id, body, workRefs = refs, attachments = attachments)
                onSent()
            } catch (e: CancellationException) { throw e }
            catch (e: Exception) { error = messageOf(e) }
            finally { busy = false }
        }
    }
    val editor: @Composable ((() -> Unit)?, (() -> Unit)?, @Composable () -> Unit) -> Unit = { pickImage, addAttachment, uploadDetails ->
        CommentComposer(
            text, { text = it; error = null }, ::dismiss, ::submit,
            limit = 10_000, replyTo = parent?.author?.nickname,
            title = if (editingPost == null) "写评论" else "编辑评论",
            submitLabel = if (editingPost == null) "发布" else "保存",
            busyLabel = when {
                busy -> if (editingPost == null) "发布中…" else "保存中…"
                uploading -> "上传中…"
                else -> null
            },
            length = length, locked = locked || !session.loggedIn || currentAccount != account,
            editorReady = !loadingSource, error = error, canMention = true, onPickImage = pickImage,
            preview = {
                CommentDraftImages(env, images, if (locked) null else { id -> images = images.filterNot { it.image.id == id } })
                uploadDetails()
                selected.forEach { chosen ->
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Column(Modifier.weight(1f)) {
                            Text(chosen.card?.title ?: "引用内容 #${chosen.reference.id}", style = XType.Secondary, maxLines = 2)
                            Text(if (chosen.reference.kind == "scheme") "方案" else chosen.reference.version?.let { "固定版本 v$it" } ?: "发表时固定为当前版本",
                                style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        if (!locked) IconTap(XIcons.Close, "移除引用", { selected = selected.filterNot { it.reference == chosen.reference } }, size = 16.dp)
                    }
                }
            },
            more = {
                if (addAttachment != null) GhostPillButton("添加附件", addAttachment, Modifier.fillMaxWidth(),
                    enabled = !locked && attachments.size < 5, height = 44.dp)
                ReplyReferenceOptions(env, session.accountId, selected, locked, resolving, resolve = { url ->
                    if (!locked && selected.size < 3) {
                        resolving = true
                        error = null
                        scope.launch {
                            try {
                                val next = refsApi.fromUrl(url)
                                if (currentAccount != account) throw ApiException(401, "账号已切换，请重新撰写回复")
                                val all = selected + next
                                validateWorkReferences(all.map { it.reference })
                                selected = all
                            } catch (e: CancellationException) { throw e }
                            catch (e: Exception) { error = messageOf(e) }
                            finally { resolving = false }
                        }
                    }
                })
            },
            loading = {
                if (sourceError == null) Spinner(Modifier.fillMaxWidth().height(60.dp))
                else Banner(sourceError.orEmpty(), warn = true, actionText = "重试读取", onAction = { sourceAttempt++ })
            },
        )
    }
    if (!loadingSource && (env.platform?.forumPhotoPickerSupported == true ||
            env.platform?.forumFilePickerSupported == true || attachments.isNotEmpty())) {
        ForumUploadControls(
            env, enabled = !busy && !resolving && !leaving,
            onImage = { image, name ->
                if (images.none { it.image.id == image.id }) images += CommentDraftImage(image, name)
                error = null
            },
            onBusy = { uploading = it },
            attachments = attachments,
            onAttachments = { next -> attachments = next.map { if (it.id in retainedIds) it else it.copy(price = 0) } },
            retainedAttachmentIds = retainedIds,
            commentLayout = editor,
        )
    } else editor(null, null) {}
}

@Composable
private fun ReplyReferenceOptions(
    env: CommunityEnv,
    accountId: String?,
    selected: List<SelectedWorkReference>,
    locked: Boolean,
    resolving: Boolean,
    resolve: (String) -> Unit,
) {
    var link by remember { mutableStateOf("") }
    var showMine by remember { mutableStateOf(false) }
    LaunchedEffect(selected.size) {
        link = ""
        showMine = false
    }
    Text("引用作品或方案", style = XType.Secondary, fontWeight = FontWeight.Medium)
    AreaBox(link, { link = it }, placeholder = "粘贴社区作品或方案链接", minHeight = 48.dp, enabled = !locked, filled = true)
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        GhostPillButton(if (resolving) "正在读取…" else "添加引用", { resolve(link) }, Modifier.weight(1f),
            enabled = !locked && link.isNotBlank() && selected.size < 3, height = 40.dp)
        GhostPillButton("我的作品", { showMine = !showMine }, Modifier.weight(1f),
            enabled = !locked && selected.size < 3, height = 40.dp)
    }
    if (showMine) {
        val loader = rememberLoader(env, "reply-my-packs-$accountId") { env.content.myPacks() }
        val state by loader.state.collectAsState()
        when (val mine = state) {
            is Load.Ready -> {
                val packs = ownedChallengePacks(mine.value.items, accountId)
                if (packs.isEmpty()) Text("还没有可引用的已上架作品。", style = XType.Caption)
                packs.forEach { pack -> GhostPillButton(pack.title, { resolve(env.forum.packUrl(pack.id)) },
                    Modifier.fillMaxWidth(), enabled = !locked, height = 40.dp) }
            }
            is Load.Failed -> Banner(mine.message, warn = true, actionText = "重试", onAction = loader::reload)
            else -> Text("正在读取我的作品…", style = XType.Caption)
        }
    }
}
