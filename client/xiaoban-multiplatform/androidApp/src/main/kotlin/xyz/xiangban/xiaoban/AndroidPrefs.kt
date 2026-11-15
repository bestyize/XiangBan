package xyz.xiangban.xiaoban

import android.content.Context
import xyz.xiangban.xiaoban.business.communitynative.AppPrefs

/** 界面偏好（外观、主题色）存在应用私有的 SharedPreferences 里，不含敏感信息。 */
class AndroidPrefs(context: Context) : AppPrefs {
    private val prefs = context.applicationContext.getSharedPreferences("xiaoban_ui_prefs", Context.MODE_PRIVATE)

    init {
        // 旧版首页追踪的提醒开关已下线。
        if (prefs.contains("quota_remind")) prefs.edit().remove("quota_remind").apply()
    }

    override fun get(key: String): String? = prefs.getString(key, null)
    override fun put(key: String, value: String) { prefs.edit().putString(key, value).apply() }
}
