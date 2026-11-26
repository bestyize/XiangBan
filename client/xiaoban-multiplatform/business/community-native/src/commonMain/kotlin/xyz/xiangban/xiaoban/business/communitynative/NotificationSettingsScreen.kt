package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlin.time.Clock
import kotlin.time.ExperimentalTime
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.notificationHeadline
import xyz.xiangban.xiaoban.common.community.taskProgress
import xyz.xiangban.xiaoban.common.community.toolLabel

private val NotificationWarnColor: Color get() = Color(if (ThemeState.dark) 0xFFEBA446L else 0xFFC77710L)

/**
 * 设置 → 通知与提醒（对应设计稿 mobile-notifications.html 画板 04）：
 * - 通栏白底列表，无圆角，0.5px 细分隔线
 * - 零同系色浅底搭配：统一中性底 + 实心主题主按钮 + 状态小圆点
 */
@Composable
fun NotificationSettingsScreen(env: CommunityEnv, back: () -> Unit) {
    val bridge = env.platform?.notifications
    val permState by (bridge?.permissionState ?: EmptyNotificationPermission).collectAsState()
    val sysEnabled = permState == NotificationPermissionState.Granted
    val settings = NotificationState.settings
    var toast by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(bridge) { bridge?.refreshPermission() }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("通知与提醒", onBack = back)
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(bottom = 28.dp)) {
                if (bridge != null && !sysEnabled) {
                    SystemNotificationOffBanner(
                        channelOnly = permState == NotificationPermissionState.ChannelDisabled,
                        onEnable = {
                            if (permState == NotificationPermissionState.NotDetermined) {
                                bridge.triggerPermissionGuide(PermissionGuideReason.Settings)
                            } else {
                                if (!bridge.openSystemNotificationSettings()) {
                                    toast = "请在系统设置中开启小伴通知"
                                }
                            }
                        },
                    )
                }

                GroupLabel("通知类型")
                Column(Modifier.fillMaxWidth().alpha(if (sysEnabled) 1f else 0.48f)) {
                    NotificationSwitchRow(
                        title = "定时提醒",
                        badge = "高优先级",
                        desc = "小伴帮你设定的本机定时提醒",
                        checked = settings.reminders,
                        enabled = sysEnabled,
                        first = true,
                    ) { on -> NotificationState.update { it.copy(reminders = on) } }
                    NotificationSwitchRow(
                        title = "后台任务进度",
                        badge = "静默",
                        desc = "AI 后台制作与执行时的常驻进度条，不响铃不震动",
                        checked = settings.aiProgress,
                        enabled = sysEnabled,
                    ) { on ->
                        NotificationState.update { it.copy(aiProgress = on) }
                        if (!on) bridge?.cancelTaskProgress("active")
                    }
                    NotificationSwitchRow(
                        title = "任务确认与交付",
                        desc = "方案待确认、操作审批、小伴提问、交付完成与异常中断",
                        checked = settings.aiTasks,
                        enabled = sysEnabled,
                    ) { on -> NotificationState.update { it.copy(aiTasks = on) } }
                    NotificationSwitchRow(
                        title = "社区互动",
                        desc = "评论、回复、点赞与作品审核动态",
                        checked = settings.community,
                        enabled = sysEnabled,
                    ) { on ->
                        NotificationState.update { it.copy(community = on) }
                        if (!on) bridge?.cancelCommunitySummary()
                    }
                    NotificationSwitchRow(
                        title = "设备互联与下载",
                        desc = "电脑端发来文件链接、应用包下载完成",
                        checked = settings.linkTransfer,
                        enabled = sysEnabled,
                    ) { on -> NotificationState.update { it.copy(linkTransfer = on) } }
                }

                GroupLabel("隐私与打扰控制")
                Column(Modifier.fillMaxWidth().alpha(if (sysEnabled) 1f else 0.48f)) {
                    NotificationSwitchRow(
                        title = "锁屏显示通知详情",
                        desc = "关闭时锁屏仅显示“收到一条新通知”",
                        checked = settings.lockScreenPreview,
                        enabled = sysEnabled,
                        first = true,
                    ) { on ->
                        NotificationState.update { it.copy(lockScreenPreview = on) }
                        toast = if (on) "锁屏将显示通知正文" else "锁屏已隐藏通知详情"
                    }
                    NotificationSwitchRow(
                        title = "声音与振动",
                        desc = "关闭后非定时提醒通知静默送达状态栏",
                        checked = settings.soundVibrate,
                        enabled = sysEnabled,
                    ) { on -> NotificationState.update { it.copy(soundVibrate = on) } }
                    if (sysEnabled) {
                        NavRow("夜间免打扰", settings.dnd.label) {
                            val next = settings.dnd.next()
                            NotificationState.update { it.copy(dnd = next) }
                            toast = if (next == DndSchedule.Off) "已关闭夜间免打扰" else "免打扰时段已切换为 ${next.label}"
                        }
                    } else {
                        DisabledNavRow("夜间免打扰", settings.dnd.label)
                    }
                }

                if (bridge != null) {
                    GroupLabel("系统级设置")
                    val title = if (bridge.platformSupportsChannels) "系统通知渠道管理" else "系统通知样式设置"
                    NavRow(title, first = true) {
                        if (!bridge.openSystemNotificationSettings()) {
                            toast = "请在系统设置中管理小伴通知"
                        }
                    }
                }
            }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
private fun SystemNotificationOffBanner(channelOnly: Boolean, onEnable: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Column(Modifier.fillMaxWidth().background(colors.surface)) {
        Row(
            Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 13.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Column(Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    Box(Modifier.size(7.dp).clip(CircleShape).background(NotificationWarnColor))
                    Text(
                        if (channelOnly) "提醒渠道已关闭" else "系统通知未开启",
                        style = XType.Body,
                        fontWeight = FontWeight.SemiBold,
                        color = colors.onSurface,
                    )
                }
                Text(
                    if (channelOnly) "开启「提醒」渠道后才能按时弹出本机定时提醒"
                    else "开启后才能在锁屏与通知栏收到提醒、任务与互动消息",
                    Modifier.padding(top = 2.dp),
                    style = XType.Caption,
                    color = colors.onSurfaceVariant,
                )
            }
            Box(
                Modifier.height(32.dp).clip(CircleShape).background(Brand)
                    .clickable(role = Role.Button, onClick = onEnable)
                    .padding(horizontal = 13.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text("去开启", color = Color.White, style = XType.Caption, fontWeight = FontWeight.SemiBold)
            }
        }
        Hairline()
    }
}

@Composable
private fun NotificationSwitchRow(
    title: String,
    desc: String,
    checked: Boolean,
    enabled: Boolean,
    first: Boolean = false,
    badge: String? = null,
    onChange: (Boolean) -> Unit,
) {
    val extra = LocalExtraColors.current
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        if (!first) Hairline(Modifier.padding(start = 16.dp))
        Row(
            Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Column(Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text(title, style = XType.Body)
                    if (badge != null) {
                        Box(
                            Modifier.clip(CircleShape).background(extra.fill).padding(horizontal = 7.dp, vertical = 2.dp),
                            contentAlignment = Alignment.Center,
                        ) {
                            Text(badge, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
                Text(desc, Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            Toggle(checked, onChange = { next -> if (enabled) onChange(next) })
        }
    }
}

@Composable
private fun DisabledNavRow(title: String, note: String) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface)) {
        Hairline(Modifier.padding(start = 16.dp))
        Row(
            Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(title, Modifier.weight(1f), style = XType.Body)
            Text(note, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

/**
 * 24px 统一通知权限引导弹层（对应设计稿 mobile-notifications.html 画板 03）：
 * - 首次触发：说明 3 个核心场景，点击「开启通知」拉起系统权限框
 * - 已被拒绝 / 渠道关闭：纯净中性提示块 + 琥珀图标点缀，点击「前往系统设置」直达系统通知设置
 */
@Composable
internal fun NotificationPermissionDialog(bridge: SystemNotificationBridge) {
    val reason by bridge.permissionGuide.collectAsState()
    val permState by bridge.permissionState.collectAsState()
    if (reason == null) return

    val colors = MaterialTheme.colorScheme
    val extra = LocalExtraColors.current
    val denied = permState == NotificationPermissionState.Denied || permState == NotificationPermissionState.ChannelDisabled

    CommunityCardDialog(
        title = if (denied) "通知权限已关闭" else "开启通知，按时收到提醒",
        description = if (denied) null else "小伴会在以下场景通过系统通知栏提醒你，随时可在设置里按类型开关：",
        confirmLabel = if (denied) "前往系统设置" else "开启通知",
        dismissLabel = if (denied) "稍后再说" else "暂不开启",
        icon = XIcons.Bell,
        iconColor = if (denied) Color(0xFF293036) else Color(0xFF293036),
        onConfirm = {
            bridge.dismissPermissionGuide()
            if (denied) bridge.openSystemNotificationSettings() else bridge.requestSystemPermission()
        },
        onDismiss = { bridge.dismissPermissionGuide() },
    ) {
        if (denied) {
            Row(
                Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(extra.fill).padding(horizontal = 12.dp, vertical = 10.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.Top,
            ) {
                XIcon(XIcons.Info, Modifier.padding(top = 2.dp), tint = NotificationWarnColor, size = 15.dp)
                Text(
                    if (permState == NotificationPermissionState.ChannelDisabled)
                        "小伴的「提醒」通知渠道已被关闭，提醒将无法按时弹出。"
                    else
                        "系统通知权限已关闭，已保存的提醒与任务节点暂时无法弹出通知。",
                    style = XType.Secondary,
                    color = colors.onSurface,
                )
            }
            Spacer(Modifier.height(10.dp))
            Box(
                Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp))
                    .background(colors.surface).border(0.5.dp, extra.hairline, RoundedCornerShape(10.dp))
                    .padding(horizontal = 12.dp, vertical = 9.dp),
            ) {
                Text(
                    if (bridge.platformSupportsChannels) "开启路径：系统设置 → 应用 → 小伴 → 通知"
                    else "开启路径：系统设置 → 小伴 → 通知",
                    style = XType.Caption,
                    color = colors.onSurfaceVariant,
                )
            }
        } else {
            Column(
                Modifier.fillMaxWidth().clip(RoundedCornerShape(12.dp)).background(extra.fill).padding(horizontal = 12.dp, vertical = 10.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                PermBullet("你让小伴设定的定时提醒到点弹出")
                PermBullet("AI 后台长任务完成或需要你确认时通知")
                PermBullet("社区回复、作品审核与电脑发来的文件")
            }
        }
    }
}

@Composable
private fun PermBullet(text: String) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Top) {
        Box(Modifier.padding(top = 6.dp).size(6.dp).clip(CircleShape).background(Brand))
        Text(text, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurface)
    }
}

/**
 * 全局系统通知桥接器：
 * - 将 AI 后台任务进度、待确认方案/审批/提问、交付与异常、电脑端审批和社区未读同步到系统通知栏
 * - 监听点击通知栏产生的 [NotificationDeepLink] 并自动路由跳转
 */
@OptIn(ExperimentalTime::class)
@Composable
internal fun NotificationSystemObserver(
    env: CommunityEnv,
    chat: CloudChatController,
    isAiTabVisible: Boolean,
    isNotificationsScreenVisible: Boolean,
    onOpenAiConversation: (String?) -> Unit,
    onOpenRemotePeer: (String) -> Unit,
    onOpenCommunityNotifications: () -> Unit,
) {
    val bridge = env.platform?.notifications ?: return
    val chatState by chat.state.collectAsState()
    val remoteState by env.remoteChat.state.collectAsState()
    val session by env.session.state.collectAsState()
    val pendingLink by bridge.pendingLink.collectAsState()

    // 1. 点击通知栏回到 App 后的直达跳转
    LaunchedEffect(pendingLink) {
        val link = pendingLink ?: return@LaunchedEffect
        bridge.consumePendingLink()
        when (link) {
            is NotificationDeepLink.AiConversation -> onOpenAiConversation(link.conversationId)
            is NotificationDeepLink.RemotePeer -> onOpenRemotePeer(link.peer)
            NotificationDeepLink.CommunityNotifications -> onOpenCommunityNotifications()
        }
    }

    // 进入社区通知页时清理系统栏的社区未读汇总
    LaunchedEffect(isNotificationsScreenVisible) {
        if (isNotificationsScreenVisible) bridge.cancelCommunitySummary()
    }

    // 2. AI 后台任务静默进度条（ai_progress）
    LaunchedEffect(
        chatState.sending,
        chatState.runningSessions,
        chatState.imageSessions,
        chatState.messages.lastOrNull()?.tools,
        chatState.creation?.title,
        isAiTabVisible,
        NotificationState.settings.aiProgress,
        NotificationState.settings.lockScreenPreview,
    ) {
        val prefs = NotificationState.settings
        val activeCount = (chatState.runningSessions + chatState.imageSessions).size.coerceAtLeast(if (chatState.sending) 1 else 0)
        val hasRunning = chatState.sending || chatState.runningSessions.isNotEmpty() || chatState.imageSessions.isNotEmpty()
        val shouldShow = prefs.aiProgress && hasRunning && (!bridge.isAppForeground() || !isAiTabVisible || chatState.backgroundTaskCount > 0 || chatState.backgroundImageCount > 0)
        if (!shouldShow) {
            bridge.cancelTaskProgress("active")
            return@LaunchedEffect
        }
        val last = chatState.messages.lastOrNull()
        val progress = last?.let { taskProgress(it.tools, running = chatState.sending) }
        val title = when {
            activeCount > 1 -> "$activeCount 个后台任务进行中"
            !chatState.creation?.title.isNullOrBlank() -> chatState.creation?.title.orEmpty()
            chatState.imageSessions.isNotEmpty() -> "正在生成图片"
            else -> "小伴正在处理任务"
        }
        val stepText = when {
            progress != null -> {
                val cur = progress.plan.steps.getOrNull(progress.current)?.content ?: "执行中"
                "步骤 ${progress.completed}/${progress.total} · $cur"
            }
            activeCount > 1 -> "点击查看并行任务进度"
            last?.tools?.lastOrNull()?.let { toolLabel(it.name) } != null -> "正在${toolLabel(last.tools.last().name)}…"
            else -> "正在后台执行，完成后将通知你"
        }
        bridge.showTaskProgress(
            id = "active",
            title = title,
            stepText = stepText,
            completed = progress?.completed ?: 0,
            total = progress?.total ?: 0,
            indeterminate = progress == null,
            lockPreview = prefs.lockScreenPreview,
        )
    }

    DisposableEffect(bridge) {
        onDispose { bridge.cancelTaskProgress("active") }
    }

    // 3. AI 任务关键节点通知（待确认方案 / 审批 / 提问 / 交付完成 / 异常中断）
    var lastApprovalId by remember { mutableStateOf<String?>(null) }
    var lastQuestionId by remember { mutableStateOf<String?>(null) }
    var lastPlanConv by remember { mutableStateOf<String?>(null) }
    var wasSending by remember { mutableStateOf(false) }

    LaunchedEffect(
        chatState.sending,
        chatState.approval?.id,
        chatState.question?.id,
        chatState.creationPlanReady,
        chatState.conversationId,
        chatState.messages.size,
    ) {
        val prefs = NotificationState.settings
        val silent = prefs.shouldSilenceNonReminder(
            bridge.currentLocalMinuteOfDay(env.utcOffsetMinutes, Clock.System.now().toEpochMilliseconds()),
        )
        val userNotWatching = !bridge.isAppForeground() || !isAiTabVisible

        val approval = chatState.approval
        if (approval != null && approval.id != lastApprovalId) {
            lastApprovalId = approval.id
            if (prefs.aiTasks && userNotWatching) {
                val highRisk = !approval.danger.isNullOrBlank()
                val title = if (highRisk) "高危操作待确认 · ${toolLabel(approval.tool)}" else "需要你的确认 · ${toolLabel(approval.tool)}"
                val body = listOfNotNull(approval.detail.takeIf { it.isNotBlank() }, approval.danger?.takeIf { it.isNotBlank() })
                    .joinToString("。").ifBlank { "请打开小伴查看并确认是否继续执行。" }
                bridge.showTaskMilestone(
                    id = "approval:${approval.id}",
                    kind = TaskNotificationKind.Approval,
                    title = title,
                    body = body,
                    conversationId = chatState.conversationId,
                    lockPreview = prefs.lockScreenPreview,
                    silent = silent,
                )
            }
        } else if (approval == null) {
            lastApprovalId = null
        }

        val question = chatState.question
        if (question != null && question.id != lastQuestionId) {
            lastQuestionId = question.id
            if (prefs.aiTasks && userNotWatching) {
                bridge.showTaskMilestone(
                    id = "question:${question.id}",
                    kind = TaskNotificationKind.Question,
                    title = "小伴向你提问",
                    body = question.question.ifBlank { "任务需要你的选择后继续执行。" },
                    conversationId = chatState.conversationId,
                    lockPreview = prefs.lockScreenPreview,
                    silent = silent,
                )
            }
        } else if (question == null) {
            lastQuestionId = null
        }

        if (chatState.creationPlanReady && chatState.conversationId != lastPlanConv) {
            lastPlanConv = chatState.conversationId
            if (prefs.aiTasks && userNotWatching) {
                val workName = chatState.creation?.title?.takeIf { it.isNotBlank() } ?: "创作方案"
                bridge.showTaskMilestone(
                    id = "plan:${chatState.conversationId}",
                    kind = TaskNotificationKind.PlanReady,
                    title = "《$workName》方案已就绪",
                    body = "打开查看方案详情，确认后小伴将开始制作。",
                    conversationId = chatState.conversationId,
                    lockPreview = prefs.lockScreenPreview,
                    silent = silent,
                )
            }
        } else if (!chatState.creationPlanReady) {
            lastPlanConv = null
        }

        // 任务本轮结束：检查是否交付完成或异常中断
        if (wasSending && !chatState.sending) {
            val last = chatState.messages.lastOrNull()
            if (prefs.aiTasks && userNotWatching && last != null && !last.cancelled && !chatState.creationPlanReady) {
                val err = last.error
                if (!err.isNullOrBlank()) {
                    bridge.showTaskMilestone(
                        id = "failed:${chatState.conversationId}",
                        kind = TaskNotificationKind.Failed,
                        title = "任务已暂停",
                        body = err.take(120),
                        conversationId = chatState.conversationId,
                        lockPreview = prefs.lockScreenPreview,
                        silent = silent,
                    )
                } else if (last.generatedImages.isNotEmpty() || !chatState.creation?.title.isNullOrBlank() || last.tools.isNotEmpty()) {
                    val doneTitle = when {
                        last.generatedImages.isNotEmpty() -> "图片已生成"
                        !chatState.creation?.title.isNullOrBlank() -> "《${chatState.creation?.title}》处理完成"
                        else -> "小伴任务已完成"
                    }
                    val doneBody = last.text.lineSequence().firstOrNull { it.isNotBlank() }?.take(100)
                        ?: "点击打开小伴查看任务结果。"
                    bridge.showTaskMilestone(
                        id = "done:${chatState.conversationId}",
                        kind = TaskNotificationKind.Completed,
                        title = doneTitle,
                        body = doneBody,
                        conversationId = chatState.conversationId,
                        lockPreview = prefs.lockScreenPreview,
                        silent = silent,
                    )
                }
            }
        }
        wasSending = chatState.sending
    }

    // 4. 遥控电脑端待审批提醒（高危操作明确要求打开 App 确认）
    var lastRemoteApprovalId by remember { mutableStateOf<String?>(null) }
    LaunchedEffect(remoteState.pendingApproval?.approvalId, remoteState.selectedPeerId) {
        val req = remoteState.pendingApproval
        if (req == null) {
            lastRemoteApprovalId = null
            return@LaunchedEffect
        }
        if (req.approvalId == lastRemoteApprovalId) return@LaunchedEffect
        lastRemoteApprovalId = req.approvalId
        val prefs = NotificationState.settings
        if (!prefs.aiTasks || bridge.isAppForeground()) return@LaunchedEffect
        val peerId = remoteState.selectedPeerId ?: return@LaunchedEffect
        val peerName = remoteState.peers.firstOrNull { it.id == peerId }?.name?.ifBlank { "电脑" } ?: "电脑"
        val silent = prefs.shouldSilenceNonReminder(
            bridge.currentLocalMinuteOfDay(env.utcOffsetMinutes, Clock.System.now().toEpochMilliseconds()),
        )
        val kind = if (req.kind == "question") TaskNotificationKind.Question else TaskNotificationKind.Approval
        val title = if (req.kind == "question") "$peerName · 小伴向你提问" else "$peerName · 请求确认操作"
        val body = listOfNotNull(req.title.takeIf { it.isNotBlank() }, req.danger?.takeIf { it.isNotBlank() }, req.question.takeIf { it.isNotBlank() })
            .joinToString("。").ifBlank { "请打开小伴查看电脑端请求。" }
        bridge.showTaskMilestone(
            id = "remote:${req.approvalId}",
            kind = kind,
            title = title,
            body = body,
            peerId = peerId,
            lockPreview = prefs.lockScreenPreview,
            silent = silent,
        )
    }

    // 5. 社区未读增量通知（仅记录基线后出现新增未读时推送，避免冷启动轰炸历史未读）
    LaunchedEffect(session.loggedIn, session.accountId, NotificationState.settings.community) {
        if (!session.loggedIn || !NotificationState.settings.community) return@LaunchedEffect
        var baseline: Int? = null
        while (isActive) {
            runCatching { env.forum.unreadCount() }.onSuccess { count ->
                val prev = baseline
                baseline = count
                if (prev != null && count > prev && !isNotificationsScreenVisible && NotificationState.settings.community) {
                    val latest = runCatching { env.social.notifications(page = 1, unread = true).items.firstOrNull() }.getOrNull()
                    val headline = latest?.let { notificationHeadline(it) } ?: "你有 $count 条未读社区互动"
                    val excerpt = latest?.excerpt.orEmpty()
                    val prefs = NotificationState.settings
                    val silent = prefs.shouldSilenceNonReminder(
                        bridge.currentLocalMinuteOfDay(env.utcOffsetMinutes, Clock.System.now().toEpochMilliseconds()),
                    )
                    bridge.showCommunitySummary(
                        unreadCount = count,
                        headline = headline,
                        excerpt = excerpt,
                        lockPreview = prefs.lockScreenPreview,
                        silent = silent,
                    )
                }
            }
            delay(90_000)
        }
    }
}
