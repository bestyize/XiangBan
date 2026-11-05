package xyz.xiangban.xiaoban.common.community

import kotlin.math.abs
import kotlin.math.roundToLong

/** 仅用于展示：10 美元额度 = 1,000 小伴点；接口和账本仍使用底层整数额度。 */
const val AI_CREDITS_PER_DOLLAR = 100

fun formatAiCredits(quota: Long, quotaPerDollar: Long = 500_000): String {
    val unit = if (quotaPerDollar > 0) quotaPerDollar else 500_000L
    val credits = quota.toDouble() / unit.toDouble() * AI_CREDITS_PER_DOLLAR
    val sign = if (credits < 0) "−" else ""
    val value = abs(credits)
    // 默认比例下，一个底层单位为 0.0002 点；小额消耗不能显示成 0 点。
    if (value > 0 && value < 0.0001) return "${sign}<0.0001 点"
    val scaled = (value * 10_000).roundToLong()
    val whole = (scaled / 10_000).toString()
    val grouped = buildString {
        whole.forEachIndexed { index, digit ->
            if (index > 0 && (whole.length - index) % 3 == 0) append(',')
            append(digit)
        }
    }
    val fraction = (scaled % 10_000).toString().padStart(4, '0').trimEnd('0')
    return "$sign$grouped${if (fraction.isEmpty()) "" else ".$fraction"} 点"
}
