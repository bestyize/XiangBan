package xyz.xiaban.wallpaper.data

import okhttp3.Interceptor
import okhttp3.Response

/** How long to wait before trying again after a "too many requests" answer. */
object Backoff {
    const val MAX_ATTEMPTS = 4
    private const val BASE_MS = 1_200L
    private const val CAP_MS = 8_000L

    /** [retryAfter] is the server's own hint in seconds when it gave one; otherwise wait longer each time. */
    fun delayMillis(attempt: Int, retryAfter: String?): Long {
        val hinted = retryAfter?.trim()?.toLongOrNull()?.takeIf { it in 0..60 }?.times(1_000)
        return (hinted ?: (BASE_MS shl attempt.coerceIn(0, 6))).coerceAtMost(CAP_MS)
    }
}

/**
 * Some picture hosts answer 429 when many covers are requested at once. Wait a little and ask again,
 * so a busy host slows the grid down instead of leaving holes in it.
 */
class RetryWhenBusy : Interceptor {
    override fun intercept(chain: Interceptor.Chain): Response {
        var attempt = 0
        while (true) {
            val response = chain.proceed(chain.request())
            if (response.code != 429 || attempt >= Backoff.MAX_ATTEMPTS - 1) return response
            val wait = Backoff.delayMillis(attempt, response.header("Retry-After"))
            response.close()
            try {
                Thread.sleep(wait)
            } catch (e: InterruptedException) {
                Thread.currentThread().interrupt()
                throw java.io.IOException("cancelled while waiting to retry")
            }
            attempt++
        }
    }
}
