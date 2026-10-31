package xyz.xiangban.xiaoban.common.community

import kotlin.math.ln
import kotlin.math.max

/** 模型天梯的指标。分数类越高越好，速度越快越好，价格越低越好。 */
enum class Metric(val label: String, val unit: String, val higherIsBetter: Boolean, val tiered: Boolean) {
    Iq("智能指数", "分", true, true),
    Code("编程", "分", true, true),
    Math("数学", "分", true, true),
    Agent("智能体", "分", true, true),
    Speed("速度", "token/秒", true, false),
    Price("价格", "美元/百万 token", false, false);

    fun value(m: ModelEntry): Double = when (this) {
        Iq -> m.iq; Code -> m.code; Math -> m.math; Agent -> m.agent; Speed -> m.speed; Price -> m.price
    }
}

enum class ModelFilter(val label: String) {
    All("全部"), Reasoning("推理模型"), Open("开源权重"), Cn("国产");

    fun accepts(m: ModelEntry): Boolean = when (this) {
        All -> true; Reasoning -> m.reasoning; Open -> m.open; Cn -> m.cn
    }
}

/** 榜单里的一行。[fraction] 是柱子占满宽度的比例（0～1），按分数决定长短。 */
data class RankRow(val rank: Int, val model: ModelEntry, val value: Double, val fraction: Float)

data class TierGroup(val tier: TierDef, val rows: List<RankRow>)

/** 柱子最短也留一小截，免得看不见。 */
const val MinBarFraction = 0.04f

/**
 * 按指标排序后的榜单：分数高的在前，价格低的在前。
 * 柱长和设计稿一致：越高越好的指标按本榜最大值定比例（第一名占满）；价格反过来，最便宜的占满，其余按 最低价 / 本项价格。
 */
fun rankModels(models: List<ModelEntry>, metric: Metric, filter: ModelFilter = ModelFilter.All): List<RankRow> {
    val picked = models.filter(filter::accepts)
    val sorted = if (metric.higherIsBetter) picked.sortedByDescending { metric.value(it) } else picked.sortedBy { metric.value(it) }
    val top = max(sorted.maxOfOrNull { metric.value(it) } ?: 0.0, 1e-9)
    val cheapest = sorted.minOfOrNull { metric.value(it) } ?: 0.0
    return sorted.mapIndexed { i, m ->
        val v = metric.value(m)
        val raw = if (metric.higherIsBetter) v / top else if (v <= 0.0) 1.0 else cheapest / v
        RankRow(i + 1, m, v, raw.coerceIn(0.0, 1.0).toFloat().coerceAtLeast(MinBarFraction))
    }
}

/** 散点图标签用的短名：「模型 A」→「A」。 */
fun shortLabel(name: String): String = name.substringAfterLast(" ").take(6)

/** 把分数类榜单按服务端给的档位分组；不分档的指标返回空列表。空档不返回。 */
fun groupByTier(rows: List<RankRow>, tiers: List<TierDef>): List<TierGroup> {
    if (tiers.isEmpty()) return emptyList()
    val ordered = tiers.sortedByDescending { it.min ?: Double.NEGATIVE_INFINITY }
    return ordered.mapNotNull { tier ->
        val min = tier.min ?: Double.NEGATIVE_INFINITY
        val upper = ordered.getOrNull(ordered.indexOf(tier) - 1)?.min ?: Double.POSITIVE_INFINITY
        val inTier = rows.filter { it.value >= min && it.value < upper }
        if (inTier.isEmpty()) null else TierGroup(tier, inTier)
    }
}

/** 对数坐标上 [v] 的位置：[lo] 在 0，[hi] 在 1，范围外夹在两端。散点图的横轴用它。 */
fun logFraction(v: Double, lo: Double, hi: Double): Float {
    if (v <= 0.0 || lo <= 0.0 || hi <= lo) return 0f
    return ((ln(v) - ln(lo)) / (ln(hi) - ln(lo))).coerceIn(0.0, 1.0).toFloat()
}

/** 线性坐标上 [v] 的位置：[lo] 在 0，[hi] 在 1。 */
fun linearFraction(v: Double, lo: Double, hi: Double): Float =
    if (hi <= lo) 0f else ((v - lo) / (hi - lo)).coerceIn(0.0, 1.0).toFloat()

/** 价格显示：不到 1 美元保留两位，其余保留一位，如 `$0.15`、`$3.4`。 */
fun formatPrice(v: Double): String {
    val scaled = if (v < 1.0) (v * 100 + 0.5).toLong().let { "${it / 100}.${(it % 100).toString().padStart(2, '0')}" }
    else (v * 10 + 0.5).toLong().let { "${it / 10}.${it % 10}" }
    return "$$scaled"
}

/** 分数显示：整数不带小数，其余保留一位。 */
fun formatScore(v: Double): String = (v * 10 + 0.5).toLong().let { if (it % 10 == 0L) "${it / 10}" else "${it / 10}.${it % 10}" }

/** 开源热榜的星数：1000 以上写「12.4k」，10 万以上写「12.0万」。 */
fun formatStars(n: Long): String = when {
    n >= 100_000 -> "${n / 10_000}.${(n % 10_000) / 1000}万"
    n >= 1000 -> "${n / 1000}.${(n % 1000) / 100}k"
    else -> n.toString()
}

/** 带正负号和千分位的新增数，如 `+3,214`。 */
fun formatGain(n: Long): String = (if (n >= 0) "+" else "-") + abs(n).toString().reversed().chunked(3).joinToString(",").reversed()

private fun abs(n: Long) = if (n < 0) -n else n

data class AxisRange(val lo: Double, val hi: Double)

/** 散点图纵轴（智能指数）：默认 45～75，数据超出时按 5 分向外扩。 */
fun scoreAxis(models: List<ModelEntry>): AxisRange {
    val values = models.map { it.iq }.filter { it > 0 }
    if (values.isEmpty()) return AxisRange(45.0, 75.0)
    val lo = kotlin.math.floor(values.min() / 5) * 5
    val hi = kotlin.math.ceil(values.max() / 5) * 5
    return AxisRange(minOf(45.0, lo), maxOf(75.0, hi))
}

/** 散点图横轴（价格，对数刻度）：默认 $0.1～$10，数据超出时按十倍向外扩。 */
fun priceAxis(models: List<ModelEntry>): AxisRange {
    val values = models.map { it.price }.filter { it > 0 }
    var lo = 0.1
    var hi = 10.0
    if (values.isNotEmpty()) {
        while (values.min() < lo && lo > 1e-4) lo /= 10
        while (values.max() > hi && hi < 1e4) hi *= 10
    }
    return AxisRange(lo, hi)
}

/** 对数横轴上的刻度：每个数量级取 1 和 3，如 0.1、0.3、1、3、10。 */
fun priceTicks(range: AxisRange): List<Double> {
    val ticks = mutableListOf<Double>()
    var d = range.lo
    while (d <= range.hi * 1.0001) {
        ticks += d
        if (d * 3 <= range.hi * 1.0001) ticks += d * 3
        d *= 10
    }
    return ticks
}

/** 刻度文字：`$0.1`、`$0.03`、`$3`、`$10`。 */
fun formatTick(v: Double): String {
    if (v >= 1.0) return "$" + v.toLong().toString()
    val text = (v * 10000 + 0.5).toLong().let { n -> "${n / 10000}.${(n % 10000).toString().padStart(4, '0')}" }.trimEnd('0').trimEnd('.')
    return "$$text"
}
