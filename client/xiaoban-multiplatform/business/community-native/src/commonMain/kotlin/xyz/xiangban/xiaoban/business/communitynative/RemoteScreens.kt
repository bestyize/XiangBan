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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
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
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.taskPlan
import xyz.xiangban.xiaoban.common.community.taskProgress
import xyz.xiangban.xiaoban.common.community.CloudRecognizer
import xyz.xiangban.xiaoban.common.community.SpokenCall
import xyz.xiangban.xiaoban.common.link.DesktopRecognizer
import xyz.xiangban.xiaoban.common.link.PeerSummary
import xyz.xiangban.xiaoban.common.link.RemoteAssistant
import xyz.xiangban.xiaoban.common.link.RemoteApprovalRequest
import xyz.xiangban.xiaoban.common.link.RemoteChatState
import xyz.xiangban.xiaoban.common.link.folderName
import xyz.xiangban.xiaoban.common.link.RemoteSessionSummary
import xyz.xiangban.xiaoban.common.link.RemoteTurn
import xyz.xiangban.xiaoban.common.link.TurnCancelled

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
    var lastManage by remember { mutableStateOf<PeerSummary?>(null) }

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
                            IconTap(XIcons.More, "管理这台设备", { lastManage = peer; manage = peer })
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
    OverlayTransition(manage != null) { panel ->
        lastManage?.let { peer ->
            BottomSheet({ manage = null }, panel) {
                Text(peer.name, fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                SheetAction(XIcons.Close, "从账号中移除这台设备", "它再次启动并开启手机遥控后会重新出现") {
                    manage = null
                    env.remoteChat.removePeer(peer.id)
                }
                GhostPillButton("取消", { manage = null }, Modifier.padding(top = 8.dp).fillMaxWidth())
            }
        }
    }
}

// ───────────── 遥控会话页 ─────────────

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
    var drawer by rememberSaveable { mutableStateOf(false) }
    var attach by rememberSaveable { mutableStateOf(false) }
    // 0 关着，1 选权限，2 确认开启完全信任
    var permissionStep by rememberSaveable { mutableStateOf(0) }
    var toast by remember { mutableStateOf<String?>(null) }
    val listState = rememberLazyListState()
    val uiScope = rememberCoroutineScope()
    var follow by remember { mutableStateOf(true) }
    var adjust by remember { mutableStateOf(false) }
    var modelSheet by remember { mutableStateOf(false) }
    // 语音：麦克风是“说完变成文字放进输入框”，波形键是“语音通话”。识别先用手机本地的 SenseVoice，不行再让电脑识别，最后云端。
    val platform = env.platform
    val voiceInput = rememberVoiceInput(
        env, { env.recognizers(DesktopRecognizer(env.remoteChat), CloudRecognizer(env.ai)) },
        onText = { text -> input = if (input.isBlank()) text else input.trimEnd() + " " + text },
        onMessage = { toast = it },
    )
    val callHost = rememberVoiceCallHost(env)
    val uiScopeForCall = uiScope
    val startCall: () -> Unit = {
        when {
            platform == null -> toast = "这个平台暂不支持语音通话"
            !online -> toast = "设备不在线，没法通话"
            voiceInput.busy -> toast = "先结束当前的录音"
            else -> callHost.request {
                SpokenCall(
                    uiScopeForCall, platform, RemoteAssistant(env.remoteChat),
                    env.recognizers(DesktopRecognizer(env.remoteChat), CloudRecognizer(env.ai)), env.synthesizers(),
                )
            }
        }
    }
    DisposableEffect(Unit) { onDispose { platform?.stopAudio() } }
    val openModels: () -> Unit = {
        when {
            remote.selectedSessionId == null -> toast = "先发一条消息开始对话，之后就能切换模型"
            remote.sending -> toast = "小伴正在回答，停下后再切换模型"
            else -> { env.remoteChat.fetchModels(); modelSheet = true }
        }
    }
    var adjustSteps by remember { mutableStateOf(emptyList<String>()) }
    BackHandler(enabled = drawer) { drawer = false }

    LaunchedEffect(activePeer) {
        if (remote.peers.none { it.id == activePeer }) env.remoteChat.refreshPeers()
        if (env.remoteChat.state.value.selectedPeerId != activePeer) env.remoteChat.selectPeer(activePeer)
    }

    // 打开对话后读一下电脑上的模型，输入框上的模型名才对得上。
    LaunchedEffect(remote.selectedSessionId, online) { if (online && remote.selectedSessionId != null) env.remoteChat.fetchModels() }

    val current = remote.selectedSession
    val lastTurn = remote.messages.lastOrNull()
    val liveProgress = if (lastTurn != null && lastTurn.role == "assistant" && lastTurn.streaming) {
        taskProgress(lastTurn.tools, running = true, waiting = remote.pendingApproval?.let { if (it.isQuestion) "question" else "approval" })
    } else null
    val lastIndex = remote.messages.lastIndex
    // 任务卡滚出屏幕（最后一条回复已经往上滚了一截）后，在输入框上方吸一行进度。
    val showMini by remember(liveProgress != null, lastIndex) {
        derivedStateOf { liveProgress != null && listState.firstVisibleItemIndex == lastIndex && listState.firstVisibleItemScrollOffset > 100 }
    }
    val projectName = remote.currentWorkspace.takeIf { it.isNotBlank() }?.let(::folderName)
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize().imePadding()) {
            ScreenBar(
                current?.title?.ifBlank { null } ?: "新对话", onBack = null,
                leading = { IconTap(XIcons.Menu, "打开项目与对话", { drawer = true }, size = 22.dp) },
                trailing = { IconTap(XIcons.NewChat, "新对话", { env.remoteChat.newSession() }, size = 22.dp) },
                subtitleContent = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(listOfNotNull(peer?.name ?: "设备", projectName).joinToString(" · ") + " · ", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        StatusDot(online, if (online) "在线" else "离线", fontSize = 11)
                    }
                },
            )
            if (!session.loggedIn) {
                Banner("登录已过期，请重新登录", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Lock, actionText = "去登录", onAction = { open(Route.Login) })
            } else if (peer == null) {
                Banner("这台设备已退出账号或被移除", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Warning, actionText = "回到我的设备", onAction = back)
            } else if (!online) {
                Banner("${peer.name}现在不在线，当前显示的是离线缓存内容。上线后自动恢复同步。", Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true, icon = XIcons.Monitor, actionText = "改用云端小伴", onAction = { open(Route.AiChat) })
            }
            remote.error?.let { Banner(it, Modifier.padding(start = 12.dp, end = 12.dp, top = 8.dp), warn = true) }

            Box(Modifier.weight(1f).fillMaxWidth()) {
                RemoteMessages(
                    remote.messages, remote.pendingApproval, online, remote.selectedSessionId == null, env.platform, listState, follow, { follow = it },
                    onStop = env.remoteChat::cancel, onSend = env.remoteChat::send,
                    onAdjust = {
                        adjustSteps = lastTurn?.let { turn -> taskPlan(turn.tools)?.steps?.filter { !it.done }?.map { it.content } }.orEmpty()
                        adjust = true
                    },
                    savedTurns = remote.savedTurns, onSave = env.remoteChat::saveToLibrary, onUndo = env.remoteChat::undoRun,
                    onSwitchModel = openModels,
                )
            }

            remote.pendingApproval?.let { pending ->
                val cardModifier = Modifier.padding(start = 12.dp, end = 12.dp, bottom = 8.dp)
                if (pending.isQuestion) {
                    AskCard(pending, { answer -> env.remoteChat.answerApproval(pending.approvalId, answer) }, cardModifier)
                } else {
                    // 高风险的卡片上已经写明风险和完整命令，不再额外弹二次确认（开发阶段便利优先）。
                    ApprovalCard(pending, { choice -> env.remoteChat.answerApproval(pending.approvalId, choice) }, cardModifier)
                }
            }

            Column(Modifier.padding(start = 12.dp, end = 12.dp, bottom = 12.dp, top = 4.dp)) {
                if (voiceInput.busy) {
                    VoiceBar(
                        if (voiceInput.recording) "正在听… ${voiceInput.seconds / 60}:${(voiceInput.seconds % 60).toString().padStart(2, '0')}" else "正在识别…",
                        recording = voiceInput.recording,
                        hint = if (voiceInput.recording) "再点一下麦克风结束，文字会放进输入框" else null,
                        onCancel = { voiceInput.finish(false) },
                        modifier = Modifier.padding(bottom = 8.dp),
                    )
                }
                if (showMini && liveProgress != null) {
                    TaskMiniBar(
                        liveProgress, modifier = Modifier.padding(bottom = 8.dp),
                        onClick = { follow = false; uiScope.launch { listState.animateScrollToItem(lastIndex) } },
                        onStop = env.remoteChat::cancel,
                    )
                }
                Row(Modifier.padding(bottom = 8.dp, start = 4.dp)) {
                    ChipPill(
                        if (remote.fullTrust) "完全信任" else "默认权限", icon = XIcons.Lock, trailingIcon = XIcons.ChevronDown,
                        selected = remote.fullTrust, onClick = if (online) ({ permissionStep = 1 }) else null,
                    )
                }
                ChatComposer(
                    input, { input = it }, if (online) "对设备上的小伴说点什么…" else "设备不在线，上线后可继续对话",
                    remote.models.firstOrNull { it.id == remote.currentModel }?.name ?: remote.currentModel.ifBlank { "跟随电脑" },
                    onModel = openModels,
                    onAttach = { attach = true },
                    onWeb = { toast = "联网搜索即将支持" },
                    onVoice = { voiceInput.toggle() },
                    onLive = startCall,
                    onSend = { val text = input.trim(); if (text.isNotEmpty()) { input = ""; env.remoteChat.send(text) } },
                    enabled = online, sending = remote.sending,
                )
            }
        }
        OverlayTransition(permissionStep != 0) { panel ->
            BottomSheet({ permissionStep = 0 }, panel) {
                if (permissionStep == 2) {
                    Text("开启完全信任？", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                    Text(
                        "开启后，这个对话里小伴在“${peer?.name ?: "设备"}”上读写文件、运行命令都不再逐步问你，会直接执行。\n删除文件、危险命令这类高风险操作仍然每次都问你。\n只在你信任这次任务时开启，随时可以关掉。",
                        Modifier.padding(top = 10.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    Row(Modifier.padding(top = 16.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        GhostPillButton("取消", { permissionStep = 0 }, Modifier.weight(1f))
                        PillButton("开启完全信任", { permissionStep = 0; env.remoteChat.setPermission("auto") }, Modifier.weight(1f))
                    }
                } else {
                    Text("这个对话的权限", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                    Text("只对当前这个对话生效，和电脑上的开关是同一个。", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    RadioRow(
                        selected = !remote.fullTrust, title = "默认权限", desc = "改动文件、运行命令之前先问你",
                        onClick = { permissionStep = 0; env.remoteChat.setPermission("ask") },
                    )
                    RadioRow(
                        selected = remote.fullTrust, title = "完全信任", desc = "不再逐步询问，直接执行；高风险操作仍然每次都问",
                        onClick = { permissionStep = 0; env.remoteChat.setPermission("auto") },
                    )
                    GhostPillButton("取消", { permissionStep = 0 }, Modifier.padding(top = 8.dp).fillMaxWidth())
                }
            }
        }
        OverlayTransition(modelSheet) { panel ->
            BottomSheet({ modelSheet = false }, panel) {
                Text("这个对话用的模型", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                Text("只对当前这个对话生效，模型来自${peer?.name ?: "设备"}上已经配置好的服务。", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Column(Modifier.heightIn(max = 360.dp).verticalScroll(rememberScrollState())) {
                    if (remote.models.isEmpty()) {
                        Text("还没有读到可选的模型。请确认设备在线，并且在设备上配置过模型服务。", Modifier.padding(vertical = 12.dp), fontSize = 13.sp, lineHeight = 20.sp, color = LocalExtraColors.current.ink3)
                    }
                    remote.models.forEach { model ->
                        RadioRow(
                            selected = model.id == remote.currentModel, title = model.name, desc = model.providerName,
                            onClick = { modelSheet = false; env.remoteChat.setModel(model.provider, model.id) },
                        )
                    }
                }
                GhostPillButton("取消", { modelSheet = false }, Modifier.padding(top = 8.dp).fillMaxWidth())
            }
        }
        OverlayTransition(adjust) { panel ->
            AdjustPlanSheet(adjustSteps, onDismiss = { adjust = false }, onSubmit = { adjust = false; follow = true; env.remoteChat.revisePlan(it) }, panelModifier = panel)
        }
        OverlayTransition(attach) { panel -> AttachSheet({ attach = false }, { attach = false; toast = "$it 即将支持" }, panel) }
        VoiceCallLayer(env, callHost, peer?.name ?: "设备", "语音通话") { toast = it }
        Toast(toast) { toast = null }
        OverlayTransition(drawer, direction = "left") { panel ->
            RemoteDrawer(
                panelModifier = panel,
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
private fun RemoteMessages(
    messages: List<RemoteTurn>,
    pending: RemoteApprovalRequest?,
    online: Boolean,
    isNew: Boolean,
    platform: PlatformActions?,
    listState: LazyListState,
    follow: Boolean,
    onFollow: (Boolean) -> Unit,
    onStop: () -> Unit,
    onSend: (String) -> Unit,
    onAdjust: () -> Unit,
    savedTurns: Set<String>,
    onSave: (RemoteTurn) -> Unit,
    onUndo: (String) -> Unit,
    onSwitchModel: () -> Unit,
) {
    val last = messages.lastOrNull()
    val lastUserText = messages.lastOrNull { it.role == "user" }?.text
    val progressMark = (last?.text?.length ?: 0) + (last?.reasoning?.length ?: 0) + (last?.tools?.size ?: 0) * 7 + (last?.records?.size ?: 0) * 13
    // 跟着最新内容往下走；用户往上翻就不再拽回去，发新消息或回到底部后恢复。
    LaunchedEffect(listState.lastScrolledBackward) { if (listState.lastScrolledBackward) onFollow(false) }
    LaunchedEffect(listState.canScrollForward) { if (!listState.canScrollForward) onFollow(true) }
    LaunchedEffect(messages.lastOrNull { it.role == "user" }?.id) { onFollow(true) }
    LaunchedEffect(messages.size, progressMark, pending?.approvalId, follow) {
        if (follow && messages.isNotEmpty()) {
            if (listState.layoutInfo.visibleItemsInfo.lastOrNull()?.index != messages.lastIndex) listState.scrollToItem(messages.lastIndex)
            listState.scrollBy(1_000_000f)
        }
    }
    if (messages.isEmpty()) {
        Box(Modifier.fillMaxSize().padding(24.dp), contentAlignment = Alignment.Center) {
            Text(
                if (!online) "设备上线后，这里会显示对话。" else if (isNew) "新对话：发一条消息，电脑上的小伴会开始处理。" else "这个对话还没有内容，直接发消息开始。",
                fontSize = 13.sp, lineHeight = 21.sp, color = LocalExtraColors.current.ink3, textAlign = TextAlign.Center,
            )
        }
        return
    }
    Box(Modifier.fillMaxSize()) {
        LazyColumn(Modifier.fillMaxSize(), state = listState, contentPadding = PaddingValues(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            items(messages, key = { it.id }) { turn ->
                if (turn.role == "user") {
                    UserBubble(turn.text, images = turn.images)
                } else {
                    val isLast = turn === last
                    val again = if (isLast && !turn.streaming && lastUserText != null) ({ onSend(lastUserText) }) else null
                    val waiting = if (turn.streaming && pending != null) (if (pending.isQuestion) "question" else "approval") else null
                    val failed = turn.error != null && turn.error != TurnCancelled
                    val extras = TurnExtras(
                        promptTokens = turn.promptTokens, completionTokens = turn.completionTokens,
                        truncated = turn.truncated && isLast, onContinue = if (isLast) ({ onSend("继续") }) else null,
                        checkpoint = turn.checkpoint, onUndo = onUndo,
                        onSave = if (turn.text.isNotBlank() && !turn.streaming) ({ onSave(turn) }) else null, saved = turn.id in savedTurns,
                        onSwitchModel = if (isLast && failed) onSwitchModel else null,
                    )
                    AssistantRow {
                        AssistantTurnCards(
                            text = turn.text, streaming = turn.streaming, error = turn.error, reasoning = turn.reasoning, tools = turn.tools, records = turn.records,
                            waiting = waiting, platform = platform, idleText = "电脑正在处理…", onStop = onStop, onSend = onSend,
                            onRetry = if (failed) again else null, onRegenerate = again,
                            onAdjust = if (isLast) onAdjust else null, extras = extras,
                        )
                    }
                }
            }
        }
        // 往上翻之后，右下角一个“回到最新”。
        if (!follow && listState.canScrollForward) {
            Box(
                Modifier.align(Alignment.BottomEnd).padding(end = 16.dp, bottom = 12.dp).size(40.dp).shadow(4.dp, CircleShape).clip(CircleShape)
                    .background(MaterialTheme.colorScheme.surface).clickable { onFollow(true) },
                contentAlignment = Alignment.Center,
            ) { XIcon(XIcons.ChevronDown, size = 20.dp, description = "回到最新") }
        }
    }
}

// ───────────── 左侧栏：设备、项目、对话 ─────────────

@Composable
private fun RemoteDrawer(
    panelModifier: Modifier,
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
        Column(panelModifier.width(312.dp).fillMaxHeight().background(MaterialTheme.colorScheme.surface).statusBarsPadding().padding(top = 12.dp)) {
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
    OverlayTransition(newProject) { panel -> NewProjectSheet(remote, onDismiss = { newProject = false }, onPick = { newProject = false; onAddProject(it) }, panelModifier = panel) }
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
private fun NewProjectSheet(remote: RemoteChatState, onDismiss: () -> Unit, onPick: (String) -> Unit, panelModifier: Modifier) {
    val have = remote.sidebar.projects.map { it.path.trimEnd('/', '\\') }.toSet()
    BottomSheet(onDismiss, panelModifier) {
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
