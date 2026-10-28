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

private val StarterPrompts = listOf(
    "帮我理清今天最重要的三件事",
    "用通俗的话解释一个复杂概念",
    "帮我把这段话改得更自然得体",
)

/**
 * 手机端“小伴”页（云端纯聊天）：
 * 直接对接 `server-ai`，支持流式对话、档位选择（日常/深入）、额度展示与兑换码充值，不依赖电脑在线。
 */
@Composable
fun AiChatScreen(env: CommunityEnv, chat: CloudChatController, login: () -> Unit) {
    val session by env.session.state.collectAsState()
    val state by chat.state.collectAsState()
    var showQuota by rememberSaveable { mutableStateOf(false) }
    var input by rememberSaveable { mutableStateOf("") }
    var copiedId by remember { mutableStateOf<Long?>(null) }

    LaunchedEffect(session.loggedIn) {
        if (session.loggedIn) chat.loadMeta() else chat.reset()
    }

    Column(Modifier.fillMaxSize()) {
        PageHeader(
            title = "小伴 · 云端",
            actions = {
                if (session.loggedIn) {
                    TextButton(onClick = {
                        showQuota = !showQuota
                        if (showQuota) chat.clearRedeemFeedback()
                    }) {
                        Text(if (showQuota) "收起额度" else "额度与兑换", color = MaterialTheme.colorScheme.onSurface, fontSize = 13.sp)
                    }
                    if (state.messages.isNotEmpty()) {
                        TextButton(onClick = { chat.clearConversation() }) {
                            Text("新对话", color = Brand, fontSize = 13.sp)
                        }
                    }
                }
            },
        )

        if (!session.loggedIn) {
            UnauthenticatedAiCard(login)
            return@Column
        }

        TierAndQuotaBar(
            state = state,
            onSelectTier = { tier ->
                if (tier.available) {
                    chat.selectTier(tier.tier)
                } else {
                    showQuota = true
                }
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
                state = state,
                onRedeem = chat::redeem,
                onRefresh = chat::loadMeta,
            )
            Hairline()
        }

        Box(Modifier.weight(1f).fillMaxWidth()) {
            if (state.messages.isEmpty()) {
                WelcomeGuide(onPickPrompt = { prompt -> chat.send(prompt) })
            } else {
                MessageList(
                    messages = state.messages,
                    canCopy = env.platform != null,
                    copiedId = copiedId,
                    onCopy = { turn ->
                        env.platform?.copy(turn.text)
                        copiedId = turn.id
                    },
                    onRetry = chat::retryLast,
                    onOpenQuota = { showQuota = true },
                )
            }
        }

        Hairline()
        ComposerBar(
            input = input,
            onInputChange = { input = it },
            sending = state.sending,
            onSend = {
                val text = input.trim()
                if (text.isNotEmpty()) {
                    input = ""
                    chat.send(text)
                }
            },
            onStop = chat::stop,
        )
    }
}

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
            Text("登录即可与云端小伴对话", fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
            Text(
                "使用统一小伴账号直接开启流式对话，新用户享有免费试用额度，无需自己准备模型密钥。电脑不在线时也能随时提问。",
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
private fun MessageList(
    messages: List<ChatTurn>,
    canCopy: Boolean,
    copiedId: Long?,
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
            ChatBubble(
                turn = turn,
                canCopy = canCopy,
                copied = copiedId == turn.id,
                onCopy = { onCopy(turn) },
                onRetry = onRetry,
                onOpenQuota = onOpenQuota,
            )
        }
    }
}

@Composable
private fun ChatBubble(
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
            placeholder = { Text("发给云端小伴…") },
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
