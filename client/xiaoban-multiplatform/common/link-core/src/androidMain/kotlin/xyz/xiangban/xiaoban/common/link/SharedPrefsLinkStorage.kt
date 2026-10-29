package xyz.xiangban.xiaoban.common.link

import android.content.Context
import kotlinx.serialization.json.Json

private const val PREFS_NAME = "xiaoban_link_storage_v2"
private const val KEY_STATE = "state"

private val StorageJson = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
    encodeDefaults = true
}

/** Android 上的 [LinkStorage]：只有设备编号这类非敏感信息，存在应用私有的 SharedPreferences 里。 */
class SharedPrefsLinkStorage(context: Context) : LinkStorage {
    private val prefs = context.applicationContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)

    override fun load(): LinkState {
        val text = prefs.getString(KEY_STATE, null) ?: return LinkState()
        return try {
            StorageJson.decodeFromString(LinkState.serializer(), text)
        } catch (e: Exception) {
            LinkState()
        }
    }

    override fun save(state: LinkState) {
        prefs.edit().putString(KEY_STATE, StorageJson.encodeToString(LinkState.serializer(), state)).apply()
    }
}
