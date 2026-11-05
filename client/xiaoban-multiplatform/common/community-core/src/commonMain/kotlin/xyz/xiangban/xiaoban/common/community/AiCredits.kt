package xyz.xiangban.xiaoban.common.community

import kotlin.math.abs

/** 仅用于展示：10 美元额度 = 1,000 小伴点；接口和账本仍使用底层整数额度。 */
const val AI_CREDITS_PER_DOLLAR = 100

fun formatAiCredits(quota: Long, quotaPerDollar: Long = 500_000): String {
    val unit = if (quotaPerDollar > 0) quotaPerDollar else 500_000L
    val credits = quota.toDouble() / (unit.toDouble() / AI_CREDITS_PER_DOLLAR)
    val sign = if (credits < 0) "−" else ""
    val value = abs(credits)
    // 展示向下取整，不高估余额；不足一点也不显示成零，账本精度保持不变。
    if (value > 0 && value < 1) return "${sign}<1 点"
    val whole = value.toLong().toString()
    val grouped = buildString {
        whole.forEachIndexed { index, digit ->
            if (index > 0 && (whole.length - index) % 3 == 0) append(',')
            append(digit)
        }
    }
    return "$sign$grouped 点"
}
