package xyz.xiangban.xiaoban

import android.Manifest
import android.app.AlarmManager
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import androidx.core.app.NotificationCompat
import org.json.JSONObject
import java.util.UUID
import xyz.xiangban.xiaoban.common.community.ReminderScheduler
import xyz.xiangban.xiaoban.common.community.ReminderScheduleResult
import xyz.xiangban.xiaoban.common.community.LocalReminder
import xyz.xiangban.xiaoban.common.community.ReminderWriteResult

private const val CHANNEL_ID = XiaobanNotificationChannels.REMINDERS
private const val ACTION_FIRE = "xyz.xiangban.xiaoban.REMINDER_FIRE"
private const val ACTION_SNOOZE = "xyz.xiangban.xiaoban.REMINDER_SNOOZE"
private const val ACTION_DONE = "xyz.xiangban.xiaoban.REMINDER_DONE"
private const val SNOOZE_MS = 10 * 60 * 1000L
private val reminderLock = Any()

private enum class Status { WaitingForPermission, Scheduled, Fired, Failed }

private data class Reminder(val id: String, val title: String, val at: Long, val status: Status)

private fun Reminder.publicRecord() = LocalReminder(id, title, at, when (status) {
    Status.WaitingForPermission -> "waiting_for_permission"
    Status.Scheduled -> "scheduled"
    Status.Fired -> "fired"
    Status.Failed -> "failed"
})

private class ReminderStore(context: Context) {
    private val prefs = context.getSharedPreferences("local_reminders", Context.MODE_PRIVATE)

    fun read(id: String): Reminder? = prefs.getString(id, null)?.let { raw ->
        runCatching {
            val row = JSONObject(raw)
            Reminder(id, row.getString("title"), row.getLong("at"), Status.valueOf(row.getString("status")))
        }.getOrNull()
    }

    fun all(): List<Reminder> = prefs.all.keys.mapNotNull(::read)

    fun save(reminder: Reminder): Boolean = prefs.edit().putString(reminder.id, JSONObject()
        .put("title", reminder.title).put("at", reminder.at).put("status", reminder.status.name).toString()).commit()

    fun remove(id: String): Boolean = prefs.edit().remove(id).commit()

    fun prune(now: Long) {
        val expired = all().filter { (it.status == Status.Fired || it.status == Status.Failed) && now - it.at > 30 * 24 * 60 * 60 * 1000L }
        if (expired.isEmpty()) return
        val edit = prefs.edit()
        expired.forEach { edit.remove(it.id) }
        edit.apply()
    }
}

private fun missingPermission(context: Context): Boolean =
    Build.VERSION.SDK_INT >= 33 && context.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED

private fun notificationManager(context: Context): NotificationManager =
    context.getSystemService(NotificationManager::class.java)

private fun notificationsEnabled(context: Context): Boolean {
    if (missingPermission(context)) return false
    val manager = notificationManager(context)
    if (!manager.areNotificationsEnabled()) return false
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return true
    XiaobanNotificationChannels.ensureAll(context)
    return manager.getNotificationChannel(CHANNEL_ID)?.importance != NotificationManager.IMPORTANCE_NONE
}

private fun alarmIntent(context: Context, id: String, action: String = ACTION_FIRE): Intent =
    Intent(context, ReminderReceiver::class.java)
        .setAction(action)
        .setData(Uri.Builder().scheme("xiaoban-reminder").authority("alarm").appendPath(id).build())

private fun registerAlarm(context: Context, reminder: Reminder) {
    val pending = PendingIntent.getBroadcast(context, 0, alarmIntent(context, reminder.id), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
    // Deliberately use an inexact alarm. The system can defer it under battery restrictions.
    context.getSystemService(AlarmManager::class.java)
        .setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, maxOf(reminder.at, System.currentTimeMillis() + 1), pending)
}

private fun cancelAlarm(context: Context, id: String) {
    val pending = PendingIntent.getBroadcast(context, 0, alarmIntent(context, id), PendingIntent.FLAG_NO_CREATE or PendingIntent.FLAG_IMMUTABLE) ?: return
    context.getSystemService(AlarmManager::class.java).cancel(pending)
    pending.cancel()
}

/** The reminder is committed before requesting permission or registering its system alarm. */
class AndroidReminders(
    context: Context,
    private val askForNotifications: () -> Unit,
    private val onNotificationsDisabled: () -> Unit = {},
) : ReminderScheduler {
    private val app = context.applicationContext
    private val store = ReminderStore(app)
    override val supportsManagement = true

    override fun schedule(title: String, atEpochMs: Long): Boolean =
        scheduleResult(title, atEpochMs) == ReminderScheduleResult.Scheduled

    override fun scheduleResult(title: String, atEpochMs: Long): ReminderScheduleResult =
        createReminder(title, atEpochMs).status

    override fun createReminder(title: String, atEpochMs: Long): ReminderWriteResult {
        val result = synchronized(reminderLock) {
            val reminder = store.all().firstOrNull {
                it.title == title && it.at == atEpochMs && it.status in setOf(Status.WaitingForPermission, Status.Scheduled)
            } ?: Reminder(UUID.randomUUID().toString(), title, atEpochMs, Status.Scheduled)
            saveAndSchedule(reminder)
        }
        when (result.status) {
            ReminderScheduleResult.AwaitingPermission -> askForNotifications()
            ReminderScheduleResult.NotificationsDisabled -> onNotificationsDisabled()
            else -> Unit
        }
        return result
    }

    override fun listReminders(): List<LocalReminder> = synchronized(reminderLock) {
        store.prune(System.currentTimeMillis())
        store.all().map { it.publicRecord() }
    }

    override fun updateReminder(id: String, title: String?, atEpochMs: Long?): ReminderWriteResult {
        val result = synchronized(reminderLock) {
            val existing = store.read(id) ?: return@synchronized ReminderWriteResult(ReminderScheduleResult.NotFound)
            saveAndSchedule(existing.copy(title = title ?: existing.title, at = atEpochMs ?: existing.at))
        }
        when (result.status) {
            ReminderScheduleResult.AwaitingPermission -> askForNotifications()
            ReminderScheduleResult.NotificationsDisabled -> onNotificationsDisabled()
            else -> Unit
        }
        return result
    }

    override fun cancelReminder(id: String): Boolean = synchronized(reminderLock) {
        if (store.read(id) == null) return@synchronized false
        // Remove durable state first: an already dispatched broadcast must also become harmless.
        check(store.remove(id)) { "提醒删除失败，请稍后重试" }
        runCatching { cancelAlarm(app, id) }
        runCatching { notificationManager(app).cancel(id, 0) }
        true
    }

    private fun saveAndSchedule(reminder: Reminder): ReminderWriteResult {
        if (reminder.at <= System.currentTimeMillis() || reminder.title.isBlank()) return ReminderWriteResult(ReminderScheduleResult.Failed)
        val needsPermission = missingPermission(app)
        if (!needsPermission && !notificationsEnabled(app)) return ReminderWriteResult(ReminderScheduleResult.NotificationsDisabled)
        val status = if (needsPermission) {
            if (store.save(reminder.copy(status = Status.WaitingForPermission))) {
                runCatching { cancelAlarm(app, reminder.id) }
                ReminderScheduleResult.AwaitingPermission
            } else ReminderScheduleResult.Failed
        } else activate(reminder)
        return ReminderWriteResult(status, store.read(reminder.id)?.publicRecord())
    }

    private fun activate(reminder: Reminder): ReminderScheduleResult {
        if (!notificationsEnabled(app)) return ReminderScheduleResult.NotificationsDisabled
        val scheduled = reminder.copy(status = Status.Scheduled)
        if (!store.save(scheduled)) return ReminderScheduleResult.Failed
        return try {
            registerAlarm(app, scheduled)
            ReminderScheduleResult.Scheduled
        } catch (_: RuntimeException) {
            store.save(scheduled.copy(status = Status.Failed))
            runCatching { cancelAlarm(app, scheduled.id) }
            ReminderScheduleResult.Failed
        }
    }

    fun permissionResult(granted: Boolean): Pair<Int, Int> = synchronized(reminderLock) {
        val waiting = store.all().filter { it.status == Status.WaitingForPermission }
        var activated = 0
        waiting.forEach { reminder ->
            if (granted && activate(reminder) == ReminderScheduleResult.Scheduled) activated++
            else store.save(reminder.copy(status = Status.Failed))
        }
        activated to (waiting.size - activated)
    }

    fun restore() = synchronized(reminderLock) {
        store.prune(System.currentTimeMillis())
        if (!notificationsEnabled(app)) return@synchronized
        store.all().filter { it.status == Status.Scheduled || it.status == Status.WaitingForPermission }
            .forEach { activate(it) }
    }
}

class ReminderReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val app = context.applicationContext
        val action = intent.action
        val pending = goAsync()
        Thread {
            try {
                when (action) {
                    Intent.ACTION_BOOT_COMPLETED, Intent.ACTION_MY_PACKAGE_REPLACED -> AndroidReminders(app, {}) .restore()
                    ACTION_FIRE -> intent.data?.lastPathSegment?.let { fire(app, it) }
                    ACTION_SNOOZE -> intent.data?.lastPathSegment?.let { snooze(app, it) }
                    ACTION_DONE -> intent.data?.lastPathSegment?.let { done(app, it) }
                    null -> intent.getStringExtra("title")?.let { title ->
                        // Alarms registered by the previous app version can still arrive after an update.
                        if (notificationsEnabled(app)) notifyReminder(app, "legacy-" + title.hashCode(), title)
                    }
                }
            } finally { pending.finish() }
        }.start()
    }

    private fun fire(context: Context, id: String) = synchronized(reminderLock) {
        val store = ReminderStore(context)
        val reminder = store.read(id)?.takeIf { it.status == Status.Scheduled } ?: return@synchronized
        // A broadcast from the old alarm can race an update using the same stable id.
        if (reminder.at > System.currentTimeMillis()) {
            runCatching { registerAlarm(context, reminder) }
            return@synchronized
        }
        val delivered = runCatching {
            check(notificationsEnabled(context))
            notifyReminder(context, id, reminder.title)
        }.isSuccess
        store.save(reminder.copy(status = if (delivered) Status.Fired else Status.Failed))
        Unit
    }

    private fun snooze(context: Context, id: String) = synchronized(reminderLock) {
        runCatching { notificationManager(context).cancel(id, 0) }
        val store = ReminderStore(context)
        val reminder = store.read(id) ?: return@synchronized
        if (!notificationsEnabled(context)) return@synchronized
        val next = reminder.copy(at = System.currentTimeMillis() + SNOOZE_MS, status = Status.Scheduled)
        if (store.save(next)) {
            runCatching { registerAlarm(context, next) }
        }
    }

    private fun done(context: Context, id: String) = synchronized(reminderLock) {
        runCatching { notificationManager(context).cancel(id, 0) }
    }
}

private fun notifyReminder(context: Context, id: String, title: String) {
    XiaobanNotificationChannels.ensureAll(context)
    val openIntent = Intent(context, MainActivity::class.java).apply {
        flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_NEW_TASK
        putExtra(XiaobanNotificationChannels.EXTRA_TARGET, XiaobanNotificationChannels.TARGET_AI)
    }
    val open = PendingIntent.getActivity(context, id.hashCode(), openIntent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
    val snooze = PendingIntent.getBroadcast(
        context,
        ("snooze:$id").hashCode(),
        alarmIntent(context, id, ACTION_SNOOZE),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )
    val done = PendingIntent.getBroadcast(
        context,
        ("done:$id").hashCode(),
        alarmIntent(context, id, ACTION_DONE),
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
    )
    val lockPreview = XiaobanNotificationChannels.readLockPreviewPref(context)
    val builder = NotificationCompat.Builder(context, CHANNEL_ID)
        .setSmallIcon(R.drawable.ic_stat_xiaoban)
        .setContentTitle("小伴提醒")
        .setContentText(title)
        .setStyle(NotificationCompat.BigTextStyle().bigText(title))
        .setContentIntent(open)
        .setAutoCancel(true)
        .setOnlyAlertOnce(true)
        .setCategory(NotificationCompat.CATEGORY_REMINDER)
        .setPriority(NotificationCompat.PRIORITY_HIGH)
        .addAction(0, "延迟 10 分钟", snooze)
        .addAction(0, "已完成", done)

    if (lockPreview) {
        builder.setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
    } else {
        val publicVersion = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_stat_xiaoban)
            .setContentTitle("小伴提醒")
            .setContentText("解锁后查看 1 条提醒内容")
            .setVisibility(NotificationCompat.VISIBILITY_PUBLIC)
            .build()
        builder.setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
        builder.setPublicVersion(publicVersion)
    }
    notificationManager(context).notify(id, 0, builder.build())
}
