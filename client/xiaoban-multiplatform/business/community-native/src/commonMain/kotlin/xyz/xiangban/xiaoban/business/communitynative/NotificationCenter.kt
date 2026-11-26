package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

/** 系统通知权限当前状态。 */
enum class NotificationPermissionState {
    /** 已允许通知且关键渠道可用。 */
    Granted,
    /** 尚未向系统申请（或 Android 13+ 仍可弹系统授权框）。 */
    NotDetermined,
    /** 用户已在系统侧关闭应用通知或永久拒绝。 */
    Denied,
    /** 应用通知已开，但关键渠道（如提醒）在系统设置中被关闭。 */
    ChannelDisabled,
}

/** 触发权限引导弹窗的来源场景。 */
enum class PermissionGuideReason {
    Reminder,
    Settings,
}

/** AI 任务关键节点通知类型（过程静默，仅在阻塞/终态时高优提醒）。 */
enum class TaskNotificationKind {
    PlanReady,
    Approval,
    Question,
    Completed,
    Failed,
}

/** 点击系统通知栏回到 App 后的直达目标。 */
sealed interface NotificationDeepLink {
    data class AiConversation(val conversationId: String? = null) : NotificationDeepLink
    data class RemotePeer(val peer: String) : NotificationDeepLink
    data object CommunityNotifications : NotificationDeepLink
}

/** 夜间免打扰预设时段。 */
enum class DndSchedule(val id: String, val label: String, val startMinute: Int, val endMinute: Int) {
    Night23("23:00-08:00", "23:00 – 08:00", 23 * 60, 8 * 60),
    Night22("22:00-07:30", "22:00 – 07:30", 22 * 60, 7 * 60 + 30),
    Off("off", "已关闭", -1, -1);

    fun next(): DndSchedule = when (this) {
        Night23 -> Night22
        Night22 -> Off
        Off -> Night23
    }

    /** 判断本地分钟数 [localMinuteOfDay]（0..1439）是否处于免打扰时段内。 */
    fun isActiveAt(localMinuteOfDay: Int): Boolean {
        if (this == Off) return false
        val m = ((localMinuteOfDay % 1440) + 1440) % 1440
        return if (startMinute > endMinute) m >= startMinute || m < endMinute else m in startMinute until endMinute
    }

    companion object {
        fun byId(id: String?): DndSchedule = entries.firstOrNull { it.id == id } ?: Night23
    }
}

/** 应用内通知偏好设置（保存在本机轻量偏好中）。 */
data class NotificationSettings(
    val reminders: Boolean = true,
    val aiProgress: Boolean = true,
    val aiTasks: Boolean = true,
    val community: Boolean = true,
    val linkTransfer: Boolean = true,
    val lockScreenPreview: Boolean = true,
    val soundVibrate: Boolean = true,
    val dnd: DndSchedule = DndSchedule.Night23,
) {
    /** 当前时刻发非定时提醒通知时，是否应静默送达。 */
    fun shouldSilenceNonReminder(localMinuteOfDay: Int): Boolean =
        !soundVibrate || dnd.isActiveAt(localMinuteOfDay)

    fun encode(): Map<String, String> = mapOf(
        KEY_REMINDERS to reminders.toString(),
        KEY_AI_PROGRESS to aiProgress.toString(),
        KEY_AI_TASKS to aiTasks.toString(),
        KEY_COMMUNITY to community.toString(),
        KEY_LINK_TRANSFER to linkTransfer.toString(),
        KEY_LOCK_PREVIEW to lockScreenPreview.toString(),
        KEY_SOUND_VIBRATE to soundVibrate.toString(),
        KEY_DND to dnd.id,
    )

    companion object {
        const val KEY_REMINDERS = "notif.reminders"
        const val KEY_AI_PROGRESS = "notif.ai_progress"
        const val KEY_AI_TASKS = "notif.ai_tasks"
        const val KEY_COMMUNITY = "notif.community"
        const val KEY_LINK_TRANSFER = "notif.link_transfer"
        const val KEY_LOCK_PREVIEW = "notif.lock_preview"
        const val KEY_SOUND_VIBRATE = "notif.sound_vibrate"
        const val KEY_DND = "notif.dnd"

        private fun parseBool(raw: String?, default: Boolean): Boolean = when (raw) {
            "true" -> true
            "false" -> false
            else -> default
        }

        fun decode(get: (String) -> String?): NotificationSettings = NotificationSettings(
            reminders = parseBool(get(KEY_REMINDERS), true),
            aiProgress = parseBool(get(KEY_AI_PROGRESS), true),
            aiTasks = parseBool(get(KEY_AI_TASKS), true),
            community = parseBool(get(KEY_COMMUNITY), true),
            linkTransfer = parseBool(get(KEY_LINK_TRANSFER), true),
            lockScreenPreview = parseBool(get(KEY_LOCK_PREVIEW), true),
            soundVibrate = parseBool(get(KEY_SOUND_VIBRATE), true),
            dnd = DndSchedule.byId(get(KEY_DND)),
        )
    }
}

/** 全局可观察通知设置状态，与 [ThemeState] / [SpeechState] 保持一致。 */
object NotificationState {
    var settings by mutableStateOf(NotificationSettings())
        private set

    private var prefs: AppPrefs? = null

    fun load(prefs: AppPrefs) {
        this.prefs = prefs
        settings = NotificationSettings.decode { prefs.get(it) }
    }

    fun update(change: (NotificationSettings) -> NotificationSettings) {
        val next = change(settings)
        settings = next
        prefs?.let { store -> next.encode().forEach { (k, v) -> store.put(k, v) } }
    }
}

/** 平台侧系统通知栏能力桥接（Android 5 渠道实现，iOS UNUserNotificationCenter 实现）。 */
interface SystemNotificationBridge {
    val platformSupportsChannels: Boolean get() = false
    val permissionState: StateFlow<NotificationPermissionState>
    val pendingLink: StateFlow<NotificationDeepLink?>
    val permissionGuide: StateFlow<PermissionGuideReason?>

    fun isAppForeground(): Boolean = true
    fun refreshPermission() {}
    fun requestSystemPermission() {}
    fun openSystemNotificationSettings(): Boolean = false
    fun triggerPermissionGuide(reason: PermissionGuideReason) {}
    fun dismissPermissionGuide() {}
    fun consumePendingLink() {}

    /** 常驻静默进度条通知（Android ai_progress 渠道，不响铃不弹横幅）。 */
    fun showTaskProgress(
        id: String,
        title: String,
        stepText: String,
        completed: Int,
        total: Int,
        indeterminate: Boolean,
        lockPreview: Boolean,
    ) {}

    fun cancelTaskProgress(id: String) {}

    /** 任务关键节点通知（待确认方案、高危/普通审批、提问、交付完成、异常中断）。 */
    fun showTaskMilestone(
        id: String,
        kind: TaskNotificationKind,
        title: String,
        body: String,
        conversationId: String? = null,
        peerId: String? = null,
        lockPreview: Boolean = true,
        silent: Boolean = false,
    ) {}

    fun cancelTaskMilestone(id: String) {}

    /** 社区未读消息汇总通知。 */
    fun showCommunitySummary(
        unreadCount: Int,
        headline: String,
        excerpt: String,
        lockPreview: Boolean = true,
        silent: Boolean = false,
    ) {}

    fun cancelCommunitySummary() {}

    /** 判断本地当前时间（分钟 0..1439），用于夜间免打扰判定。 */
    fun currentLocalMinuteOfDay(utcOffsetMinutes: Int, nowEpochMs: Long): Int {
        val localMs = nowEpochMs + utcOffsetMinutes * 60_000L
        val minutes = ((localMs / 60_000L) % 1440L).toInt()
        return if (minutes < 0) minutes + 1440 else minutes
    }
}

internal val EmptyNotificationPermission = MutableStateFlow(NotificationPermissionState.Granted)
