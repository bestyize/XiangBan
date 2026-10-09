package xyz.xiaban.wallpaper.data

import android.content.Context
import androidx.media3.database.StandaloneDatabaseProvider
import androidx.media3.datasource.DefaultDataSource
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.datasource.cache.CacheDataSource
import androidx.media3.datasource.cache.LeastRecentlyUsedCacheEvictor
import androidx.media3.datasource.cache.SimpleCache
import java.io.File

/**
 * Online videos are large, so what has been streamed is kept on disk: looping and replaying read from the phone
 * instead of the network. It lives in the app's cache folder, so the system may clear it when space runs low.
 */
object VideoCache {
    private const val MAX_BYTES = 512L * 1024 * 1024
    private const val CONNECT_MS = 15_000
    private const val READ_MS = 30_000

    @Volatile
    private var cache: SimpleCache? = null

    private fun get(context: Context): SimpleCache = cache ?: synchronized(this) {
        cache ?: SimpleCache(
            File(context.applicationContext.cacheDir, "video"),
            LeastRecentlyUsedCacheEvictor(MAX_BYTES),
            StandaloneDatabaseProvider(context.applicationContext),
        ).also { cache = it }
    }

    /** Web addresses go through the disk cache; pictures and videos from the gallery are read directly. */
    fun dataSourceFactory(context: Context, remote: Boolean): androidx.media3.datasource.DataSource.Factory {
        val http = DefaultHttpDataSource.Factory().setConnectTimeoutMs(CONNECT_MS).setReadTimeoutMs(READ_MS)
        val upstream = DefaultDataSource.Factory(context.applicationContext, http)
        return if (!remote) {
            upstream
        } else {
            CacheDataSource.Factory().setCache(get(context)).setUpstreamDataSourceFactory(upstream)
                .setFlags(CacheDataSource.FLAG_IGNORE_CACHE_ON_ERROR)
        }
    }

    fun sizeBytes(context: Context): Long = runCatching { get(context).cacheSpace }.getOrDefault(0L)

    fun clear(context: Context) {
        runCatching {
            val c = get(context)
            for (key in c.keys.toList()) c.removeResource(key)
        }
    }
}
