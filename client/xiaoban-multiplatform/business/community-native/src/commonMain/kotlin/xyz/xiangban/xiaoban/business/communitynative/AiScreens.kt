package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import xyz.xiangban.xiaoban.common.community.ChatTurn
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CloudChatState
import xyz.xiangban.xiaoban.common.community.formatAiMoney

private val StarterPrompts = listOf(
    "帮我理清今天最重要的三件事",
    "用通俗的话解释一个复杂概念",
    "帮我把这段话改得更自然得体",
)

private fun quotaText(balance: AiBalance?): String = when {
    balance == null -> "读取额度中…"
    balance.total > 0L -> "剩余 ${formatAiMoney(balance.total, balance.quotaPerUnit)}"
    balance.trial.available -> "试用 ${formatAiMoney(balance.trial.quota, balance.quotaPerUnit)}"
    else -> "额度已用完"
}

private fun CloudChatState.modelName(): String = activeTier?.displayName ?: tiers.firstOrNull()?.displayName ?: "小伴AI"

private fun CloudChatState.title(): String {
    val first = messages.firstOrNull { it.role == "user" }?.text?.trim().orEmpty()
    return if (first.isEmpty()) "新对话" else if (first.length > 14) first.take(14) + "…" else first
}

/** 没登录时的整页提示。 */
@Composable
private fun LoginPrompt(title: String, desc: String, onLogin: () -> Unit, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth().padding(horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        EmptyState(XIcons.Sparkle, title, desc)
        PillButton("登录小伴账号", onLogin, Modifier.padding(top = 20.dp).fillMaxWidth())
    }
}

// ───────────── 小伴AI 首页 ─────────────

/** 小伴AI 首页：问候、输入卡片（点一下进入对话页）、功能入口、试试这样问我、当前对话。 */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun AiHomeScreen(env: CommunityEnv, chat: CloudChatController, open: (Route) -> Unit, onOpenDrawer: () -> Unit) {
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()

    LaunchedEffect(session.loggedIn) {
        if (session.loggedIn) {
            chat.loadMeta()
            env.remoteChat.refreshPeers()
        } else {
            chat.reset()
        }
    }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            Column(Modifier.fillMaxWidth().statusBarsPadding()) {
                Row(Modifier.fillMaxWidth().height(52.dp).padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                    IconTap(XIcons.Menu, "对话列表", onOpenDrawer)
                    Text("小伴AI", Modifier.weight(1f), fontSize = 17.sp, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.Center)
                    Spacer(Modifier.width(44.dp))
                }
            }
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(start = 16.dp, end = 16.dp, top = 12.dp)) {
                Text("你好，想做点什么？", fontSize = 24.sp, fontWeight = FontWeight.SemiBold, lineHeight = 31.sp)
                Text("问问题、办点事，或者遥控电脑上的小伴", Modifier.padding(top = 6.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)

                // 输入卡片：整张卡片是进入对话页的入口，真正的输入在对话页。
                Column(
                    Modifier.padding(top = 16.dp).fillMaxWidth().clip(RoundedCornerShape(20.dp)).background(MaterialTheme.colorScheme.surface)
                        .clickable { if (session.loggedIn) open(Route.AiChat) else open(Route.Login) }.padding(horizontal = 16.dp, vertical = 14.dp),
                ) {
                    Text("问点什么，或描述想要的内容…", fontSize = 15.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Row(Modifier.fillMaxWidth().padding(top = 14.dp), verticalAlignment = Alignment.CenterVertically) {
                        ChipPill(cloud.modelName(), trailingIcon = XIcons.ChevronDown)
                        Spacer(Modifier.weight(1f))
                        Box(Modifier.size(36.dp).clip(CircleShape).background(Brand), contentAlignment = Alignment.Center) {
                            XIcon(XIcons.ArrowRight, tint = Color.White, size = 20.dp)
                        }
                    }
                }

                Text("功能", Modifier.padding(top = 22.dp, bottom = 10.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                val peers = remote.peers
                val computerDesc = when {
                    !session.loggedIn -> "登录后可用"
                    peers.isEmpty() -> "还没有找到设备"
                    peers.any { it.online } -> peers.first { it.online }.name + " · 在线"
                    else -> "设备都不在线"
                }
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    FeatureTile(XIcons.Monitor, "我的设备", computerDesc, Modifier.weight(1f)) { open(Route.Devices) }
                    FeatureTile(XIcons.Star, "模型与额度", if (session.loggedIn) quotaText(cloud.balance) else "登录后可用", Modifier.weight(1f)) { open(Route.Models) }
                }

                Text("试试这样问我", Modifier.padding(top = 22.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                Column(Modifier.padding(top = 6.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface)) {
                    StarterPrompts.forEachIndexed { i, prompt ->
                        if (i > 0) Hairline(Modifier.padding(start = 14.dp))
                        Row(
                            Modifier.fillMaxWidth().clickable {
                                if (session.loggedIn) { chat.clearConversation(); chat.send(prompt); open(Route.AiChat) } else open(Route.Login)
                            }.padding(14.dp),
                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Text(prompt, Modifier.weight(1f), fontSize = 15.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                        }
                    }
                }

                if (cloud.messages.isNotEmpty()) {
                    Text("最近对话", Modifier.padding(top = 22.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    Row(
                        Modifier.padding(top = 6.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable { open(Route.AiChat) }.padding(14.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Text(cloud.title(), Modifier.weight(1f), fontSize = 15.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                    }
                    Text("对话只保存在这次打开应用期间。", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                Spacer(Modifier.height(NavInset))
            }
        }
    }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun AiDrawerOverlay(chat: CloudChatController, visible: Boolean, onClose: () -> Unit, open: (Route) -> Unit) {
    val cloud by chat.state.collectAsState()
    BackHandler(enabled = visible, onBack = onClose)
    OverlayTransition(visible, direction = "left") { panel ->
        AiDrawer(
            cloud = cloud,
            onClose = onClose,
            onNew = { onClose(); chat.clearConversation(); open(Route.AiChat) },
            onOpenCurrent = { onClose(); open(Route.AiChat) },
            onModels = { onClose(); open(Route.Models) },
            onSettings = { onClose(); open(Route.Settings) },
            panelModifier = panel,
        )
    }
}

@Composable
private fun FeatureTile(icon: androidx.compose.ui.graphics.vector.ImageVector, title: String, desc: String, modifier: Modifier = Modifier, onClick: () -> Unit) {
    Row(
        modifier.clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        IconBox(icon, 40.dp, 12.dp)
        Column(Modifier.weight(1f)) {
            Text(title, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(desc, Modifier.padding(top = 2.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
    }
}

/** 左侧对话列表抽屉：新对话、当前对话、模型与额度、设置。 */
@Composable
private fun AiDrawer(cloud: CloudChatState, onClose: () -> Unit, onNew: () -> Unit, onOpenCurrent: () -> Unit, onModels: () -> Unit, onSettings: () -> Unit, panelModifier: Modifier) {
    Row(Modifier.fillMaxSize()) {
        Column(panelModifier.width(300.dp).fillMaxHeight().background(MaterialTheme.colorScheme.surface).statusBarsPadding().navigationBarsPadding().padding(vertical = 16.dp)) {
            Text("小伴AI", Modifier.padding(horizontal = 16.dp), fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
            PillButton("新对话", onNew, Modifier.padding(start = 16.dp, end = 16.dp, top = 14.dp).fillMaxWidth(), height = 44.dp, icon = XIcons.Plus)
            Text("最近", Modifier.padding(start = 16.dp, top = 14.dp, bottom = 4.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            if (cloud.messages.isNotEmpty()) {
                Text(
                    cloud.title(),
                    Modifier.padding(horizontal = 8.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).clickable(onClick = onOpenCurrent).padding(horizontal = 8.dp, vertical = 11.dp),
                    fontSize = 15.sp, maxLines = 1, overflow = TextOverflow.Ellipsis,
                )
            } else {
                Text("还没有对话", Modifier.padding(horizontal = 16.dp, vertical = 8.dp), fontSize = 14.sp, color = LocalExtraColors.current.ink3)
            }
            Spacer(Modifier.weight(1f))
            Hairline()
            DrawerLink(XIcons.Star, "模型与额度", onModels)
            DrawerLink(XIcons.Sliders, "设置", onSettings)
        }
        Box(Modifier.weight(1f).fillMaxHeight().background(Color.Black.copy(alpha = 0.45f)).clickable(indication = null, interactionSource = null) { onClose() })
    }
}

@Composable
private fun DrawerLink(icon: androidx.compose.ui.graphics.vector.ImageVector, text: String, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().height(44.dp).clickable(onClick = onClick).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        XIcon(icon, size = 22.dp)
        Text(text, fontSize = 15.sp)
    }
}

// ───────────── 对话页（云端小伴） ─────────────

@Composable
fun AiChatPage(env: CommunityEnv, chat: CloudChatController, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    var input by rememberSaveable { mutableStateOf("") }
    var modelSheet by rememberSaveable { mutableStateOf(false) }
    var drawer by rememberSaveable { mutableStateOf(false) }
    var attach by rememberSaveable { mutableStateOf(false) }
    var toast by remember { mutableStateOf<String?>(null) }
    // 语音：麦克风把一句话变成文字放进输入框；波形键是语音通话。识别先用手机本地的 SenseVoice，不行再用云端；合成直连 Edge 朗读。
    val voiceInput = rememberVoiceInput(
        env, { env.recognizers(CloudRecognizer(env.ai)) },
        onText = { text -> input = if (input.isBlank()) text else input.trimEnd() + " " + text },
        onSend = { text -> chat.send(text) },
        onMessage = { toast = it },
    )
    var voiceMode by rememberSaveable { mutableStateOf(false) }
    val callHost = rememberVoiceCallHost(env)
    val callScope = rememberCoroutineScope()

    LaunchedEffect(session.loggedIn) { if (session.loggedIn) chat.loadMeta() }
    cloud.messages.lastOrNull().let { last -> AutoRead(env.reader, last?.id, last?.role, last?.text.orEmpty(), last?.streaming == true) }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().imePadding()) {
            ScreenBar(
                cloud.title(), onBack = back,
                trailing = {
                    Row {
                        IconTap(XIcons.NewChat, "新对话", { chat.clearConversation(); input = "" }, size = 22.dp)
                        IconTap(XIcons.Menu, "对话列表", { drawer = true }, size = 22.dp)
                    }
                },
            )
            if (!session.loggedIn) {
                Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                    LoginPrompt("登录即可与小伴对话", "使用统一小伴账号开启对话，新用户享有免费试用额度，不用自己准备模型密钥。", { open(Route.Login) })
                }
            } else {
                Box(Modifier.weight(1f).fillMaxWidth()) {
                    if (cloud.messages.isEmpty()) {
                        ChatWelcome()
                    } else {
                        CloudMessages(cloud.messages, cloud.sending, env.platform, onRetry = chat::retryLast, onRegenerate = chat::regenerate, onStop = chat::stop, onSend = chat::send, onQuota = { open(Route.Models) })
                    }
                }
                cloud.approval?.let { pending -> ApprovalCard(pending.toRequest(), { chat.answerApproval(it) }, Modifier.padding(start = 12.dp, end = 12.dp, bottom = 8.dp)) }
                if (cloud.messages.isEmpty()) {
                    Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(start = 16.dp, end = 16.dp, bottom = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        StarterPrompts.forEach { prompt -> ChipPill(prompt, onClick = { chat.send(prompt) }) }
                    }
                }
                Column(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp, top = 4.dp)) {
                    PressTip(env, voiceMode, Modifier.padding(bottom = 8.dp))
                    ChatComposer(
                        input, { input = it }, "问点什么，或描述想要的装扮、游戏…",
                        onModel = { modelSheet = true },
                        onAttach = { attach = true },
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
                                    SpokenCall(callScope, platform, CloudAssistant(chat), env.recognizers(CloudRecognizer(env.ai)), env.synthesizers())
                                }
                            }
                        },
                        onSend = { val text = input.trim(); if (text.isNotEmpty()) { input = ""; chat.send(text) } },
                        sending = cloud.sending, onStop = chat::stop,
                    )
                }
            }
        }
        OverlayTransition(attach) { panel ->
            AttachSheet({ attach = false }, { attach = false; toast = "$it 即将支持" }, panel)
        }
        AiDrawerOverlay(chat, drawer, { drawer = false }) { route -> if (route != Route.AiChat) open(route) }
        Toast(toast) { toast = null }
        PressOverlay(voiceInput)
        MicPermissionLayer(env, voiceInput) { toast = it }
        VoiceCallLayer(env, callHost, "小伴AI") { toast = it }
        OverlayTransition(modelSheet) { panel ->
            BottomSheet({ modelSheet = false }, panel) {
                Text("选择模型", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                Text("只对当前这个对话生效。", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Text("小伴AI · 官方", Modifier.padding(top = 12.dp), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                if (cloud.tiers.isEmpty()) {
                    Text(if (cloud.loadingMeta) "读取中…" else (cloud.metaError ?: "暂时没有可选的模型"), Modifier.padding(vertical = 12.dp), fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                }
                cloud.tiers.forEach { tier ->
                    RadioRow(
                        selected = cloud.selectedTier == tier.tier,
                        title = tier.displayName,
                        desc = (if (!tier.available && tier.requiresPaid) "需要充值后使用　" else "") + tier.description,
                        enabled = tier.available,
                        onClick = { if (tier.available) { chat.selectTier(tier.tier); modelSheet = false } else { modelSheet = false; open(Route.Models) } },
                    )
                }
                Row(
                    Modifier.padding(top = 8.dp).fillMaxWidth().clickable { modelSheet = false; open(Route.Models) }.padding(vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("模型与额度（余额、兑换）", Modifier.weight(1f), fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                    XIcon(XIcons.ChevronRight, size = 16.dp)
                }
                GhostPillButton("取消", { modelSheet = false }, Modifier.padding(top = 4.dp).fillMaxWidth())
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
private fun CloudMessages(messages: List<ChatTurn>, sending: Boolean, platform: PlatformActions?, onRetry: () -> Unit, onRegenerate: () -> Unit, onStop: () -> Unit, onSend: (String) -> Unit, onQuota: () -> Unit) {
    val listState = rememberLazyListState()
    val last = messages.lastOrNull()
    val progressMark = (last?.text?.length ?: 0) + (last?.tools?.size ?: 0) * 7 + (last?.records?.size ?: 0) * 13
    LaunchedEffect(messages.size, progressMark) { if (messages.isNotEmpty()) listState.scrollToItem(messages.lastIndex) }
    LazyColumn(Modifier.fillMaxSize(), state = listState, contentPadding = PaddingValues(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        items(messages, key = { it.id }) { turn ->
            if (turn.role == "user") {
                UserBubble(turn.text)
            } else {
                AssistantRow {
                    AssistantTurnCards(
                        text = turn.text, streaming = turn.streaming, error = turn.error, reasoning = "", tools = turn.tools, records = turn.records,
                        waiting = null, platform = platform, idleText = "思考中…", onStop = onStop, onSend = onSend,
                        onRetry = if (turn.error != null && !sending) onRetry else null,
                        onRegenerate = if (turn.id == last?.id && !turn.streaming && !sending) onRegenerate else null,
                    )
                    if (turn.error.orEmpty().let { it.contains("额度") || it.contains("兑换") }) LinkText("去兑换额度 ›", onQuota)
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
        if (danger == null) add(RemoteApprovalOption("always", "本对话都允许"))
        add(RemoteApprovalOption("deny", "拒绝"))
    },
    tool = tool, danger = danger,
)

// ───────────── 模型与额度 ─────────────

@Composable
fun ModelsScreen(env: CommunityEnv, chat: CloudChatController, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    var sheet by rememberSaveable { mutableStateOf(false) }
    var code by remember { mutableStateOf("") }

    LaunchedEffect(session.loggedIn) { if (session.loggedIn) chat.loadMeta() }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("模型与额度", onBack = back)
            if (!session.loggedIn) {
                Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                    LoginPrompt("登录后查看额度", "小伴AI 的额度跟着账号走，换手机、换电脑自动同步。", { open(Route.Login) })
                }
            } else {
                Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 24.dp)) {
                    GroupLabel("小伴AI · 官方")
                    Column(Modifier.padding(horizontal = 16.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(16.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            Box(Modifier.size(40.dp).clip(RoundedCornerShape(8.dp)).background(Brand), contentAlignment = Alignment.Center) { XIcon(XIcons.SparkleSmall, tint = Color.White, size = 22.dp) }
                            Column(Modifier.weight(1f)) {
                                Text("小伴AI", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                Text("跟着账号走，换手机、换电脑自动同步", Modifier.padding(top = 2.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                        Text(
                            "当前模型：${cloud.modelName()}\n${quotaText(cloud.balance)}" + (expiryText(cloud.balance)?.let { " · 到期 $it" } ?: ""),
                            Modifier.padding(top = 12.dp), fontSize = 14.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        GhostPillButton("配置", { sheet = true; chat.clearRedeemFeedback() }, Modifier.padding(top = 12.dp).fillMaxWidth(), height = 40.dp, fontSize = 14)
                    }
                    GroupLabel("自己的模型服务")
                    Text(
                        "用自己的密钥接入 DeepSeek、OpenAI 兼容服务等，目前只能在电脑版的“设置”里添加；手机上的填写入口会在后续版本提供。",
                        Modifier.padding(horizontal = 16.dp), fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
        OverlayTransition(sheet) { panel ->
            BottomSheet({ sheet = false }, panel) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Box(Modifier.size(40.dp).clip(RoundedCornerShape(8.dp)).background(Brand), contentAlignment = Alignment.Center) { XIcon(XIcons.SparkleSmall, tint = Color.White, size = 22.dp) }
                    Column(Modifier.weight(1f)) {
                        Text("小伴AI", fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                        Text("跟着账号走，换设备自动同步", Modifier.padding(top = 2.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    IconTap(XIcons.Close, "关闭", { sheet = false })
                }
                Row(Modifier.padding(top = 14.dp).fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(MaterialTheme.colorScheme.background).padding(vertical = 12.dp)) {
                    Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(cloud.balance?.let { formatAiMoney(it.total, it.quotaPerUnit) } ?: "—", fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                        Text("剩余额度", Modifier.padding(top = 2.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
                        Text(expiryText(cloud.balance) ?: "—", fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                        Text("到期时间", Modifier.padding(top = 2.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
                Row(Modifier.padding(top = 14.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    TextBox(code, { code = it }, Modifier.weight(1f), placeholder = "输入兑换码", filled = true)
                    PillButton(if (cloud.redeeming) "兑换中…" else "兑换", { chat.redeem(code); code = "" }, enabled = !cloud.redeeming && code.isNotBlank(), height = 48.dp)
                }
                cloud.redeemMessage?.let { Text(it, Modifier.padding(top = 8.dp), fontSize = 14.sp, fontWeight = FontWeight.Medium) }
                cloud.redeemError?.let { Text(it, Modifier.padding(top = 8.dp), fontSize = 14.sp, color = Danger) }
                Text("可用模型", Modifier.padding(top = 16.dp), fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Text("点选一个作为默认；在对话里也可以单独为每个对话换。", Modifier.padding(top = 2.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                cloud.tiers.forEach { tier ->
                    RadioRow(
                        selected = cloud.selectedTier == tier.tier,
                        title = tier.displayName + if (tier.isDefault) "　推荐" else "",
                        desc = (if (!tier.available && tier.requiresPaid) "需要充值后使用　" else "") + tier.description,
                        enabled = tier.available,
                        onClick = { chat.selectTier(tier.tier) },
                    )
                }
                PillButton("完成", { sheet = false }, Modifier.padding(top = 14.dp).fillMaxWidth())
            }
        }
    }
}

/** 取额度里最早到期的一项，只显示日期；没有到期时间返回 null。 */
private fun expiryText(balance: AiBalance?): String? =
    balance?.buckets?.filter { it.remaining > 0 }?.mapNotNull { it.expiresAt }?.minOrNull()?.take(10)
