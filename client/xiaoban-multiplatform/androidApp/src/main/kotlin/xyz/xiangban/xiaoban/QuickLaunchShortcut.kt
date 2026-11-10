package xyz.xiangban.xiaoban

import android.app.Activity
import android.app.PendingIntent
import android.content.Intent
import android.content.pm.ShortcutInfo
import android.content.pm.ShortcutManager
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.Path
import android.graphics.drawable.Icon
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.widget.Toast
import java.util.UUID

/** Targets are chosen by the host, never supplied as an arbitrary URI by a work. */
internal object QuickLaunchShortcut {
    data class Target(val name: String, val packageName: String, val uri: String, val image: String)
    val targets = mapOf(
        "微信" to Target("微信", "com.tencent.mm", "weixin://", "wechat.webp"),
        "抖音" to Target("抖音", "com.ss.android.ugc.aweme", "snssdk1128://", "douyin.webp"),
        "迅雷" to Target("迅雷", "com.xunlei.downloadprovider", "thunder://", "appstore-1503466530.webp"),
        "千问" to Target("千问", "com.aliyun.tongyi", "tongyi://", "appstore-6466733523.webp"),
        "QQ" to Target("QQ", "com.tencent.mobileqq", "mqq://", "qq.webp"),
        "支付宝" to Target("支付宝", "com.eg.android.AlipayGphone", "alipay://", "alipay.webp"),
        "微博" to Target("微博", "com.sina.weibo", "sinaweibo://", "weibo.webp"),
        "知乎" to Target("知乎", "com.zhihu.android", "zhihu://", "zhihu.webp"),
        "美团" to Target("美团", "com.sankuai.meituan", "imeituan://", "meituan.webp"),
        "淘宝" to Target("淘宝", "com.taobao.taobao", "taobao://", "taobao.webp"),
        "京东" to Target("京东", "com.jingdong.app.mall", "openapp.jdmobile://", "jd.webp"),
        "哔哩哔哩" to Target("哔哩哔哩", "tv.danmaku.bili", "bilibili://", "bilibili.webp"),
        "钉钉" to Target("钉钉", "com.alibaba.android.rimet", "dingtalk://", "dingtalk.webp"),
        "企业微信" to Target("企业微信", "com.tencent.wework", "wxwork://", "appstore-1087897068.webp"),
        "飞书" to Target("飞书", "com.ss.android.lark", "lark://", "feishu.webp"),
        "小红书" to Target("小红书", "com.xingin.xhs", "xhsdiscover://home/explore", "xiaohongshu.webp"),
        "拼多多" to Target("拼多多", "com.xunmeng.pinduoduo", "pinduoduo://", "pdd.webp"),
        "快手" to Target("快手", "com.smile.gifmaker", "kwai://home", "kuaishou.webp"),
        "高德地图" to Target("高德地图", "com.autonavi.minimap", "androidamap://", "amap.webp"),
        "网易云音乐" to Target("网易云音乐", "com.netease.cloudmusic", "orpheus://", "netease.webp"),
        "QQ音乐" to Target("QQ音乐", "com.tencent.qqmusic", "qqmusic://", "appstore-414603431.webp"),
    )
    fun available(activity: Activity, name: String): Boolean {
        val target = targets[name] ?: return false
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(target.uri)).setPackage(target.packageName)
        return intent.resolveActivity(activity.packageManager) != null
    }
    fun open(activity: Activity, name: String): Boolean {
        val target = targets[name] ?: return false
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(target.uri)).setPackage(target.packageName)
        return try { activity.startActivity(intent); true } catch (_: Exception) { false }
    }
    fun isPinned(activity: Activity, name: String): Boolean =
        Build.VERSION.SDK_INT >= 26 && activity.getSystemService(ShortcutManager::class.java).pinnedShortcuts.any {
            it.id == "quick-launch:$name" && it.isEnabled
        }
    fun pin(activity: Activity, name: String): Boolean {
        if (Build.VERSION.SDK_INT < 26) return false
        val target = targets[name] ?: return false
        if (!available(activity, name)) return false
        if (isPinned(activity, name)) return true
        val manager = activity.getSystemService(ShortcutManager::class.java)
        if (!manager.isRequestPinShortcutSupported) return false
        val token = UUID.randomUUID().toString()
        val launch = Intent(activity, QuickLaunchEntryActivity::class.java).setAction(ACTION)
            .putExtra("name", name).putExtra("token", token)
        val bitmap = activity.assets.open("game-packs-v2/quick-launch/icons/${target.image}").use { BitmapFactory.decodeStream(it) }
        val icon = bitmap.copy(android.graphics.Bitmap.Config.ARGB_8888, true)
        val canvas = Canvas(icon)
        val scale = icon.width / 256f
        val p = Paint(Paint.ANTI_ALIAS_FLAG)
        p.color = Color.WHITE; canvas.drawCircle(218f * scale, 218f * scale, 32f * scale, p)
        p.color = Color.rgb(255, 215, 69); canvas.drawCircle(218f * scale, 218f * scale, 27f * scale, p)
        p.color = Color.WHITE
        val bolt = Path().apply { moveTo(220f*scale,198f*scale); lineTo(207f*scale,218f*scale); lineTo(216f*scale,218f*scale); lineTo(213f*scale,237f*scale); lineTo(229f*scale,214f*scale); lineTo(220f*scale,214f*scale); close() }
        canvas.drawPath(bolt, p)
        val shortcut = ShortcutInfo.Builder(activity, "quick-launch:$name").setShortLabel(name)
            .setIcon(Icon.createWithBitmap(icon)).setIntent(launch).build()
        val callback = PendingIntent.getBroadcast(activity, name.hashCode(), Intent(activity, RuntimeWorkShortcutResult::class.java), PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_ONE_SHOT)
        return manager.requestPinShortcut(shortcut, callback.intentSender)
    }
    const val ACTION = "xyz.xiangban.xiaoban.action.QUICK_LAUNCH"
}

class QuickLaunchEntryActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val name = intent.getStringExtra("name")
        val token = intent.getStringExtra("token")
        val valid = if (Build.VERSION.SDK_INT >= 26 && intent.action == QuickLaunchShortcut.ACTION && name != null && token != null) {
            getSystemService(ShortcutManager::class.java).pinnedShortcuts.any {
                it.id == "quick-launch:$name" && it.isEnabled && it.intent?.getStringExtra("token") == token
            }
        } else false
        if (!valid || !QuickLaunchShortcut.open(this, name.orEmpty())) Toast.makeText(this, "无法打开应用，请确认已安装", Toast.LENGTH_SHORT).show()
        finish()
    }
}
