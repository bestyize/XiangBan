package xyz.xiangban.xiaoban

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import xyz.xiangban.xiaoban.common.community.MobileGame
import xyz.xiangban.xiaoban.common.community.MobileGameHistory

internal class AndroidGameLibrary(private val context: Context) {
    suspend fun games(): List<MobileGame> = withContext(Dispatchers.IO) {
        BuiltinMobileGames.ids.map { id ->
            BuiltinMobileGames.read(id, context.assets.open("game-packs/$id/manifest.json").bufferedReader().use { it.readText() })
        }
    }
    suspend fun cover(id: String): ByteArray? = withContext(Dispatchers.IO) {
        if (id !in BuiltinMobileGames.ids) return@withContext null
        context.assets.open("game-packs/$id/cover.svg").use { it.readBytes() }
    }
}

/** Same process as the runner; the StateFlow refreshes the library when returning from play. */
internal object AndroidGameHistory {
    private var initialized = false
    private val value = MutableStateFlow<List<String>>(emptyList())
    @Synchronized fun flow(context: Context): StateFlow<List<String>> {
        if (!initialized) {
            value.value = MobileGameHistory.decode(context.getSharedPreferences("game_library", Context.MODE_PRIVATE).getString("recent", null))
            initialized = true
        }
        return value
    }
    @Synchronized fun opened(context: Context, id: String) {
        flow(context)
        val ids = MobileGameHistory.opened(value.value, id)
        context.getSharedPreferences("game_library", Context.MODE_PRIVATE).edit().putString("recent", MobileGameHistory.encode(ids)).apply()
        value.value = ids
    }
}
