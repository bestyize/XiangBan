package xyz.xiaban.wallpaper

import android.app.Application
import coil3.ImageLoader
import coil3.PlatformContext
import coil3.SingletonImageLoader
import coil3.disk.DiskCache
import coil3.memory.MemoryCache
import coil3.network.okhttp.OkHttpNetworkFetcherFactory
import okhttp3.Dispatcher
import okhttp3.OkHttpClient
import okio.Path.Companion.toOkioPath
import xyz.xiaban.wallpaper.data.RetryWhenBusy
import java.util.concurrent.TimeUnit

class XiaobanApp : Application(), SingletonImageLoader.Factory {
    /**
     * Covers come from third-party hosts that can be slow or rate-limited, so each host gets only a couple of
     * requests at a time, a busy answer is retried, timeouts are longer than OkHttp's defaults, and everything
     * fetched once is kept on disk.
     */
    override fun newImageLoader(context: PlatformContext): ImageLoader {
        val dispatcher = Dispatcher().apply {
            maxRequests = MAX_REQUESTS
            maxRequestsPerHost = MAX_PER_HOST
        }
        val client = OkHttpClient.Builder()
            .dispatcher(dispatcher)
            .addInterceptor(RetryWhenBusy())
            .connectTimeout(20, TimeUnit.SECONDS)
            .readTimeout(40, TimeUnit.SECONDS)
            .build()
        return ImageLoader.Builder(context)
            .components { add(OkHttpNetworkFetcherFactory(callFactory = { client })) }
            .memoryCache { MemoryCache.Builder().maxSizePercent(context, 0.25).build() }
            .diskCache { DiskCache.Builder().directory(cacheDir.resolve("images").toOkioPath()).maxSizeBytes(MAX_DISK_BYTES).build() }
            .build()
    }

    private companion object {
        const val MAX_DISK_BYTES = 256L * 1024 * 1024
        const val MAX_REQUESTS = 12
        const val MAX_PER_HOST = 2
    }
}
