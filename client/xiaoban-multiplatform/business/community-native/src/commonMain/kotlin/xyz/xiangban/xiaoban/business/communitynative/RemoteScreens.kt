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
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.link.PeerSummary
import xyz.xiangban.xiaoban.common.link.RemoteApprovalRequest
import xyz.xiangban.xiaoban.common.link.RemoteSessionSummary
import xyz.xiangban.xiaoban.common.link.RemoteToolProgress
import xyz.xiangban.xiaoban.common.link.RemoteTurn

private val SetupSteps = listOf(
    "在电脑上安装小伴并登录" to "用和手机相同的账号登录。",
    "打开“设置 → 手机遥控”" to "默认关闭；打开后，这台电脑才允许被手机遥控。",
    "回到这里点开电脑" to "电脑在线时会出现在列表里，点一下就能开始遥控。",
)

/** 毫秒时间戳 → “刚刚 / N 分钟前 / N 小时前 / N 天前”。 */
@OptIn(kotlin.time.ExperimentalTime::class)
private fun relativeTime(ms: Long): String {
    if (ms <= 0) return ""
    val diff = (kotlin.time.Clock.System.now().toEpochMilliseconds() - ms) / 1000
    return when {
        diff < 60 -> "刚刚"
        diff < 3600 -> "${diff / 60} 分钟前"
        diff < 86400 -> "${diff / 3600} 小时前"
        else -> "${diff / 86400} 天前"
    }
}

private fun PeerSummary.statusText(): String = if (online) "在线" else "离线"

// ───────────── 我的电脑 ─────────────

/** 同账号下的电脑列表；没有电脑时给出开启步骤。 */
@Composable
fun DevicesScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()
    var searching by remember { mutableStateOf(false) }

    LaunchedEffect(session.loggedIn) { if (session.loggedIn) env.remoteChat.refreshPeers() }
    val refresh: () -> Unit = {
        if (!searching) scope.launch { searching = true; env.remoteChat.refreshPeers(); searching = false }
    }

    Column(Modifier.fillMaxSize()) {
        ScreenBar("我的电脑", onBack = back)
        when {
            !session.loggedIn -> Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                Column(Modifier.padding(horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                    EmptyState(XIcons.Monitor, "先登录账号", "电脑和手机登录同一个小伴账号，电脑就会自动出现在这里。")
                    PillButton("登录小伴账号", { open(Route.Login) }, Modifier.padding(top = 20.dp).fillMaxWidth())
                }
            }
            remote.peers.isEmpty() -> Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(vertical = 24.dp)) {
                EmptyState(XIcons.Monitor, "还没有可遥控的电脑", "用手机说一句话，电脑上的小伴就开始干活，做完再通知你。")
                Column(Modifier.padding(top = 24.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                    SetupSteps.forEachIndexed { i, (title, desc) ->
                        if (i > 0) Hairline()
                        Row(Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            Box(Modifier.size(28.dp).clip(CircleShape).background(MaterialTheme.colorScheme.onSurface), contentAlignment = Alignment.Center) {
                                Text("${i + 1}", color = MaterialTheme.colorScheme.surface, fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                            }
                            Column(Modifier.weight(1f)) {
                                Text(title, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                Text(desc, Modifier.padding(top = 2.dp), fontSize = 13.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    }
                }
                PillButton(if (searching) "查找中…" else "重新查找", refresh, Modifier.padding(start = 16.dp, end = 16.dp, top = 20.dp).fillMaxWidth(), enabled = !searching, icon = XIcons.Refresh)
                GhostPillButton("先用云端小伴聊聊", { open(Route.AiChat) }, Modifier.padding(start = 16.dp, end = 16.dp, top = 10.dp).fillMaxWidth(), height = 44.dp)
            }
            else -> Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(top = 12.dp, bottom = 24.dp)) {
                Text("登录同一个账号的电脑会自动出现在这里，不需要扫码。点一台在线的电脑，就能用手机遥控它上面的小伴。", Modifier.padding(horizontal = 16.dp), fontSize = 13.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Column(Modifier.padding(top = 12.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                    remote.peers.forEachIndexed { i, peer ->
                        if (i > 0) Hairline(Modifier.padding(start = 16.dp))
                        Row(
                            Modifier.fillMaxWidth().clickable { open(Route.RemoteList(peer.id)) }.padding(horizontal = 16.dp, vertical = 14.dp),
                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            IconBox(XIcons.Monitor, 44.dp, 12.dp)
                            Column(Modifier.weight(1f)) {
                                Text(peer.name, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                Text(if (peer.online) "点一下开始遥控" else "不在线，上线后自动恢复", Modifier.padding(top = 3.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                            StatusDot(peer.online, peer.statusText())
                            XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                        }
                    }
                }
                Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).padding(horizontal = 14.dp, vertical = 12.dp)) {
                    Text("没看到你的电脑？", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                    Text("1. 电脑上的小伴已登录同一个账号\n2. 电脑小伴里“设置 → 手机遥控”已打开（默认关闭）\n3. 电脑已联网，小伴正在运行", Modifier.padding(top = 6.dp), fontSize = 13.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    LinkText(if (searching) "查找中…" else "重新查找 ›", refresh, Modifier.padding(top = 4.dp))
                }
                Text("当前账号：${session.user?.username ?: ""}。手机和电脑之间的消息经中转服务转发。", Modifier.padding(horizontal = 16.dp, vertical = 16.dp), fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

// ───────────── 某台电脑上的会话列表 ─────────────

@Composable
fun RemoteListScreen(env: CommunityEnv, peerId: String, back: () -> Unit, open: (Route) -> Unit) {
    val remote by env.remoteChat.state.collectAsState()
    val peer = remote.peers.firstOrNull { it.id == peerId }
    var menu by rememberSaveable { mutableStateOf(false) }

    LaunchedEffect(peerId) {
        if (remote.peers.none { it.id == peerId }) env.remoteChat.refreshPeers()
        if (env.remoteChat.state.value.selectedPeerId != peerId) env.remoteChat.selectPeer(peerId) else env.remoteChat.fetchSessions(peerId)
    }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar(
                peer?.name ?: "电脑", onBack = back,
                subtitle = if (peer == null) "不在账号的电脑列表里" else null,
                trailing = { IconTap(XIcons.More, "更多", { menu = true }) },
                subtitleContent = if (peer != null) ({ StatusDot(peer.online, peer.statusText(), fontSize = 11) }) else null,
            )
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(top = 12.dp, bottom = 24.dp)) {
                if (peer == null) {
                    Banner("这台电脑已退出账号或被移除，它重新登录同一个账号后会再次出现。", Modifier.padding(horizontal = 16.dp), warn = true, icon = XIcons.Warning, actionText = "回到我的电脑", onAction = back)
                } else if (!peer.online) {
                    Banner("${peer.name}现在不在线，暂时不能遥控。电脑上线后会自动恢复。", Modifier.padding(horizontal = 16.dp), warn = true, icon = XIcons.Monitor, actionText = "改用云端小伴", onAction = { open(Route.AiChat) })
                } else {
                    PillButton("新对话", { env.remoteChat.newSession(); open(Route.RemoteChat(peerId)) }, Modifier.padding(horizontal = 16.dp).fillMaxWidth(), icon = XIcons.Plus)
                    Text("电脑上的对话", Modifier.padding(start = 16.dp, end = 16.dp, top = 20.dp, bottom = 6.dp), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    if (remote.sessions.isEmpty()) {
                        Text(if (remote.loadingSessions) "读取电脑上的对话…" else "电脑上还没有对话。", Modifier.padding(horizontal = 16.dp, vertical = 8.dp), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    } else {
                        Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                            remote.sessions.forEachIndexed { i, s -> SessionRow(s, first = i == 0) { env.remoteChat.selectSession(s.id); open(Route.RemoteChat(peerId)) } }
                        }
                    }
                    Text("这里是电脑小伴里的对话，手机上发的消息也会出现在电脑上。电脑关机或断网时，这里会提示不在线。", Modifier.padding(horizontal = 16.dp, vertical = 12.dp), fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
        OverlayTransition(menu) { panel ->
            BottomSheet({ menu = false }, panel) {
                Text(peer?.name ?: "电脑", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                SheetAction(XIcons.Close, "从账号中移除这台电脑", "它再次启动并开启手机遥控后会重新出现") {
                    menu = false
                    env.remoteChat.removePeer(peerId)
                    back()
                }
                GhostPillButton("取消", { menu = false }, Modifier.padding(top = 8.dp).fillMaxWidth())
            }
        }
    }
}

@Composable
private fun SessionRow(s: RemoteSessionSummary, first: Boolean, onClick: () -> Unit) {
    if (!first) Hairline(Modifier.padding(start = 16.dp))
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 16.dp, vertical = 14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(s.title.ifBlank { "未命名对话" }, Modifier.weight(1f), fontSize = 15.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        if (s.state == "running") {
            Text("执行中", Modifier.clip(CircleShape).background(MaterialTheme.colorScheme.onSurface).padding(horizontal = 8.dp, vertical = 3.dp), color = MaterialTheme.colorScheme.surface, fontSize = 11.sp, fontWeight = FontWeight.SemiBold)
        }
        Text(relativeTime(s.updatedAt), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

// ───────────── 遥控会话页 ─────────────

private fun toolLabel(name: String): String = when (name.lowercase()) {
    "bash", "shell", "run_command", "exec", "execute_command", "command" -> "运行命令"
    "read", "read_file", "view" -> "读取文件"
    "write", "write_file", "edit", "edit_file", "apply_patch", "str_replace" -> "修改文件"
    "grep", "glob", "search", "find", "list_files", "ls" -> "搜索文件"
    else -> name
}

@Composable
fun RemoteChatScreen(env: CommunityEnv, peerId: String, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()
    val peer = remote.peers.firstOrNull { it.id == peerId }
    val online = peer?.online == true
    var input by rememberSaveable { mutableStateOf("") }
    var highRisk by remember { mutableStateOf<Pair<String, String>?>(null) }
    var lastHighRisk by remember { mutableStateOf<Pair<String, String>?>(null) }
    var confirmed by remember { mutableStateOf(false) }

    LaunchedEffect(peerId) {
        if (remote.peers.none { it.id == peerId }) env.remoteChat.refreshPeers()
        if (env.remoteChat.state.value.selectedPeerId != peerId) env.remoteChat.selectPeer(peerId)
    }

    val current = remote.sessions.firstOrNull { it.id == remote.selectedSessionId }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().imePadding()) {
            ScreenBar(
                current?.title?.ifBlank { null } ?: "新对话", onBack = back,
                subtitleContent = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text((peer?.name ?: "电脑") + " · ", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1)
                        StatusDot(online, if (online) "在线" else "离线", fontSize = 11)
                    }
                },
            )
            if (!session.loggedIn) {
                Banner("登录已过期，请重新登录", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Lock, actionText = "去登录", onAction = { open(Route.Login) })
            } else if (peer == null) {
                Banner("这台电脑已退出账号或被移除", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Warning, actionText = "回到我的电脑", onAction = back)
            } else if (!online) {
                Banner("${peer.name}现在不在线，暂时不能遥控。电脑上线后会自动恢复。", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Monitor, actionText = "改用云端小伴", onAction = { open(Route.AiChat) })
            }
            remote.error?.let { Banner(it, Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true) }

            Box(Modifier.weight(1f).fillMaxWidth()) {
                RemoteMessages(remote.messages, remote.toolProgress, remote.sending, remote.pendingApproval != null, online, remote.selectedSessionId == null)
            }

            remote.pendingApproval?.let { pending ->
                ApprovalCard(pending, peer?.name ?: "电脑") { choice ->
                    val denies = choice == "deny" || choice == "__dismiss__"
                    if (pending.isHighRisk && !denies) { confirmed = false; lastHighRisk = pending.approvalId to choice; highRisk = lastHighRisk } else env.remoteChat.answerApproval(pending.approvalId, choice)
                }
            }

            Column(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp, top = 4.dp)) {
                ComposerCard(input, { input = it }, if (online) "对电脑上的小伴说点什么…" else "电脑不在线，暂时不能发送", enabled = online) {
                    ChipPill(peer?.name ?: "电脑", icon = XIcons.Monitor)
                    Spacer(Modifier.weight(1f))
                    if (remote.sending) {
                        // 协议里还没有“取消”指令，所以这里只表示电脑正在处理。
                        RoundIconButton(XIcons.Stop, "电脑正在处理", {}, MaterialTheme.colorScheme.onSurface.copy(alpha = 0.35f), MaterialTheme.colorScheme.surface, enabled = false)
                    } else {
                        val canSend = online && input.trim().isNotEmpty()
                        RoundIconButton(
                            XIcons.ArrowRight, "发送",
                            { val text = input.trim(); if (text.isNotEmpty()) { input = ""; env.remoteChat.send(text) } },
                            if (canSend) Brand else LocalExtraColors.current.line, Color.White, enabled = canSend,
                        )
                    }
                }
            }
        }
        OverlayTransition(highRisk != null) { panel ->
          lastHighRisk?.let { (approvalId, choice) ->
            val pending = remote.pendingApproval
            BottomSheet({ highRisk = null }, panel) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    Box(Modifier.size(36.dp).clip(CircleShape).background(LocalExtraColors.current.brandSoft), contentAlignment = Alignment.Center) { XIcon(XIcons.Warning, tint = Brand, size = 20.dp) }
                    Text("这是高危操作", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                }
                Text("它会在“${peer?.name ?: "电脑"}”上执行下面的操作，可能无法撤销。请看清内容再决定。", Modifier.padding(top = 10.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                CodeBox(pending?.summary.orEmpty().ifBlank { pending?.title.orEmpty() }, Modifier.padding(top = 10.dp))
                Row(Modifier.padding(top = 14.dp).fillMaxWidth().height(44.dp).clickable { confirmed = !confirmed }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    CheckBox(confirmed, { confirmed = it })
                    Text("我已看清上面的内容", fontSize = 14.sp)
                }
                Row(Modifier.padding(top = 8.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    DarkPillButton("拒绝", { if (highRisk != null) { highRisk = null; env.remoteChat.answerApproval(approvalId, "deny") } }, Modifier.weight(1f))
                    GhostPillButton("仍然允许", { if (highRisk != null) { highRisk = null; env.remoteChat.answerApproval(approvalId, choice) } }, Modifier.weight(1f), enabled = confirmed && highRisk != null)
                }
                Text("勾选后“仍然允许”才可点；手机上只能允许这一次。", Modifier.padding(top = 8.dp).fillMaxWidth(), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
            }
          }
        }
    }
}

@Composable
private fun CodeBox(text: String, modifier: Modifier = Modifier) {
    Text(text, modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(horizontal = 10.dp, vertical = 8.dp), fontSize = 12.sp, lineHeight = 19.sp, maxLines = 6, overflow = TextOverflow.Ellipsis)
}

@Composable
private fun RemoteMessages(messages: List<RemoteTurn>, tools: List<RemoteToolProgress>, sending: Boolean, waitingApproval: Boolean, online: Boolean, isNew: Boolean) {
    val listState = rememberLazyListState()
    val lastLen = messages.lastOrNull()?.text?.length ?: 0
    LaunchedEffect(messages.size, lastLen, tools.size) { if (messages.isNotEmpty()) listState.scrollToItem(messages.lastIndex + (if (tools.isNotEmpty()) 1 else 0)) }
    if (messages.isEmpty()) {
        Box(Modifier.fillMaxSize().padding(24.dp), contentAlignment = Alignment.Center) {
            Text(
                if (!online) "电脑上线后，这里会显示对话。" else if (isNew) "新对话：发一条消息，电脑上的小伴会开始处理。" else "这个对话还没有内容，直接发消息开始。",
                fontSize = 13.sp, lineHeight = 21.sp, color = LocalExtraColors.current.ink3, textAlign = TextAlign.Center,
            )
        }
        return
    }
    LazyColumn(Modifier.fillMaxSize(), state = listState, contentPadding = PaddingValues(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        items(messages, key = { it.id }) { turn ->
            if (turn.role == "user") {
                UserBubble(turn.text)
            } else {
                AssistantRow {
                    when {
                        turn.streaming && turn.text.isEmpty() -> AssistantBubble("电脑正在处理…", muted = true)
                        turn.text.isNotEmpty() -> AssistantBubble(if (turn.streaming) turn.text + " ▍" else turn.text)
                    }
                    if (turn.error != null) Banner(turn.error.orEmpty(), warn = true)
                }
            }
        }
        if (tools.isNotEmpty()) {
            item(key = "tools") {
                AssistantRow(avatar = false) {
                    tools.forEach { t ->
                        ToolCard(
                            XIcons.Tool, toolLabel(t.name), t.detail,
                            status = if (t.running) "执行中…" else if (t.isError == true) "出错" else "完成",
                            done = !t.running && t.isError != true,
                        )
                    }
                }
            }
        }
        if (waitingApproval || sending) {
            item(key = "hint") {
                Text(if (waitingApproval) "等你确认后继续…" else "电脑正在处理…", Modifier.fillMaxWidth(), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
            }
        }
    }
}

/** 电脑请求确认：标题、要做的事、选项。高危操作的“允许”会先弹二次确认。 */
@Composable
private fun ApprovalCard(pending: RemoteApprovalRequest, peerName: String, onDecision: (String) -> Unit) {
    val shape = RoundedCornerShape(16.dp)
    Column(
        Modifier.padding(start = 12.dp, end = 12.dp, bottom = 8.dp).fillMaxWidth().clip(shape).background(MaterialTheme.colorScheme.surface)
            .border(0.5.dp, LocalExtraColors.current.line, shape).padding(14.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            XIcon(if (pending.isHighRisk) XIcons.Warning else XIcons.Lock, tint = if (pending.isHighRisk) Brand else MaterialTheme.colorScheme.onSurface, size = 22.dp)
            Text(if (pending.isHighRisk) "电脑请求高危操作" else "电脑请求你的确认", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
        }
        Text(pending.title.ifBlank { "允许在“$peerName”上执行操作吗？" }, Modifier.padding(top = 6.dp), fontSize = 14.sp, lineHeight = 21.sp)
        val summary = pending.summary
        if (summary.isNotBlank()) CodeBox(summary, Modifier.padding(top = 8.dp))
        Text("在电脑上也可以处理，任一边处理后另一边会自动消失。", Modifier.padding(top = 6.dp), fontSize = 12.sp, lineHeight = 18.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        // 高危操作手机上只能“允许一次”，不给“总是允许”。
        val options = pending.options.filter { !(pending.isHighRisk && it.id == "always") }
        Row(Modifier.padding(top = 12.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            options.sortedBy { if (it.id == "deny" || it.id == "__dismiss__") 0 else 1 }.forEach { opt ->
                val negative = opt.id == "deny" || opt.id == "__dismiss__"
                if (negative) GhostPillButton(opt.label, { onDecision(opt.id) }, Modifier.weight(1f), height = 44.dp)
                else PillButton(opt.label, { onDecision(opt.id) }, Modifier.weight(1f), height = 44.dp)
            }
        }
    }
}
