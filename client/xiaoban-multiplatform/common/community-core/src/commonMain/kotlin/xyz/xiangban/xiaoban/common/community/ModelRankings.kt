package xyz.xiangban.xiaoban.common.community

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** v2 数据保持来源的空值、指数版本和价格单位，不借用旧榜单的四项评分。 */
@Serializable
data class LiveModelBoard(
    val source: String,
    @SerialName("source_url") val sourceUrl: String,
    @SerialName("as_of") val asOf: String,
    @SerialName("index_version") val indexVersion: String,
    @SerialName("schema_version") val schemaVersion: String,
    val tier: String,
    @SerialName("price_unit") val priceUnit: String,
    val vendors: List<LiveModelVendor>,
    val models: List<LiveModel>,
    @SerialName("updated_at") val updatedAt: String? = null,
)

@Serializable
data class LiveModelVendor(val id: String, val name: String, val country: String? = null)

@Serializable
data class LiveModel(
    val id: String,
    val slug: String,
    val name: String,
    @SerialName("vendor_id") val vendorId: String,
    @SerialName("release_date") val releaseDate: String? = null,
    val metrics: LiveModelMetrics,
    val pricing: LiveModelPricing,
    val performance: LiveModelPerformance,
)

@Serializable
data class LiveModelMetrics(
    val intelligence: Double? = null,
    @SerialName("finance_and_accounting") val financeAndAccounting: Double? = null,
    @SerialName("strategy_and_ops") val strategyAndOps: Double? = null,
    val legal: Double? = null,
    @SerialName("healthcare_and_medical") val healthcareAndMedical: Double? = null,
    val engineering: Double? = null,
    val economics: Double? = null,
    val coding: Double? = null,
    val agentic: Double? = null,
)

@Serializable
data class LiveModelPricing(val input: Double? = null, val output: Double? = null)

@Serializable
data class LiveModelPerformance(
    @SerialName("output_tokens_per_second") val outputTokensPerSecond: Double? = null,
    @SerialName("time_to_first_token_seconds") val timeToFirstTokenSeconds: Double? = null,
)

enum class LiveModelMetric(val label: String, val unit: String, val higherIsBetter: Boolean = true) {
    Intelligence("智能指数", "AA 智能指数"),
    Finance("财务会计", "能力指数"), Strategy("战略运营", "能力指数"), Legal("法律", "能力指数"),
    Healthcare("医疗健康", "能力指数"), Engineering("工程", "能力指数"), Economics("经济", "能力指数"),
    InputPrice("输入价格", "美元 / 百万 Token", false), OutputPrice("输出价格", "美元 / 百万 Token", false),
    Speed("输出速度", "Token / 秒");

    fun value(model: LiveModel): Double? = when (this) {
        Intelligence -> model.metrics.intelligence
        Finance -> model.metrics.financeAndAccounting
        Strategy -> model.metrics.strategyAndOps
        Legal -> model.metrics.legal
        Healthcare -> model.metrics.healthcareAndMedical
        Engineering -> model.metrics.engineering
        Economics -> model.metrics.economics
        InputPrice -> model.pricing.input
        OutputPrice -> model.pricing.output
        Speed -> model.performance.outputTokensPerSecond
    }?.takeIf { it.isFinite() && it >= 0.0 }
}

/** 无测量的条目保留在末尾，名次为空，避免把缺失数值算成 0。 */
data class LiveModelRank(val model: LiveModel, val value: Double?, val rank: Int?, val fraction: Float?)

fun rankLiveModels(models: List<LiveModel>, metric: LiveModelMetric): List<LiveModelRank> {
    val sorted = models.sortedWith { a, b ->
        val av = metric.value(a); val bv = metric.value(b)
        when {
            av == null && bv == null -> a.id.compareTo(b.id)
            av == null -> 1
            bv == null -> -1
            av == bv -> a.id.compareTo(b.id)
            metric.higherIsBetter -> bv.compareTo(av)
            else -> av.compareTo(bv)
        }
    }
    val max = sorted.mapNotNull(metric::value).maxOrNull() ?: 0.0
    val min = sorted.mapNotNull(metric::value).minOrNull() ?: 0.0
    var rank = 0
    return sorted.map { model ->
        val value = metric.value(model)
        val fraction = value?.let {
            when {
                metric.higherIsBetter -> if (max == 0.0) 0.0 else it / max
                it == 0.0 -> 1.0
                min == 0.0 -> 0.04
                else -> min / it
            }.coerceIn(0.0, 1.0).toFloat()
        }
        LiveModelRank(model, value, if (value == null) null else ++rank, fraction)
    }
}

/** 只显示该快照确实提供的指标；全空时仍显示智能指数和“暂无数据”。 */
fun availableLiveMetrics(board: LiveModelBoard): List<LiveModelMetric> =
    LiveModelMetric.entries.filter { metric -> board.models.any { metric.value(it) != null } }
        .ifEmpty { listOf(LiveModelMetric.Intelligence) }

data class LiveScatterPoint(val id: String, val name: String, val vendorId: String, val intelligence: Double, val weightedPrice: Double)

/** 图表只使用真实、完整且能用于对数轴的测量；价格明确采用输入:输出 = 3:1。 */
fun liveScatterPoints(board: LiveModelBoard): List<LiveScatterPoint> = board.models.mapNotNull { model ->
    val score = LiveModelMetric.Intelligence.value(model) ?: return@mapNotNull null
    val input = LiveModelMetric.InputPrice.value(model) ?: return@mapNotNull null
    val output = LiveModelMetric.OutputPrice.value(model) ?: return@mapNotNull null
    val price = input * 0.75 + output * 0.25
    if (!price.isFinite() || price <= 0.0) return@mapNotNull null
    LiveScatterPoint(model.id, model.name, model.vendorId, score, price)
}

@Serializable
data class ModelUsageBoard(
    val source: String,
    val date: String,
    @SerialName("as_of") val asOf: String,
    @SerialName("source_url") val sourceUrl: String,
    val version: String,
    val period: String,
    val timezone: String,
    @SerialName("token_unit") val tokenUnit: String,
    @SerialName("total_tokens") val totalTokens: String,
    @SerialName("other_tokens") val otherTokens: String,
    val models: List<ModelUsageEntry>,
    @SerialName("previous_date_available") val previousDateAvailable: Boolean = false,
    @SerialName("updated_at") val updatedAt: String? = null,
)

@Serializable
data class ModelUsageEntry(
    val id: String, val name: String, val tokens: String, val rank: Int,
    @SerialName("share_pct") val sharePct: Double,
    @SerialName("previous_rank") val previousRank: Int? = null,
    @SerialName("rank_change") val rankChange: Int? = null,
    @SerialName("new_entry") val newEntry: Boolean = false,
)

@Serializable
data class ModelUsageDates(val items: List<ModelUsageDate> = emptyList())
@Serializable
data class ModelUsageDate(val date: String, @SerialName("updated_at") val updatedAt: String? = null)

/** 不转 Long/Double，不缩写，超过 JavaScript 安全整数或 64 位的 Token 数仍完整显示。 */
fun formatTokenCount(value: String): String = if (isTokenCount(value))
    value.reversed().chunked(3).joinToString(",").reversed() else "暂无数据"

fun isTokenCount(value: String): Boolean = value.isNotEmpty() && value.length <= 80 &&
    value.all { it in '0'..'9' } && (value.length == 1 || value.first() != '0')

fun compareTokenCounts(a: String, b: String): Int {
    require(isTokenCount(a) && isTokenCount(b))
    return a.length.compareTo(b.length).takeIf { it != 0 } ?: a.compareTo(b)
}

/** 前一整日未发布时，不把未知变化标为新入榜。 */
fun modelUsageMovement(current: ModelUsageEntry, previousDateAvailable: Boolean): String {
    if (!previousDateAvailable) return "暂无对比"
    if (current.newEntry) return "新入榜"
    val delta = current.rankChange ?: return "暂无对比"
    return when { delta > 0 -> "↑ $delta"; delta < 0 -> "↓ ${-delta}"; else -> "—" }
}

/** 保留来源报价的小数精度；例如 0.001 美元不能四舍五入显示成免费。 */
fun formatLivePrice(value: Double?): String = value?.takeIf { it.isFinite() && it >= 0.0 }
    ?.let { "$" + it.toString().removeSuffix(".0") } ?: "暂无数据"
