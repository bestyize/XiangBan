package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import kotlinx.serialization.encodeToString
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid
import xyz.xiangban.xiaoban.common.community.Category
import xyz.xiangban.xiaoban.common.community.CommunityJson
import xyz.xiangban.xiaoban.common.community.ContentPublishDraft
import xyz.xiangban.xiaoban.common.community.FORUM_IMAGE_MAX_BYTES
import xyz.xiangban.xiaoban.common.community.ForumAttachment
import xyz.xiangban.xiaoban.common.community.ForumCategories
import xyz.xiangban.xiaoban.common.community.ForumImage
import xyz.xiangban.xiaoban.common.community.ForumSection
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.PreparedForumWork
import xyz.xiangban.xiaoban.common.community.formatSize
import xyz.xiangban.xiaoban.common.community.forumImageMarkdown
import xyz.xiangban.xiaoban.common.community.isHelpBoard
import xyz.xiangban.xiaoban.common.community.isUncategorized
import xyz.xiangban.xiaoban.common.community.kindLabel
import xyz.xiangban.xiaoban.common.community.messageOf
import xyz.xiangban.xiaoban.common.community.ownedChallengePacks
import xyz.xiangban.xiaoban.common.community.pollDraftError
import xyz.xiangban.xiaoban.common.community.requiredPostLevel

private enum class PostingSection(val label: String) { Ideas("创意"), Discuss("讨论"), Resources("资源") }

private fun ForumSection.postingSection(): PostingSection = when (this) {
    ForumSection.Ideas -> PostingSection.Ideas
    ForumSection.Discuss, ForumSection.Site -> PostingSection.Discuss
    ForumSection.Resources -> PostingSection.Resources
}

private data class PostingBoardChoice(
    val key: String,
    val label: String,
    val category: Category,
    val topicTag: String? = null,
)

private val PostingSections = PostingSection.entries
private val PostingLevels = listOf("新人", "成员", "老用户", "守护者")

private sealed interface DraftPostWork {
    val title: String

    data class Linked(val id: Long, override val title: String, val kind: String) : DraftPostWork
    data class Uploaded(val prepared: PreparedForumWork, val rightsAccepted: Boolean) : DraftPostWork {
        override val title: String get() = prepared.title.ifBlank { prepared.originalName.substringBeforeLast('.') }
        val badge: String
            get() = when {
                prepared.originalName.endsWith(".htm", ignoreCase = true) -> "HTM → V2"
                prepared.originalName.endsWith(".html", ignoreCase = true) -> "HTML → V2"
                else -> "V2 容器"
            }
        val converted: Boolean
            get() = prepared.originalName.endsWith(".html", ignoreCase = true) ||
                prepared.originalName.endsWith(".htm", ignoreCase = true)
    }
}

private fun boardRuleLabel(catalog: ForumCategories, choice: PostingBoardChoice, trustLevel: Int): String? = when {
    !catalog.canPost(choice.category, trustLevel) ->
        if (catalog.postingEnabled(choice.category)) "需${PostingLevels[choice.category.requiredPostLevel]}" else "已关闭发帖"
    choice.topicTag != null -> "必带 #${choice.topicTag}"
    catalog.allowsShortTitle(choice.category) -> "可不写标题"
    choice.category.slug == "resource-movies" -> "需合法授权"
    else -> null
}

private fun cleanWorkLink(title: String, url: String): String {
    val label = title.filter { !it.isISOControl() && it !in "[]\\<>`*_!" }.trim().ifBlank { "查看作品" }
    return "[$label]($url)"
}

/**
 * 发帖：顶部栏无底线、单行元信息入口、无框写作区、内联话题胶囊、结构化浅灰功能卡、48dp 固定单行底栏。
 * 「作品」支持上传 V2 ZIP 或 .html/.htm 自动转为 V2 容器，或关联我的创作，每帖限 1 个。
 */
@OptIn(ExperimentalComposeUiApi::class, ExperimentalLayoutApi::class, ExperimentalUuidApi::class)
@Composable
fun NewPostScreen(
    env: CommunityEnv,
    back: () -> Unit,
    open: (Route) -> Unit,
    help: Boolean = false,
    initialTitle: String = "",
    initialBody: String = "",
    initialSection: ForumSection? = null,
    initialBoardSlug: String? = null,
) {
    val extra = LocalExtraColors.current
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    var categories by remember(session.accountId) { mutableStateOf<ForumCategories?>(null) }
    var loadError by remember { mutableStateOf<String?>(null) }
    val startingSection = initialSection?.postingSection() ?: if (help) PostingSection.Resources else PostingSection.Discuss
    var sectionIndex by rememberSaveable { mutableIntStateOf(PostingSections.indexOf(startingSection)) }
    var trustLevel by remember { mutableIntStateOf(0) }
    var selectedChoiceKey by rememberSaveable { mutableStateOf(initialBoardSlug ?: "") }
    var title by rememberSaveable { mutableStateOf(initialTitle) }
    var body by rememberSaveable { mutableStateOf(initialBody) }
    var tagsJson by rememberSaveable(session.accountId) { mutableStateOf("[]") }
    val userTags = remember(tagsJson) { CommunityJson.decodeFromString<List<String>>(tagsJson) }
    fun updateUserTags(next: List<String>) {
        tagsJson = CommunityJson.encodeToString(next.distinctBy { it.lowercase() }.take(5))
    }

    var busy by remember { mutableStateOf(false) }
    var topicRequestId by rememberSaveable(session.accountId) { mutableStateOf<String?>(null) }
    var publishStep by remember { mutableStateOf("") }
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

    var draftWork by remember(session.accountId) { mutableStateOf<DraftPostWork?>(null) }
    var uploadedWorkKey by remember(session.accountId) { mutableStateOf<String?>(null) }
    var choosingWork by remember { mutableStateOf(false) }
    var workTab by remember { mutableStateOf(0) } // 0: 上传成品, 1: 关联我的创作
    var workPicking by remember { mutableStateOf(false) }
    var workPickerError by remember { mutableStateOf<String?>(null) }

    var choosingCategory by remember { mutableStateOf(false) }
    var choosingTags by remember { mutableStateOf(false) }
    var tagDraft by remember { mutableStateOf("") }
    var confirmingLeave by remember { mutableStateOf(false) }

    var pollEnabled by rememberSaveable { mutableStateOf(false) }
    var pollOptionsJson by rememberSaveable(session.accountId) { mutableStateOf("""["",""]""") }
    val pollInputs = remember(pollOptionsJson) { CommunityJson.decodeFromString<List<String>>(pollOptionsJson) }
    fun updatePollInputs(next: List<String>) {
        pollOptionsJson = CommunityJson.encodeToString(next.take(6))
    }
    var pollDays by rememberSaveable { mutableStateOf(7) }

    val focus = LocalFocusManager.current
    fun cancelDraft() {
        if (leaving) return
        leaving = true
        val pending = attachments.toList()
        env.scope.launch { pending.forEach { attachment -> runCatching { env.forum.deletePendingAttachment(attachment.id) } } }
        back()
    }
    fun requestBack() {
        if (busy || uploading || leaving || workPicking) return
        focus.clearFocus()
        val dirty = title.isNotBlank() || body.isNotBlank() || userTags.isNotEmpty() || attachments.isNotEmpty() || draftWork != null || cover != null
        if (dirty) confirmingLeave = true else cancelDraft()
    }
    BackHandler { requestBack() }

    var error by remember { mutableStateOf<String?>(null) }
    var attempt by remember { mutableStateOf(0) }

    LaunchedEffect(attempt, session.accountId, session.loggedIn) {
        categories = null
        loadError = null
        try {
            if (session.loggedIn) {
                trustLevel = env.forum.uploadSession().user.trustLevel
                categories = ForumCategories(env.forum.categories())
            }
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            loadError = messageOf(e)
        }
    }

    val catalog = categories
    val destinations: Map<PostingSection, List<PostingBoardChoice>> = remember(catalog) {
        if (catalog == null) emptyMap()
        else {
            val discussBoards = catalog.discussionBoards
            val discussDefault = discussBoards.firstOrNull { it.isUncategorized } ?: discussBoards.firstOrNull()
            val discussChoices = catalog.discussionTabs.mapNotNull { tab ->
                val board = tab.category?.takeIf { it in catalog.boards } ?: discussDefault?.takeIf { tab.topicTag != null }
                board?.let { PostingBoardChoice(tab.key, tab.label, it, tab.topicTag) }
            }
            mapOf(
                PostingSection.Ideas to catalog.boardsIn(ForumSection.Ideas).map { PostingBoardChoice(it.slug, it.name, it) },
                PostingSection.Discuss to discussChoices,
                PostingSection.Resources to catalog.boardsIn(ForumSection.Resources)
                    .filterNot { it.isHelpBoard }
                    .map { PostingBoardChoice(it.slug, it.name, it) },
            ).filterValues { it.isNotEmpty() }
        }
    }

    val allChoices = remember(destinations) { destinations.values.flatten() }
    val selectedChoice = allChoices.firstOrNull { it.key == selectedChoiceKey && catalog?.canPost(it.category, trustLevel) == true }
    val activeSection = PostingSections.getOrNull(sectionIndex) ?: PostingSection.Discuss

    LaunchedEffect(catalog, trustLevel) {
        if (catalog != null && destinations.isNotEmpty()) {
            val currentValid = allChoices.firstOrNull { it.key == selectedChoiceKey && catalog.canPost(it.category, trustLevel) }
            if (currentValid == null) {
                val slugMatch = initialBoardSlug?.let { slug ->
                    allChoices.firstOrNull {
                        (it.key == slug || it.category.slug == slug || it.category.id == catalog.discussionTab(slug)?.category?.id) &&
                            catalog.canPost(it.category, trustLevel)
                    }
                }
                val fallbackList = destinations[activeSection] ?: destinations.values.firstOrNull().orEmpty()
                val preferred = slugMatch ?: fallbackList.firstOrNull { it.category.isUncategorized && catalog.canPost(it.category, trustLevel) }
                    ?: fallbackList.firstOrNull { catalog.canPost(it.category, trustLevel) }
                if (preferred != null) {
                    selectedChoiceKey = preferred.key
                    val prefSec = PostingSections.firstOrNull { sec -> destinations[sec]?.any { it.key == preferred.key } == true }
                    if (prefSec != null) sectionIndex = PostingSections.indexOf(prefSec)
                } else {
                    selectedChoiceKey = ""
                }
            }
        }
    }

    val requiredTag = selectedChoice?.topicTag
    val allTags = remember(userTags, requiredTag) {
        if (requiredTag != null) (listOf(requiredTag) + userTags.filterNot { it.equals(requiredTag, ignoreCase = true) }).take(5)
        else userTags.take(5)
    }

    val chosenBoard = selectedChoice?.category
    val isCasual = chosenBoard?.let { catalog?.allowsShortTitle(it) } == true
    val pollOptions = pollInputs.map { it.trim() }.filter { it.isNotEmpty() }
    val pollError = if (pollEnabled) {
        if (pollOptions.size < 2) "请至少填写 2 个有效选项"
        else pollDraftError(pollOptions, pollDays.takeIf { it != 0 })
    } else null
    val workError = (draftWork as? DraftPostWork.Uploaded)?.let { if (!it.rightsAccepted) "请确认作品原创或已获授权" else null }
    val titleLength = title.trim().split(Regex("\\s+")).joinToString(" ").count { !it.isLowSurrogate() }
    val bodyLength = body.trim().count { !it.isLowSurrogate() }
    val validTitle = if (isCasual) titleLength <= 80 else titleLength in 4..80
    val canPost = session.loggedIn && !busy && !uploading && !leaving && chosenBoard != null &&
        validTitle && bodyLength in 10..20_000 && pollError == null && workError == null

    fun addUserTag(raw: String) {
        val cleaned = raw.trim().removePrefix("#").trim().take(20)
        if (cleaned.isEmpty()) return
        if (requiredTag != null && cleaned.equals(requiredTag, ignoreCase = true)) return
        if (allTags.size >= 5 || allTags.any { it.equals(cleaned, ignoreCase = true) }) return
        updateUserTags(userTags + cleaned)
    }

    fun publish() {
        val board = chosenBoard ?: return
        if (!canPost) return
        focus.clearFocus()
        busy = true
        publishStep = ""
        error = null
        scope.launch {
            try {
                val account = session.accountId
                val selectedCover = cover
                if (selectedCover != null && uploadedCover == null) {
                    publishStep = "正在上传封面…"
                    val limits = env.forum.uploadSession().limits
                    check(currentAccount == account) { "账号已切换，请重新编辑" }
                    uploadedCover = env.forum.uploadImage(selectedCover.file, minOf(limits.imageMaxBytes, FORUM_IMAGE_MAX_BYTES))
                }
                check(currentAccount == account) { "账号已切换，请重新编辑" }

                var linkedWork: DraftPostWork.Linked? = draftWork as? DraftPostWork.Linked
                val pendingUpload = draftWork as? DraftPostWork.Uploaded
                if (pendingUpload != null) {
                    check(pendingUpload.rightsAccepted) { "请确认作品原创或已获授权" }
                    val packTitle = pendingUpload.title.trim().take(60).ifBlank { title.trim().take(60) }.ifBlank { "社区作品" }
                    val packSummary = pendingUpload.prepared.summary.trim().ifBlank { body.trim().take(500) }
                    val fileKey = uploadedWorkKey ?: run {
                        publishStep = "正在上传作品…"
                        env.content.upload(pendingUpload.prepared.kind, pendingUpload.prepared.file).fileKey.also { uploadedWorkKey = it }
                    }
                    check(currentAccount == account) { "账号已切换，请重新编辑" }
                    publishStep = "正在校验作品…"
                    check(env.content.awaitVerified(fileKey, recoverPublished = true)) { "作品校验尚未完成，请稍后重试" }
                    check(currentAccount == account) { "账号已切换，请重新编辑" }
                    publishStep = "正在登记作品…"
                    val result = env.content.publish(
                        ContentPublishDraft(pendingUpload.prepared.kind, packTitle, packSummary, allTags, rightsAck = true),
                        fileKey,
                    )
                    check(currentAccount == account) { "账号已切换，请到我的作品核对" }
                    val linked = DraftPostWork.Linked(result.id, packTitle, kindLabel(pendingUpload.prepared.kind))
                    draftWork = linked
                    uploadedWorkKey = null
                    linkedWork = linked
                }

                val postBody = buildList {
                    uploadedCover?.let { add(forumImageMarkdown(it, "封面").trim()) }
                    linkedWork?.let { add(cleanWorkLink(it.title, env.forum.packUrl(it.id))) }
                    add(body.trim())
                }.joinToString("\n\n")
                require(postBody.count { !it.isLowSurrogate() } <= 20_000) { "加上封面和作品链接后正文超过字数上限，请精简正文" }

                publishStep = "正在发布…"
                val requestId = topicRequestId ?: Uuid.random().toString().also { topicRequestId = it }
                val id = env.forum.createTopic(
                    categoryId = board.id,
                    title = title.trim(),
                    body = postBody,
                    tags = allTags,
                    bounty = null,
                    pollOptions = if (pollEnabled) pollOptions else emptyList(),
                    pollDays = pollDays.takeIf { pollEnabled && it != 0 },
                    attachments = attachments,
                    requestId = requestId,
                )
                back()
                open(Route.Topic(id))
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                error = messageOf(e)
            } finally {
                busy = false
                publishStep = ""
            }
        }
    }

    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)) {
        Column(Modifier.fillMaxSize().imePadding()) {
            // 1. 无底线顶栏：禁用发布改为浅灰底+次要文字
            ScreenBar("发帖", onBack = ::requestBack, trailing = {
                if (session.loggedIn) {
                    Box(
                        Modifier.padding(end = 10.dp).height(34.dp).clip(CircleShape)
                            .background(if (canPost) Brand else extra.fill)
                            .clickable(enabled = canPost) { publish() }
                            .padding(horizontal = 16.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            if (busy) (publishStep.ifEmpty { "发布中" }) else "发布",
                            color = if (canPost) Color.White else extra.ink3,
                            style = XType.Secondary,
                            fontWeight = FontWeight.SemiBold,
                            maxLines = 1,
                        )
                    }
                }
            })

            if (!session.loggedIn) {
                CenterMessage("登录后分享你的想法", "登录", { open(Route.Login) }, Modifier.weight(1f).fillMaxWidth())
            } else {
                ForumUploadControls(
                    env = env,
                    enabled = !busy && !uploading && !leaving,
                    onImage = { image, name -> if (!leaving) body += forumImageMarkdown(image, name) },
                    onBusy = { uploading = it },
                    attachments = attachments,
                    onAttachments = { next ->
                        if (!leaving) attachmentsJson = CommunityJson.encodeToString(next)
                        else env.scope.launch {
                            next.filterNot { file -> attachments.any { it.id == file.id } }
                                .forEach { file -> runCatching { env.forum.deletePendingAttachment(file.id) } }
                        }
                    },
                    composerLayout = { pickImage, addAttachment, removeAttachment, showLimits, toggleLimits, limitsSummary, uploadErrorBanner, toolsBusy ->
                        Column(Modifier.weight(1f).fillMaxWidth()) {
                            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = 18.dp)) {
                                // 2. 单行板块 + 封面元信息条
                                Row(
                                    Modifier.fillMaxWidth().padding(top = 6.dp, bottom = 14.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                                ) {
                                    Row(
                                        Modifier.weight(1f, fill = false).height(34.dp).clip(CircleShape)
                                            .background(extra.fill)
                                            .clickable(enabled = !busy && !uploading) {
                                                focus.clearFocus()
                                                choosingCategory = true
                                            }
                                            .padding(horizontal = 12.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(7.dp),
                                    ) {
                                        Box(Modifier.size(7.dp).clip(CircleShape).background(if (selectedChoice != null) Brand else extra.ink3))
                                        Text(
                                            selectedChoice?.let { "${activeSection.label} · ${it.label}" } ?: "选择发布板块",
                                            style = XType.Secondary,
                                            fontWeight = FontWeight.Medium,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                        XIcon(XIcons.ChevronDown, size = 14.dp, tint = extra.ink3)
                                    }
                                    Spacer(Modifier.weight(1f))
                                    if (env.platform?.forumCoverSupported == true) {
                                        val hasCover = cover != null
                                        Row(
                                            Modifier.height(34.dp).clip(CircleShape)
                                                .background(if (hasCover) extra.brandSoft else extra.fill)
                                                .clickable(enabled = !busy && !uploading) {
                                                    focus.clearFocus()
                                                    choosingCover = true
                                                }
                                                .padding(horizontal = 12.dp),
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(5.dp),
                                        ) {
                                            XIcon(XIcons.Image, size = 15.dp, tint = if (hasCover) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant)
                                            Text(
                                                if (hasCover) "已选封面" else "封面",
                                                style = XType.Secondary,
                                                fontWeight = if (hasCover) FontWeight.SemiBold else FontWeight.Medium,
                                                color = if (hasCover) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant,
                                            )
                                        }
                                    }
                                }

                                // 3. 紧凑封面缩略条（仅在已选封面时出现）
                                val currentCover = cover
                                if (currentCover != null) {
                                    Row(
                                        Modifier.fillMaxWidth().padding(bottom = 14.dp)
                                            .clip(RoundedCornerShape(14.dp)).background(extra.fill)
                                            .padding(horizontal = 12.dp, vertical = 10.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                                    ) {
                                        val thumbWidth = if (coverSpec?.portrait == false) 48.dp else 42.dp
                                        val thumbHeight = if (coverSpec?.portrait == false) 48.dp else 54.dp
                                        Image(
                                            currentCover.preview,
                                            "帖子封面",
                                            Modifier.size(thumbWidth, thumbHeight).clip(RoundedCornerShape(8.dp))
                                                .clickable(enabled = !busy && !uploading) {
                                                    if (coverSpec != null) editingCover = true else choosingCover = true
                                                },
                                            contentScale = ContentScale.Crop,
                                        )
                                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                                            Text(
                                                coverSpec?.template?.label ?: "相册封面",
                                                style = XType.Secondary,
                                                fontWeight = FontWeight.SemiBold,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                            )
                                            Text(
                                                when (val spec = coverSpec) {
                                                    null -> "相册图片 · 将作为首图展示"
                                                    else -> if (spec.portrait) "文字封面 · 竖版 3:4" else "文字封面 · 方形 1:1"
                                                },
                                                style = XType.Caption,
                                                color = extra.ink3,
                                            )
                                        }
                                        Box(
                                            Modifier.height(28.dp).clip(CircleShape).background(MaterialTheme.colorScheme.surface)
                                                .clickable(enabled = !busy && !uploading) { choosingCover = true }
                                                .padding(horizontal = 10.dp),
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            Text("更换", style = XType.Caption, fontWeight = FontWeight.Medium)
                                        }
                                        IconTap(XIcons.Close, "移除封面", {
                                            if (!busy && !uploading) { cover = null; coverSpec = null }
                                        }, size = 16.dp)
                                    }
                                }

                                // 4. 无框标题与正文
                                Row(Modifier.fillMaxWidth().padding(vertical = 4.dp), verticalAlignment = Alignment.CenterVertically) {
                                    BasicTextField(
                                        value = title,
                                        onValueChange = { title = it },
                                        modifier = Modifier.weight(1f),
                                        enabled = !busy,
                                        textStyle = XType.Title.copy(fontSize = 20.sp, lineHeight = 28.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface),
                                        cursorBrush = SolidColor(Brand),
                                        decorationBox = { inner ->
                                            Box {
                                                if (title.isEmpty()) Text(
                                                    if (isCasual) "添加标题（可选）" else "添加标题",
                                                    style = XType.Title.copy(fontSize = 20.sp, lineHeight = 28.sp, fontWeight = FontWeight.SemiBold),
                                                    color = extra.ink3,
                                                )
                                                inner()
                                            }
                                        },
                                    )
                                    if (titleLength > 60) {
                                        Text(
                                            "$titleLength/80",
                                            Modifier.padding(start = 8.dp),
                                            style = XType.Caption,
                                            color = if (titleLength > 80) Danger else extra.ink3,
                                        )
                                    }
                                }
                                Hairline(Modifier.padding(vertical = 10.dp))
                                BasicTextField(
                                    value = body,
                                    onValueChange = { body = it },
                                    modifier = Modifier.fillMaxWidth().heightIn(min = if (pollEnabled || draftWork != null) 120.dp else 200.dp),
                                    enabled = !busy,
                                    textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface),
                                    cursorBrush = SolidColor(Brand),
                                    decorationBox = { inner ->
                                        Box {
                                            if (body.isEmpty()) Text("分享你的想法、经验或新发现…", style = XType.Body, color = extra.ink3)
                                            inner()
                                        }
                                    },
                                )

                                // 5. 正文下方内联话题胶囊（取代堆叠 OutlinedTextField）
                                FlowRow(
                                    Modifier.fillMaxWidth().padding(top = 14.dp),
                                    horizontalArrangement = Arrangement.spacedBy(7.dp),
                                    verticalArrangement = Arrangement.spacedBy(7.dp),
                                ) {
                                    allTags.forEach { tag ->
                                        val isReq = requiredTag != null && tag.equals(requiredTag, ignoreCase = true)
                                        Row(
                                            Modifier.height(28.dp).clip(CircleShape)
                                                .background(extra.brandSoft)
                                                .padding(start = 10.dp, end = if (isReq) 10.dp else 6.dp),
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(4.dp),
                                        ) {
                                            Text("# $tag", style = XType.Caption, fontWeight = FontWeight.Medium, color = extra.brandInk, maxLines = 1)
                                            if (!isReq && !busy) {
                                                Box(
                                                    Modifier.size(16.dp).clip(CircleShape)
                                                        .clickable { updateUserTags(userTags.filterNot { it == tag }) },
                                                    contentAlignment = Alignment.Center,
                                                ) {
                                                    XIcon(XIcons.Close, size = 11.dp, tint = extra.brandInk)
                                                }
                                            }
                                        }
                                    }
                                    Row(
                                        Modifier.height(28.dp).clip(CircleShape).background(extra.fill)
                                            .clickable(enabled = !busy) {
                                                focus.clearFocus()
                                                choosingTags = true
                                            }
                                            .padding(horizontal = 10.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(4.dp),
                                    ) {
                                        XIcon(XIcons.Hash, size = 12.dp, tint = extra.ink3)
                                        Text(
                                            if (allTags.isEmpty()) "添加话题" else "话题",
                                            style = XType.Caption,
                                            fontWeight = FontWeight.Medium,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        )
                                    }
                                }

                                // 6. 投票卡 / 单作品 V2 卡 / 普通附件卡（同列滚动区内 --fill 视觉分组）
                                if (pollEnabled) {
                                    Column(
                                        Modifier.fillMaxWidth().padding(top = 14.dp)
                                            .clip(RoundedCornerShape(16.dp)).background(extra.fill).padding(13.dp),
                                        verticalArrangement = Arrangement.spacedBy(10.dp),
                                    ) {
                                        Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                                            XIcon(XIcons.Poll, size = 16.dp, tint = extra.brandInk)
                                            Text(" 发起投票", Modifier.weight(1f), style = XType.Secondary, fontWeight = FontWeight.SemiBold)
                                            IconTap(XIcons.Close, "移除投票", { if (!busy) pollEnabled = false }, size = 16.dp)
                                        }
                                        pollInputs.forEachIndexed { idx, opt ->
                                            Row(
                                                Modifier.fillMaxWidth().height(40.dp).clip(RoundedCornerShape(10.dp))
                                                    .background(MaterialTheme.colorScheme.surface).padding(horizontal = 10.dp),
                                                verticalAlignment = Alignment.CenterVertically,
                                                horizontalArrangement = Arrangement.spacedBy(6.dp),
                                            ) {
                                                BasicTextField(
                                                    value = opt,
                                                    onValueChange = { valNext ->
                                                        updatePollInputs(pollInputs.toMutableList().also { it[idx] = valNext.take(40) })
                                                    },
                                                    modifier = Modifier.weight(1f),
                                                    enabled = !busy,
                                                    singleLine = true,
                                                    textStyle = XType.Secondary.copy(color = MaterialTheme.colorScheme.onSurface),
                                                    cursorBrush = SolidColor(Brand),
                                                    decorationBox = { inner ->
                                                        Box {
                                                            if (opt.isEmpty()) Text("选项 ${idx + 1}", style = XType.Secondary, color = extra.ink3)
                                                            inner()
                                                        }
                                                    },
                                                )
                                                if (pollInputs.size > 2 && !busy) {
                                                    IconTap(XIcons.Close, "删除选项", {
                                                        updatePollInputs(pollInputs.toMutableList().also { it.removeAt(idx) })
                                                    }, size = 14.dp)
                                                }
                                            }
                                        }
                                        if (pollInputs.size < 6) {
                                            Row(
                                                Modifier.height(30.dp).clip(CircleShape).background(MaterialTheme.colorScheme.surface)
                                                    .clickable(enabled = !busy) { updatePollInputs(pollInputs + "") }
                                                    .padding(horizontal = 11.dp),
                                                verticalAlignment = Alignment.CenterVertically,
                                                horizontalArrangement = Arrangement.spacedBy(4.dp),
                                            ) {
                                                XIcon(XIcons.Plus, size = 13.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
                                                Text("添加选项", style = XType.Caption, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                            }
                                        }
                                        Text("截止时间", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                        LazyRow(horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                                            items(listOf(0, 1, 3, 7, 14, 30)) { day ->
                                                SurfaceChip(if (day == 0) "不限时" else "$day 天", pollDays == day) {
                                                    if (!busy) pollDays = day
                                                }
                                            }
                                        }
                                        Text(
                                            pollError ?: "发布后不能修改投票选项",
                                            style = XType.Caption,
                                            color = if (pollError != null) Danger else extra.ink3,
                                        )
                                    }
                                }

                                val currentWork = draftWork
                                if (currentWork != null) {
                                    DraftWorkCard(
                                        work = currentWork,
                                        enabled = !busy && !uploading,
                                        onReplace = {
                                            focus.clearFocus()
                                            workTab = if (currentWork is DraftPostWork.Linked) 1 else 0
                                            workPickerError = null
                                            choosingWork = true
                                        },
                                        onRemove = { draftWork = null; uploadedWorkKey = null },
                                        onToggleRights = {
                                            if (currentWork is DraftPostWork.Uploaded) {
                                                draftWork = currentWork.copy(rightsAccepted = !currentWork.rightsAccepted)
                                            }
                                        },
                                    )
                                }

                                attachments.forEach { file ->
                                    Row(
                                        Modifier.fillMaxWidth().padding(top = 10.dp)
                                            .clip(RoundedCornerShape(14.dp)).background(extra.fill).padding(11.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                                    ) {
                                        Box(
                                            Modifier.size(36.dp).clip(RoundedCornerShape(10.dp)).background(MaterialTheme.colorScheme.surface),
                                            contentAlignment = Alignment.Center,
                                        ) {
                                            XIcon(XIcons.File, size = 18.dp, tint = extra.brandInk)
                                        }
                                        Column(Modifier.weight(1f)) {
                                            Text(file.name, style = XType.Secondary, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                            Text(
                                                "${formatSize(file.size)} · ${if (file.price == 0) "免费下载" else "${file.price} 积分"}",
                                                style = XType.Caption,
                                                color = extra.ink3,
                                            )
                                        }
                                        IconTap(XIcons.Close, "移除附件", { removeAttachment(file) }, size = 16.dp)
                                    }
                                }

                                if (chosenBoard?.slug?.startsWith("share") == true || chosenBoard?.slug == "resource-movies") {
                                    Text(
                                        "请分享原创、开源或已获合法授权的内容，禁止破解与盗版链接。",
                                        Modifier.fillMaxWidth().padding(top = 12.dp)
                                            .clip(RoundedCornerShape(12.dp)).background(extra.fill)
                                            .padding(horizontal = 12.dp, vertical = 8.dp),
                                        style = XType.Caption,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    )
                                }
                                Spacer(Modifier.height(20.dp))
                            }

                            error?.let { Banner(it, Modifier.padding(horizontal = 16.dp, vertical = 6.dp), warn = true) }
                            uploadErrorBanner?.let { banner ->
                                Box(Modifier.padding(horizontal = 16.dp, vertical = 6.dp)) { banner() }
                            }
                            if (showLimits && limitsSummary != null) {
                                Column(Modifier.fillMaxWidth().background(extra.fill)) {
                                    Hairline()
                                    Row(
                                        Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                                    ) {
                                        Text(limitsSummary, Modifier.weight(1f), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                        IconTap(XIcons.Close, "收起上传说明", toggleLimits, size = 14.dp)
                                    }
                                }
                            }
                            Hairline()

                            // 7. 固定 48dp 单行图标工具栏
                            val statusText: String
                            val statusBad: Boolean
                            when {
                                toolsBusy || uploading -> { statusText = "正在上传…"; statusBad = false }
                                chosenBoard == null -> { statusText = "请选择发布板块"; statusBad = false }
                                !validTitle -> { statusText = if (isCasual) "标题最多 80 字" else "标题需 4～80 字"; statusBad = titleLength > 80 }
                                bodyLength < 10 -> { statusText = "正文还差 ${10 - bodyLength} 字"; statusBad = false }
                                bodyLength > 20_000 -> { statusText = "超出 20,000 字上限"; statusBad = true }
                                pollError != null -> { statusText = pollError; statusBad = true }
                                workError != null -> { statusText = workError; statusBad = true }
                                else -> { statusText = "$bodyLength 字"; statusBad = false }
                            }
                            Row(
                                Modifier.fillMaxWidth().height(48.dp).padding(horizontal = 8.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(2.dp),
                            ) {
                                ToolbarIconButton(XIcons.Image, "插入图片", enabled = pickImage != null, active = false) {
                                    focus.clearFocus(); pickImage?.invoke()
                                }
                                ToolbarIconButton(
                                    XIcons.File,
                                    "添加附件",
                                    enabled = addAttachment != null,
                                    active = attachments.isNotEmpty(),
                                    badge = attachments.size.takeIf { it > 0 }?.toString(),
                                ) {
                                    focus.clearFocus(); addAttachment?.invoke()
                                }
                                ToolbarIconButton(
                                    XIcons.Grid,
                                    "添加作品",
                                    enabled = !busy && !uploading && !leaving,
                                    active = draftWork != null,
                                    badge = if (draftWork != null) "1" else null,
                                ) {
                                    focus.clearFocus()
                                    workTab = if (draftWork is DraftPostWork.Linked) 1 else 0
                                    workPickerError = null
                                    choosingWork = true
                                }
                                ToolbarIconButton(XIcons.Poll, "发起投票", enabled = !busy && !uploading, active = pollEnabled) {
                                    focus.clearFocus(); pollEnabled = !pollEnabled
                                }
                                Spacer(Modifier.weight(1f))
                                Text(
                                    statusText,
                                    Modifier.widthIn(max = 152.dp).padding(end = 2.dp),
                                    style = XType.Caption,
                                    color = if (statusBad) Danger else extra.ink3,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                                IconTap(XIcons.Info, "上传说明", toggleLimits, tint = if (showLimits) extra.brandInk else extra.ink3, size = 18.dp)
                            }
                        }
                    },
                )
            }
        }

        // ─── Bottom Sheets ──────────────────────────────────────────────────────
        // A. 一级分段槽 + 3 列紧凑网格板块选择面板
        OverlayTransition(choosingCategory) { panel ->
            BoxWithConstraints(Modifier.fillMaxSize()) {
                BottomSheet({ choosingCategory = false }, panel.heightIn(max = maxHeight * 0.78f)) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Text("选择发布板块", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                        IconTap(XIcons.Close, "关闭分类", { choosingCategory = false })
                    }
                    when {
                        loadError != null -> Banner(loadError.orEmpty(), warn = true, actionText = "重试", onAction = { attempt++ })
                        catalog == null -> Spinner(Modifier.fillMaxWidth().height(120.dp))
                        else -> {
                            Row(
                                Modifier.fillMaxWidth().padding(top = 8.dp, bottom = 10.dp)
                                    .clip(RoundedCornerShape(12.dp)).background(extra.fill).padding(3.dp),
                                horizontalArrangement = Arrangement.spacedBy(3.dp),
                            ) {
                                PostingSections.forEachIndexed { index, item ->
                                    val list = destinations[item] ?: return@forEachIndexed
                                    val on = sectionIndex == index
                                    Box(
                                        Modifier.weight(1f).height(34.dp).clip(RoundedCornerShape(9.dp))
                                            .background(if (on) MaterialTheme.colorScheme.surface else Color.Transparent)
                                            .clickable { sectionIndex = index },
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        Text(
                                            "${item.label} ${list.size}",
                                            style = XType.Secondary,
                                            fontWeight = if (on) FontWeight.SemiBold else FontWeight.Medium,
                                            color = if (on) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant,
                                            maxLines = 1,
                                        )
                                    }
                                }
                            }
                            key(activeSection) {
                                val options = destinations[activeSection].orEmpty()
                                Column(Modifier.heightIn(max = 320.dp).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                                    options.chunked(3).forEach { rowItems ->
                                        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                            for (idx in 0 until 3) {
                                                val choice = rowItems.getOrNull(idx)
                                                if (choice == null) {
                                                    Spacer(Modifier.weight(1f))
                                                } else {
                                                    val enabled = catalog.canPost(choice.category, trustLevel)
                                                    val isOn = choice.key == selectedChoiceKey
                                                    Row(
                                                        Modifier.weight(1f).height(40.dp).clip(RoundedCornerShape(10.dp))
                                                            .background(if (isOn) extra.brandSoft else extra.fill)
                                                            .then(if (isOn) Modifier.border(1.dp, Brand.copy(alpha = 0.25f), RoundedCornerShape(10.dp)) else Modifier)
                                                            .alpha(if (enabled) 1f else 0.45f)
                                                            .clickable(enabled = enabled) {
                                                                selectedChoiceKey = choice.key
                                                                choosingCategory = false
                                                            }
                                                            .padding(horizontal = 6.dp),
                                                        verticalAlignment = Alignment.CenterVertically,
                                                        horizontalArrangement = Arrangement.Center,
                                                    ) {
                                                        Text(
                                                            choice.label,
                                                            style = XType.Secondary,
                                                            fontWeight = if (isOn) FontWeight.SemiBold else FontWeight.Medium,
                                                            color = if (isOn) extra.brandInk else MaterialTheme.colorScheme.onSurface,
                                                            maxLines = 1,
                                                            overflow = TextOverflow.Ellipsis,
                                                        )
                                                        if (choice.topicTag != null) {
                                                            Text(
                                                                "#",
                                                                Modifier.padding(start = 4.dp)
                                                                    .clip(RoundedCornerShape(4.dp))
                                                                    .background(MaterialTheme.colorScheme.surface)
                                                                    .padding(horizontal = 4.dp, vertical = 1.dp),
                                                                style = XType.Caption.copy(fontSize = 10.sp),
                                                                fontWeight = FontWeight.SemiBold,
                                                                color = if (isOn) extra.brandInk else extra.ink3,
                                                            )
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if (options.isEmpty()) {
                                        Text("暂时没有板块", Modifier.padding(vertical = 20.dp), style = XType.Secondary, color = extra.ink3)
                                    }
                                }
                                val previewChoice = options.firstOrNull { it.key == selectedChoiceKey } ?: options.firstOrNull()
                                if (previewChoice != null) {
                                    val rule = boardRuleLabel(catalog, previewChoice, trustLevel)
                                    Row(
                                        Modifier.fillMaxWidth().padding(top = 12.dp)
                                            .clip(RoundedCornerShape(12.dp)).background(extra.fill)
                                            .padding(horizontal = 12.dp, vertical = 10.dp),
                                        verticalAlignment = Alignment.Top,
                                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                                    ) {
                                        Column(Modifier.weight(1f)) {
                                            Text(previewChoice.label, style = XType.Secondary, fontWeight = FontWeight.SemiBold)
                                            if (previewChoice.category.description.isNotBlank()) {
                                                Text(previewChoice.category.description, Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                            }
                                        }
                                        if (rule != null) {
                                            Box(
                                                Modifier.clip(CircleShape).background(MaterialTheme.colorScheme.surface)
                                                    .padding(horizontal = 8.dp, vertical = 2.dp),
                                            ) {
                                                Text(rule, style = XType.Caption.copy(fontSize = 11.sp), fontWeight = FontWeight.SemiBold, color = extra.brandInk)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // B. 统一单作品面板（上传 V2 ZIP / HTML·HTM 转 V2 + 关联我的创作）
        OverlayTransition(choosingWork) { panel ->
            BoxWithConstraints(Modifier.fillMaxSize()) {
                BottomSheet({ if (!workPicking) choosingWork = false }, panel.heightIn(max = maxHeight * 0.78f)) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Text("添加作品", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                        IconTap(XIcons.Close, "关闭作品选择", { if (!workPicking) choosingWork = false })
                    }
                    Text(
                        "每篇帖子最多附带 1 个可运行作品（切换方式或重新选择将替换当前作品）",
                        Modifier.padding(top = 4.dp, bottom = 10.dp),
                        style = XType.Caption,
                        color = extra.ink3,
                    )
                    Row(
                        Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(extra.fill).padding(3.dp),
                        horizontalArrangement = Arrangement.spacedBy(3.dp),
                    ) {
                        listOf("上传成品", "关联我的创作").forEachIndexed { idx, label ->
                            val on = workTab == idx
                            Box(
                                Modifier.weight(1f).height(34.dp).clip(RoundedCornerShape(9.dp))
                                    .background(if (on) MaterialTheme.colorScheme.surface else Color.Transparent)
                                    .clickable(enabled = !workPicking) { workTab = idx; workPickerError = null },
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    label,
                                    style = XType.Secondary,
                                    fontWeight = if (on) FontWeight.SemiBold else FontWeight.Medium,
                                    color = if (on) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                    }

                    if (workTab == 0) {
                        val platform = env.platform
                        val canPickWork = platform?.forumWorkPickerSupported == true
                        Column(Modifier.padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                            if (canPickWork) {
                                Column(
                                    Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(extra.fill)
                                        .border(1.dp, extra.line, RoundedCornerShape(16.dp))
                                        .clickable(enabled = !workPicking) {
                                            workPicking = true
                                            workPickerError = null
                                            val account = session.accountId
                                            scope.launch {
                                                try {
                                                    val prepared = platform?.pickForumWork() ?: return@launch
                                                    check(currentAccount == account) { "账号已切换，请重新选择" }
                                                    draftWork = DraftPostWork.Uploaded(prepared, rightsAccepted = false)
                                                    uploadedWorkKey = null
                                                    choosingWork = false
                                                } catch (e: CancellationException) {
                                                    throw e
                                                } catch (e: Exception) {
                                                    workPickerError = messageOf(e)
                                                } finally {
                                                    workPicking = false
                                                }
                                            }
                                        }
                                        .padding(horizontal = 16.dp, vertical = 20.dp),
                                    horizontalAlignment = Alignment.CenterHorizontally,
                                    verticalArrangement = Arrangement.spacedBy(8.dp),
                                ) {
                                    Box(
                                        Modifier.size(40.dp).clip(RoundedCornerShape(12.dp)).background(MaterialTheme.colorScheme.surface),
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        XIcon(XIcons.Upload, size = 20.dp, tint = extra.brandInk)
                                    }
                                    Text(
                                        if (workPicking) "正在检查与封装作品…" else "选择 .zip / .html / .htm 成品",
                                        style = XType.Body,
                                        fontWeight = FontWeight.SemiBold,
                                    )
                                    Text(
                                        "ZIP 需包含 V2 manifest.json；单个 HTML / HTM 上传后自动封装转为 V2 容器",
                                        style = XType.Caption,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        textAlign = TextAlign.Center,
                                    )
                                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                        listOf(".zip V2 容器", ".html → 转 V2", ".htm → 转 V2").forEach { fmt ->
                                            Box(
                                                Modifier.clip(CircleShape).background(MaterialTheme.colorScheme.surface)
                                                    .border(0.5.dp, extra.hairline, CircleShape)
                                                    .padding(horizontal = 8.dp, vertical = 3.dp),
                                            ) {
                                                Text(fmt, style = XType.Caption.copy(fontSize = 11.sp), fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                            }
                                        }
                                    }
                                }
                            } else {
                                Column(
                                    Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(extra.fill).padding(16.dp),
                                    verticalArrangement = Arrangement.spacedBy(10.dp),
                                ) {
                                    Text("当前设备暂不支持本地挑选作品文件，可到网页版上传后回此页「关联我的创作」选用。", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                    GhostPillButton("打开网页版", {
                                        platform?.openUrl("${env.config.forumBase.trimEnd('/')}/publish")
                                    }, onPage = true, height = 40.dp)
                                }
                            }
                            workPickerError?.let { Banner(it, warn = true) }
                        }
                    } else {
                        val loader = rememberLoader(env, "compose-works-${session.accountId}") { env.content.myPacks() }
                        val state by loader.state.collectAsState()
                        val selectedMineId = (draftWork as? DraftPostWork.Linked)?.id
                        Column(Modifier.weight(1f, fill = false).padding(top = 10.dp).verticalScroll(rememberScrollState())) {
                            when (val result = state) {
                                Load.Loading -> Spinner(Modifier.fillMaxWidth().height(80.dp))
                                is Load.Failed -> Banner(result.message, warn = true, actionText = "重试", onAction = loader::reload)
                                is Load.Ready -> {
                                    val available = ownedChallengePacks(result.value.items, session.accountId)
                                    if (available.isEmpty()) {
                                        Text("暂无可关联的已发布作品，可切到「上传成品」直接提交。", Modifier.padding(vertical = 16.dp), style = XType.Secondary, color = extra.ink3)
                                    }
                                    available.forEach { pack ->
                                        WorkCatalogRow(pack, selected = pack.id == selectedMineId) {
                                            draftWork = DraftPostWork.Linked(pack.id, pack.title, kindLabel(pack.kind))
                                            uploadedWorkKey = null
                                            choosingWork = false
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // C. 话题标签弹层
        OverlayTransition(choosingTags) { panel ->
            BottomSheet({ choosingTags = false }, panel) {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text("添加话题标签", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                    IconTap(XIcons.Close, "关闭话题", { choosingTags = false })
                }
                Row(
                    Modifier.fillMaxWidth().padding(top = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    TextBox(
                        tagDraft,
                        { tagDraft = it.take(20) },
                        Modifier.weight(1f),
                        placeholder = if (allTags.size >= 5) "已达 5 个标签上限" else "输入话题名…",
                        enabled = allTags.size < 5,
                        filled = true,
                    )
                    PillButton("添加", {
                        addUserTag(tagDraft)
                        tagDraft = ""
                    }, enabled = tagDraft.isNotBlank() && allTags.size < 5, height = 46.dp)
                }
                if (allTags.isNotEmpty()) {
                    FlowRow(
                        Modifier.fillMaxWidth().padding(top = 12.dp),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        allTags.forEach { tag ->
                            val isReq = requiredTag != null && tag.equals(requiredTag, ignoreCase = true)
                            Row(
                                Modifier.height(30.dp).clip(CircleShape).background(extra.brandSoft)
                                    .padding(start = 11.dp, end = if (isReq) 11.dp else 7.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(4.dp),
                            ) {
                                Text("# $tag", style = XType.Secondary, color = extra.brandInk, fontWeight = FontWeight.Medium)
                                if (!isReq) {
                                    Box(
                                        Modifier.size(16.dp).clip(CircleShape).clickable { updateUserTags(userTags.filterNot { it == tag }) },
                                        contentAlignment = Alignment.Center,
                                    ) { XIcon(XIcons.Close, size = 12.dp, tint = extra.brandInk) }
                                }
                            }
                        }
                    }
                }
                val presets = chosenBoard?.presetTags.orEmpty()
                if (presets.isNotEmpty()) {
                    Text("板块常用话题", Modifier.padding(top = 14.dp, bottom = 8.dp), style = XType.Caption, color = extra.ink3)
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        presets.forEach { preset ->
                            val isReq = requiredTag != null && preset.equals(requiredTag, ignoreCase = true)
                            val on = allTags.any { it.equals(preset, ignoreCase = true) }
                            Box(
                                Modifier.height(32.dp).clip(CircleShape)
                                    .background(if (on) extra.brandSoft else extra.fill)
                                    .clickable(enabled = !isReq && (on || allTags.size < 5)) {
                                        if (on) updateUserTags(userTags.filterNot { it.equals(preset, ignoreCase = true) })
                                        else addUserTag(preset)
                                    }
                                    .padding(horizontal = 12.dp),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    "# $preset",
                                    style = XType.Secondary,
                                    color = if (on) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant,
                                    fontWeight = if (on) FontWeight.SemiBold else FontWeight.Normal,
                                )
                            }
                        }
                    }
                }
            }
        }

        // D. 封面选择与文字封面
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
                        } catch (e: CancellationException) { throw e }
                        catch (e: Exception) { error = messageOf(e) }
                        finally { uploading = false }
                    }
                }
                SheetAction(XIcons.File, "文字封面", "选择模板，用文字做封面") { choosingCover = false; editingCover = true }
            }
        }
        if (editingCover) env.platform?.let { platform ->
            ForumCoverEditor(
                platform,
                coverSpec ?: ForumCoverSpec(title),
                onDismiss = { editingCover = false },
                onUse = { spec, image -> coverSpec = spec; cover = image; editingCover = false },
            )
        }
        if (confirmingLeave) CenterDialog({ confirmingLeave = false }) {
            Text("放弃这次编辑？", style = XType.Title, fontWeight = FontWeight.SemiBold)
            Text("尚未发布的内容不会保留。", Modifier.padding(top = 8.dp, bottom = 20.dp), style = XType.Secondary, color = extra.ink3)
            PillButton("继续编辑", { confirmingLeave = false }, Modifier.fillMaxWidth())
            GhostPillButton("放弃编辑", { confirmingLeave = false; cancelDraft() }, Modifier.fillMaxWidth().padding(top = 8.dp), textColor = Danger)
        }
    }
}

@Composable
private fun DraftWorkCard(
    work: DraftPostWork,
    enabled: Boolean,
    onReplace: () -> Unit,
    onRemove: () -> Unit,
    onToggleRights: () -> Unit,
) {
    val extra = LocalExtraColors.current
    val badge = when (work) {
        is DraftPostWork.Linked -> "我的创作"
        is DraftPostWork.Uploaded -> work.badge
    }
    val subtitle = when (work) {
        is DraftPostWork.Linked -> "${work.kind} · V2 容器"
        is DraftPostWork.Uploaded -> buildString {
            append(work.prepared.originalName)
            append(if (work.converted) " · 已自动转为 V2 容器 · " else " · V2 作品包 · ")
            append(formatSize(work.prepared.file.bytes.size.toLong()))
        }
    }
    Column(
        Modifier.fillMaxWidth().padding(top = 12.dp)
            .clip(RoundedCornerShape(16.dp)).background(extra.fill).padding(12.dp),
        verticalArrangement = Arrangement.spacedBy(9.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Box(
                Modifier.size(36.dp).clip(RoundedCornerShape(10.dp)).background(MaterialTheme.colorScheme.surface),
                contentAlignment = Alignment.Center,
            ) {
                XIcon(XIcons.Grid, size = 18.dp, tint = extra.brandInk)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text(work.title, Modifier.weight(1f, fill = false), style = XType.Secondary, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Box(
                        Modifier.clip(RoundedCornerShape(5.dp)).background(extra.brandSoft)
                            .padding(horizontal = 6.dp, vertical = 1.dp),
                    ) {
                        Text(badge, style = XType.Caption.copy(fontSize = 10.sp), fontWeight = FontWeight.SemiBold, color = extra.brandInk, maxLines = 1)
                    }
                }
                Text(subtitle, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Box(
                Modifier.height(26.dp).clip(CircleShape).background(MaterialTheme.colorScheme.surface)
                    .clickable(enabled = enabled, onClick = onReplace).padding(horizontal = 9.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text("更换", style = XType.Caption, fontWeight = FontWeight.SemiBold)
            }
            IconTap(XIcons.Close, "移除作品 ${work.title}", { if (enabled) onRemove() }, size = 16.dp)
        }
        if (work is DraftPostWork.Uploaded) {
            Hairline()
            Row(
                Modifier.fillMaxWidth().clickable(enabled = enabled, onClick = onToggleRights).padding(top = 2.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                CheckBox(work.rightsAccepted, { if (enabled) onToggleRights() })
                Text(
                    "我确认这是原创或已获授权，并同意按社区规则发布${if (!work.rightsAccepted) "（必选）" else ""}",
                    style = XType.Caption,
                    color = if (!work.rightsAccepted) Danger else MaterialTheme.colorScheme.onSurfaceVariant,
                    fontWeight = if (!work.rightsAccepted) FontWeight.Medium else FontWeight.Normal,
                )
            }
        }
    }
}

@Composable
private fun WorkCatalogRow(pack: Pack, selected: Boolean, onSelect: () -> Unit) {
    val extra = LocalExtraColors.current
    Row(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).clickable(onClick = onSelect).padding(vertical = 11.dp, horizontal = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box(
            Modifier.size(38.dp).clip(RoundedCornerShape(10.dp)).background(extra.fill),
            contentAlignment = Alignment.Center,
        ) {
            XIcon(XIcons.Grid, size = 18.dp, tint = if (selected) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Column(Modifier.weight(1f)) {
            Text(pack.title, style = XType.Body, fontWeight = FontWeight.SemiBold, color = if (selected) extra.brandInk else MaterialTheme.colorScheme.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text("${kindLabel(pack.kind)} · v${pack.version} · V2 容器", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        if (selected) XIcon(XIcons.Check, size = 17.dp, tint = extra.brandInk)
    }
}

@Composable
private fun SurfaceChip(text: String, selected: Boolean, onClick: () -> Unit) {
    val extra = LocalExtraColors.current
    Box(
        Modifier.height(32.dp).clip(RoundedCornerShape(10.dp))
            .background(if (selected) extra.brandSoft else MaterialTheme.colorScheme.surface)
            .then(if (!selected) Modifier.border(0.5.dp, extra.hairline, RoundedCornerShape(10.dp)) else Modifier)
            .clickable(onClick = onClick).padding(horizontal = 12.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text,
            style = XType.Secondary,
            fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Medium,
            color = if (selected) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant,
            maxLines = 1,
        )
    }
}

@Composable
private fun ToolbarIconButton(
    icon: ImageVector,
    description: String,
    enabled: Boolean,
    active: Boolean,
    badge: String? = null,
    onClick: () -> Unit,
) {
    val extra = LocalExtraColors.current
    Box(
        Modifier.size(40.dp).clip(CircleShape).alpha(if (enabled) 1f else 0.38f)
            .clickable(enabled = enabled, onClickLabel = description, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        XIcon(icon, size = 20.dp, tint = if (active) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant)
        if (badge != null) {
            Box(
                Modifier.align(Alignment.TopEnd).padding(top = 4.dp, end = 3.dp)
                    .height(14.dp).widthIn(min = 14.dp).clip(CircleShape).background(Brand)
                    .padding(horizontal = 3.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(badge, color = Color.White, fontSize = 9.sp, lineHeight = 12.sp, fontWeight = FontWeight.Bold)
            }
        }
    }
}
