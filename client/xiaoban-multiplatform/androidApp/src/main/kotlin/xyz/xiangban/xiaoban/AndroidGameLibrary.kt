package xyz.xiangban.xiaoban

import android.content.Context
import android.content.res.AssetManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.withContext
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import xyz.xiangban.xiaoban.common.community.MobileGame
import xyz.xiangban.xiaoban.common.community.MobileGameHistory

internal class AndroidGameLibrary(private val context: Context) {
    suspend fun games(): List<MobileGame> = withContext(Dispatchers.IO) {
        val ids = if (BuildConfig.DEBUG) BuiltinMobileGames.ids + RUNTIME_V2_FIXTURE else BuiltinMobileGames.ids
        ids.map { id ->
            context.assets.loadBundledRuntimeV2(id, checkNotNull(bundledGameRoot(id))).manifest.let {
                MobileGame(it.id, it.name, it.description, it.orientation, it.kind, it.category, it.input, it.capabilities.toList(), it.version)
            }
        }
    }
    suspend fun cover(id: String): ByteArray? = withContext(Dispatchers.IO) {
        val root = bundledGameRoot(id) ?: return@withContext null
        val manifest = context.assets.loadBundledRuntimeV2(id, root).manifest
        context.assets.open("$root/${manifest.cover}").use { it.readBytes() }
    }
}

internal const val RUNTIME_V2_FIXTURE = "runtime-v2-fixture"

internal fun bundledGameRoot(id: String): String? = when {
    BuildConfig.DEBUG && id == RUNTIME_V2_FIXTURE -> RUNTIME_V2_FIXTURE
    id in BuiltinMobileGames.ids -> "game-packs-v2/$id"
    else -> null
}

internal fun AssetManager.loadBundledRuntimeV2(id: String, root: String): RuntimeV2Package {
    val host = if (id == RUNTIME_V2_FIXTURE) "local-$id" else "official-$id"
    return RuntimeV2Package.load(this, root, host).also { require(it.manifest.id == id) { "内置作品清单不匹配" } }
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
    @Synchronized fun forget(context: Context, id: String) {
        flow(context)
        val ids = value.value.filterNot { it == id }
        context.getSharedPreferences("game_library", Context.MODE_PRIVATE).edit().putString("recent", MobileGameHistory.encode(ids)).apply()
        value.value = ids
    }
    @Synchronized fun opened(context: Context, id: String) {
        flow(context)
        val ids = MobileGameHistory.opened(value.value, id)
        context.getSharedPreferences("game_library", Context.MODE_PRIVATE).edit().putString("recent", MobileGameHistory.encode(ids)).apply()
        value.value = ids
    }
}
