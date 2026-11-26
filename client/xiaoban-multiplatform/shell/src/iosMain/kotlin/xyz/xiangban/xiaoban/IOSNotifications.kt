package xyz.xiangban.xiaoban

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import platform.Foundation.NSURL
import platform.UIKit.UIApplication
import platform.UIKit.UIApplicationOpenSettingsURLString
import platform.UIKit.UIApplicationState
import platform.UserNotifications.UNAuthorizationOptionAlert
import platform.UserNotifications.UNAuthorizationOptionBadge
import platform.UserNotifications.UNAuthorizationOptionSound
import platform.UserNotifications.UNAuthorizationStatusAuthorized
import platform.UserNotifications.UNAuthorizationStatusDenied
import platform.UserNotifications.UNAuthorizationStatusEphemeral
import platform.UserNotifications.UNAuthorizationStatusNotDetermined
import platform.UserNotifications.UNAuthorizationStatusProvisional
import platform.UserNotifications.UNMutableNotificationContent
import platform.UserNotifications.UNNotificationRequest
import platform.UserNotifications.UNNotificationSound
import platform.UserNotifications.UNUserNotificationCenter
import xyz.xiangban.xiaoban.business.communitynative.NotificationDeepLink
import xyz.xiangban.xiaoban.business.communitynative.NotificationPermissionState
import xyz.xiangban.xiaoban.business.communitynative.PermissionGuideReason
import xyz.xiangban.xiaoban.business.communitynative.SystemNotificationBridge
import xyz.xiangban.xiaoban.business.communitynative.TaskNotificationKind

/**
 * iOS 本地系统通知实现：
 * 基于 [UNUserNotificationCenter] 查询/请求权限、投递任务节点与社区汇总通知并支持跳系统通知设置。
 * 灵动岛与锁屏实时活动（Live Activity）需独立 Widget Extension，当前阶段用标准本地通知承载。
 */
class IOSNotifications : SystemNotificationBridge {
    private val center: UNUserNotificationCenter get() = UNUserNotificationCenter.currentNotificationCenter()

    private val _permissionState = MutableStateFlow(NotificationPermissionState.NotDetermined)
    override val permissionState: StateFlow<NotificationPermissionState> = _permissionState.asStateFlow()

    private val _pendingLink = MutableStateFlow<NotificationDeepLink?>(null)
    override val pendingLink: StateFlow<NotificationDeepLink?> = _pendingLink.asStateFlow()

    private val _permissionGuide = MutableStateFlow<PermissionGuideReason?>(null)
    override val permissionGuide: StateFlow<PermissionGuideReason?> = _permissionGuide.asStateFlow()

    init {
        refreshPermission()
    }

    override fun isAppForeground(): Boolean =
        UIApplication.sharedApplication.applicationState == UIApplicationState.UIApplicationStateActive

    override fun refreshPermission() {
        center.getNotificationSettingsWithCompletionHandler { settings ->
            val mapped = when (settings?.authorizationStatus) {
                UNAuthorizationStatusAuthorized,
                UNAuthorizationStatusProvisional,
                UNAuthorizationStatusEphemeral,
                -> NotificationPermissionState.Granted
                UNAuthorizationStatusDenied -> NotificationPermissionState.Denied
                UNAuthorizationStatusNotDetermined -> NotificationPermissionState.NotDetermined
                else -> NotificationPermissionState.NotDetermined
            }
            _permissionState.value = mapped
        }
    }

    override fun requestSystemPermission() {
        val options = UNAuthorizationOptionAlert or UNAuthorizationOptionSound or UNAuthorizationOptionBadge
        center.requestAuthorizationWithOptions(options) { _, _ -> refreshPermission() }
    }

    override fun openSystemNotificationSettings(): Boolean {
        val url = NSURL.URLWithString(UIApplicationOpenSettingsURLString) ?: return false
        UIApplication.sharedApplication.openURL(url, emptyMap<Any?, Any>(), null)
        return true
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
        if (_permissionState.value != NotificationPermissionState.Granted) return
        val content = UNMutableNotificationContent().apply {
            setTitle(if (lockPreview) title else "小伴 AI")
            setBody(if (lockPreview) body else "有 1 条任务动态，解锁后查看详情")
            setThreadIdentifier("ai_tasks")
            if (!silent) setSound(UNNotificationSound.defaultSound())
        }
        val req = UNNotificationRequest.requestWithIdentifier("xb-task-$id", content, null)
        center.addNotificationRequest(req, null)
    }

    override fun cancelTaskMilestone(id: String) {
        center.removeDeliveredNotificationsWithIdentifiers(listOf("xb-task-$id"))
    }

    override fun showCommunitySummary(
        unreadCount: Int,
        headline: String,
        excerpt: String,
        lockPreview: Boolean,
        silent: Boolean,
    ) {
        if (unreadCount <= 0 || _permissionState.value != NotificationPermissionState.Granted) return
        val content = UNMutableNotificationContent().apply {
            setTitle(if (lockPreview) (if (unreadCount > 1) "$unreadCount 条新社区互动" else headline.ifBlank { "新的社区互动" }) else "小伴社区")
            setBody(if (lockPreview) excerpt.ifBlank { headline } else "收到 $unreadCount 条新消息")
            setThreadIdentifier("community")
            if (!silent) setSound(UNNotificationSound.defaultSound())
        }
        val req = UNNotificationRequest.requestWithIdentifier("xb-community-summary", content, null)
        center.addNotificationRequest(req, null)
    }

    override fun cancelCommunitySummary() {
        center.removeDeliveredNotificationsWithIdentifiers(listOf("xb-community-summary"))
    }
}
