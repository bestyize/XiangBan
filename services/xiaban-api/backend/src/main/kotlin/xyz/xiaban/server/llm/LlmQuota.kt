package xyz.xiaban.server.llm

import kotlinx.coroutines.future.await
import xyz.xiaban.server.redis.Redis
import java.time.LocalDate

/** Caps model calls per user per 下班日 to keep cost predictable (PRD 11). */
class LlmQuota(private val redis: Redis, private val dailyLimit: Int) {
    suspend fun tryAcquire(userId: Long, workDay: LocalDate): Boolean {
        val key = "xb:llm:quota:$workDay:$userId"
        val used = redis.cmd.incr(key).await()
        if (used == 1L) redis.cmd.expire(key, 2 * 24 * 3600L).await()
        return used <= dailyLimit
    }
}
