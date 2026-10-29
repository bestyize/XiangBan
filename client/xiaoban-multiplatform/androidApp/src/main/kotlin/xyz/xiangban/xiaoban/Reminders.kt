package xyz.xiangban.xiaoban

import android.Manifest
import android.app.AlarmManager
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import xyz.xiangban.xiaoban.common.community.ReminderScheduler

private const val CHANNEL_ID = "reminders"
private const val EXTRA_TITLE = "title"

private fun ensureChannel(context: Context) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
    val manager = context.getSystemService(NotificationManager::class.java)
    if (manager.getNotificationChannel(CHANNEL_ID) == null) {
        manager.createNotificationChannel(NotificationChannel(CHANNEL_ID, "提醒", NotificationManager.IMPORTANCE_HIGH).apply { description = "小伴帮你设的提醒" })
    }
}

/**
 * 智能体的“设提醒”：用系统闹钟到点触发，弹一条通知。
 * 用的是不要求“精确闹钟”权限的闹钟，到点前后可能差几分钟；手机重启后还没响的提醒会丢，后面再补。
 */
class AndroidReminders(private val context: Context, private val askForNotifications: () -> Unit) : ReminderScheduler {
    override fun schedule(title: String, atEpochMs: Long): Boolean {
        val app = context.applicationContext
        if (Build.VERSION.SDK_INT >= 33 && app.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED) {
            askForNotifications()
            return false
        }
        ensureChannel(app)
        val intent = Intent(app, ReminderReceiver::class.java).putExtra(EXTRA_TITLE, title)
        val code = (atEpochMs xor title.hashCode().toLong()).hashCode()
        val pending = PendingIntent.getBroadcast(app, code, intent, PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE)
        val alarms = app.getSystemService(Context.ALARM_SERVICE) as AlarmManager
        alarms.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, atEpochMs, pending)
        return true
    }
}

class ReminderReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val title = intent.getStringExtra(EXTRA_TITLE).orEmpty().ifBlank { "提醒" }
        ensureChannel(context)
        val open = PendingIntent.getActivity(context, 0, Intent(context, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK), PendingIntent.FLAG_IMMUTABLE)
        @Suppress("DEPRECATION")
        val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) Notification.Builder(context, CHANNEL_ID) else Notification.Builder(context)
        val notification = builder
            .setSmallIcon(android.R.drawable.ic_popup_reminder)
            .setContentTitle("小伴提醒")
            .setContentText(title)
            .setContentIntent(open)
            .setAutoCancel(true)
            .build()
        context.getSystemService(NotificationManager::class.java).notify(title.hashCode(), notification)
    }
}
