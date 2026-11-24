package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.tween
import androidx.compose.animation.core.updateTransition
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.DragInteraction
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.key
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import xyz.xiangban.xiaoban.common.community.creationInterrupted
import xyz.xiangban.xiaoban.common.community.creationProgress
import androidx.compose.ui.semantics.hideFromAccessibility
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.AiBalance
import xyz.xiangban.xiaoban.common.community.ChatApproval
import xyz.xiangban.xiaoban.common.community.MobileGame
import xyz.xiangban.xiaoban.common.community.CloudAssistant
import xyz.xiangban.xiaoban.common.community.CloudRecognizer
import xyz.xiangban.xiaoban.common.community.SpokenCall
import xyz.xiangban.xiaoban.common.community.toolLabel
import xyz.xiangban.xiaoban.common.link.RemoteApprovalOption
import xyz.xiangban.xiaoban.common.link.RemoteApprovalRequest
import androidx.compose.runtime.snapshotFlow
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ChatAttachment
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.longOrNull
import xyz.xiangban.xiaoban.common.community.CommunityJson
import xyz.xiangban.xiaoban.common.community.ToolStatus
import xyz.xiangban.xiaoban.common.community.ChatSummary
import xyz.xiangban.xiaoban.common.community.ChatTurn
import xyz.xiangban.xiaoban.common.community.ChatTaskStatus
import xyz.xiangban.xiaoban.common.community.StoredChat
import xyz.xiangban.xiaoban.common.community.isCustomModelTier
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CloudChatState
import xyz.xiangban.xiaoban.common.community.CustomModelConfig
import xyz.xiangban.xiaoban.common.community.CUSTOM_MODEL_TIER
import xyz.xiangban.xiaoban.common.community.customModelTier
import xyz.xiangban.xiaoban.common.community.newCustomProviderId
import xyz.xiangban.xiaoban.common.community.remoteProviderId
import xyz.xiangban.xiaoban.common.community.formatAiCredits
import xyz.xiangban.xiaoban.common.community.bucketKindLabel

internal val StarterPrompts = listOf(
    "帮我理清今天最重要的三件事",
    "用通俗的话解释一个复杂概念",
    "帮我把这段话改得更自然得体",
)

internal fun quotaText(balance: AiBalance?): String = when {
    balance == null -> "读取点数中…"
    balance.total > 0L -> "剩余 ${formatAiCredits(balance.total, balance.quotaPerUnit)}"
    balance.trial.available -> "试用 ${formatAiCredits(balance.trial.quota, balance.quotaPerUnit)}"
    else -> "点数已用完"
}

internal fun CloudChatState.modelName(): String = activeTier?.takeIf { officialAiEnabled || xyz.xiangban.xiaoban.common.community.isCustomModelTier(selectedTier) }?.displayName
    ?: if (officialAiEnabled) tiers.firstOrNull()?.displayName ?: "小伴AI" else "选择模型"

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun AiDrawerOverlay(chat: CloudChatController, visible: Boolean, onClose: () -> Unit, open: (Route) -> Unit) {
    val cloud by chat.state.collectAsState()
    var deleting by remember { mutableStateOf<ChatSummary?>(null) }
    var moving by remember { mutableStateOf<ChatSummary?>(null) }
    var editing by remember { mutableStateOf(false) }
    var original by remember { mutableStateOf<String?>(null) }
    var name by remember { mutableStateOf("") }
    BackHandler(enabled = visible && deleting == null && moving == null && !editing, onBack = onClose)
    SwipeChatDrawer(visible, onClose) { panel ->
        AiDrawer(
            cloud = cloud,
            onClose = onClose,
            onNew = { onClose(); chat.clearConversation(); open(Route.AiChat) },
            onOpen = { id -> onClose(); chat.openConversation(id); open(Route.AiChat) },
            onDelete = { moving = it },
            onCreateWorkspace = { original = null; name = ""; editing = true },
            onEditWorkspace = { original = it; name = it; editing = true },
            onModels = { onClose(); open(Route.Models) },
            onSettings = { onClose(); open(Route.Settings) },
            panelModifier = panel,
        )
    }
    if (editing) CenterDialog({ editing = false }) {
        Text(if (original == null) "新建工作空间" else "管理工作空间", style = XType.Title, fontWeight = FontWeight.SemiBold)
        Text("把相关的聊天收在一起", Modifier.padding(top = 6.dp, bottom = 16.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        OutlinedTextField(name, { name = it.take(40) }, label = { Text("名称") }, singleLine = true, modifier = Modifier.fillMaxWidth())
        val valid = name.trim().isNotEmpty() && (name.trim() == original || name.trim() !in cloud.workspaces)
        Row(Modifier.fillMaxWidth().padding(top = 16.dp, bottom = 12.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            GhostPillButton("取消", { editing = false }, Modifier.weight(1f))
            if (valid) PillButton("保存", {
                original?.let { chat.renameWorkspace(it, name) } ?: chat.addWorkspace(name)
                editing = false
            }, Modifier.weight(1f))
        }
        original?.let { group ->
            Text("删除分组后，对话保留在未分组中。", style = XType.Caption)
            GhostPillButton("删除工作空间", { chat.deleteWorkspace(group); editing = false })
        }
    }
    moving?.let { item ->
        BottomSheet({ moving = null }) {
            Text(item.title, style = XType.Title, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Text("移动到工作空间", Modifier.padding(top = 8.dp, bottom = 12.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Column(Modifier.heightIn(max = 320.dp).verticalScroll(rememberScrollState())) {
                (listOf<String?>(null) + cloud.workspaces).forEach { group ->
                    Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).clickable {
                        chat.moveConversation(item.id, group); moving = null
                    }.padding(14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        XIcon(if (group == null) XIcons.Message else XIcons.Folder, size = 20.dp)
                        Text(group ?: "未分组", Modifier.weight(1f), style = XType.Body)
                        if (group == item.workspace) Text("当前", style = XType.Caption, color = MaterialTheme.colorScheme.primary)
                    }
                }
            }
            Hairline()
            SheetAction(XIcons.Trash, "删除对话", "删除后无法恢复", { deleting = item; moving = null })
            GhostPillButton("取消", { moving = null })
        }
    }
    deleting?.let { item ->
        CenterDialog({ deleting = null }) {
            Text("删除这个对话？", style = XType.Title, fontWeight = FontWeight.SemiBold)
            Text("“${item.title}”会从这台手机上删掉，删了就找不回来了。", Modifier.padding(top = 8.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Row(Modifier.padding(top = 18.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                GhostPillButton("取消", { deleting = null }, Modifier.weight(1f), height = 44.dp)
                PillButton("删除", { chat.deleteConversation(item.id); deleting = null }, Modifier.weight(1f), height = 44.dp)
            }
        }
    }
}

// ───────────── 对话页（云端小伴） ─────────────

@OptIn(ExperimentalComposeUiApi::class)
@Composable
internal fun AiChatPage(
    env: CommunityEnv, chat: CloudChatController, open: (Route) -> Unit, back: (() -> Unit)? = null, visible: Boolean = true,
    onSelectTab: (Tab) -> Unit = {}, onPublish: () -> Unit = {},
) {
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    val conversation = cloud.conversationId
    val accountGeneration = session.generation
    val main = back == null
    val currentMainTopic = cloud.mainTopic
    val latestMainTopic = cloud.conversations.mapNotNull { it.mainTopic }.maxOrNull()
    val archived = !main && currentMainTopic != null && latestMainTopic != null && currentMainTopic < latestMainTopic
    val ime = WindowInsets.ime
    val navigationBars = WindowInsets.navigationBars
    val density = LocalDensity.current
    val compact by remember(ime, navigationBars, density) {
        // The keyboard can briefly report the gesture area again after its closing animation.
        val threshold = with(density) { MainBottomNavHeight.roundToPx() }
        derivedStateOf { ime.getBottom(density) - navigationBars.getBottom(density) >= threshold }
    }
    val aboveKeyboard = Modifier.revealAboveKeyboard(ime, navigationBars)
        .semantics { if (compact) hideFromAccessibility() }
    val focus = LocalFocusManager.current
    val remote by env.remoteChat.state.collectAsState()
    val onlineComputer = remote.peers.firstOrNull { it.online }?.name?.takeIf { session.loggedIn }
    val canChat = session.loggedIn && cloud.officialAiEnabled || cloud.customModelName != null
    val selectedCustom = isCustomModelTier(cloud.selectedTier)
    val exhausted = session.loggedIn && cloud.officialAiEnabled && !selectedCustom && cloud.balance != null && quotaText(cloud.balance) == "点数已用完"
    val model = cloud.activeTier?.displayName ?: if (canChat && cloud.officialAiEnabled) "Auto" else "选择模型"
    val quota = when {
        selectedCustom -> "自有服务"
        !cloud.officialAiEnabled -> ""
        !session.loggedIn -> "未登录"
        else -> quotaText(cloud.balance)
    }
    var config by rememberSaveable { mutableStateOf(false) }
    var tasksSheet by rememberSaveable { mutableStateOf(false) }
    var historySheet by rememberSaveable { mutableStateOf(false) }
    var clearHistory by remember { mutableStateOf(false) }
    fun stillInConversation() = chat.state.value.conversationId == conversation &&
        env.session.state.value.generation == accountGeneration
    var activeMainId by remember(accountGeneration) { mutableStateOf(conversation) }
    if (main && visible && cloud.mainTopic != null) SideEffect { activeMainId = conversation }
    val draftOwner = if (main) activeMainId else conversation
    var attachments by remember(draftOwner, accountGeneration) { mutableStateOf(emptyList<ChatAttachment>()) }
    val attachmentScope = key(draftOwner, accountGeneration) { rememberCoroutineScope() }
    val archiveSummaries = if (main) cloud.conversations.filter { it.mainTopic != null && it.id != conversation }.sortedBy { it.mainTopic } else emptyList()
    var archive by remember(accountGeneration) { mutableStateOf(emptyList<StoredChat>()) }
    LaunchedEffect(archiveSummaries, accountGeneration) {
        archive = archiveSummaries.mapNotNull { chat.readChatHistory(it.id) }
    }
    var input by rememberSaveable(draftOwner, accountGeneration) { mutableStateOf("") }
    var modelSheet by rememberSaveable { mutableStateOf(false) }
    var permissionSheet by remember(cloud.conversationId) { mutableStateOf(false) }
    var executionSheet by remember(cloud.conversationId) { mutableStateOf(false) }
    var drawer by rememberSaveable { mutableStateOf(false) }
    var attach by rememberSaveable { mutableStateOf(false) }
    var imageSheet by remember(cloud.conversationId) { mutableStateOf(false) }
    var preview by remember(conversation, accountGeneration) { mutableStateOf<ChatImagePreview?>(null) }
    var archivePreview by remember(conversation, accountGeneration) { mutableStateOf<Pair<String, ChatImagePreview>?>(null) }
    val resolveImage = remember(chat, conversation, accountGeneration) {
        val resolve: suspend (String) -> String? = { value -> chat.resolveImage(conversation, value) }
        resolve
    }
    val resolveThumbnail = remember(chat, conversation, accountGeneration) {
        val resolve: suspend (String) -> String? = { value -> chat.resolveImage(conversation, value, thumbnail = true) }
        resolve
    }
    var toast by remember { mutableStateOf<String?>(null) }
    var workKind by rememberSaveable { mutableStateOf<String?>(null) }
    var workBrief by rememberSaveable { mutableStateOf("") }
    var workForComputer by rememberSaveable { mutableStateOf<String?>(null) }
    // 语音：麦克风把一句话变成文字放进输入框；波形键是语音通话。识别先用手机本地的 SenseVoice，不行再用云端；合成直连 Edge 朗读。
    val voiceInput = key(conversation, accountGeneration) {
        rememberVoiceInput(
            env, { env.recognizers(CloudRecognizer(env.ai)) },
            onText = { text ->
                if (stillInConversation()) input = if (input.isBlank()) text else input.trimEnd() + " " + text
            },
            onSend = { text -> if (stillInConversation()) chat.send(text) },
            onMessage = { if (stillInConversation()) toast = it },
        )
    }
    var voiceMode by rememberSaveable { mutableStateOf(false) }
    val inputFocus = remember { FocusRequester() }
    val keyboard = LocalSoftwareKeyboardController.current
    var editingPlan by remember(conversation) { mutableStateOf(false) }
    LaunchedEffect(editingPlan) {
        if (editingPlan) { inputFocus.requestFocus(); keyboard?.show(); editingPlan = false }
    }
    val callHost = key(conversation, accountGeneration) { rememberVoiceCallHost(env) }
    val callScope = key(conversation, accountGeneration) { rememberCoroutineScope() }

    fun requireChat(action: () -> Unit) {
        when {
            main && cloud.mainTopic == null -> Unit
            canChat -> action()
            cloud.officialAiEnabled -> open(Route.Login)
            else -> open(Route.Models)
        }
    }
    fun openConfig() {
        focus.clearFocus()
        config = true
    }
    fun openImage() {
        when {
            !session.loggedIn -> open(Route.Login)
            cloud.sending && !main -> toast = "请先结束当前任务"
            cloud.imageCapability?.available == false -> toast = "生图暂不可用，请检查登录和额度"
            else -> imageSheet = true
        }
    }
    fun openWork(kind: String) {
        focus.clearFocus()
        workBrief = ""
        workKind = kind
    }
    fun sendPrompt(text: String) {
        requireChat {
            try {
                if (chat.sendWithAttachments(text, attachments)) {
                    input = ""
                    attachments = emptyList()
                }
            } catch (e: Exception) {
                toast = e.message ?: "附件无法发送"
            }
        }
    }
    fun openConversation(id: String) {
        focus.clearFocus()
        chat.openConversation(id)
        if (main) open(Route.AiChat)
    }
    LaunchedEffect(visible, session.loggedIn, session.accountId, accountGeneration) {
        if (visible && session.loggedIn && main) {
            chat.loadMeta()
            env.remoteChat.refreshPeers()
        }
    }
    LaunchedEffect(cloud.officialAiEnabled) { if (!cloud.officialAiEnabled) imageSheet = false }
    if (visible && !archived) key(conversation) {
        cloud.messages.lastOrNull().let { last ->
            AutoRead(env.reader, last?.id, last?.role, last?.text.orEmpty(), last?.streaming == true,
                paused = callHost.call != null, afterCompletion = true)
        }
    }
    BackHandler(enabled = visible && config) { config = false }

    val configTransition = updateTransition(config, label = "小伴配置")
    val chatShift by configTransition.animateFloat(
        transitionSpec = { tween(PageMillis, easing = FastOutSlowInEasing) },
        label = "主线让位",
    ) { if (it) 1f else 0f }
    val sliding = configTransition.currentState != config || configTransition.isRunning
    val blockTouches = if (sliding) Modifier.pointerInput(Unit) {
        awaitPointerEventScope {
            while (true) awaitPointerEvent(PointerEventPass.Initial).changes.forEach { it.consume() }
        }
    } else Modifier
    Box(Modifier.fillMaxSize().clipToBounds().then(blockTouches)) {
        Column(
            Modifier.fillMaxSize()
                .graphicsLayer { translationX = -size.width * chatShift }
                .semantics { if (config) hideFromAccessibility() }
                .imePadding(),
        ) {
            val hasWaitingTask = cloud.parallelTasks().any { it.id in cloud.waitingSessions || it.taskStatus == ChatTaskStatus.WaitingForPlan }
            AgentTopBar(model, exhausted, hasWaitingTask, { focus.clearFocus(); tasksSheet = true }, ::openConfig, back)
            if (main) AgentTaskStrip(cloud, compact, ::openConversation) { focus.clearFocus(); tasksSheet = true }
            Box(Modifier.weight(1f).fillMaxWidth()) {
                if (!archived && cloud.creationPlanReady) {
                    val plan = cloud.messages.last()
                    key(plan.id) {
                        Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 16.dp, vertical = 8.dp)) {
                            Text("制作方案", style = XType.Section, fontWeight = FontWeight.SemiBold)
                            MarkdownBubble(plan.text, false, env.platform, Modifier.padding(top = 8.dp), flat = true)
                        }
                    }
                } else if (cloud.messages.isEmpty() && archive.isEmpty()) {
                    AgentWelcome(env.utcOffsetMinutes, compact) { sendPrompt(it) }
                } else {
                    val deliveredWork = if (!archived && cloud.creation?.approved == true && !cloud.sending && cloud.messages.lastOrNull()?.creationInterrupted() != true) {
                        remember(cloud.messages) { creationProgress(cloud.messages) }.installed
                    } else null
                    key(conversation) {
                        CloudMessages(cloud.messages, cloud.sending, env.platform, resolveThumbnail, onPreview = { preview = it },
                            onRetry = chat::retryLast, onRegenerate = chat::regenerate, onStop = chat::stop, onSend = chat::send,
                            onQuota = { open(Route.Models) }, onRecoverImage = chat::recoverImageJob,
                            onReviewed = { try { chat.acknowledgeOutcome() } catch (e: Exception) { toast = e.message } },
                            waiting = if (cloud.question != null) "question" else if (cloud.approval != null) "approval" else null,
                            showQuota = cloud.officialAiEnabled, archive = archive, offsetMinutes = env.utcOffsetMinutes,
                            topicStarted = if (main && (cloud.mainTopic ?: 0) > 1L) {
                                cloud.messages.firstOrNull()?.createdAtMs
                                    ?: cloud.conversations.firstOrNull { it.id == conversation }?.updatedAtMs
                            } else null,
                            readOnly = archived, creation = cloud.creation?.approved == true,
                            deliveredWork = deliveredWork,
                            onOpenWork = { work ->
                                if (env.platform?.openGame(work.id) != true) toast = "当前无法打开这件作品"
                            },
                            onArchivedPreview = { id, image -> archivePreview = id to image },
                            archivedThumbnail = { id, image -> chat.resolveArchivedImage(id, image, thumbnail = true) })
                    }
                }
            }
            if (!archived) {
                cloud.question?.let { question ->
                    AskCard(RemoteApprovalRequest(
                        approvalId = question.id, session = cloud.conversationId, title = "小伴想问你",
                        kind = "question", question = question.question, choices = question.options, freeText = true,
                    ), { answer -> chat.answerQuestion(answer.takeUnless { it == "__dismiss__" }) }, Modifier.padding(start = 12.dp, end = 12.dp, bottom = 8.dp))
                }
                cloud.approval?.let { pending -> ApprovalCard(pending.toRequest(), { chat.answerApproval(it) }, Modifier.padding(start = 12.dp, end = 12.dp, bottom = 8.dp)) }
                if (main) {
                    Column(aboveKeyboard) {
                        AgentCapabilities(cloud.officialAiEnabled, onlineComputer != null, ::openImage,
                            { openWork("游戏") }, { openWork("壁纸") }, { open(Route.Devices) })
                        if (exhausted) AgentQuotaBanner(::openConfig)
                    }
                }
                Column(Modifier.padding(horizontal = 12.dp)) {
                    if (cloud.planningCreation) {
                        AgentPlanConfirmCard(
                            sending = cloud.sending,
                            ready = cloud.creationPlanReady,
                            canConfirm = input.isBlank() && attachments.isEmpty(),
                            onEdit = { voiceMode = false; editingPlan = true },
                            onPrimary = if (cloud.creationPlanReady) chat::confirmCreationPlan else chat::requestCreationPlan,
                            modifier = Modifier.padding(bottom = 8.dp),
                        )
                    }
                    if (cloud.creation?.approved == true && !cloud.sending) {
                        val last = cloud.messages.lastOrNull()
                        val progress = remember(cloud.messages) { creationProgress(cloud.messages) }
                        if (last?.creationInterrupted() == true) {
                            AgentCreationResumeCard(
                                stage = last.failureStage,
                                summary = progress.summary,
                                nextStep = progress.nextStep,
                                canResume = !last.error.orEmpty().contains("结果尚未确认"),
                                onResume = chat::resumeCreation,
                                modifier = Modifier.padding(bottom = 8.dp),
                            )
                        }
                    }
                    cloud.saveError?.let { message ->
                        Row(Modifier.fillMaxWidth().padding(bottom = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            Text(message, Modifier.weight(1f), style = XType.Caption, color = MaterialTheme.colorScheme.error)
                            LinkText("重试保存", chat::retrySave)
                        }
                    }
                    PressTip(env, voiceMode, Modifier.padding(bottom = 8.dp))
                    if (attachments.isNotEmpty()) Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(bottom = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        attachments.forEachIndexed { index, item -> ChipPill("${item.name} ×", onClick = { attachments = attachments.filterIndexed { i, _ -> i != index } }) }
                    }
                    cloud.inputError?.let { Text(it, style = XType.Caption, color = MaterialTheme.colorScheme.error, modifier = Modifier.padding(bottom = 4.dp)) }
                }
                AgentComposer(
                    input, { input = it }, if (cloud.creationPlanReady) "想调整哪些功能？" else if (cloud.messages.isEmpty() && archive.isEmpty()) "和小伴说点什么…" else "问点什么…",
                    inputFocus = inputFocus,
                    compact = compact, hasAttachments = attachments.isNotEmpty(),
                    voice = voiceInput, voiceMode = voiceMode, onVoiceMode = { focus.clearFocus(); voiceMode = it },
                    onAttach = { focus.clearFocus(); attach = true },
                    onLive = {
                        focus.clearFocus()
                        val platform = env.platform
                        when {
                            platform == null -> toast = "这个平台暂不支持语音通话"
                            !session.loggedIn -> open(Route.Login)
                            cloud.sending -> toast = "小伴正在回答，稍后再通话"
                            voiceInput.busy -> toast = "先结束当前的录音"
                            else -> callHost.request {
                                SpokenCall(callScope, platform, CloudAssistant(chat), env.recognizers(CloudRecognizer(env.ai)), env.synthesizers(), env.turnJudge())
                            }
                        }
                    },
                    onSend = { sendPrompt(input) }, sending = cloud.sending, onStop = chat::stop,
                )
            }
            if (main) {
                MainBottomNav(Tab.Ai, onSelect = { target ->
                    if (target != Tab.Ai) focus.clearFocus()
                    onSelectTab(target)
                }, onPublish = onPublish, modifier = aboveKeyboard)
            } else Spacer(Modifier.navigationBarsPadding())
        }
        val pageMotion = tween<IntOffset>(PageMillis, easing = FastOutSlowInEasing)
        configTransition.AnimatedVisibility(
            visible = { it },
            modifier = Modifier.fillMaxSize().semantics { if (!config) hideFromAccessibility() },
            enter = slideInHorizontally(pageMotion) { it },
            exit = slideOutHorizontally(pageMotion) { it },
        ) {
            AgentConfiguration(
                cloud, model, quota, exhausted, session.loggedIn, onlineComputer,
                back = { config = false },
                onTopic = {
                    if (main) chat.startMainTopic() else chat.clearConversation()
                    config = false
                },
                onHistory = { historySheet = true },
                onModel = { modelSheet = true },
                onQuota = { open(Route.Models) },
                onPermission = { if (cloud.nativeAgentAvailable) permissionSheet = true else toast = "本机操作当前不可用" },
                onBudget = { executionSheet = true },
                onDevices = { open(Route.Devices) },
                onAutoRead = { on -> SpeechState.update { it.copy(autoRead = on) }; if (!on) env.reader.stop() },
                onClear = { clearHistory = true },
            )
        }
        workKind?.let { kind ->
            MobileCreationSheet(env, chat, kind, workBrief, onKind = { workKind = it }, onIdea = { workBrief = it },
                onDismiss = { workKind = null }, onStart = {
                    val brief = workBrief.trim()
                    if (kind in LocalCreationTypes) {
                        requireChat {
                            if (env.agent == null) {
                                toast = "当前安装包不支持手机本地创作，可连接电脑创作。"
                            } else {
                                val type = if (kind == "小程序") "小程序（kind=app，category 取 tool 或 quiz）" else "离线小游戏（kind=game）"
                                val title = xyz.xiangban.xiaoban.common.community.creationTitle(brief, kind)
                                if (chat.startParallelCreation(title, brief, "制作支持手机触控和电脑键鼠的$type。需求：$brief")) {
                                    workKind = null
                                } else toast = "暂时无法发起创作，请检查当前对话模型。"
                            }
                        }
                    } else if (!session.loggedIn) {
                        open(Route.Login)
                    } else {
                        workKind = null
                        workForComputer = "请在这台电脑上帮我创作小伴桌面的${kind}。我的想法：${brief}。先读取对应的资源规范，给出简短方案让我确认，再制作并预览。"
                    }
                })
        }
        workForComputer?.let { prompt ->
            MobileCreationDeviceSheet(env, prompt, { workForComputer = null }, open)
        }
        OverlayTransition(tasksSheet) { panel ->
            AgentTasks(cloud, { tasksSheet = false }, panel, open = { id ->
                tasksSheet = false
                openConversation(id)
            })
        }
        OverlayTransition(historySheet) { panel ->
            AgentHistorySheet(chat, cloud.conversations, { historySheet = false }, panel,
                onOpen = { id -> historySheet = false; config = false; openConversation(id) },
                onAllConversations = { historySheet = false; drawer = true })
        }
        if (clearHistory) CenterDialog({ clearHistory = false }) {
            Text("清空主线聊天记录？", style = XType.Title, fontWeight = FontWeight.SemiBold)
            Text("将删除本机保存的主线记录，作品与已发布内容会保留。", Modifier.padding(vertical = 12.dp), style = XType.Secondary)
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                GhostPillButton("取消", { clearHistory = false }, Modifier.weight(1f))
                PillButton("清空", { clearHistory = false; chat.clearMainHistory() }, Modifier.weight(1f))
            }
        }
        OverlayTransition(permissionSheet) { panel ->
            AgentSheet("本机操作权限", { permissionSheet = false }, panel) {
                CloudPermissionSettings(chat, cloud) { permissionSheet = false }
            }
        }
        OverlayTransition(executionSheet) { panel ->
            BoxWithConstraints(Modifier.fillMaxSize()) {
                BottomSheet({ executionSheet = false }, panel.heightIn(max = maxHeight * 0.85f)) {
                    CloudExecutionSettings(chat, cloud) { executionSheet = false }
                }
            }
        }
        OverlayTransition(attach) { panel ->
            fun pick(label: String) {
                attach = false
                if (label == "相册" || label == "文件") {
                    if (env.platform?.supportsChatAttachments != true) toast = "当前平台尚未接入附件选择"
                    else if (attachments.size >= 3) toast = "每条消息最多添加 3 个附件"
                    else {
                        val conversation = cloud.conversationId
                        attachmentScope.launch {
                            try {
                                val item = env.platform?.pickChatAttachment(label == "相册")
                                if (item != null && chat.state.value.conversationId == conversation) {
                                    val next = attachments + item
                                    chat.validateDraftAttachments(next)
                                    attachments = next
                                }
                            } catch (e: CancellationException) { throw e }
                            catch (e: Exception) { toast = e.message ?: "无法读取附件" }
                        }
                    }
                }
            }
            AgentSheet("添加附件", { attach = false }, panel) {
                AgentChoiceRow(false, "相册", null, { pick("相册") })
                AgentChoiceRow(false, "文件", null, { pick("文件") })
            }
        }
        if (imageSheet && cloud.officialAiEnabled) {
            ImageGenerationSheet(
                initialPrompt = input,
                sizes = cloud.imageCapability?.sizes.orEmpty(),
                formats = cloud.imageCapability?.formats.orEmpty(),
                quota = quotaText(cloud.balance),
                basePointsPerImage = cloud.imageCapability?.basePointsPerImage,
                onDismiss = { imageSheet = false },
                onGenerate = { prompt, count, size, format, frames, duration ->
                    val started = if (main) chat.generateParallelImages(prompt, count, size, format, frames, duration)
                        else chat.generateImages(prompt, count, size, format, frames, duration)
                    if (started) {
                        if (!main) input = ""
                        imageSheet = false
                    } else toast = "请先结束当前任务"
                },
            )
        }
        preview?.let { image ->
            var referencing by remember(image) { mutableStateOf(false) }
            fun reference(review: Boolean) {
                if (attachments.size >= 3) toast = "每条消息最多添加 3 个附件"
                else {
                    referencing = true
                    attachmentScope.launch {
                        try {
                            val item = chat.referenceGeneratedImage(conversation, image.data, image.label) { data ->
                                env.platform?.prepareGeneratedChatImage(data)
                            }
                            if (stillInConversation() && preview == image) {
                                val next = attachments + item
                                chat.validateDraftAttachments(next)
                                attachments = next
                                if (review) {
                                    val brief = image.prompt.orEmpty()
                                    val reviewPrompt = "请按原需求检查这张图的文案、画幅和构图，逐字列出你能辨认的文字、偏差和看不清的部分。\n原需求：$brief"
                                    input = listOf(input.trim(), reviewPrompt).filter { it.isNotEmpty() }.joinToString("\n\n")
                                }
                                preview = null
                                toast = if (review) "已填入检查要求，发送后开始检查" else "已添加到输入框"
                            }
                        } catch (e: CancellationException) { throw e }
                        catch (e: Exception) { if (stillInConversation()) toast = e.message ?: "无法引用这张图片" }
                        finally { referencing = false }
                    }
                }
            }
            ChatImageViewer(image, env.platform, resolveImage, referencing = referencing,
                onReference = if (image.generated) ({ reference(false) }) else null,
                onReview = if (image.generated) ({ reference(true) }) else null,
            ) { preview = null }
        }
        archivePreview?.let { (id, image) ->
            ChatImageViewer(image, env.platform, { chat.resolveArchivedImage(id, it) }) { archivePreview = null }
        }
        AiDrawerOverlay(chat, drawer, { drawer = false }) { route ->
            if (main || route != Route.AiChat) open(route)
        }
        Toast(toast) { toast = null }
        PressOverlay(voiceInput)
        MicPermissionLayer(env, voiceInput) { toast = it }
        VoiceCallLayer(env, callHost, "小伴AI") { toast = it }
        var reasoningSheet by remember { mutableStateOf<Triple<String, String, List<String>>?>(null) }
        OverlayTransition(modelSheet) { panel ->
            val extra = LocalExtraColors.current
            val scheme = MaterialTheme.colorScheme
            BottomSheet({ modelSheet = false; reasoningSheet = null }, panel, flush = true, cornerRadius = 18.dp) {
                SheetDragHandle()
                Row(
                    Modifier.fillMaxWidth().padding(start = 16.dp, end = 14.dp, bottom = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("选择对话模型", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
                    SheetCloseCircle { modelSheet = false; reasoningSheet = null }
                }
                Column(Modifier.weight(1f, fill = false).heightIn(max = 460.dp).verticalScroll(rememberScrollState()).padding(bottom = 8.dp)) {
                    cloud.customProviders.forEach { provider ->
                        Text(
                            provider.name,
                            Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 5.dp),
                            style = XType.Caption,
                            fontWeight = FontWeight.SemiBold,
                            color = extra.ink3,
                        )
                        provider.models.forEach { model ->
                            val id = customModelTier(provider.id, model)
                            val legacyMatch = provider.id == "legacy" && cloud.selectedTier == CUSTOM_MODEL_TIER && cloud.legacyCustomTier?.displayName == model
                            val activeTierId = if (legacyMatch) CUSTOM_MODEL_TIER else id
                            val selected = cloud.selectedTier == id || legacyMatch
                            ChatModelPickerRow(
                                title = model,
                                description = if (provider.available) null else "暂不可用",
                                badge = null,
                                selected = selected,
                                enabled = provider.available && !cloud.sending,
                                reasoningLevels = if (provider.available) provider.reasoningEfforts else emptyList(),
                                currentReasoning = cloud.reasoningByTier[activeTierId],
                                onSelect = { if (provider.available && !cloud.sending) { chat.selectTier(id); modelSheet = false } },
                                onOpenReasoning = { reasoningSheet = Triple(activeTierId, model, provider.reasoningEfforts) },
                            )
                        }
                    }
                    if (cloud.officialAiEnabled) {
                        Text(
                            "小伴 AI · 官方",
                            Modifier.padding(start = 16.dp, end = 16.dp, top = 12.dp, bottom = 5.dp),
                            style = XType.Caption,
                            fontWeight = FontWeight.SemiBold,
                            color = extra.ink3,
                        )
                        if (cloud.tiers.isEmpty()) {
                            Text(
                                if (cloud.loadingMeta) "读取中…" else (cloud.metaError ?: "暂时没有可选的模型"),
                                Modifier.padding(horizontal = 16.dp, vertical = 12.dp),
                                style = XType.Secondary,
                                color = extra.ink3,
                            )
                        }
                        cloud.tiers.forEach { tier ->
                            val desc = (if (!tier.available && tier.requiresPaid) "需要充值后使用　" else "") + tier.description
                            ChatModelPickerRow(
                                title = tier.displayName,
                                description = desc.takeIf(String::isNotBlank),
                                badge = if (tier.isDefault) "推荐" else null,
                                selected = cloud.selectedTier == tier.tier,
                                enabled = tier.available && !cloud.sending,
                                reasoningLevels = if (tier.available) tier.reasoningEfforts else emptyList(),
                                currentReasoning = cloud.reasoningByTier[tier.tier],
                                onSelect = { if (tier.available && !cloud.sending) { chat.selectTier(tier.tier); modelSheet = false } },
                                onOpenReasoning = { reasoningSheet = Triple(tier.tier, tier.displayName, tier.reasoningEfforts) },
                            )
                        }
                    }
                }
                Hairline()
                Row(
                    Modifier
                        .fillMaxWidth()
                        .clickable { modelSheet = false; reasoningSheet = null; open(Route.Models) }
                        .padding(horizontal = 16.dp, vertical = 14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        if (cloud.officialAiEnabled) "模型配置（余额、兑换与第三方服务）" else "模型配置",
                        Modifier.weight(1f),
                        style = XType.Secondary,
                        fontWeight = FontWeight.Medium,
                        color = scheme.onSurfaceVariant,
                    )
                    XIcon(XIcons.ChevronRight, size = 16.dp, tint = extra.ink3)
                }
            }
        }
        reasoningSheet?.let { (tierId, modelName, levels) ->
            ReasoningEffortBottomSheet(
                modelName = modelName,
                levels = levels,
                selected = cloud.reasoningByTier[tierId],
                onDismiss = { reasoningSheet = null },
                onSelect = { effort ->
                    chat.selectReasoning(tierId, effort)
                    reasoningSheet = null
                    modelSheet = false
                },
            )
        }
    }
}

/** Within imePadding, reveal only the part of this row that fits above the moving keyboard. */
private fun Modifier.revealAboveKeyboard(ime: WindowInsets, navigationBars: WindowInsets): Modifier =
    clipToBounds().layout { measurable, constraints ->
        val row = measurable.measure(constraints)
        val keyboardHeight = (ime.getBottom(this) - navigationBars.getBottom(this)).coerceAtLeast(0)
        val height = (row.height - keyboardHeight).coerceAtLeast(constraints.minHeight)
        layout(row.width, height) { row.placeRelative(0, 0) }
    }

@Composable
private fun AgentHistorySheet(
    chat: CloudChatController, history: List<ChatSummary>, onClose: () -> Unit, panel: Modifier,
    onOpen: (String) -> Unit, onAllConversations: () -> Unit,
) {
    var query by rememberSaveable { mutableStateOf("") }
    val main = history.filter { it.mainTopic != null && it.title != "新对话" }
    var found by remember { mutableStateOf<List<Pair<ChatSummary, String>>>(emptyList()) }
    LaunchedEffect(query, main) {
        found = emptyList()
        val term = query.trim()
        if (term.isNotEmpty()) {
            delay(200)
            found = main.mapNotNull { item ->
                val text = chat.readChatHistory(item.id)?.messages?.firstOrNull { it.text.contains(term, ignoreCase = true) }?.text
                if (text == null && !item.title.contains(term, ignoreCase = true)) null
                else item to (text ?: item.title)
            }
        }
    }
    AgentSheet("查找聊天记录", onClose, panel) {
        OutlinedTextField(query, { query = it }, Modifier.fillMaxWidth(), singleLine = true,
            placeholder = { Text("搜索主线聊天记录", style = XType.Body) })
        val matches = if (query.isBlank()) main.map { it to relativeTime(it.updatedAtMs) } else found
        Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
            matches.forEach { (item, detail) ->
                Column(Modifier.fillMaxWidth().clickable { onOpen(item.id) }.padding(vertical = 10.dp)) {
                    Text(item.title, style = XType.Body, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text(detail, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
                }
            }
            if (matches.isEmpty()) Text("暂无聊天记录", Modifier.padding(vertical = 10.dp), style = XType.Caption,
                color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        LinkText("全部对话与工作空间", onAllConversations, Modifier.padding(top = 12.dp))
    }
}

@Composable
private fun CloudMessages(
    messages: List<ChatTurn>, sending: Boolean, platform: PlatformActions?, resolveImage: suspend (String) -> String?,
    onPreview: (ChatImagePreview) -> Unit, onRetry: () -> Unit, onRegenerate: () -> Unit, onStop: () -> Unit,
    onSend: (String) -> Unit, onQuota: () -> Unit, onRecoverImage: (Long) -> Unit, onReviewed: () -> Unit,
    waiting: String?, showQuota: Boolean, archive: List<StoredChat>, offsetMinutes: Int, topicStarted: Long?,
    readOnly: Boolean, creation: Boolean, deliveredWork: MobileGame? = null, onOpenWork: (MobileGame) -> Unit = {},
    onArchivedPreview: (String, ChatImagePreview) -> Unit,
    archivedThumbnail: suspend (String, String) -> String?,
) {
    val listState = rememberLazyListState()
    val last = messages.lastOrNull()
    var followLatest by remember { mutableStateOf(true) }
    LaunchedEffect(listState) {
        listState.interactionSource.interactions.collect {
            if (it is DragInteraction.Start) followLatest = false
        }
    }
    LaunchedEffect(listState) {
        snapshotFlow { !listState.canScrollForward && !listState.isScrollInProgress }.collect {
            if (it) followLatest = true
        }
    }
    LaunchedEffect(listState) {
        snapshotFlow {
            if (followLatest && listState.canScrollForward && !listState.isScrollInProgress) listState.layoutInfo.totalItemsCount else 0
        }.collect { count -> if (count > 0) listState.requestScrollToItem(count - 1) }
    }
    LazyColumn(Modifier.fillMaxSize(), state = listState,
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 18.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)) {
        archive.forEach { stored ->
            stored.messages.firstOrNull()?.createdAtMs?.let { time ->
                item(key = "${stored.id}:date") {
                    if ((stored.options.mainTopic ?: 1) > 1) AgentTopicDivider(time, offsetMinutes)
                    else AgentTimestamp(time, offsetMinutes)
                }
            }
            items(stored.messages, key = { "${stored.id}:${it.id}" }) { turn ->
                AgentChatTurn(turn, stored.messages, platform, { archivedThumbnail(stored.id, it) },
                    { onArchivedPreview(stored.id, it) }, offsetMinutes = offsetMinutes)
            }
        }
        if (topicStarted != null) item(key = "current-topic") { AgentTopicDivider(topicStarted, offsetMinutes) }
        else messages.firstOrNull()?.createdAtMs?.let { time ->
            item(key = "current-date") { AgentTimestamp(time, offsetMinutes) }
        }
        items(messages, key = { it.id }) { turn ->
            val latest = !readOnly && turn.id == last?.id && !sending
            AgentChatTurn(turn, messages, platform, resolveImage, onPreview, offsetMinutes = offsetMinutes,
                waiting = if (turn.streaming) waiting else null, onStop = onStop.takeUnless { readOnly }, onSend = onSend,
                creation = creation,
                onRetry = onRetry.takeIf { latest && !creation && turn.error != null && !turn.imageGeneration },
                onRegenerate = onRegenerate.takeIf { latest && !creation && !turn.streaming && !turn.imageGeneration },
                onContinue = if (latest && !creation) ({ onSend("请继续上一次任务，先核对已有结果，不要重复执行已完成的操作。") }) else null,
                onRecover = if (!readOnly && turn.imageJobRecoverable && !sending) ({ onRecoverImage(turn.id) }) else null,
                onReviewed = onReviewed.takeIf { !turn.imageGeneration && latest && turn.error.orEmpty().contains("结果尚未确认") },
                onQuota = onQuota.takeIf { showQuota && turn.error.orEmpty().let { it.contains("额度") || it.contains("点数") || it.contains("兑换") } })
        }
        if (deliveredWork != null) {
            item(key = "delivered-work:${deliveredWork.id}") {
                AgentDeliverableCard(deliveredWork, onOpen = { onOpenWork(deliveredWork) })
            }
        }
        item(key = "latest-edge") { Spacer(Modifier.height(1.dp)) }
    }
}

@Composable
private fun AgentChatTurn(
    turn: ChatTurn, messages: List<ChatTurn>, platform: PlatformActions?, resolveImage: suspend (String) -> String?,
    onPreview: (ChatImagePreview) -> Unit, offsetMinutes: Int, waiting: String? = null, onStop: (() -> Unit)? = null,
    onSend: (String) -> Unit = {}, onRetry: (() -> Unit)? = null, onRegenerate: (() -> Unit)? = null,
    onContinue: (() -> Unit)? = null, onRecover: (() -> Unit)? = null, onReviewed: (() -> Unit)? = null, onQuota: (() -> Unit)? = null,
    creation: Boolean = false,
) {
    if (turn.role == "user") {
        AgentUserMessage((if (turn.imageGeneration) "生成图片 · " else "") + turn.text +
            turn.attachments.filter { it.imageDataUrl == null }.joinToString("") { "\n📎 ${it.name}" })
        turn.attachments.forEach { attachment ->
            attachment.imageDataUrl?.let { ChatImageThumbnail(ChatImagePreview(it, attachment.name), resolveImage, onPreview) }
        }
    } else AgentReply {
        if (turn.streaming && turn.activity.isNotBlank() && (turn.text.isNotBlank() || turn.tools.isNotEmpty())) {
            Text(turn.activity, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        val reminders = remember(turn.tools) {
            turn.tools.filter { it.name == "set_reminder" && it.status == ToolStatus.Done }.mapNotNull { call ->
                val data = runCatching { CommunityJson.parseToJsonElement(call.output) as? JsonObject }.getOrNull()
                val reminder = data?.get("reminder") as? JsonObject ?: return@mapNotNull null
                val status = (reminder["status"] as? JsonPrimitive)?.contentOrNull
                val title = (reminder["title"] as? JsonPrimitive)?.contentOrNull
                val time = (reminder["at_epoch_ms"] as? JsonPrimitive)?.longOrNull
                if (status == "scheduled" && title != null && time != null) Triple(call, title, time) else null
            }
        }
        reminders.forEach { (call, title, time) ->
            var details by rememberSaveable(call.id) { mutableStateOf(false) }
            AgentReminderResult(title, time, offsetMinutes) { details = !details }
            if (details) ToolGroup(listOf(call), live = false)
        }
        AssistantTurnCards(
            text = turn.text, streaming = turn.streaming, error = if (turn.cancelled) "已取消" else turn.error,
            reasoning = turn.reasoning, tools = turn.tools.filterNot { tool -> reminders.any { it.first.id == tool.id } },
            records = turn.records, waiting = waiting, platform = platform,
            idleText = turn.activity.ifBlank { "思考中…" }, onStop = onStop, onSend = onSend,
            onRetry = onRetry, onRegenerate = onRegenerate, flat = true,
            showTaskRecovery = !creation || !turn.creationInterrupted(),
            extras = TurnExtras(promptTokens = turn.promptTokens.coerceAtMost(Int.MAX_VALUE.toLong()).toInt(),
                completionTokens = turn.completionTokens.coerceAtMost(Int.MAX_VALUE.toLong()).toInt(),
                truncated = turn.truncated, onContinue = onContinue),
        )
        if (turn.generatedImages.isNotEmpty()) {
            val brief = messages.takeWhile { it.id != turn.id }.lastOrNull { it.role == "user" }?.text
            turn.generatedImages.forEachIndexed { index, data ->
                ChatImageThumbnail(ChatImagePreview(data, "生成的图片 ${index + 1}", generated = true, prompt = brief), resolveImage, onPreview)
            }
        }
        if (onRecover != null) LinkText("查回生图结果", onRecover)
        if (onReviewed != null) LinkText("已核对，继续", onReviewed)
        if (onQuota != null) LinkText("去兑换点数 ›", onQuota)
    }
}

/** 手机内核要的授权，画成和设备聊天同一张紧凑授权卡。 */
private fun ChatApproval.toRequest(): RemoteApprovalRequest = RemoteApprovalRequest(
    approvalId = id, session = "", title = toolLabel(tool),
    detail = JsonObject(mapOf("detail" to JsonPrimitive(detail))),
    risk = if (danger != null) "high" else "low",
    options = buildList {
        add(RemoteApprovalOption("once", "允许一次"))
        if (canRemember) add(RemoteApprovalOption("always", "本对话都允许"))
        add(RemoteApprovalOption("deny", "拒绝"))
    },
    tool = tool, danger = danger,
)

/** 对话模型选择弹层内的单行项：名称严格左对齐，右侧等宽思考胶囊与固定对号列垂直对齐。 */
@Composable
private fun ChatModelPickerRow(
    title: String,
    description: String?,
    badge: String?,
    selected: Boolean,
    enabled: Boolean,
    reasoningLevels: List<String>,
    currentReasoning: String?,
    onSelect: () -> Unit,
    onOpenReasoning: () -> Unit,
) {
    val extra = LocalExtraColors.current
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 8.dp, vertical = 2.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(if (selected) extra.fill else Color.Transparent)
            .alpha(if (enabled) 1f else 0.45f)
            .padding(horizontal = 8.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Column(
            modifier = Modifier
                .weight(1f)
                .clickable(enabled = enabled, role = Role.RadioButton, onClick = onSelect)
                .padding(vertical = 1.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    text = title,
                    modifier = Modifier.weight(1f, fill = false),
                    style = XType.Body,
                    fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                    color = scheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (!badge.isNullOrBlank()) {
                    Text(
                        badge,
                        Modifier
                            .clip(RoundedCornerShape(6.dp))
                            .background(if (selected) scheme.surface else extra.fill)
                            .padding(horizontal = 6.dp, vertical = 1.dp),
                        style = XType.Caption,
                        color = scheme.onSurfaceVariant,
                    )
                }
            }
            if (!description.isNullOrBlank()) {
                Text(description, Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
            }
        }
        if (reasoningLevels.isNotEmpty()) {
            ReasoningEffortChip(
                modelName = title,
                current = currentReasoning,
                enabled = enabled,
                selectedRow = selected,
                onClick = onOpenReasoning,
            )
        }
        Box(Modifier.size(18.dp), contentAlignment = Alignment.Center) {
            if (selected) XIcon(XIcons.Check, size = 17.dp, tint = extra.brandInk)
        }
    }
}
