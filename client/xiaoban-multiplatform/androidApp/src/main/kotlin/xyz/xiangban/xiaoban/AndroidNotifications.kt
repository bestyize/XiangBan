package xyz.xiangban.xiaoban

import android.Manifest
import android.app.Activity
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.provider.Settings
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import xyz.xiangban.xiaoban.business.communitynative.NotificationDeepLink
import xyz.xiangban.xiaoban.business.communitynative.NotificationPermissionState
import xyz.xiangban.xiaoban.business.communitynative.NotificationSettings
import xyz.xiangban.xiaoban.business.communitynative.PermissionGuideReason
import xyz.xiangban.xiaoban.business.communitynative.SystemNotificationBridge
import xyz.xiangban.xiaoban.business.communitynative.TaskNotificationKind

internal object XiaobanNotificationChannels {
    const val REMINDERS = "reminders"
    const val AI_PROGRESS = "ai_progress"
    const val AI_TASKS = "ai_tasks"
    const val COMMUNITY = "community"
    const val LINK_TRANSFER = "link_transfer"

    const val EXTRA_TARGET = "xiaoban_notif_target"
    const val EXTRA_CONVERSATION_ID = "xiaoban_notif_conv_id"
    const val EXTRA_PEER_ID = "xiaoban_notif_peer_id"

    const val TARGET_AI = "ai"
    const val TARGET_REMOTE = "remote"
    const val TARGET_COMMUNITY = "community"

    const val TAG_PROGRESS = "xb_progress"
    const val TAG_TASK = "xb_task"
    const val TAG_COMMUNITY = "xb_community"
    const val COMMUNITY_ID = 4001

    fun ensureAll(context: Context) {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        val channels = listOf(
            NotificationChannel(REMINDERS, "提醒", NotificationManager.IMPORTANCE_HIGH).apply {
                description = "小伴帮你设的定时提醒"
                lockscreenVisibility = Notification.VISIBILITY_PUBLIC
            },
            NotificationChannel(AI_PROGRESS, "任务进度", NotificationManager.IMPORTANCE_LOW).apply {
                description = "AI 后台制作与执行的静默进度条，不响铃不震动"
                setShowBadge(false)
                enableVibration(false)
                setSound(null, null)
            },
            NotificationChannel(AI_TASKS, "任务确认与交付", NotificationManager.IMPORTANCE_HIGH).apply {
                description = "待确认方案、操作审批、提问、交付完成与异常中断提醒"
            },
            NotificationChannel(COMMUNITY, "社区互动", NotificationManager.IMPORTANCE_DEFAULT).apply {
                description = "评论、回复、点赞与作品审核动态"
            },
            NotificationChannel(LINK_TRANSFER, "互联与下载", NotificationManager.IMPORTANCE_LOW).apply {
                description = "电脑端文件链接互传与应用包下载完成"
                setShowBadge(false)
            },
        )
        manager.createNotificationChannels(channels)
    }

    fun readLockPreviewPref(context: Context): Boolean {
        val prefs = context.applicationContext.getSharedPreferences("xiaoban_ui_prefs", Context.MODE_PRIVATE)
        return prefs.getString(NotificationSettings.KEY_LOCK_PREVIEW, "true") != "false"
    }

    fun readAskedFlag(context: Context): Boolean {
        val prefs = context.applicationContext.getSharedPreferences("xiaoban_ui_prefs", Context.MODE_PRIVATE)
        return prefs.getBoolean("notif.permission_asked", false)
    }

    fun markAskedFlag(context: Context) {
        context.applicationContext.getSharedPreferences("xiaoban_ui_prefs", Context.MODE_PRIVATE)
            .edit().putBoolean("notif.permission_asked", true).apply()
    }
}

/**
 * Android 5 渠道系统通知实现：
 * - 统一使用 24dp 单色双星矢量图标 [R.drawable.ic_stat_xiaoban]
 * - 支持锁屏隐藏详情（[NotificationCompat.VISIBILITY_PRIVATE] + 公共占位版本）
 * - 支持 AI 后台任务静默常驻进度条、关键节点通知、社区未读汇总与点击直达路由
 */
class AndroidNotifications(
    private val activity: Activity,
    private val launchSystemRequest: () -> Unit,
) : SystemNotificationBridge {
    private val app = activity.applicationContext
    private val manager: NotificationManager get() = app.getSystemService(NotificationManager::class.java)

    override val platformSupportsChannels: Boolean = Build.VERSION.SDK_INT >= Build.VERSION_CODES.O

    private val _permissionState = MutableStateFlow(computePermissionState())
    override val permissionState: StateFlow<NotificationPermissionState> = _permissionState.asStateFlow()

    private val _pendingLink = MutableStateFlow<NotificationDeepLink?>(null)
    override val pendingLink: StateFlow<NotificationDeepLink?> = _pendingLink.asStateFlow()

    private val _permissionGuide = MutableStateFlow<PermissionGuideReason?>(null)
    override val permissionGuide: StateFlow<PermissionGuideReason?> = _permissionGuide.asStateFlow()

    @Volatile
    private var foreground: Boolean = true

    init {
        XiaobanNotificationChannels.ensureAll(app)
        refreshPermission()
    }

    fun setForeground(active: Boolean) {
        foreground = active
        if (active) refreshPermission()
    }

    override fun isAppForeground(): Boolean = foreground

    override fun refreshPermission() {
        XiaobanNotificationChannels.ensureAll(app)
        _permissionState.value = computePermissionState()
    }

    fun onPermissionResult() {
        XiaobanNotificationChannels.markAskedFlag(app)
        refreshPermission()
    }

    override fun requestSystemPermission() {
        if (Build.VERSION.SDK_INT >= 33) {
            XiaobanNotificationChannels.markAskedFlag(app)
            launchSystemRequest()
        } else {
            openSystemNotificationSettings()
        }
    }

    override fun openSystemNotificationSettings(): Boolean {
        val intent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).apply {
                putExtra(Settings.EXTRA_APP_PACKAGE, app.packageName)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
        } else {
            Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
                data = android.net.Uri.parse("package:${app.packageName}")
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
        }
        return runCatching { app.startActivity(intent) }.isSuccess
    }

    override fun triggerPermissionGuide(reason: PermissionGuideReason) {
        refreshPermission()
        _permissionGuide.value = reason
    }

    override fun dismissPermissionGuide() {
        _permissionGuide.value = null
    }

    override fun consumePendingLink() {
        _pendingLink.value = null
    }

    fun acceptIntent(intent: Intent?): Boolean {
        val target = intent?.getStringExtra(XiaobanNotificationChannels.EXTRA_TARGET) ?: return false
        val link = when (target) {
            XiaobanNotificationChannels.TARGET_AI ->
                NotificationDeepLink.AiConversation(intent.getStringExtra(XiaobanNotificationChannels.EXTRA_CONVERSATION_ID))
            XiaobanNotificationChannels.TARGET_REMOTE ->
                intent.getStringExtra(XiaobanNotificationChannels.EXTRA_PEER_ID)?.takeIf { it.isNotBlank() }?.let(NotificationDeepLink::RemotePeer)
            XiaobanNotificationChannels.TARGET_COMMUNITY -> NotificationDeepLink.CommunityNotifications
            else -> null
        } ?: return false
        intent.removeExtra(XiaobanNotificationChannels.EXTRA_TARGET)
        _pendingLink.value = link
        return true
    }

    override fun showTaskProgress(
        id: String,
        title: String,
        stepText: String,
        completed: Int,
        total: Int,
        indeterminate: Boolean,
        lockPreview: Boolean,
    ) {
        if (!canPost(XiaobanNotificationChannels.AI_PROGRESS)) return
        val contentIntent = buildOpenIntent(
            requestCode = ("prog:$id").hashCode(),
            target = XiaobanNotificationChannels.TARGET_AI,
            conversationId = id.takeIf { it != "global" },
        )
        val builder = NotificationCompat.Builder(app, XiaobanNotificationChannels.AI_PROGRESS)
            .setSmallIcon(R.drawable.ic_stat_xiaoban)
            .setContentTitle(title)
            .setContentText(stepText)
            .setContentIntent(contentIntent)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .setSilent(true)
            .setCategory(NotificationCompat.CATEGORY_PROGRESS)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setProgress(if (indeterminate) 0 else total.coerceAtLeast(1), if (indeterminate) 0 else completed.coerceIn(0, total.coerceAtLeast(1)), indeterminate)

        applyLockPrivacy(builder, XiaobanNotificationChannels.AI_PROGRESS, "小伴后台任务", "正在后台处理任务", lockPreview)
        runCatching { manager.notify(XiaobanNotificationChannels.TAG_PROGRESS, id.hashCode(), builder.build()) }
    }

    override fun cancelTaskProgress(id: String) {
        runCatching { manager.cancel(XiaobanNotificationChannels.TAG_PROGRESS, id.hashCode()) }
    }

    override fun showTaskMilestone(
        id: String,
        kind: TaskNotificationKind,
        title: String,
        body: String,
        conversationId: String?,
        peerId: String?,
        lockPreview: Boolean,
        silent: Boolean,
    ) {
        if (!canPost(XiaobanNotificationChannels.AI_TASKS)) return
        val target = if (!peerId.isNullOrBlank()) XiaobanNotificationChannels.TARGET_REMOTE else XiaobanNotificationChannels.TARGET_AI
        val contentIntent = buildOpenIntent(
            requestCode = ("task:$id:${kind.name}").hashCode(),
            target = target,
            conversationId = conversationId,
            peerId = peerId,
        )
        val actionLabel = when (kind) {
            TaskNotificationKind.PlanReady -> "查看方案"
            TaskNotificationKind.Approval -> "打开确认"
            TaskNotificationKind.Question -> "回到对话"
            TaskNotificationKind.Completed -> "打开查看"
            TaskNotificationKind.Failed -> "查看原因"
        }
        val builder = NotificationCompat.Builder(app, XiaobanNotificationChannels.AI_TASKS)
            .setSmallIcon(R.drawable.ic_stat_xiaoban)
            .setContentTitle(title)
            .setContentText(body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(body))
            .setContentIntent(contentIntent)
            .setAutoCancel(true)
            .setOnlyAlertOnce(true)
            .setSilent(silent)
            .setCategory(
                when (kind) {
                    TaskNotificationKind.Approval, TaskNotificationKind.Question -> NotificationCompat.CATEGORY_STATUS
                    TaskNotificationKind.Failed -> NotificationCompat.CATEGORY_ERROR
                    else -> NotificationCompat.CATEGORY_MESSAGE
                },
            )
            .setPriority(if (silent) NotificationCompat.PRIORITY_LOW else NotificationCompat.PRIORITY_HIGH)
            .addAction(0, actionLabel, contentIntent)

        val publicSubtitle = when (kind) {
            TaskNotificationKind.PlanReady -> "有 1 个创作方案等待确认"
            TaskNotificationKind.Approval -> "有 1 项操作等待打开确认"
            TaskNotificationKind.Question -> "小伴正在等待你的回复"
            TaskNotificationKind.Completed -> "有 1 个任务已完成"
            TaskNotificationKind.Failed -> "有 1 个任务已暂停"
        }
        applyLockPrivacy(builder, XiaobanNotificationChannels.AI_TASKS, "小伴 AI", publicSubtitle, lockPreview)
        runCatching { manager.notify(XiaobanNotificationChannels.TAG_TASK, id.hashCode(), builder.build()) }
    }

    override fun cancelTaskMilestone(id: String) {
        runCatching { manager.cancel(XiaobanNotificationChannels.TAG_TASK, id.hashCode()) }
    }

    override fun showCommunitySummary(
        unreadCount: Int,
        headline: String,
        excerpt: String,
        lockPreview: Boolean,
        silent: Boolean,
    ) {
        if (unreadCount <= 0 || !canPost(XiaobanNotificationChannels.COMMUNITY)) return
        val contentIntent = buildOpenIntent(
            requestCode = XiaobanNotificationChannels.COMMUNITY_ID,
            target = XiaobanNotificationChannels.TARGET_COMMUNITY,
        )
        val title = if (unreadCount > 1) "$unreadCount 条新社区互动" else headline.ifBlank { "新的社区互动" }
        val text = if (unreadCount > 1) {
            listOf(headline, excerpt).filter { it.isNotBlank() }.joinToString("：")
        } else {
            excerpt.ifBlank { headline }
        }
        val builder = NotificationCompat.Builder(app, XiaobanNotificationChannels.COMMUNITY)
            .setSmallIcon(R.drawable.ic_stat_xiaoban)
            .setContentTitle(title)
            .setContentText(text)
            .setStyle(NotificationCompat.BigTextStyle().bigText(text))
            .setNumber(unreadCount)
            .setContentIntent(contentIntent)
            .setAutoCancel(true)
            .setOnlyAlertOnce(true)
            .setSilent(silent)
            .setCategory(NotificationCompat.CATEGORY_SOCIAL)
            .setPriority(if (silent) NotificationCompat.PRIORITY_LOW else NotificationCompat.PRIORITY_DEFAULT)
            .addAction(0, "查看互动", contentIntent)

        applyLockPrivacy(builder, XiaobanNotificationChannels.COMMUNITY, "小伴社区", "收到 $unreadCount 条新消息", lockPreview)
        runCatching { manager.notify(XiaobanNotificationChannels.TAG_COMMUNITY, XiaobanNotificationChannels.COMMUNITY_ID, builder.build()) }
    }

    override fun cancelCommunitySummary() {
        runCatching { manager.cancel(XiaobanNotificationChannels.TAG_COMMUNITY, XiaobanNotificationChannels.COMMUNITY_ID) }
    }

    private fun buildOpenIntent(
        requestCode: Int,
        target: String,
        conversationId: String? = null,
        peerId: String? = null,
    ): PendingIntent {
        val intent = Intent(app, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_NEW_TASK
            putExtra(XiaobanNotificationChannels.EXTRA_TARGET, target)
            if (!conversationId.isNullOrBlank()) putExtra(XiaobanNotificationChannels.EXTRA_CONVERSATION_ID, conversationId)
            if (!peerId.isNullOrBlank()) putExtra(XiaobanNotificationChannels.EXTRA_PEER_ID, peerId)
        }
        return PendingIntent.getActivity(app, requestCode, intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
    }

    private fun applyLockPrivacy(
        builder: NotificationCompat.Builder,
        channelId: String,
        publicTitle: String,
        publicText: String,
        lockPreview: Boolean,
    ) {
        if (lockPreview) {
            builder.setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
        } else {
            val publicVersion = NotificationCompat.Builder(app, channelId)
                .setSmallIcon(R.drawable.ic_stat_xiaoban)
                .setContentTitle(publicTitle)
                .setContentText(publicText)
                .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
                .build()
            builder.setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
            builder.setPublicVersion(publicVersion)
        }
    }

    private fun canPost(channelId: String): Boolean {
        if (Build.VERSION.SDK_INT >= 33 &&
            app.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) return false
        val nm = manager
        if (!nm.areNotificationsEnabled()) return false
        XiaobanNotificationChannels.ensureAll(app)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val ch = nm.getNotificationChannel(channelId)
            if (ch != null && ch.importance == NotificationManager.IMPORTANCE_NONE) return false
        }
        return true
    }

    private fun computePermissionState(): NotificationPermissionState {
        if (Build.VERSION.SDK_INT >= 33) {
            val granted = app.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED
            if (!granted) {
                val askedBefore = XiaobanNotificationChannels.readAskedFlag(app)
                val shouldShowRationale = ActivityCompat.shouldShowRequestPermissionRationale(activity, Manifest.permission.POST_NOTIFICATIONS)
                return if (!askedBefore || shouldShowRationale) NotificationPermissionState.NotDetermined
                else NotificationPermissionState.Denied
            }
        }
        val nm = manager
        if (!nm.areNotificationsEnabled()) return NotificationPermissionState.Denied
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            XiaobanNotificationChannels.ensureAll(app)
            val reminders = nm.getNotificationChannel(XiaobanNotificationChannels.REMINDERS)
            if (reminders != null && reminders.importance == NotificationManager.IMPORTANCE_NONE) {
                return NotificationPermissionState.ChannelDisabled
            }
        }
        return NotificationPermissionState.Granted
    }
}
