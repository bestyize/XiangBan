package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.combinedClickable
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
import androidx.compose.runtime.key
import androidx.compose.runtime.collectAsState
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.AiBalance
import xyz.xiangban.xiaoban.common.community.ChatApproval
import xyz.xiangban.xiaoban.common.community.CloudAssistant
import xyz.xiangban.xiaoban.common.community.CloudRecognizer
import xyz.xiangban.xiaoban.common.community.SpokenCall
import xyz.xiangban.xiaoban.common.community.toolLabel
import xyz.xiangban.xiaoban.common.link.RemoteApprovalOption
import xyz.xiangban.xiaoban.common.link.RemoteApprovalRequest
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.ChatAttachment
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import xyz.xiangban.xiaoban.common.community.ChatSummary
import xyz.xiangban.xiaoban.common.community.ChatTurn
import xyz.xiangban.xiaoban.common.community.chatTitle
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CloudChatState
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

internal fun CloudChatState.modelName(): String = activeTier?.displayName ?: tiers.firstOrNull()?.displayName ?: "小伴AI"

private fun CloudChatState.title(): String = chatTitle(messages)

/** 没登录时的整页提示。 */
@Composable
private fun LoginPrompt(title: String, desc: String, onLogin: () -> Unit, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth().padding(horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        EmptyState(XIcons.Sparkle, title, desc)
        PillButton("登录小伴账号", onLogin, Modifier.padding(top = 20.dp).fillMaxWidth())
    }
}

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

@Composable
fun AiChatPage(env: CommunityEnv, chat: CloudChatController, back: () -> Unit, open: (Route) -> Unit, openImageSheet: Boolean = false, onImageSheetOpened: () -> Unit = {}) {
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    val conversation = cloud.conversationId
    val accountGeneration = session.generation
    fun stillInConversation() = chat.state.value.conversationId == conversation &&
        env.session.state.value.generation == accountGeneration
    var attachments by remember(conversation, accountGeneration) { mutableStateOf(emptyList<ChatAttachment>()) }
    val attachmentScope = key(conversation, accountGeneration) { rememberCoroutineScope() }
    var input by rememberSaveable(conversation, accountGeneration) { mutableStateOf("") }
    var modelSheet by rememberSaveable { mutableStateOf(false) }
    var permissionSheet by remember(cloud.conversationId) { mutableStateOf(false) }
    var executionSheet by remember(cloud.conversationId) { mutableStateOf(false) }
    var drawer by rememberSaveable { mutableStateOf(false) }
    var attach by rememberSaveable { mutableStateOf(false) }
    var imageSheet by remember(cloud.conversationId) { mutableStateOf(false) }
    var preview by remember(cloud.conversationId, session.accountId) { mutableStateOf<ChatImagePreview?>(null) }
    var toast by remember { mutableStateOf<String?>(null) }
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
    val callHost = key(conversation, accountGeneration) { rememberVoiceCallHost(env) }
    val callScope = key(conversation, accountGeneration) { rememberCoroutineScope() }

    LaunchedEffect(session.loggedIn, session.accountId, accountGeneration) { if (session.loggedIn) chat.loadMeta() }
    LaunchedEffect(openImageSheet) {
        if (openImageSheet) {
            onImageSheetOpened()
            when {
                !session.loggedIn -> Unit
                cloud.sending -> toast = "请先结束当前任务"
                cloud.imageCapability?.available == false -> toast = "生图暂不可用，请检查登录和额度"
                else -> imageSheet = true
            }
        }
    }
    cloud.messages.lastOrNull().let { last -> AutoRead(env.reader, last?.id, last?.role, last?.text.orEmpty(), last?.streaming == true, paused = callHost.call != null) }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().imePadding()) {
            ScreenBar(
                cloud.title(), onBack = back,
                trailing = {
                    Row {
                        IconTap(XIcons.Sliders, "任务设置", { executionSheet = true }, size = 22.dp)
                        IconTap(XIcons.NewChat, "新对话", { chat.clearConversation(); input = "" }, size = 22.dp)
                        IconTap(XIcons.Menu, "对话列表", { drawer = true }, size = 22.dp)
                    }
                },
            )
            if (!session.loggedIn) {
                Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                    LoginPrompt("登录即可与小伴对话", "使用统一小伴账号开启对话，新用户享有免费试用点数，不用自己准备模型密钥。", { open(Route.Login) })
                }
            } else {
                Box(Modifier.weight(1f).fillMaxWidth()) {
                    if (cloud.messages.isEmpty()) {
                        ChatWelcome()
                    } else {
                        CloudMessages(cloud.messages, cloud.sending, env.platform, onPreview = { preview = it }, onRetry = chat::retryLast, onRegenerate = chat::regenerate, onStop = chat::stop, onSend = chat::send, onQuota = { open(Route.Models) }, onReviewed = { try { chat.acknowledgeOutcome() } catch (e: Exception) { toast = e.message } }, waiting = if (cloud.question != null) "question" else if (cloud.approval != null) "approval" else null)
                    }
                }
                cloud.question?.let { question ->
                    AskCard(RemoteApprovalRequest(
                        approvalId = question.id, session = cloud.conversationId, title = "小伴想问你",
                        kind = "question", question = question.question, choices = question.options, freeText = true,
                    ), { answer -> chat.answerQuestion(answer.takeUnless { it == "__dismiss__" }) }, Modifier.padding(horizontal = 12.dp))
                }
                cloud.approval?.let { pending -> ApprovalCard(pending.toRequest(), { chat.answerApproval(it) }, Modifier.padding(start = 12.dp, end = 12.dp, bottom = 8.dp)) }
                if (cloud.messages.isEmpty()) {
                    Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(start = 16.dp, end = 16.dp, bottom = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        StarterPrompts.forEach { prompt -> ChipPill(prompt, onClick = { chat.send(prompt) }) }
                    }
                }
                Column(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp, top = 4.dp)) {
                    cloud.saveError?.let { message ->
                        Row(Modifier.fillMaxWidth().padding(bottom = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            Text(message, Modifier.weight(1f), style = XType.Caption, color = MaterialTheme.colorScheme.error)
                            LinkText("重试保存", chat::retrySave)
                        }
                    }
                    if (!cloud.nativeAgentAvailable) Text("当前仅支持云端对话，本机任务暂不可用", Modifier.padding(bottom = 6.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    PressTip(env, voiceMode, Modifier.padding(bottom = 8.dp))
                    if (attachments.isNotEmpty()) Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(bottom = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        attachments.forEachIndexed { index, item -> ChipPill("${item.name} ×", onClick = { attachments = attachments.filterIndexed { i, _ -> i != index } }) }
                    }
                    cloud.inputError?.let { Text(it, style = XType.Caption, color = MaterialTheme.colorScheme.error, modifier = Modifier.padding(bottom = 4.dp)) }
                    ChatComposer(
                        input, { input = it }, "问点什么，或描述想要的装扮、游戏…",
                        onModel = { modelSheet = true },
                        onAttach = { attach = true },
                        onPermission = if (cloud.nativeAgentAvailable) ({ permissionSheet = true }) else null,
                        fullTrust = cloud.permissionMode == "auto",
                        voice = voiceInput, voiceMode = voiceMode, onVoiceMode = { voiceMode = it },
                        onVoicePress = { if (cloud.sending) chat.stop() },
                        onLive = {
                            val platform = env.platform
                            when {
                                platform == null -> toast = "这个平台暂不支持语音通话"
                                !session.loggedIn -> toast = "请先登录小伴账号"
                                cloud.sending -> toast = "小伴正在回答，稍后再通话"
                                voiceInput.busy -> toast = "先结束当前的录音"
                                else -> callHost.request {
                                    SpokenCall(callScope, platform, CloudAssistant(chat), env.recognizers(CloudRecognizer(env.ai)), env.synthesizers(), env.turnJudge())
                                }
                            }
                        },
                        onSend = {
                            if (!cloud.sending && (input.isNotBlank() || attachments.isNotEmpty())) {
                                try {
                                    if (chat.sendWithAttachments(input, attachments)) { input = ""; attachments = emptyList() }
                                }
                                catch (e: Exception) { toast = e.message ?: "附件无法发送" }
                            }
                        },
                        hasAttachments = attachments.isNotEmpty(),
                        sending = cloud.sending, onStop = chat::stop,
                    )
                }
            }
        }
        OverlayTransition(permissionSheet) { panel ->
            BottomSheet({ permissionSheet = false }, panel) { CloudPermissionSettings(chat, cloud) { permissionSheet = false } }
        }
        OverlayTransition(executionSheet) { panel ->
            BoxWithConstraints(Modifier.fillMaxSize()) {
                BottomSheet({ executionSheet = false }, panel.heightIn(max = maxHeight * 0.85f)) {
                    CloudExecutionSettings(chat, cloud) { executionSheet = false }
                }
            }
        }
        OverlayTransition(attach) { panel ->
            AttachSheet({ attach = false }, { label ->
                attach = false
                if (label == "生成图片") {
                    if (cloud.sending) toast = "请先结束当前任务"
                    else if (cloud.imageCapability?.available == false) toast = "生图暂不可用，请检查登录和额度"
                    else imageSheet = true
                } else if (label == "相册" || label == "文件") {
                    if (env.platform?.supportsChatAttachments != true) toast = "当前平台尚未接入附件选择"
                    else if (attachments.size >= 3) toast = "每条消息最多添加 3 个附件"
                    else {
                        val conversation = cloud.conversationId
                        attachmentScope.launch {
                            try {
                                val item = env.platform?.pickChatAttachment(label == "相册")
                                if (item != null && chat.state.value.conversationId == conversation) {
                                    val next = attachments + item
                                    xyz.xiangban.xiaoban.common.community.validateChatAttachments(next)
                                    attachments = next
                                }
                            } catch (e: CancellationException) { throw e }
                            catch (e: Exception) { toast = e.message ?: "无法读取附件" }
                        }
                    }
                } else toast = "$label 尚未接入，请从相册或文件选择"
            }, panel, showImageGeneration = true)
        }
        if (imageSheet) {
            ImageGenerationSheet(
                initialPrompt = input,
                sizes = cloud.imageCapability?.sizes.orEmpty(),
                formats = cloud.imageCapability?.formats.orEmpty(),
                onDismiss = { imageSheet = false },
                onGenerate = { prompt, count, size, format, frames, duration ->
                    if (chat.generateImages(prompt, count, size, format, frames, duration)) {
                        input = ""
                        imageSheet = false
                    } else toast = "请先结束当前任务"
                },
            )
        }
        preview?.let { ChatImageViewer(it, env.platform) { preview = null } }
        AiDrawerOverlay(chat, drawer, { drawer = false }) { route -> if (route != Route.AiChat) open(route) }
        Toast(toast) { toast = null }
        PressOverlay(voiceInput)
        MicPermissionLayer(env, voiceInput) { toast = it }
        VoiceCallLayer(env, callHost, "小伴AI") { toast = it }
        OverlayTransition(modelSheet) { panel ->
            BoxWithConstraints(Modifier.fillMaxSize()) {
                BottomSheet({ modelSheet = false }, panel.heightIn(max = maxHeight * 0.8f)) {
                    Text("选择模型", style = XType.Title, fontWeight = FontWeight.SemiBold)
                    Text("选择模型，右侧可调整思考等级。", Modifier.padding(top = 4.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text("小伴AI · 官方", Modifier.padding(top = 12.dp), style = XType.Caption, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
                        if (cloud.tiers.isEmpty()) {
                            Text(if (cloud.loadingMeta) "读取中…" else (cloud.metaError ?: "暂时没有可选的模型"), Modifier.padding(vertical = 12.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3)
                        }
                        cloud.tiers.forEach { tier ->
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                RadioRow(
                                    selected = cloud.selectedTier == tier.tier,
                                    title = tier.displayName,
                                    desc = (if (!tier.available && tier.requiresPaid) "需要充值后使用　" else "") + tier.description,
                                    modifier = Modifier.weight(1f),
                                    enabled = tier.available && !cloud.sending,
                                    onClick = { if (tier.available && !cloud.sending) { chat.selectTier(tier.tier); modelSheet = false } },
                                )
                                if (tier.available && tier.reasoningEfforts.isNotEmpty()) {
                                    ReasoningMenu(
                                        modelName = tier.displayName,
                                        levels = tier.reasoningEfforts,
                                        selected = cloud.reasoningByTier[tier.tier],
                                        enabled = !cloud.sending,
                                        onSelect = { effort ->
                                            chat.selectReasoning(tier.tier, effort)
                                            modelSheet = false
                                        },
                                    )
                                }
                            }
                        }
                    }
                    Row(
                        Modifier.padding(top = 8.dp).fillMaxWidth().clickable { modelSheet = false; open(Route.Models) }.padding(vertical = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text("模型与点数（余额、兑换）", Modifier.weight(1f), style = XType.Body, fontWeight = FontWeight.SemiBold)
                        XIcon(XIcons.ChevronRight, size = 16.dp)
                    }
                    GhostPillButton("取消", { modelSheet = false }, Modifier.padding(top = 4.dp).fillMaxWidth())
                }
            }
        }
    }
}

@Composable
private fun ChatWelcome() {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(16.dp), verticalArrangement = Arrangement.Bottom) {
        AssistantRow { AssistantBubble("你好，我是小伴。当前是云端对话，不依赖电脑在线。你可以随时提问、整理待办或润色文字。") }
    }
}

@Composable
private fun CloudMessages(messages: List<ChatTurn>, sending: Boolean, platform: PlatformActions?, onPreview: (ChatImagePreview) -> Unit, onRetry: () -> Unit, onRegenerate: () -> Unit, onStop: () -> Unit, onSend: (String) -> Unit, onQuota: () -> Unit, onReviewed: () -> Unit = {}, waiting: String? = null) {
    val listState = rememberLazyListState()
    val last = messages.lastOrNull()
    val progressMark = (last?.text?.length ?: 0) + (last?.tools?.size ?: 0) * 7 + (last?.records?.size ?: 0) * 13
    LaunchedEffect(messages.size, progressMark) { if (messages.isNotEmpty()) listState.scrollToItem(messages.lastIndex) }
    LazyColumn(Modifier.fillMaxSize(), state = listState, contentPadding = PaddingValues(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        items(messages, key = { it.id }) { turn ->
            if (turn.role == "user") {
                UserBubble((if (turn.imageGeneration) "生成图片 · " else "") + turn.text +
                    turn.attachments.filter { it.imageDataUrl == null }.joinToString("") { "\n📎 ${it.name}" })
                turn.attachments.forEach { attachment ->
                    attachment.imageDataUrl?.let { data ->
                        ChatImageThumbnail(ChatImagePreview(data, attachment.name), onPreview)
                    }
                }
            } else {
                AssistantRow {
                    AssistantTurnCards(
                        text = turn.text, streaming = turn.streaming, error = if (turn.cancelled) "已取消" else turn.error, reasoning = turn.reasoning, tools = turn.tools, records = turn.records,
                        waiting = if (turn.streaming) waiting else null, platform = platform, idleText = turn.activity.ifBlank { "思考中…" }, onStop = onStop, onSend = onSend,
                        onRetry = if (turn.id == last?.id && turn.error != null && !sending && !turn.imageGeneration) onRetry else null,
                        onRegenerate = if (turn.id == last?.id && !turn.streaming && !sending && !turn.imageGeneration) onRegenerate else null,
                        extras = TurnExtras(promptTokens = turn.promptTokens.coerceAtMost(Int.MAX_VALUE.toLong()).toInt(), completionTokens = turn.completionTokens.coerceAtMost(Int.MAX_VALUE.toLong()).toInt(), truncated = turn.truncated,
                            onContinue = if (turn.id == last?.id && !sending) ({ onSend("请继续上一次任务，先核对已有结果，不要重复执行已完成的操作。") }) else null),
                    )
                    turn.generatedImages.forEachIndexed { index, data ->
                        ChatImageThumbnail(ChatImagePreview(data, "生成的图片 ${index + 1}"), onPreview)
                    }
                    if (!turn.imageGeneration && turn.id == last?.id && !sending && turn.error.orEmpty().contains("结果尚未确认")) LinkText("已核对，继续", onReviewed)
                    if (turn.error.orEmpty().let { it.contains("额度") || it.contains("点数") || it.contains("兑换") }) LinkText("去兑换点数 ›", onQuota)
                }
            }
        }
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

// ───────────── 模型与点数 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ModelsScreen(env: CommunityEnv, chat: CloudChatController, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    var sheet by rememberSaveable { mutableStateOf(false) }
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val sheetScope = rememberCoroutineScope()
    val dismissSheet: () -> Unit = {
        sheetScope.launch {
            sheetState.hide()
            if (!sheetState.isVisible) sheet = false
        }
    }
    var code by remember { mutableStateOf("") }

    LaunchedEffect(cloud.redeemMessage) { if (cloud.redeemMessage != null) code = "" }

    LaunchedEffect(session.loggedIn, session.accountId) { if (session.loggedIn) chat.loadMeta() }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("模型与点数", onBack = back)
            if (!session.loggedIn) {
                Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                    LoginPrompt("登录后查看点数", "小伴点跟着账号走，换手机、换电脑自动同步。", { open(Route.Login) })
                }
            } else {
                Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
                    GroupLabel("小伴AI · 官方")
                    Column(Modifier.padding(horizontal = 16.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(16.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            Box(Modifier.size(40.dp).clip(RoundedCornerShape(8.dp)).background(Brand), contentAlignment = Alignment.Center) { XIcon(XIcons.SparkleSmall, tint = Color.White, size = 22.dp) }
                            Column(Modifier.weight(1f)) {
                                Text("小伴AI", style = XType.Section, fontWeight = FontWeight.SemiBold)
                                Text("跟着账号走，换手机、换电脑自动同步", Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                        Text(
                            "当前模型：${cloud.modelName()}\n${quotaText(cloud.balance)}" + (expiryText(cloud.balance)?.let { " · 到期 $it" } ?: ""),
                            Modifier.padding(top = 12.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        GhostPillButton("配置", { sheet = true; chat.clearRedeemFeedback() }, Modifier.padding(top = 12.dp).fillMaxWidth(), height = 40.dp, fontSize = 14)
                    }
                    GroupLabel("自己的模型服务")
                    Text(
                        "用自己的密钥接入 DeepSeek、OpenAI 兼容服务等，目前只能在电脑版的“设置”里添加；手机上的填写入口会在后续版本提供。",
                        Modifier.padding(horizontal = 16.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
        if (sheet) {
            BoxWithConstraints(Modifier.fillMaxSize().statusBarsPadding().imePadding()) {
                val contentHeight = (maxHeight * 0.85f - 48.dp).coerceAtLeast(1.dp)
                ModalBottomSheet(
                    onDismissRequest = { sheet = false },
                    sheetState = sheetState,
                    containerColor = MaterialTheme.colorScheme.surface,
                ) {
                Column(Modifier.fillMaxWidth().heightIn(max = contentHeight).padding(start = 16.dp, end = 16.dp, bottom = 16.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Box(Modifier.size(40.dp).clip(RoundedCornerShape(8.dp)).background(Brand), contentAlignment = Alignment.Center) { XIcon(XIcons.SparkleSmall, tint = Color.White, size = 22.dp) }
                    Column(Modifier.weight(1f)) {
                        Text("小伴AI", style = XType.Title, fontWeight = FontWeight.SemiBold)
                        Text("跟着账号走，换设备自动同步", Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    IconTap(XIcons.Close, "关闭", dismissSheet)
                }
                Column(Modifier.weight(1f, fill = false).verticalScroll(rememberScrollState())) {
                Row(Modifier.padding(top = 14.dp).fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(MaterialTheme.colorScheme.background).padding(vertical = 12.dp)) {
                    Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(cloud.balance?.let { formatAiCredits(it.total, it.quotaPerUnit) } ?: "—", style = XType.Headline, fontWeight = FontWeight.SemiBold)
                        Text("剩余小伴点", Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(expiryText(cloud.balance) ?: "—", style = XType.Headline, fontWeight = FontWeight.SemiBold)
                        Text("到期时间", Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
                cloud.balance?.let { balance ->
                    Text(
                        "其中付费点数 ${formatAiCredits(balance.paid, balance.quotaPerUnit)}",
                        Modifier.padding(top = 8.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    if (balance.trial.available) {
                        Text(
                            "首次使用自动发放 ${formatAiCredits(balance.trial.quota, balance.quotaPerUnit)} 试用点数，${balance.trial.days} 天内有效。",
                            Modifier.padding(top = 8.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    balance.buckets.forEach { bucket ->
                        Text(
                            "${bucketKindLabel(bucket.kind)} · ${formatAiCredits(bucket.remaining, balance.quotaPerUnit)} / ${formatAiCredits(bucket.total, balance.quotaPerUnit)}" +
                                (bucket.expiresAt?.let { " · ${it.take(10)} 到期" } ?: " · 长期有效"),
                            Modifier.padding(top = 8.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                Row(Modifier.padding(top = 14.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    TextBox(code, { code = it }, Modifier.weight(1f), placeholder = "输入兑换码", filled = true)
                    PillButton(if (cloud.redeeming) "兑换中…" else "兑换", { chat.redeem(code) }, enabled = !cloud.redeeming && code.isNotBlank(), height = 48.dp)
                }
                cloud.redeemMessage?.let { Text(it, Modifier.padding(top = 8.dp), style = XType.Body, fontWeight = FontWeight.Medium) }
                cloud.redeemError?.let { Text(it, Modifier.padding(top = 8.dp), style = XType.Body, color = Danger) }
                Text("可用模型", Modifier.padding(top = 16.dp), style = XType.Body, fontWeight = FontWeight.SemiBold)
                Text("选择聊天使用的模型，也可以在对话页切换。", Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                cloud.tiers.forEach { tier ->
                    RadioRow(
                        selected = cloud.selectedTier == tier.tier,
                        title = tier.displayName + if (tier.isDefault) "　推荐" else "",
                        desc = (if (!tier.available && tier.requiresPaid) "需要充值后使用　" else "") + tier.description,
                        enabled = tier.available,
                        onClick = { chat.selectTier(tier.tier) },
                    )
                }
                }
                PillButton("完成", dismissSheet, Modifier.padding(top = 14.dp).fillMaxWidth())
                }
            }
            }
        }
    }
}

/** 取额度里最早到期的一项，只显示日期；没有到期时间返回 null。 */
private fun expiryText(balance: AiBalance?): String? =
    balance?.buckets?.filter { it.remaining > 0 }?.mapNotNull { it.expiresAt }?.minOrNull()?.take(10)

