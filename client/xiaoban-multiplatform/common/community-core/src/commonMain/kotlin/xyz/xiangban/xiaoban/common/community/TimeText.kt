package xyz.xiangban.xiaoban.common.community

import kotlin.time.Clock

private const val Minute = 60_000L
private const val Hour = 60 * Minute
private const val Day = 24 * Hour

/** 国内用户为主，日期按东八区显示。 */
private const val ChinaOffset = 8 * Hour

fun nowMillis(): Long = Clock.System.now().toEpochMilliseconds()

private fun daysFromCivil(y0: Int, m: Int, d: Int): Long {
    val y = if (m <= 2) y0 - 1 else y0
    val era = (if (y >= 0) y else y - 399) / 400
    val yoe = y - era * 400
    val doy = (153 * (m + (if (m > 2) -3 else 9)) + 2) / 5 + d - 1
    val doe = yoe * 365 + yoe / 4 - yoe / 100 + doy
    return era * 146097L + doe - 719468
}

private fun civilFromDays(z0: Long): Triple<Int, Int, Int> {
    val z = z0 + 719468
    val era = (if (z >= 0) z else z - 146096) / 146097
    val doe = z - era * 146097
    val yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365
    val doy = doe - (365 * yoe + yoe / 4 - yoe / 100)
    val mp = (5 * doy + 2) / 153
    val d = (doy - (153 * mp + 2) / 5 + 1).toInt()
    val m = (if (mp < 10) mp + 3 else mp - 9).toInt()
    val y = (yoe + era * 400 + (if (m <= 2) 1 else 0)).toInt()
    return Triple(y, m, d)
}

/** 解析服务端的 ISO 8601 时间（`2026-10-31T10:00:00.123456Z`，或带 `+08:00` 偏移），返回毫秒时间戳；解析不了返回 null。 */
fun parseIsoMillis(text: String): Long? {
    val m = Regex("""^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})(?::(\d{2}))?(?:\.(\d+))?(Z|[+-]\d{2}:?\d{2})?$""").matchEntire(text.trim()) ?: return null
    val g = m.groupValues
    val millis = g[7].take(3).padEnd(3, '0').ifEmpty { "0" }.toLong()
    val local = daysFromCivil(g[1].toInt(), g[2].toInt(), g[3].toInt()) * Day +
        g[4].toLong() * Hour + g[5].toLong() * Minute + (g[6].ifEmpty { "0" }.toLong()) * 1000 + millis
    val zone = g[8]
    val offset = if (zone.isEmpty() || zone == "Z") 0L else {
        val sign = if (zone[0] == '-') -1 else 1
        val digits = zone.drop(1).replace(":", "")
        sign * (digits.take(2).toLong() * Hour + digits.drop(2).toLong() * Minute)
    }
    return local - offset
}

/** 「刚刚」「5 分钟前」「3 小时前」「昨天」「10 月 20 日」。解析不了就原样返回前 10 个字。 */
fun agoText(iso: String, now: Long = nowMillis()): String {
    val t = parseIsoMillis(iso) ?: return iso.take(10)
    val diff = now - t
    if (diff < 0) return "刚刚"
    return when {
        diff < Minute -> "刚刚"
        diff < Hour -> "${diff / Minute} 分钟前"
        diff < Day -> "${diff / Hour} 小时前"
        else -> {
            val today = Math2.floorDiv(now + ChinaOffset, Day)
            val that = Math2.floorDiv(t + ChinaOffset, Day)
            if (today - that == 1L) "昨天" else {
                val (y, m, d) = civilFromDays(that)
                val (ny, _, _) = civilFromDays(today)
                if (y == ny) "$m 月 $d 日" else "$y 年 $m 月 $d 日"
            }
        }
    }
}

/** 「3 小时 12 分」，不到一小时只写分钟，已经过了写「0 分」。 */
fun countdownText(targetIso: String, now: Long = nowMillis()): String {
    val t = parseIsoMillis(targetIso) ?: return ""
    val left = (t - now).coerceAtLeast(0)
    val h = left / Hour
    val min = left % Hour / Minute
    return if (h > 0) "$h 小时 $min 分" else "$min 分"
}

/** 目标时刻在东八区的「今天 18:40」「明天 09:00」「10 月 20 日 09:00」。 */
fun clockText(targetIso: String, now: Long = nowMillis()): String {
    val t = parseIsoMillis(targetIso) ?: return ""
    val local = t + ChinaOffset
    val hh = (local % Day / Hour).toString().padStart(2, '0')
    val mm = (local % Hour / Minute).toString().padStart(2, '0')
    val diffDays = Math2.floorDiv(local, Day) - Math2.floorDiv(now + ChinaOffset, Day)
    return when (diffDays) {
        0L -> "今天 $hh:$mm"
        1L -> "明天 $hh:$mm"
        else -> civilFromDays(Math2.floorDiv(local, Day)).let { (_, m, d) -> "$m 月 $d 日 $hh:$mm" }
    }
}

/** 求助帖的剩余时间：「剩 5 天」，不到一天写「剩 3 小时」，已结束写「已结束」。 */
fun remainText(expiresIso: String, now: Long = nowMillis()): String {
    val t = parseIsoMillis(expiresIso) ?: return ""
    val left = t - now
    return when {
        left <= 0 -> "已结束"
        left >= Day -> "剩 ${(left + Day - 1) / Day} 天"
        else -> "剩 ${maxOf(1, left / Hour)} 小时"
    }
}

/** 播放量、浏览量的紧凑写法：`4.1千`、`1.2万`。 */
fun compactCount(n: Int): String = when {
    n >= 10_000 -> "${n / 10_000}.${n % 10_000 / 1000}万"
    n >= 1000 -> "${n / 1000}.${n % 1000 / 100}千"
    else -> n.toString()
}

private object Math2 {
    fun floorDiv(a: Long, b: Long): Long = if (a >= 0) a / b else -((-a + b - 1) / b)
}
