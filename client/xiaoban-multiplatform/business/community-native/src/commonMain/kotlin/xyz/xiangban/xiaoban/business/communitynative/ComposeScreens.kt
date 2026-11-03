package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.layout.*
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.TextStyle
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString
import xyz.xiangban.xiaoban.common.community.CommunityJson
import xyz.xiangban.xiaoban.common.community.ForumAttachment
import xyz.xiangban.xiaoban.common.community.ForumImage
import xyz.xiangban.xiaoban.common.community.FORUM_IMAGE_MAX_BYTES
import xyz.xiangban.xiaoban.common.community.forumImageMarkdown
import xyz.xiangban.xiaoban.common.community.Category
import xyz.xiangban.xiaoban.common.community.messageOf
import xyz.xiangban.xiaoban.common.community.parsePollOptions
import xyz.xiangban.xiaoban.common.community.pollDraftError

/**
 * 发帖：选一个具体的分类，写标题和正文，发布。发成功后直接打开这个帖子。
 * 图片上传成功后回填站内 Markdown；附件通过既有 attachment_ids 挂到帖子上。
 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun NewPostScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit, help: Boolean = false, initialTitle: String = "", initialBody: String = "") {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var categories by remember { mutableStateOf<List<Category>?>(null) }
    var loadError by remember { mutableStateOf<String?>(null) }
    var parentId by rememberSaveable { mutableStateOf(-1) }
    var categoryId by rememberSaveable { mutableStateOf(-1) }
    var title by rememberSaveable { mutableStateOf(initialTitle) }
    var body by rememberSaveable { mutableStateOf(initialBody) }
    var topicTags by rememberSaveable { mutableStateOf("") }
    var busy by remember { mutableStateOf(false) }
    var uploading by remember { mutableStateOf(false) }
    var leaving by remember { mutableStateOf(false) }
    var cover by remember(session.accountId) { mutableStateOf<ForumCoverImage?>(null) }
    var coverSpec by remember(session.accountId) { mutableStateOf<ForumCoverSpec?>(null) }
    var uploadedCover by remember(cover, session.accountId) { mutableStateOf<ForumImage?>(null) }
    var choosingCover by remember { mutableStateOf(false) }
    var editingCover by remember { mutableStateOf(false) }
    val currentAccount by rememberUpdatedState(session.accountId)
    var attachmentsJson by rememberSaveable { mutableStateOf("[]") }
    val attachments = remember(attachmentsJson) { CommunityJson.decodeFromString<List<ForumAttachment>>(attachmentsJson) }
    fun cancelDraft() {
        if (leaving) return
        leaving = true
        // 只撤销本次草稿上传的 pending ID，服务端再次保证关联到帖子后不可删除。
        val pending = attachments.toList()
        env.scope.launch { pending.forEach { attachment -> runCatching { env.forum.deletePendingAttachment(attachment.id) } } }
        back()
    }
    var choosingCategory by remember { mutableStateOf(false) }
    var confirmingLeave by remember { mutableStateOf(false) }
    val focus = LocalFocusManager.current
    fun requestBack() {
        if (busy || uploading || leaving) return
        focus.clearFocus()
        if (title.isNotBlank() || body.isNotBlank() || topicTags.isNotBlank() || attachments.isNotEmpty() || cover != null) confirmingLeave = true else cancelDraft()
    }
    BackHandler { requestBack() }
    var error by remember { mutableStateOf<String?>(null) }
    var attempt by remember { mutableStateOf(0) }
    var bounty by rememberSaveable { mutableStateOf(0) }
    var bountyDays by rememberSaveable { mutableStateOf(7) }
    var pollEnabled by rememberSaveable { mutableStateOf(false) }
    var pollText by rememberSaveable { mutableStateOf("") }
    var pollDays by rememberSaveable { mutableStateOf(7) }

    LaunchedEffect(attempt) {
        loadError = null
        try {
            val all = env.forum.categories()
            categories = all
            val helpBoard = all.firstOrNull { it.slug in setOf("share-help", "resource-help") }
            if (help && helpBoard != null && categoryId < 0) {
                parentId = helpBoard.parentId ?: parentId
                categoryId = helpBoard.id
            } else if (parentId < 0) {
                val firstParent = all.firstOrNull { it.parentId == null && !it.adminOnly && all.any { c -> c.parentId == it.id } }
                if (firstParent != null) parentId = firstParent.id
            }
        } catch (e: kotlinx.coroutines.CancellationException) {
            throw e
        } catch (e: Exception) {
            loadError = messageOf(e)
        }
    }

    val all = categories.orEmpty()
    val parents = all.filter { it.parentId == null && !it.adminOnly && all.any { c -> c.parentId == it.id } }
    val children = all.filter { it.parentId == parentId && !it.adminOnly }
    val chosen = all.firstOrNull { it.id == categoryId }
    val isHelp = chosen?.slug in setOf("share-help", "resource-help")
    val isCasual = chosen?.let { category -> all.any { it.id == category.parentId && it.slug == "casual" } } == true
    val pollOptions = parsePollOptions(pollText)
    val pollError = if (pollEnabled) pollDraftError(pollOptions, pollDays.takeIf { it != 0 }) else null
    val titleLength = title.trim().split(Regex("\\s+")).joinToString(" ").count { !it.isLowSurrogate() }
    val bodyLength = body.trim().count { !it.isLowSurrogate() }
    val validTitle = if (isCasual) titleLength <= 80 else titleLength in 4..80
    val canPost = session.loggedIn && !busy && !uploading && !leaving && chosen != null && validTitle && bodyLength in 10..20_000 && pollError == null
    fun publish() {
        if (!canPost) return
        focus.clearFocus(); busy = true; error = null
        scope.launch {
            try {
                val account = session.accountId
                val selectedCover = cover
                if (selectedCover != null && uploadedCover == null) {
                    val limits = env.forum.uploadSession().limits
                    check(currentAccount == account) { "账号已切换，请重新编辑" }
                    uploadedCover = env.forum.uploadImage(selectedCover.file, minOf(limits.imageMaxBytes, FORUM_IMAGE_MAX_BYTES))
                }
                check(currentAccount == account) { "账号已切换，请重新编辑" }
                val postBody = uploadedCover?.let { forumImageMarkdown(it, "封面").trim() + "\n\n" + body.trim() } ?: body.trim()
                require(postBody.count { !it.isLowSurrogate() } <= 20_000) { "加上封面后正文超过字数上限，请精简正文" }
                val tags = topicTags.split(',', '，').map { it.trim().removePrefix("#").trim() }
                    .filter { it.isNotEmpty() }.map { it.take(20) }.distinctBy { it.lowercase() }.take(5)
                val id = env.forum.createTopic(categoryId, title.trim(), postBody, tags = tags, bounty = if (isHelp) bounty else null, bountyDays = bountyDays,
                    pollOptions = if (pollEnabled) pollOptions else emptyList(), pollDays = pollDays.takeIf { pollEnabled && it != 0 }, attachments = attachments)
                back(); open(Route.Topic(id))
            } catch (e: kotlinx.coroutines.CancellationException) { throw e }
            catch (e: Exception) { error = messageOf(e) }
            finally { busy = false }
        }
    }

    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        Column(Modifier.fillMaxSize().imePadding()) {
            ScreenBar(if (help) "发求助" else "发帖", onBack = ::requestBack, trailing = {
                if (session.loggedIn) PillButton(if (busy) "发布中" else "发布", ::publish,
                    Modifier.padding(end = 8.dp), enabled = canPost, height = 36.dp)
            })
            if (!session.loggedIn) {
                CenterMessage("登录后分享你的想法", "登录", { open(Route.Login) }, Modifier.weight(1f).fillMaxWidth())
            } else {
                Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = 20.dp)) {
                    Row(Modifier.padding(top = 8.dp, bottom = 20.dp).clip(CircleShape).background(LocalExtraColors.current.fill)
                        .clickable(enabled = !busy && !uploading) { focus.clearFocus(); choosingCategory = true }
                        .padding(horizontal = 12.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        XIcon(XIcons.Hash, size = 16.dp, tint = if (chosen != null) Brand else LocalExtraColors.current.ink3)
                        Text(chosen?.name ?: "选择发布版块", style = XType.Secondary, fontWeight = FontWeight.Medium)
                        XIcon(XIcons.ChevronDown, size = 14.dp, tint = LocalExtraColors.current.ink3)
                    }
                    OutlinedTextField(
                        value = topicTags, onValueChange = { if (it.length <= 110) topicTags = it },
                        modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
                        enabled = !busy, singleLine = true, label = { Text("话题标签（可选）") },
                        placeholder = { Text("用逗号分隔，最多 5 个") },
                    )
                    if (env.platform?.forumCoverSupported == true) ForumCoverEntry(cover, !busy && !uploading,
                        onChoose = { focus.clearFocus(); choosingCover = true }, onRemove = { cover = null; coverSpec = null })
                    BasicTextField(title, { title = it }, Modifier.fillMaxWidth().padding(vertical = 8.dp), enabled = !busy,
                        textStyle = XType.Headline.copy(fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface),
                        cursorBrush = SolidColor(Brand), decorationBox = { inner ->
                            Box { if (title.isEmpty()) Text(if (isCasual) "添加标题（可选）" else "添加标题", style = XType.Headline, fontWeight = FontWeight.SemiBold, color = LocalExtraColors.current.ink3); inner() }
                        })
                    if (titleLength > 70) Text("$titleLength / 80", Modifier.align(Alignment.End), style = XType.Caption, color = if (titleLength > 80) Danger else LocalExtraColors.current.ink3)
                    Hairline(Modifier.padding(top = 12.dp, bottom = 20.dp))
                    BasicTextField(body, { body = it }, Modifier.fillMaxWidth().heightIn(min = 240.dp), enabled = !busy,
                        textStyle = XType.Section.copy(color = MaterialTheme.colorScheme.onSurface),
                        cursorBrush = SolidColor(Brand), decorationBox = { inner ->
                            Box { if (body.isEmpty()) Text(if (isHelp) "描述你遇到的问题，以及已经尝试的方法…" else "分享你的想法、经验或新发现…", style = XType.Section, color = LocalExtraColors.current.ink3); inner() }
                        })
                    if (pollEnabled) {
                        Hairline(Modifier.padding(vertical = 16.dp))
                        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                            Text("投票", Modifier.weight(1f), style = XType.Body, fontWeight = FontWeight.SemiBold)
                            IconTap(XIcons.Close, "移除投票", { if (!busy) pollEnabled = false })
                        }
                        AreaBox(pollText, { pollText = it }, placeholder = "每行一个选项，填写 2～6 项", minHeight = 108.dp, enabled = !busy, filled = true)
                        Text("截止时间", Modifier.padding(top = 12.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                        ChipRow(listOf(0, 1, 3, 7, 14, 30).map { it to if (it == 0) "不限时" else "$it 天" }, pollDays) { if (!busy) pollDays = it }
                        Text(pollError ?: "发布后不能修改选项", Modifier.padding(vertical = 10.dp), style = XType.Caption, color = if (pollError != null) Danger else LocalExtraColors.current.ink3)
                    }
                    if (isHelp) {
                        Hairline(Modifier.padding(vertical = 16.dp))
                        Text("悬赏积分", style = XType.Body, fontWeight = FontWeight.SemiBold)
                        ChipRow(listOf(0, 10, 30, 50, 100).map { it to if (it == 0) "不悬赏" else "$it 分" }, bounty) { if (!busy) bounty = it }
                        if (bounty > 0) {
                            Text("有效期", Modifier.padding(top = 12.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                            ChipRow(listOf(7, 14, 30).map { it to "$it 天" }, bountyDays) { if (!busy) bountyDays = it }
                            Text("发布时扣除，采纳后发给回答者；到期未采纳或关闭求助时退还。", Modifier.padding(top = 8.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                        }
                    }
                    if (chosen?.slug?.startsWith("share") == true) Text("请分享原创、开源或已获授权的资源。", Modifier.padding(top = 16.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                    Spacer(Modifier.height(24.dp))
                }
                error?.let { Banner(it, Modifier.padding(horizontal = 16.dp, vertical = 8.dp), warn = true) }
                Hairline()
                Column(Modifier.fillMaxWidth().heightIn(max = 220.dp).verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 8.dp)) {
                    ForumUploadControls(env, enabled = !busy && !uploading && !leaving, compact = true,
                        onImage = { if (!leaving) body += it }, onBusy = { uploading = it }, attachments = attachments,
                        onAttachments = { next ->
                            if (!leaving) attachmentsJson = CommunityJson.encodeToString(next)
                            else env.scope.launch { next.filterNot { file -> attachments.any { it.id == file.id } }.forEach { file -> runCatching { env.forum.deletePendingAttachment(file.id) } } }
                        }, trailing = {
                            EditorTool(XIcons.Poll, "投票", enabled = !busy && !uploading, selected = pollEnabled) { focus.clearFocus(); pollEnabled = !pollEnabled }
                        })
                    Text(when {
                        uploading -> "正在上传，请稍候…"
                        chosen == null -> "先选择发布版块"
                        !validTitle -> if (isCasual) "标题最多 80 个字" else "标题需要 4～80 个字"
                        bodyLength < 10 -> "正文至少 10 个字"
                        bodyLength > 20_000 -> "正文最多 20,000 个字"
                        else -> "$bodyLength 字"
                    }, Modifier.padding(top = 6.dp, bottom = 2.dp), style = XType.Caption, color = if (bodyLength > 20_000) Danger else LocalExtraColors.current.ink3)
                }
            }
        }
        OverlayTransition(choosingCategory) { panel ->
            BottomSheet({ choosingCategory = false }, panel) {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text("选择版块", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                    IconTap(XIcons.Close, "关闭分类", { choosingCategory = false })
                }
                when {
                    loadError != null -> Banner(loadError.orEmpty(), warn = true, actionText = "重试", onAction = { attempt++ })
                    categories == null -> Spinner(Modifier.fillMaxWidth().height(100.dp))
                    else -> Column(Modifier.heightIn(max = 420.dp).verticalScroll(rememberScrollState())) {
                        ChipRow(parents.map { it.id to it.name }, parentId) { parentId = it }
                        children.forEach { category ->
                            Row(Modifier.fillMaxWidth().clickable { categoryId = category.id; choosingCategory = false }.padding(vertical = 16.dp), verticalAlignment = Alignment.CenterVertically) {
                                Text(category.name, Modifier.weight(1f), style = XType.Body, color = if (categoryId == category.id) Brand else MaterialTheme.colorScheme.onSurface)
                                if (categoryId == category.id) XIcon(XIcons.Check, size = 18.dp, tint = Brand)
                            }
                        }
                        if (children.isEmpty()) Text("这个分类暂时没有可发布的版块", Modifier.padding(vertical = 20.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3)
                    }
                }
            }
        }
        OverlayTransition(choosingCover) { panel ->
            BottomSheet({ choosingCover = false }, panel) {
                Text("帖子封面", style = XType.Title, fontWeight = FontWeight.SemiBold)
                SheetAction(XIcons.Image, "从相册选择", "使用自己的图片") {
                    choosingCover = false
                    val platform = env.platform ?: return@SheetAction
                    val account = session.accountId
                    uploading = true; error = null
                    scope.launch {
                        try {
                            val file = platform.pickForumFile(true, FORUM_IMAGE_MAX_BYTES) ?: return@launch
                            val preview = platform.previewForumCover(file) ?: kotlin.error("无法读取封面")
                            check(currentAccount == account) { "账号已切换，请重新选择" }
                            cover = preview; coverSpec = null
                        } catch (e: kotlinx.coroutines.CancellationException) { throw e }
                        catch (e: Exception) { error = messageOf(e) }
                        finally { uploading = false }
                    }
                }
                SheetAction(XIcons.File, "文字封面", "选择模板，用文字做封面") { choosingCover = false; editingCover = true }
            }
        }
        if (editingCover) env.platform?.let { platform ->
            ForumCoverEditor(platform, coverSpec ?: ForumCoverSpec(title), onDismiss = { editingCover = false },
                onUse = { spec, image -> coverSpec = spec; cover = image; editingCover = false })
        }
        if (confirmingLeave) CenterDialog({ confirmingLeave = false }) {
            Text("放弃这次编辑？", style = XType.Title, fontWeight = FontWeight.SemiBold)
            Text("尚未发布的内容不会保留。", Modifier.padding(top = 8.dp, bottom = 20.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3)
            PillButton("继续编辑", { confirmingLeave = false }, Modifier.fillMaxWidth())
            GhostPillButton("放弃编辑", { confirmingLeave = false; cancelDraft() }, Modifier.fillMaxWidth().padding(top = 8.dp), textColor = Danger)
        }
    }
}

@Composable
private fun ChipRow(items: List<Pair<Int, String>>, selected: Int, onSelect: (Int) -> Unit) {
    LazyRow(Modifier.fillMaxWidth().padding(top = 8.dp), contentPadding = PaddingValues(vertical = 2.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items(items, key = { it.first }) { (id, name) ->
            PageChip(name, id == selected) { onSelect(id) }
        }
    }
}
