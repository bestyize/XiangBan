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
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.link.PeerSummary
import xyz.xiangban.xiaoban.common.link.RemoteApprovalRequest
import xyz.xiangban.xiaoban.common.link.RemoteChatState
import xyz.xiangban.xiaoban.common.link.folderName
import xyz.xiangban.xiaoban.common.link.RemoteSessionSummary
import xyz.xiangban.xiaoban.common.link.RemoteToolProgress
import xyz.xiangban.xiaoban.common.link.RemoteTurn

private val SetupSteps = listOf(
    "在设备上安装小伴并登录" to "用和手机相同的账号登录。",
    "打开“设置 → 手机遥控”" to "默认关闭；打开后，这台设备才允许被手机遥控。",
    "回到这里点开设备" to "设备在线时会出现在列表里，点一下就能进入它的小伴聊天。",
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

// ───────────── 我的设备 ─────────────

/** 同账号下的设备列表；点一台设备直接进它的聊天页（左侧栏是项目和对话）。没有设备时给出开启步骤。 */
@Composable
fun DevicesScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()
    var searching by remember { mutableStateOf(false) }
    var manage by remember { mutableStateOf<PeerSummary?>(null) }

    LaunchedEffect(session.loggedIn) { if (session.loggedIn) env.remoteChat.refreshPeers() }
    val refresh: () -> Unit = {
        if (!searching) scope.launch { searching = true; env.remoteChat.refreshPeers(); searching = false }
    }

    Column(Modifier.fillMaxSize()) {
        ScreenBar("我的设备", onBack = back)
        when {
            !session.loggedIn -> Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                Column(Modifier.padding(horizontal = 24.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                    EmptyState(XIcons.Monitor, "先登录账号", "设备和手机登录同一个小伴账号，设备就会自动出现在这里。")
                    PillButton("登录小伴账号", { open(Route.Login) }, Modifier.padding(top = 20.dp).fillMaxWidth())
                }
            }
            remote.peers.isEmpty() -> Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(vertical = 24.dp)) {
                EmptyState(XIcons.Monitor, "还没有可用的设备", "用手机说一句话，电脑上的小伴就开始干活，做完再通知你。")
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
                Text("登录同一个账号的设备会自动出现在这里，不需要扫码。点一台在线的设备，进入它上面的小伴聊天；每台设备有自己的项目和对话，可以同时进行。", Modifier.padding(horizontal = 16.dp), fontSize = 13.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Column(Modifier.padding(top = 12.dp).fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
                    remote.peers.forEachIndexed { i, peer ->
                        if (i > 0) Hairline(Modifier.padding(start = 16.dp))
                        Row(
                            Modifier.fillMaxWidth().clickable { open(Route.RemoteChat(peer.id)) }.padding(start = 16.dp, top = 8.dp, bottom = 8.dp, end = 4.dp),
                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            IconBox(XIcons.Monitor, 44.dp, 12.dp)
                            Column(Modifier.weight(1f)) {
                                Text(peer.name, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                Text(if (peer.online) "点一下进入聊天" else "不在线，上线后自动恢复", Modifier.padding(top = 3.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                            StatusDot(peer.online, peer.statusText())
                            IconTap(XIcons.More, "管理这台设备", { manage = peer })
                        }
                    }
                }
                Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).padding(horizontal = 14.dp, vertical = 12.dp)) {
                    Text("没看到你的设备？", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                    Text("1. 设备上的小伴已登录同一个账号\n2. 设备上小伴里“设置 → 手机遥控”已打开（默认关闭）\n3. 设备已联网，小伴正在运行", Modifier.padding(top = 6.dp), fontSize = 13.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    LinkText(if (searching) "查找中…" else "重新查找 ›", refresh, Modifier.padding(top = 4.dp))
                }
                Text("当前账号：${session.user?.username ?: ""}。手机和设备之间的消息经中转服务转发。", Modifier.padding(horizontal = 16.dp, vertical = 16.dp), fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
    manage?.let { peer ->
        BottomSheet({ manage = null }) {
            Text(peer.name, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
            SheetAction(XIcons.Close, "从账号中移除这台设备", "它再次启动并开启手机遥控后会重新出现") {
                manage = null
                env.remoteChat.removePeer(peer.id)
            }
            GhostPillButton("取消", { manage = null }, Modifier.padding(top = 8.dp).fillMaxWidth())
        }
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

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun RemoteChatScreen(env: CommunityEnv, peerId: String, back: () -> Unit, open: (Route) -> Unit) {
    val session by env.session.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()
    // 侧栏里可以换设备；换了以后页面跟着新设备走，返回仍回到“我的设备”。
    var activePeer by rememberSaveable(peerId) { mutableStateOf(peerId) }
    val peer = remote.peers.firstOrNull { it.id == activePeer }
    val online = peer?.online == true
    var input by rememberSaveable { mutableStateOf("") }
    var highRisk by remember { mutableStateOf<Pair<String, String>?>(null) }
    var confirmed by remember { mutableStateOf(false) }
    var drawer by rememberSaveable { mutableStateOf(false) }
    BackHandler(enabled = drawer) { drawer = false }

    LaunchedEffect(activePeer) {
        if (remote.peers.none { it.id == activePeer }) env.remoteChat.refreshPeers()
        if (env.remoteChat.state.value.selectedPeerId != activePeer) env.remoteChat.selectPeer(activePeer)
    }

    val current = remote.selectedSession
    val projectName = remote.currentWorkspace.takeIf { it.isNotBlank() }?.let(::folderName)
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().imePadding()) {
            ScreenBar(
                current?.title?.ifBlank { null } ?: "新对话", onBack = null,
                subtitle = listOfNotNull(peer?.name ?: "设备", projectName, if (online) "● 在线" else "○ 离线").joinToString(" · "),
                leading = { IconTap(XIcons.Menu, "打开项目与对话", { drawer = true }, size = 22.dp) },
                trailing = { IconTap(XIcons.NewChat, "新对话", { env.remoteChat.newSession() }, size = 22.dp) },
            )
            if (!session.loggedIn) {
                Banner("登录已过期，请重新登录", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Lock, actionText = "去登录", onAction = { open(Route.Login) })
            } else if (peer == null) {
                Banner("这台设备已退出账号或被移除", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Warning, actionText = "回到我的设备", onAction = back)
            } else if (!online) {
                Banner("${peer.name}现在不在线，暂时不能遥控。上线后会自动恢复。", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Monitor, actionText = "改用云端小伴", onAction = { open(Route.AiChat) })
            }
            remote.error?.let { Banner(it, Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true) }

            Box(Modifier.weight(1f).fillMaxWidth()) {
                RemoteMessages(remote.messages, remote.toolProgress, remote.sending, remote.pendingApproval != null, online, remote.selectedSessionId == null)
            }

            remote.pendingApproval?.let { pending ->
                ApprovalCard(pending, peer?.name ?: "电脑") { choice ->
                    val denies = choice == "deny" || choice == "__dismiss__"
                    if (pending.isHighRisk && !denies) { confirmed = false; highRisk = pending.approvalId to choice } else env.remoteChat.answerApproval(pending.approvalId, choice)
                }
            }

            Column(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp, top = 4.dp)) {
                ComposerCard(input, { input = it }, if (online) "对设备上的小伴说点什么…" else "设备不在线，暂时不能发送", enabled = online) {
                    ChipPill(listOfNotNull(peer?.name ?: "设备", projectName).joinToString(" · "), icon = XIcons.Monitor)
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
        highRisk?.let { (approvalId, choice) ->
            val pending = remote.pendingApproval
            BottomSheet({ highRisk = null }) {
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
                    DarkPillButton("拒绝", { highRisk = null; env.remoteChat.answerApproval(approvalId, "deny") }, Modifier.weight(1f))
                    GhostPillButton("仍然允许", { highRisk = null; env.remoteChat.answerApproval(approvalId, choice) }, Modifier.weight(1f), enabled = confirmed)
                }
                Text("勾选后“仍然允许”才可点；手机上只能允许这一次。", Modifier.padding(top = 8.dp).fillMaxWidth(), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center)
            }
        }
        if (drawer) {
            RemoteDrawer(
                remote = remote,
                onClose = { drawer = false },
                onSwitchPeer = { activePeer = it },
                onNewChat = { drawer = false; env.remoteChat.newSession() },
                onNewChatIn = { path -> drawer = false; env.remoteChat.newSession(path) },
                onOpenSession = { drawer = false; env.remoteChat.selectSession(it) },
                onAddProject = { path -> drawer = false; env.remoteChat.addProject(path) },
                onLoadProjects = { env.remoteChat.fetchProjects() },
                onAllDevices = { drawer = false; back() },
            )
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
                if (!online) "设备上线后，这里会显示对话。" else if (isNew) "新对话：发一条消息，电脑上的小伴会开始处理。" else "这个对话还没有内容，直接发消息开始。",
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


// ───────────── 左侧栏：设备、项目、对话 ─────────────

@Composable
private fun RemoteDrawer(
    remote: RemoteChatState,
    onClose: () -> Unit,
    onSwitchPeer: (String) -> Unit,
    onNewChat: () -> Unit,
    onNewChatIn: (String) -> Unit,
    onOpenSession: (String) -> Unit,
    onAddProject: (String) -> Unit,
    onLoadProjects: () -> Unit,
    onAllDevices: () -> Unit,
) {
    val extra = LocalExtraColors.current
    var switchOpen by rememberSaveable { mutableStateOf(false) }
    var query by rememberSaveable { mutableStateOf("") }
    var collapsed by remember { mutableStateOf(emptySet<String>()) }
    var newProject by rememberSaveable { mutableStateOf(false) }
    val peer = remote.selectedPeer
    val bar = remember(remote.sessions, remote.addedProjects) { remote.sidebar }
    val keyword = query.trim()
    fun List<RemoteSessionSummary>.matching() = if (keyword.isEmpty()) this else filter { it.title.contains(keyword, ignoreCase = true) }

    Row(Modifier.fillMaxSize()) {
        Column(Modifier.width(312.dp).fillMaxHeight().background(MaterialTheme.colorScheme.surface).statusBarsPadding().padding(top = 12.dp)) {
            // 当前设备；点开可以换设备
            val shape = RoundedCornerShape(14.dp)
            Column(Modifier.padding(horizontal = 12.dp).fillMaxWidth().clip(shape).border(0.5.dp, extra.line, shape)) {
                Row(
                    Modifier.fillMaxWidth().clickable { switchOpen = !switchOpen }.padding(horizontal = 12.dp, vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    IconBox(XIcons.Monitor, 36.dp, 10.dp, iconSize = 20.dp)
                    Column(Modifier.weight(1f)) {
                        Text(peer?.name ?: "设备", fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(
                            (if (peer?.online == true) "● 在线" else "○ 离线") + " · ${bar.projects.size} 个项目",
                            Modifier.padding(top = 1.dp), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis,
                        )
                    }
                    XIcon(XIcons.ChevronDown, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 18.dp, modifier = Modifier.rotate(if (switchOpen) 180f else 0f))
                }
                if (switchOpen) {
                    remote.peers.filter { it.id != remote.selectedPeerId }.forEach { other ->
                        Hairline()
                        Row(
                            Modifier.fillMaxWidth().clickable { switchOpen = false; onSwitchPeer(other.id) }.padding(horizontal = 12.dp, vertical = 10.dp),
                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            IconBox(XIcons.Monitor, 32.dp, 9.dp, iconSize = 18.dp)
                            Text(other.name, Modifier.weight(1f), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            StatusDot(other.online, other.statusText())
                        }
                    }
                    Hairline()
                    Row(Modifier.fillMaxWidth().clickable(onClick = onAllDevices).padding(horizontal = 12.dp, vertical = 11.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text("全部设备", Modifier.weight(1f), fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                        XIcon(XIcons.ChevronRight, tint = extra.ink3, size = 16.dp)
                    }
                }
            }
            PillButton("新对话", onNewChat, Modifier.padding(start = 12.dp, end = 12.dp, top = 12.dp).fillMaxWidth(), height = 44.dp, icon = XIcons.Plus)
            TextBox(query, { query = it }, Modifier.padding(start = 12.dp, end = 12.dp, top = 10.dp), placeholder = "搜索这台设备上的对话")
            Row(Modifier.padding(start = 16.dp, end = 12.dp, top = 12.dp).fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                Text("项目", Modifier.weight(1f), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                ChipPill("新项目", icon = XIcons.Plus, onClick = { onLoadProjects(); newProject = true })
            }
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(top = 4.dp, bottom = 8.dp)) {
                if (remote.loadingSessions && remote.sessions.isEmpty()) {
                    Text("读取设备上的对话…", Modifier.padding(horizontal = 16.dp, vertical = 8.dp), fontSize = 13.sp, color = extra.ink3)
                } else if (bar.projects.isEmpty() && bar.loose.isEmpty()) {
                    Text("这台设备上还没有对话。点“新项目”选一个文件夹，或直接点“新对话”。", Modifier.padding(horizontal = 16.dp, vertical = 8.dp), fontSize = 13.sp, lineHeight = 20.sp, color = extra.ink3)
                }
                bar.projects.forEach { group ->
                    val chats = group.sessions.matching()
                    if (keyword.isNotEmpty() && chats.isEmpty()) return@forEach
                    val open = keyword.isNotEmpty() || group.path !in collapsed
                    Row(Modifier.padding(horizontal = 8.dp).fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Row(
                            Modifier.weight(1f).clip(RoundedCornerShape(10.dp)).clickable { collapsed = if (group.path in collapsed) collapsed - group.path else collapsed + group.path }.padding(horizontal = 6.dp, vertical = 9.dp),
                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
                        ) {
                            XIcon(XIcons.ChevronDown, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 16.dp, modifier = Modifier.rotate(if (open) 0f else -90f))
                            XIcon(XIcons.Folder, size = 18.dp)
                            Column(Modifier.weight(1f)) {
                                Text(group.name, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                Text(group.path, Modifier.padding(top = 1.dp), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            }
                            Text(group.sessions.size.toString(), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        IconTap(XIcons.NewChat, "在这个项目里新对话", { onNewChatIn(group.path) }, size = 18.dp)
                    }
                    if (open) chats.forEach { DrawerSession(it, selected = it.id == remote.selectedSessionId, indent = 34.dp) { onOpenSession(it.id) } }
                }
                val loose = bar.loose.matching()
                if (loose.isNotEmpty()) {
                    Text("没有项目的对话", Modifier.padding(start = 16.dp, top = 10.dp, bottom = 4.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    loose.forEach { DrawerSession(it, selected = it.id == remote.selectedSessionId, indent = 8.dp) { onOpenSession(it.id) } }
                }
            }
            Hairline()
            Row(Modifier.fillMaxWidth().height(44.dp).clickable(onClick = onAllDevices).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                XIcon(XIcons.ChevronLeft, size = 22.dp)
                Text("返回我的设备", fontSize = 15.sp)
            }
            Spacer(Modifier.navigationBarsPadding())
        }
        Box(Modifier.weight(1f).fillMaxHeight().background(Color.Black.copy(alpha = 0.45f)).clickable(indication = null, interactionSource = null) { onClose() })
    }
    if (newProject) NewProjectSheet(remote, onDismiss = { newProject = false }, onPick = { newProject = false; onAddProject(it) })
}

@Composable
private fun DrawerSession(s: RemoteSessionSummary, selected: Boolean, indent: Dp, onClick: () -> Unit) {
    Row(
        Modifier.padding(start = indent, end = 8.dp).fillMaxWidth().clip(RoundedCornerShape(10.dp))
            .background(if (selected) LocalExtraColors.current.fill else Color.Transparent).clickable(onClick = onClick).padding(horizontal = 10.dp, vertical = 9.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text(s.title.ifBlank { "未命名对话" }, Modifier.weight(1f), fontSize = 14.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal, maxLines = 1, overflow = TextOverflow.Ellipsis)
        if (s.state == "running") {
            Text("执行中", Modifier.clip(CircleShape).background(MaterialTheme.colorScheme.onSurface).padding(horizontal = 7.dp, vertical = 2.dp), color = MaterialTheme.colorScheme.surface, fontSize = 11.sp, fontWeight = FontWeight.SemiBold)
        } else {
            Text(relativeTime(s.updatedAt * 1000), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

/** 新项目：只能从电脑上最近用过的文件夹里选（手机浏览不了电脑上的文件）。 */
@Composable
private fun NewProjectSheet(remote: RemoteChatState, onDismiss: () -> Unit, onPick: (String) -> Unit) {
    val have = remote.sidebar.projects.map { it.path.trimEnd('/', '\\') }.toSet()
    BottomSheet(onDismiss) {
        Text("新项目", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        Text("项目是${remote.selectedPeer?.name ?: "设备"}上的一个文件夹，这个项目里的对话都在这个文件夹里干活。", Modifier.padding(top = 6.dp), fontSize = 13.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text("设备上最近用过的文件夹", Modifier.padding(top = 14.dp, bottom = 4.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (remote.recentProjects.isEmpty()) {
            Text("还没有读到。请确认设备在线，并且在设备上的小伴里打开过文件夹。", Modifier.padding(vertical = 8.dp), fontSize = 13.sp, lineHeight = 20.sp, color = LocalExtraColors.current.ink3)
        }
        remote.recentProjects.forEachIndexed { i, project ->
            if (i > 0) Hairline()
            val added = project.path.trimEnd('/', '\\') in have
            Row(
                Modifier.fillMaxWidth().clickable { onPick(project.path) }.padding(vertical = 11.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                IconBox(XIcons.Folder, 40.dp, 12.dp, iconSize = 22.dp)
                Column(Modifier.weight(1f)) {
                    Text(project.name, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text(project.path, Modifier.padding(top = 2.dp), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                if (added) Text("已添加", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        Text("手机不能浏览设备上的文件，所以只列出最近用过的文件夹。其他文件夹请在设备小伴里先打开一次。", Modifier.padding(top = 12.dp), fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        GhostPillButton("取消", onDismiss, Modifier.padding(top = 14.dp).fillMaxWidth())
    }
}
