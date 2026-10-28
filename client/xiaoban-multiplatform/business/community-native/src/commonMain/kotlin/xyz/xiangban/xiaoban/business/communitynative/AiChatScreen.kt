package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.AiBalance
import xyz.xiangban.xiaoban.common.community.AiTier
import xyz.xiangban.xiaoban.common.community.ChatTurn
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CloudChatState
import xyz.xiangban.xiaoban.common.community.bucketKindLabel
import xyz.xiangban.xiaoban.common.community.formatAiMoney
import xyz.xiangban.xiaoban.common.link.RemoteApprovalRequest
import xyz.xiangban.xiaoban.common.link.RemoteChatState
import xyz.xiangban.xiaoban.common.link.RemoteToolProgress
import xyz.xiangban.xiaoban.common.link.RemoteTurn

private enum class AiMode(val label: String) { Cloud("云端小伴"), Remote("我的电脑") }

private val StarterPrompts = listOf(
    "帮我理清今天最重要的三件事",
    "用通俗的话解释一个复杂概念",
    "帮我把这段话改得更自然得体",
)

/**
 * 手机端“小伴”页：
 * - 顶部在「云端小伴」与「我的电脑」之间切换
 * - 云端模式：直接对接 `server-ai`，支持流式对话、档位选择（日常/深入）、额度展示与兑换码充值
 * - 电脑模式：遥控已连接的桌面小伴，支持会话列表、快照流式事件、工具进度、待办审批卡片（含高危二次确认）与离线自动降级
 */
@Composable
fun AiChatScreen(env: CommunityEnv, chat: CloudChatController, openRoute: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val cloudState by chat.state.collectAsState()
    val remoteState by env.remoteChat.state.collectAsState()
    var mode by rememberSaveable { mutableStateOf(AiMode.Cloud.name) }
    var showQuota by rememberSaveable { mutableStateOf(false) }
    var cloudInput by rememberSaveable { mutableStateOf("") }
    var remoteInput by rememberSaveable { mutableStateOf("") }
    var copiedId by remember { mutableStateOf<String?>(null) }
    var highRiskConfirmTarget by remember { mutableStateOf<Pair<String, String>?>(null) }

    LaunchedEffect(session.loggedIn) {
        if (session.loggedIn) {
            chat.loadMeta()
            env.remoteChat.refreshPeers()
        } else {
            chat.reset()
        }
    }

    Column(Modifier.fillMaxSize()) {
        PageHeader(
            title = if (mode == AiMode.Cloud.name) "小伴 · 云端" else "小伴 · 遥控",
            actions = {
                if (session.loggedIn) {
                    if (mode == AiMode.Cloud.name) {
                        TextButton(onClick = {
                            showQuota = !showQuota
                            if (showQuota) chat.clearRedeemFeedback()
                        }) {
                            Text(if (showQuota) "收起额度" else "额度与兑换", color = MaterialTheme.colorScheme.onSurface, fontSize = 13.sp)
                        }
                        if (cloudState.messages.isNotEmpty()) {
                            TextButton(onClick = { chat.clearConversation() }) {
                                Text("新对话", color = Brand, fontSize = 13.sp)
                            }
                        }
                    } else {
                        TextButton(onClick = { openRoute(Route.Devices) }) {
                            Text("设备管理", color = MaterialTheme.colorScheme.onSurface, fontSize = 13.sp)
                        }
                    }
                }
            },
        )

        if (!session.loggedIn) {
            UnauthenticatedAiCard { openRoute(Route.Login) }
            return@Column
        }

        // 顶部切换模式：“云端小伴” / “我的电脑”
        Row(
            Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(horizontal = 6.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Chip("云端小伴", selected = mode == AiMode.Cloud.name) { mode = AiMode.Cloud.name }
            val peer = remoteState.selectedPeer
            val peerLabel = if (peer != null) {
                if (peer.online) "${peer.name}·在线" else "${peer.name}·离线"
            } else {
                "我的电脑"
            }
            Chip(peerLabel, selected = mode == AiMode.Remote.name) { mode = AiMode.Remote.name }
        }
        Hairline()

        if (mode == AiMode.Cloud.name) {
            // 云端纯聊天模式
            TierAndQuotaBar(
                state = cloudState,
                onSelectTier = { tier ->
                    if (tier.available) chat.selectTier(tier.tier) else showQuota = true
                },
                onToggleQuota = {
                    showQuota = !showQuota
                    if (!showQuota) chat.clearRedeemFeedback()
                },
                onReloadMeta = chat::loadMeta,
            )
            Hairline()

            if (showQuota) {
                QuotaAndRedeemPanel(
                    state = cloudState,
                    onRedeem = chat::redeem,
                    onRefresh = chat::loadMeta,
                )
                Hairline()
            }

            Box(Modifier.weight(1f).fillMaxWidth()) {
                if (cloudState.messages.isEmpty()) {
                    WelcomeGuide(onPickPrompt = { prompt -> chat.send(prompt) })
                } else {
                    CloudMessageList(
                        messages = cloudState.messages,
                        canCopy = env.platform != null,
                        copiedId = copiedId,
                        onCopy = { turn ->
                            env.platform?.copy(turn.text)
                            copiedId = "cloud-${turn.id}"
                        },
                        onRetry = chat::retryLast,
                        onOpenQuota = { showQuota = true },
                    )
                }
            }

            Hairline()
            ComposerBar(
                input = cloudInput,
                onInputChange = { cloudInput = it },
                sending = cloudState.sending,
                placeholder = "发给云端小伴…",
                onSend = {
                    val text = cloudInput.trim()
                    if (text.isNotEmpty()) {
                        cloudInput = ""
                        chat.send(text)
                    }
                },
                onStop = chat::stop,
            )
        } else {
            // 电脑遥控模式
            RemoteChatView(
                env = env,
                state = remoteState,
                input = remoteInput,
                onInputChange = { remoteInput = it },
                canCopy = env.platform != null,
                copiedId = copiedId,
                onCopy = { turn ->
                    env.platform?.copy(turn.text)
                    copiedId = "remote-${turn.id}"
                },
                onSwitchToCloud = { mode = AiMode.Cloud.name },
                onOpenDevices = { openRoute(Route.Devices) },
                onRequestApprovalDecision = { approvalId, choice, isHighRisk ->
                    if (isHighRisk && choice == "once") {
                        highRiskConfirmTarget = approvalId to choice
                    } else {
                        env.remoteChat.answerApproval(approvalId, choice)
                    }
                },
            )
        }
    }

    // 高危审批二次确认弹窗
    highRiskConfirmTarget?.let { (approvalId, choice) ->
        Box(
            Modifier
                .fillMaxSize()
                .background(MaterialTheme.colorScheme.background.copy(alpha = 0.5f))
                .clickable(indication = null, interactionSource = null) { highRiskConfirmTarget = null },
            contentAlignment = Alignment.Center,
        ) {
            Column(
                Modifier
                    .padding(24.dp)
                    .clip(CardShape)
                    .background(MaterialTheme.colorScheme.surface)
                    .border(1.dp, Brand, CardShape)
                    .padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                Text("⚠️ 高危操作二次确认", fontSize = 16.sp, fontWeight = FontWeight.Bold, color = Brand)
                Text(
                    "即将批准执行高危操作（如系统命令、外部修改或文件变更）。由于来自远程手机授权，请再次核对是否确定放行？",
                    fontSize = 13.5.sp,
                    lineHeight = 20.sp,
                )
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    TextButton(onClick = { highRiskConfirmTarget = null }) { Text("取消") }
                    BrandButton("确定放行", onClick = {
                        highRiskConfirmTarget = null
                        env.remoteChat.answerApproval(approvalId, choice)
                    })
                }
            }
        }
    }
}

@Composable
private fun RemoteChatView(
    env: CommunityEnv,
    state: RemoteChatState,
    input: String,
    onInputChange: (String) -> Unit,
    canCopy: Boolean,
    copiedId: String?,
    onCopy: (RemoteTurn) -> Unit,
    onSwitchToCloud: () -> Unit,
    onOpenDevices: () -> Unit,
    onRequestApprovalDecision: (String, String, Boolean) -> Unit,
) {
    if (state.peers.isEmpty()) {
        Column(
            Modifier.fillMaxSize().padding(Gutter),
            verticalArrangement = Arrangement.Center,
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Column(
                Modifier.fillMaxWidth().clip(CardShape).background(MaterialTheme.colorScheme.surface).padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                Text("尚未连接电脑", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                Text(
                    "在电脑端小伴打开「设置 → 连接与网络」，点击「连接手机」生成二维码或配对串，即可在此绑定并遥控桌面小伴。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    fontSize = 13.5.sp,
                )
                BrandButton("去配对电脑", onClick = onOpenDevices, modifier = Modifier.fillMaxWidth().padding(top = 4.dp))
            }
        }
        return
    }

    // 离线降级：电脑离线时明确告知，并提供切回云端小伴入口
    if (!state.isPeerOnline) {
        Column(Modifier.fillMaxSize()) {
            Box(
                Modifier
                    .fillMaxWidth()
                    .background(MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.5f))
                    .padding(horizontal = Gutter, vertical = 12.dp),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.weight(1f)) {
                        Text("电脑当前不在线", fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = Brand)
                        Text("电脑小伴已离线，无法接收远程指令。你可以直接使用云端小伴对话。", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    BrandButton("切换至云端", onClick = onSwitchToCloud)
                }
            }
            Hairline()

            Box(Modifier.weight(1f).fillMaxWidth().padding(Gutter), contentAlignment = Alignment.Center) {
                Text("电脑离线中，等待电脑小伴上线后自动恢复遥控…", color = LocalExtraColors.current.ink3, fontSize = 13.sp)
            }

            Hairline()
            Row(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(Gutter), horizontalArrangement = Arrangement.Center) {
                BrandButton("切换到云端小伴直接提问", onClick = onSwitchToCloud, modifier = Modifier.fillMaxWidth())
            }
        }
        return
    }

    // 电脑在线：展示会话切换栏、会话内容、工具进度、审批卡片与输入框
    val currentSession = state.sessions.firstOrNull { it.id == state.selectedSessionId }
    val isDelegated = state.selectedSessionId?.startsWith("del-") == true || currentSession?.title?.startsWith("[") == true

    Column(Modifier.fillMaxSize()) {
        // 会话切换栏
        RemoteSessionsBar(
            sessions = state.sessions,
            selectedId = state.selectedSessionId,
            loading = state.loadingSessions,
            onSelect = env.remoteChat::selectSession,
            onNew = env.remoteChat::newSession,
            onRefresh = { env.remoteChat.fetchSessions() },
        )
        Hairline()

        if (isDelegated && currentSession != null) {
            Box(
                Modifier
                    .fillMaxWidth()
                    .background(MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.35f))
                    .padding(horizontal = Gutter, vertical = 8.dp),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "外部 Coding Agent · ${currentSession.title}",
                        fontSize = 12.5.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = Brand,
                    )
                    Text(
                        if (currentSession.state == "running") "后台执行中…" else "已结束",
                        fontSize = 12.sp,
                        color = LocalExtraColors.current.ink3,
                    )
                }
            }
            Hairline()
        }

        Box(Modifier.weight(1f).fillMaxWidth()) {
            if (state.messages.isEmpty()) {
                Box(Modifier.fillMaxSize().padding(Gutter), contentAlignment = Alignment.Center) {
                    Text(
                        if (state.selectedSessionId == null) "新电脑对话：发一条消息，电脑端将新建会话并执行" else if (isDelegated) "正在连接并读取外部 Agent 运行输出…" else "该会话暂无消息，直接发送开始提问",
                        color = LocalExtraColors.current.ink3,
                        fontSize = 13.sp,
                    )
                }
            } else {
                RemoteMessageList(
                    messages = state.messages,
                    toolProgress = state.toolProgress,
                    canCopy = canCopy,
                    copiedId = copiedId,
                    onCopy = onCopy,
                )
            }
        }

        // 待办审批卡片
        state.pendingApproval?.let { pending ->
            ApprovalCard(
                pending = pending,
                onDecision = { choice ->
                    onRequestApprovalDecision(pending.approvalId, choice, pending.isHighRisk)
                },
            )
            Hairline()
        }

        Hairline()
        ComposerBar(
            input = input,
            onInputChange = onInputChange,
            sending = state.sending,
            placeholder = if (isDelegated) "给外部 Agent 补充指令…" else "发给电脑小伴…",
            onSend = {
                val text = input.trim()
                if (text.isNotEmpty()) {
                    onInputChange("")
                    env.remoteChat.send(text)
                }
            },
            onStop = {},
        )
    }
}

@Composable
private fun RemoteSessionsBar(
    sessions: List<xyz.xiangban.xiaoban.common.link.RemoteSessionSummary>,
    selectedId: String?,
    loading: Boolean,
    onSelect: (String) -> Unit,
    onNew: () -> Unit,
    onRefresh: () -> Unit,
) {
    LazyRow(
        Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(horizontal = 6.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        item {
            Chip("+ 新对话", selected = selectedId == null, onClick = onNew)
        }
        items(sessions, key = { it.id }) { s ->
            val label = if (s.state == "running") "${s.title}·运行中" else s.title
            Chip(label, selected = s.id == selectedId) { onSelect(s.id) }
        }
        if (sessions.isEmpty()) {
            item {
                Text(if (loading) "加载电脑会话中…" else "点击刷新获取会话", fontSize = 12.sp, color = LocalExtraColors.current.ink3, modifier = Modifier.clickable(onClick = onRefresh).padding(8.dp))
            }
        }
    }
}

@Composable
private fun RemoteMessageList(
    messages: List<RemoteTurn>,
    toolProgress: List<RemoteToolProgress>,
    canCopy: Boolean,
    copiedId: String?,
    onCopy: (RemoteTurn) -> Unit,
) {
    val listState = rememberLazyListState()
    val lastLen = messages.lastOrNull()?.text?.length ?: 0
    val progressLen = toolProgress.size
    LaunchedEffect(messages.size, lastLen, progressLen) {
        if (messages.isNotEmpty()) {
            listState.scrollToItem(messages.lastIndex)
        }
    }

    LazyColumn(
        state = listState,
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(Gutter),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        items(messages, key = { it.id }) { turn ->
            RemoteChatBubble(
                turn = turn,
                canCopy = canCopy,
                copied = copiedId == "remote-${turn.id}",
                onCopy = { onCopy(turn) },
            )
        }
        if (toolProgress.isNotEmpty()) {
            item {
                Column(
                    Modifier.fillMaxWidth().clip(CardShape).background(MaterialTheme.colorScheme.surface).border(0.5.dp, LocalExtraColors.current.hairline, CardShape).padding(10.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                ) {
                    Text("电脑执行进展：", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    toolProgress.forEach { t ->
                        val statusText = when {
                            t.running -> "正在运行…"
                            t.isError == true -> "执行报错"
                            else -> "已完成"
                        }
                        Text("• ${t.name}：$statusText", fontSize = 13.sp, color = if (t.isError == true) Brand else MaterialTheme.colorScheme.onSurface)
                    }
                }
            }
        }
    }
}

@Composable
private fun RemoteChatBubble(
    turn: RemoteTurn,
    canCopy: Boolean,
    copied: Boolean,
    onCopy: () -> Unit,
) {
    val isUser = turn.role == "user"
    val bubbleShape = RoundedCornerShape(
        topStart = 12.dp,
        topEnd = 12.dp,
        bottomStart = if (isUser) 12.dp else 4.dp,
        bottomEnd = if (isUser) 4.dp else 12.dp,
    )
    Box(
        Modifier.fillMaxWidth(),
        contentAlignment = if (isUser) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        Column(
            Modifier
                .widthIn(max = 320.dp)
                .clip(bubbleShape)
                .background(if (isUser) Brand else MaterialTheme.colorScheme.surface)
                .then(
                    if (isUser) Modifier
                    else Modifier.border(0.5.dp, LocalExtraColors.current.hairline, bubbleShape)
                )
                .padding(horizontal = 14.dp, vertical = 10.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            when {
                turn.streaming && turn.text.isEmpty() ->
                    Text("电脑正在处理…", color = LocalExtraColors.current.ink3, fontSize = 14.sp)
                turn.text.isNotEmpty() ->
                    Text(
                        text = if (turn.streaming) "${turn.text} ▍" else turn.text,
                        color = if (isUser) Color.White else MaterialTheme.colorScheme.onSurface,
                        fontSize = 15.sp,
                        lineHeight = 22.sp,
                    )
            }
            if (turn.error != null) {
                Text(turn.error.orEmpty(), color = Brand, fontSize = 13.sp)
            } else if (!isUser && !turn.streaming && turn.text.isNotEmpty() && canCopy) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    Text(
                        text = if (copied) "已复制" else "复制",
                        modifier = Modifier.clickable(onClick = onCopy).padding(top = 2.dp),
                        color = LocalExtraColors.current.ink3,
                        fontSize = 12.sp,
                    )
                }
            }
        }
    }
}

/** 审批卡片：展示电脑端发过来的待确认事项，含高危提醒与多选项按钮。 */
@Composable
private fun ApprovalCard(
    pending: RemoteApprovalRequest,
    onDecision: (String) -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .background(if (pending.isHighRisk) MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.35f) else MaterialTheme.colorScheme.surface)
            .border(if (pending.isHighRisk) 1.dp else 0.5.dp, if (pending.isHighRisk) Brand else LocalExtraColors.current.hairline)
            .padding(Gutter),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            if (pending.isHighRisk) {
                Text("⚠️ 高危操作待确认", fontSize = 14.sp, fontWeight = FontWeight.Bold, color = Brand)
            } else {
                Text("操作确认", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
            }
        }
        Text(pending.title, fontSize = 14.sp, fontWeight = FontWeight.Medium)
        if (pending.detail.isNotEmpty()) {
            Text(
                pending.detail.toString(),
                fontSize = 12.sp,
                color = LocalExtraColors.current.ink3,
                maxLines = 4,
            )
        }
        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            pending.options.forEach { opt ->
                if (opt.id == "once" || opt.id == "always") {
                    BrandButton(opt.label, onClick = { onDecision(opt.id) }, modifier = Modifier.weight(1f))
                } else {
                    OutlinedButton(onClick = { onDecision(opt.id) }, shape = CardShape, modifier = Modifier.weight(1f)) {
                        Text(opt.label)
                    }
                }
            }
        }
    }
}

// ───────────── 云端纯聊天子组件 ─────────────

@Composable
private fun UnauthenticatedAiCard(login: () -> Unit) {
    Column(
        Modifier.fillMaxSize().padding(Gutter),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Column(
            Modifier.fillMaxWidth().clip(CardShape).background(MaterialTheme.colorScheme.surface).padding(20.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Text("登录即可与小伴对话", fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
            Text(
                "使用统一小伴账号直接开启对话，新用户享有免费试用额度，无需自己准备模型密钥。电脑在线时还可远程遥控电脑小伴。",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontSize = 14.sp,
            )
            BrandButton("登录小伴账号", onClick = login, modifier = Modifier.fillMaxWidth().padding(top = 4.dp))
        }
    }
}

@Composable
private fun TierAndQuotaBar(
    state: CloudChatState,
    onSelectTier: (AiTier) -> Unit,
    onToggleQuota: () -> Unit,
    onReloadMeta: () -> Unit,
) {
    Row(
        Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(horizontal = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically) {
            if (state.tiers.isEmpty()) {
                Chip("日常", selected = true) {}
            } else {
                state.tiers.forEach { tier ->
                    val label = if (!tier.available && tier.requiresPaid) "${tier.displayName}·需充值" else tier.displayName
                    Chip(label, selected = state.selectedTier == tier.tier) { onSelectTier(tier) }
                }
            }
        }
        val balance = state.balance
        when {
            state.loadingMeta && balance == null ->
                MetaText("读取额度…", Modifier.padding(horizontal = 10.dp))
            balance != null -> {
                val exhausted = balance.total <= 0L && !balance.trial.available
                val text = when {
                    balance.total > 0L -> "可用 ${formatAiMoney(balance.total, balance.quotaPerUnit)}"
                    balance.trial.available -> "试用 ${formatAiMoney(balance.trial.quota, balance.quotaPerUnit)}"
                    else -> "额度已用完 · 兑换"
                }
                Text(
                    text,
                    modifier = Modifier.clickable(onClick = onToggleQuota).padding(horizontal = 10.dp, vertical = 8.dp),
                    color = if (exhausted) Brand else LocalExtraColors.current.ink3,
                    fontSize = 12.sp,
                    fontWeight = if (exhausted) FontWeight.SemiBold else FontWeight.Normal,
                )
            }
            state.metaError != null ->
                Text(
                    "刷新额度",
                    modifier = Modifier.clickable(onClick = onReloadMeta).padding(horizontal = 10.dp, vertical = 8.dp),
                    color = Brand,
                    fontSize = 12.sp,
                )
        }
    }
}

@Composable
private fun QuotaAndRedeemPanel(
    state: CloudChatState,
    onRedeem: (String) -> Unit,
    onRefresh: () -> Unit,
) {
    var code by rememberSaveable { mutableStateOf("") }
    val balance: AiBalance? = state.balance

    Column(
        Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(Gutter),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        state.activeTier?.description?.takeIf { it.isNotBlank() }?.let { desc ->
            Text("当前模式（${state.activeTier?.displayName.orEmpty()}）：$desc", color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 13.sp)
        }
        if (balance != null) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Column {
                    Text(
                        "可用总额：${formatAiMoney(balance.total, balance.quotaPerUnit)}",
                        fontSize = 15.sp,
                        fontWeight = FontWeight.SemiBold,
                    )
                    MetaText("其中付费额度：${formatAiMoney(balance.paid, balance.quotaPerUnit)}")
                }
                TextButton(onClick = onRefresh, enabled = !state.loadingMeta) {
                    Text(if (state.loadingMeta) "刷新中…" else "刷新", fontSize = 13.sp)
                }
            }
            if (balance.total <= 0L && balance.trial.available) {
                Text(
                    "首次发送消息将自动开通并赠送 ${formatAiMoney(balance.trial.quota, balance.quotaPerUnit)} 试用额度（${balance.trial.days} 天有效）。",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    fontSize = 12.sp,
                )
            }
            balance.buckets.forEach { b ->
                MetaText("${bucketKindLabel(b.kind)}：剩余 ${formatAiMoney(b.remaining, balance.quotaPerUnit)} / 共 ${formatAiMoney(b.total, balance.quotaPerUnit)}")
            }
        } else if (state.metaError != null) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(state.metaError.orEmpty(), color = Brand, fontSize = 13.sp, modifier = Modifier.weight(1f))
                OutlinedButton(onClick = onRefresh, shape = CardShape) { Text("重试") }
            }
        }

        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            OutlinedTextField(
                value = code,
                onValueChange = { code = it },
                modifier = Modifier.weight(1f),
                label = { Text("兑换码（XXXX-XXXX-XXXX-XXXX）") },
                singleLine = true,
            )
            BrandButton(
                text = if (state.redeeming) "兑换中…" else "兑换",
                onClick = { onRedeem(code) },
                enabled = !state.redeeming && code.isNotBlank(),
            )
        }
        state.redeemMessage?.let { Text(it, color = MaterialTheme.colorScheme.onSurface, fontSize = 13.sp, fontWeight = FontWeight.Medium) }
        state.redeemError?.let { Text(it, color = Brand, fontSize = 13.sp) }
    }
}

@Composable
private fun WelcomeGuide(onPickPrompt: (String) -> Unit) {
    Column(
        Modifier.fillMaxSize().padding(Gutter),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Column(
            Modifier.fillMaxWidth().clip(CardShape).background(MaterialTheme.colorScheme.surface).padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text("你好，我是云端小伴", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
            Text(
                "当前为云端纯聊天模式，不依赖电脑在线。你可以随时提问、整理待办或润色文字。",
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontSize = 13.sp,
            )
        }
        MetaText("试试这样问我：")
        StarterPrompts.forEach { prompt ->
            Row(
                Modifier
                    .fillMaxWidth()
                    .clip(CardShape)
                    .background(MaterialTheme.colorScheme.surface)
                    .border(0.5.dp, LocalExtraColors.current.hairline, CardShape)
                    .clickable { onPickPrompt(prompt) }
                    .padding(horizontal = 14.dp, vertical = 12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(prompt, Modifier.weight(1f), fontSize = 14.sp)
                Text("›", color = LocalExtraColors.current.ink3, fontSize = 16.sp)
            }
        }
    }
}

@Composable
private fun CloudMessageList(
    messages: List<ChatTurn>,
    canCopy: Boolean,
    copiedId: String?,
    onCopy: (ChatTurn) -> Unit,
    onRetry: () -> Unit,
    onOpenQuota: () -> Unit,
) {
    val listState = rememberLazyListState()
    val lastTextLen = messages.lastOrNull()?.text?.length ?: 0
    LaunchedEffect(messages.size, lastTextLen) {
        if (messages.isNotEmpty()) {
            listState.scrollToItem(messages.lastIndex)
        }
    }

    LazyColumn(
        state = listState,
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(Gutter),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        items(messages, key = { it.id }) { turn ->
            CloudChatBubble(
                turn = turn,
                canCopy = canCopy,
                copied = copiedId == "cloud-${turn.id}",
                onCopy = { onCopy(turn) },
                onRetry = onRetry,
                onOpenQuota = onOpenQuota,
            )
        }
    }
}

@Composable
private fun CloudChatBubble(
    turn: ChatTurn,
    canCopy: Boolean,
    copied: Boolean,
    onCopy: () -> Unit,
    onRetry: () -> Unit,
    onOpenQuota: () -> Unit,
) {
    val isUser = turn.role == "user"
    val bubbleShape = RoundedCornerShape(
        topStart = 12.dp,
        topEnd = 12.dp,
        bottomStart = if (isUser) 12.dp else 4.dp,
        bottomEnd = if (isUser) 4.dp else 12.dp,
    )
    Box(
        Modifier.fillMaxWidth(),
        contentAlignment = if (isUser) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        Column(
            Modifier
                .widthIn(max = 320.dp)
                .clip(bubbleShape)
                .background(if (isUser) Brand else MaterialTheme.colorScheme.surface)
                .then(
                    if (isUser) Modifier
                    else Modifier.border(0.5.dp, LocalExtraColors.current.hairline, bubbleShape)
                )
                .padding(horizontal = 14.dp, vertical = 10.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            when {
                turn.streaming && turn.text.isEmpty() ->
                    Text("思考中…", color = LocalExtraColors.current.ink3, fontSize = 14.sp)
                turn.text.isNotEmpty() ->
                    Text(
                        text = if (turn.streaming) "${turn.text} ▍" else turn.text,
                        color = if (isUser) Color.White else MaterialTheme.colorScheme.onSurface,
                        fontSize = 15.sp,
                        lineHeight = 22.sp,
                    )
            }
            if (turn.error != null) {
                Text(turn.error.orEmpty(), color = Brand, fontSize = 13.sp)
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedButton(onClick = onRetry, shape = CardShape) { Text("重试", fontSize = 12.sp) }
                    if (turn.error.orEmpty().contains("额度") || turn.error.orEmpty().contains("兑换")) {
                        TextButton(onClick = onOpenQuota) { Text("去兑换额度", color = Brand, fontSize = 12.sp) }
                    }
                }
            } else if (!isUser && !turn.streaming && turn.text.isNotEmpty() && canCopy) {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                    Text(
                        text = if (copied) "已复制" else "复制",
                        modifier = Modifier.clickable(onClick = onCopy).padding(top = 2.dp),
                        color = LocalExtraColors.current.ink3,
                        fontSize = 12.sp,
                    )
                }
            }
        }
    }
}

@Composable
private fun ComposerBar(
    input: String,
    onInputChange: (String) -> Unit,
    sending: Boolean,
    placeholder: String = "发给云端小伴…",
    onSend: () -> Unit,
    onStop: () -> Unit,
) {
    Row(
        Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).padding(horizontal = Gutter, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        OutlinedTextField(
            value = input,
            onValueChange = onInputChange,
            modifier = Modifier.weight(1f),
            placeholder = { Text(placeholder) },
            maxLines = 4,
        )
        if (sending) {
            OutlinedButton(onClick = onStop, shape = CardShape) { Text("停止") }
        } else {
            BrandButton(
                text = "发送",
                onClick = onSend,
                enabled = input.trim().isNotEmpty(),
            )
        }
    }
}
