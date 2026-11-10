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
    // This is the same curated 104-app catalog as the bundled Quick Launch work.
    // Home launches intentionally use Android's MAIN/LAUNCHER activity rather than
    // guessing private URL schemes (e.g. a bare "bilibili://" URI).
    data class Target(
        val name: String,
        val image: String,
        val packages: List<String> = emptyList(),
        val labels: List<String> = emptyList(),
        val homepage: String? = null,
    )

    val targets: Map<String, Target> = listOf(
        Target("微信", "wechat.webp", packages = listOf("com.tencent.mm")),
        Target("支付宝", "alipay.webp", packages = listOf("com.eg.android.AlipayGphone")),
        Target("淘宝", "taobao.webp", packages = listOf("com.taobao.taobao")),
        Target("京东", "jd.webp", packages = listOf("com.jingdong.app.mall")),
        Target("拼多多", "pdd.webp", packages = listOf("com.xunmeng.pinduoduo")),
        Target("美团", "meituan.webp", packages = listOf("com.sankuai.meituan")),
        Target("哔哩哔哩", "bilibili.webp", packages = listOf("tv.danmaku.bili"), labels = listOf("bilibili"), homepage = "bilibili://home"),
        Target("微博", "weibo.webp", packages = listOf("com.sina.weibo")),
        Target("抖音", "douyin.webp", packages = listOf("com.ss.android.ugc.aweme")),
        Target("小红书", "xiaohongshu.webp", packages = listOf("com.xingin.xhs")),
        Target("QQ", "qq.webp", packages = listOf("com.tencent.mobileqq")),
        Target("高德地图", "amap.webp", packages = listOf("com.autonavi.minimap")),
        Target("淘宝闪购", "shan_gou.webp", packages = listOf("com.taobao.ele.me"), labels = listOf("饿了么")),
        Target("网易云音乐", "netease.webp", packages = listOf("com.netease.cloudmusic")),
        Target("知乎", "zhihu.webp", packages = listOf("com.zhihu.android")),
        Target("快手", "kuaishou.webp", packages = listOf("com.smile.gifmaker")),
        Target("百度网盘", "baidunetdisk.webp", packages = listOf("com.baidu.netdisk")),
        Target("钉钉", "dingtalk.webp", packages = listOf("com.alibaba.android.rimet")),
        Target("百度地图", "baidumap.webp", packages = listOf("com.baidu.BaiduMap")),
        Target("滴滴出行", "didi.webp", packages = listOf("com.sdu.didi.psnger")),
        Target("携程旅行", "ctrip.webp", packages = listOf("ctrip.android.view")),
        Target("闲鱼", "xianyu.webp", packages = listOf("com.taobao.idlefish")),
        Target("飞书", "feishu.webp", packages = listOf("com.ss.android.lark")),
        Target("红果短剧", "appstore-6451407032.webp", labels = listOf("红果免费短剧")),
        Target("红果漫剧", "appstore-6745890963.webp"),
        Target("国家网络身份认证", "appstore-1588080869.webp", labels = listOf("国家网络身份认证APP")),
        Target("千问", "appstore-6466733523.webp", packages = listOf("com.aliyun.tongyi"), labels = listOf("通义千问", "通义")),
        Target("豆包", "appstore-6459478672.webp", packages = listOf("com.larus.nova")),
        Target("汽水音乐", "appstore-1605585211.webp"),
        Target("云闪付", "appstore-600273928.webp"),
        Target("WorkBuddy", "appstore-6761374913.webp"),
        Target("芒果TV", "appstore-629774477.webp"),
        Target("抖省省", "appstore-6756863766.webp"),
        Target("剪映", "appstore-1458072671.webp"),
        Target("中国移动（手机营业厅）", "appstore-583700738.webp", labels = listOf("中国移动", "中国移动APP")),
        Target("电子税务局", "appstore-6468941332.webp"),
        Target("得物", "appstore-1012871328.webp", packages = listOf("com.shizhuang.duapp")),
        Target("夸克", "appstore-1160172628.webp", packages = listOf("com.quark.browser")),
        Target("交管12123", "appstore-1039727169.webp"),
        Target("迅雷", "appstore-1503466530.webp", packages = listOf("com.xunlei.downloadprovider")),
        Target("UC浏览器", "appstore-586871187.webp", packages = listOf("com.UCMobile")),
        Target("铁路12306", "appstore-564818797.webp", packages = listOf("com.MobileTicket"), labels = listOf("12306")),
        Target("转转", "appstore-1002355194.webp"),
        Target("阿里巴巴(1688)", "appstore-507097717.webp", labels = listOf("1688", "阿里巴巴")),
        Target("腾讯会议", "appstore-1484048379.webp", packages = listOf("com.tencent.wemeet.app")),
        Target("今日水印相机", "appstore-1319071502.webp"),
        Target("醒图", "appstore-1500526240.webp"),
        Target("中国农业银行", "appstore-515651240.webp"),
        Target("WPS Office", "appstore-599852710.webp", packages = listOf("cn.wps.moffice_eng"), labels = listOf("WPS")),
        Target("扫描全能王", "appstore-388627783.webp"),
        Target("企业微信", "appstore-1087897068.webp", packages = listOf("com.tencent.wework")),
        Target("多闪", "appstore-1435602536.webp"),
        Target("大众点评", "appstore-351091731.webp", packages = listOf("com.dianping.v1")),
        Target("BOSS直聘", "appstore-887314963.webp", packages = listOf("com.hpbr.bosszhipin")),
        Target("中国工商银行", "appstore-423514795.webp"),
        Target("优酷视频", "appstore-336141475.webp", packages = listOf("com.youku.phone")),
        Target("滴滴", "appstore-554499054.webp", packages = listOf("com.sdu.didi.psnger")),
        Target("腾讯视频", "appstore-458318329.webp", packages = listOf("com.tencent.qqlive")),
        Target("中国电信", "appstore-513836029.webp"),
        Target("国家医保服务平台", "appstore-1490383593.webp"),
        Target("中国联通", "appstore-416457422.webp"),
        Target("DJI Mimo", "appstore-1431720653.webp", labels = listOf("DJI Mimo", "大疆Mimo")),
        Target("中国建设银行", "appstore-391965015.webp"),
        Target("系统电池分析", "appstore-6752317272.webp"),
        Target("个人所得税", "appstore-1436002627.webp"),
        Target("微店", "appstore-860384719.webp"),
        Target("学信网", "appstore-1488854568.webp"),
        Target("DeepSeek", "appstore-6737597349.webp", packages = listOf("com.deepseek.chat")),
        Target("夸克网盘", "appstore-6755949705.webp"),
        Target("番茄小说", "appstore-1468454200.webp"),
        Target("米家", "appstore-957323480.webp", packages = listOf("com.xiaomi.smarthome")),
        Target("大麦", "appstore-513829338.webp"),
        Target("58同城", "appstore-480079300.webp", labels = listOf("58同城")),
        Target("美图秀秀", "appstore-416048305.webp"),
        Target("唯品会", "appstore-417200582.webp"),
        Target("中国银行", "appstore-399608199.webp"),
        Target("多邻国Duolingo英语日语法语", "appstore-570060128.webp", labels = listOf("多邻国", "Duolingo")),
        Target("山姆会员商店", "appstore-818237113.webp", labels = listOf("山姆会员商店", "山姆会员商店App")),
        Target("美颜相机", "appstore-592331499.webp"),
        Target("蚂蚁阿福", "appstore-6743828427.webp"),
        Target("爱奇艺", "appstore-393765873.webp", packages = listOf("com.qiyi.video")),
        Target("盒马", "appstore-1063183999.webp"),
        Target("Apple Developer", "appstore-640199958.webp"),
        Target("百度", "appstore-382201985.webp", packages = listOf("com.baidu.searchbox")),
        Target("X（前身为Twitter）", "appstore-333903271.webp", packages = listOf("com.twitter.android"), labels = listOf("X", "Twitter")),
        Target("倒数日 · Days Matter", "appstore-406170251.webp", labels = listOf("倒数日", "Days Matter")),
        Target("同程旅行", "appstore-475966832.webp"),
        Target("小象超市", "appstore-1444411068.webp"),
        Target("飓风相机", "appstore-6749835845.webp"),
        Target("天猫", "appstore-518966501.webp"),
        Target("即梦AI", "appstore-6503676563.webp"),
        Target("招商银行", "appstore-392899425.webp"),
        Target("麦当劳McDonald's", "appstore-1054598922.webp", labels = listOf("麦当劳", "McDonald's")),
        Target("小米运动健康", "appstore-1493500777.webp", labels = listOf("小米运动健康", "小米健康")),
        Target("邮储银行", "appstore-493489515.webp", labels = listOf("中国邮政储蓄银行")),
        Target("灵光", "appstore-6751496092.webp"),
        Target("我的桌面 · iScreen", "appstore-1534704608.webp", labels = listOf("iScreen")),
        Target("Apple 支持", "appstore-1130498044.webp"),
        Target("抖音商城", "appstore-1485191072.webp"),
        Target("快手极速版", "appstore-1472502819.webp"),
        Target("QQ邮箱", "appstore-473225145.webp", packages = listOf("com.tencent.androidqqmail")),
        Target("京东金融", "appstore-895682747.webp", labels = listOf("京东金融", "京东金融App")),
        Target("QQ浏览器", "appstore-370139302.webp", packages = listOf("com.tencent.mtt")),
        Target("QQ音乐", "appstore-414603431.webp", packages = listOf("com.tencent.qqmusic"), labels = listOf("QQ音乐")),
    ).associateBy { it.name }

    // These catalog entries are Apple-only and must not match an unrelated Android app.
    private val appleOnly = setOf("Apple Developer", "Apple 支持")

    private fun normalized(label: String): String =
        label.lowercase(java.util.Locale.ROOT).filter { it.isLetterOrDigit() }

    private fun launcherIntent(activity: Activity, target: Target): Intent? {
        if (target.name in appleOnly) return null
        val pm = activity.packageManager

        // Known package IDs are unambiguous, and continue to work even if
        // the launcher's label is customized or localized by the publisher.
        for (packageName in target.packages) {
            val launch = runCatching { pm.getLaunchIntentForPackage(packageName) }.getOrNull()
            if (launch != null) {
                return launch.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED)
            }
        }

        // Remaining entries are matched against installed launcher labels only.
        // We do not accept package names or URIs supplied by community works.
        val expected = (target.labels + target.name).map(::normalized).toSet()
        val query = Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER)
        @Suppress("DEPRECATION")
        val candidates = runCatching {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                pm.queryIntentActivities(query, android.content.pm.PackageManager.ResolveInfoFlags.of(0L))
            } else {
                pm.queryIntentActivities(query, 0)
            }
        }.getOrDefault(emptyList())

        val found = candidates.firstOrNull { candidate ->
            val appInfo = candidate.activityInfo?.applicationInfo ?: return@firstOrNull false
            val appLabel = runCatching { appInfo.loadLabel(pm).toString() }.getOrDefault("")
            val launcherLabel = runCatching { candidate.loadLabel(pm).toString() }.getOrDefault("")
            normalized(appLabel) in expected || normalized(launcherLabel) in expected
        }?.activityInfo ?: return null

        return Intent(Intent.ACTION_MAIN)
            .addCategory(Intent.CATEGORY_LAUNCHER)
            .setClassName(found.packageName, found.name)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED)
    }

    fun available(activity: Activity, name: String): Boolean {
        val target = targets[name] ?: return false
        return launcherIntent(activity, target) != null
    }

    fun open(activity: Activity, name: String): Boolean {
        val target = targets[name] ?: return false
        val fallback = launcherIntent(activity, target) ?: return false

        // A verified home deep link can open the actual home tab rather than
        // restoring the app's last screen. It is always package-constrained.
        // If an update removes the route, the system launcher still works.
        if (target.homepage != null && target.packages.isNotEmpty()) {
            val home = Intent(Intent.ACTION_VIEW, Uri.parse(target.homepage))
                .setPackage(target.packages.first())
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            val supported = runCatching { home.resolveActivity(activity.packageManager) != null }.getOrDefault(false)
            if (supported && runCatching { activity.startActivity(home) }.isSuccess) return true
        }
        return try {
            activity.startActivity(fallback)
            true
        } catch (_: Exception) {
            false
        }
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
