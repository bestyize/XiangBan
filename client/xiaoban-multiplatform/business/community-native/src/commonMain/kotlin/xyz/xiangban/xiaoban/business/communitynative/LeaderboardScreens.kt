package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.Metric
import xyz.xiangban.xiaoban.common.community.ModelBoard
import xyz.xiangban.xiaoban.common.community.ModelEntry
import xyz.xiangban.xiaoban.common.community.ModelFilter
import xyz.xiangban.xiaoban.common.community.ModelVendor
import xyz.xiangban.xiaoban.common.community.RankRow
import xyz.xiangban.xiaoban.common.community.formatPrice
import xyz.xiangban.xiaoban.common.community.formatScore
import xyz.xiangban.xiaoban.common.community.formatTick
import xyz.xiangban.xiaoban.common.community.linearFraction
import xyz.xiangban.xiaoban.common.community.logFraction
import xyz.xiangban.xiaoban.common.community.priceAxis
import xyz.xiangban.xiaoban.common.community.priceTicks
import xyz.xiangban.xiaoban.common.community.rankModels
import xyz.xiangban.xiaoban.common.community.scoreAxis
import xyz.xiangban.xiaoban.common.community.shortLabel

// 模型榜：榜单从上到下排，分数高的在上面；每行一条从左往右长的横向柱子，长度按分数比例。

private val Gap = 4.dp
private val ZoneGreen = Color(0xFFEEF6F1)

internal fun Metric.valueText(m: ModelEntry): String = when (this) {
    Metric.Price -> formatPrice(value(m))
    Metric.Speed -> "${value(m).toInt()}"
    else -> formatScore(value(m))
}

private val Metric.title: String get() = when (this) {
    Metric.Iq -> "AA 智能指数"; Metric.Code -> "编程指数"; Metric.Math -> "数学指数"; Metric.Agent -> "智能体指数"; Metric.Speed -> "输出速度"; Metric.Price -> "综合价格"
}

private val Metric.unitText: String get() = when (this) {
    Metric.Speed -> "token/秒，越高越好"
    Metric.Price -> "美元/百万 token，越低越好"
    else -> "分，越高越好"
}

private val Metric.help: String get() = when (this) {
    Metric.Iq -> "综合推理、知识、数学、编程等多项评测的加权得分。"
    Metric.Code -> "代码生成与修复类评测的综合得分。"
    Metric.Math -> "竞赛数学类评测的综合得分。"
    Metric.Agent -> "多步工具调用、长任务执行类评测的综合得分。"
    Metric.Speed -> "首个 token 之后每秒输出的 token 数，取各家 API 中位数。"
    Metric.Price -> "输入和输出按 3:1 混合计算的价格。"
}

internal fun vendorColor(vendor: ModelVendor?): Color = hexColor(vendor?.brandColor.orEmpty())

/** 厂商的官方标志：优先用设计稿里的官方图标，认不出的用品牌色加首字。 */
@Composable
internal fun VendorTile(vendor: ModelVendor?, size: Dp = 36.dp, corner: Dp = 8.dp) {
    val logo = vendor?.let { BrandLogos.find(it.name) }
    if (logo != null) {
        // 标志用厂商在榜单里的品牌色，底色用服务端给的浅底。
        val brand = vendorColor(vendor)
        Box(Modifier.size(size).clip(RoundedCornerShape(corner)).background(tintColor(brand, vendor.tint)), contentAlignment = Alignment.Center) {
            XIcon(logo.icon, tint = brand, size = size * 0.55f)
        }
    } else {
        val brand = vendorColor(vendor)
        LetterIcon(vendor?.name ?: "?", brand, tintColor(brand, vendor?.tint.orEmpty()), size = size)
    }
}

@Composable
private fun StatBlock(label: String, value: String, modifier: Modifier = Modifier, tight: Boolean = false) {
    Column(modifier.clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(vertical = if (tight) 6.dp else 8.dp, horizontal = 4.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Text(value, fontSize = 14.sp, fontWeight = FontWeight.Bold, maxLines = 1)
        Text(label, Modifier.padding(top = 2.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun SubBars(m: ModelEntry, barColor: Color, labelWidth: Dp, gap: Dp, barHeight: Dp, fontSize: Int) {
    Column(verticalArrangement = Arrangement.spacedBy(if (barHeight > 5.dp) 6.dp else 5.dp)) {
        listOf("编程" to m.code, "数学" to m.math, "智能体" to m.agent).forEach { (label, v) ->
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(gap)) {
                Text(label, Modifier.width(labelWidth), fontSize = fontSize.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, softWrap = false)
                HBar((v / 100.0).toFloat(), barColor, Modifier.weight(1f), height = barHeight, track = LocalExtraColors.current.fill)
                Text(formatScore(v), Modifier.width(if (barHeight > 5.dp) 24.dp else 20.dp), fontSize = fontSize.sp, fontWeight = FontWeight.SemiBold, textAlign = androidx.compose.ui.text.style.TextAlign.End)
            }
        }
    }
}

/**
 * 榜单里的一行（首页排行榜卡片和模型天梯共用）：厂商标志、模型名、一条从左往右的横向柱子（厂商品牌色，长度按分数）、数值。
 */
@Composable
internal fun ModelBarRow(row: RankRow, vendor: ModelVendor?, valueText: String, nameWidth: Dp = 52.dp, valueWidth: Dp = 28.dp) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Box(Modifier.width(18.dp)) { VendorTile(vendor, 18.dp, 4.dp) }
        Text(row.model.name, Modifier.width(nameWidth), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis)
        HBar(row.fraction, vendorColor(vendor), Modifier.weight(1f), height = 14.dp, track = LocalExtraColors.current.fill)
        Text(valueText, Modifier.width(valueWidth), fontSize = 12.sp, fontWeight = FontWeight.Bold, textAlign = androidx.compose.ui.text.style.TextAlign.End, maxLines = 1)
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun VendorLegend(vendors: List<ModelVendor>, modifier: Modifier = Modifier) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        vendors.forEach { v ->
            val logo = BrandLogos.find(v.name)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                if (logo != null) XIcon(logo.icon, tint = vendorColor(v), size = 14.dp) else Dot(vendorColor(v), 8.dp)
                Text(v.name, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

internal fun sourceNote(board: ModelBoard): String =
    "数据来源 ${board.source.ifEmpty { "Artificial Analysis" }}，每天同步一次，注明出处。" + if (board.sample) "图中模型与数值为示例数据。" else ""

/** 模型天梯：顶部是指标和筛选标签，下面一张白卡里按分数从高到低排成一个列表，样式和排行榜首页的卡片一样。 */
@Composable
fun LeaderboardScreen(env: CommunityEnv, back: () -> Unit) {
    Column(Modifier.fillMaxSize()) {
        ScreenBar("模型排行榜", onBack = back)
        LiveRankingsContent(env)
    }
}

// ───────────── 智能 vs 价格 散点图 ─────────────

/** 竖向是智能指数，横向是价格（对数刻度），左上角最划算。 */
enum class ScatterMode { Inline, Full }

/**
 * 内嵌版（首页排行榜）：圆点 10dp 带白边，旁边写模型字母；左上角 45%×45% 是「高智能 · 低价格」绿区，横竖各一条虚线。
 * 全屏版（横屏）：圆点里画厂商标志，选中的放大带光圈，没通过筛选的变淡但不消失；点圆点选中。
 */
@Composable
fun ScatterPlot(
    models: List<ModelEntry>,
    vendors: List<ModelVendor>,
    modifier: Modifier = Modifier,
    mode: ScatterMode = ScatterMode.Inline,
    filter: ModelFilter = ModelFilter.All,
    selected: String? = null,
    onSelect: ((String) -> Unit)? = null,
) {
    val full = mode == ScatterMode.Full
    val pAxis = remember(models) { priceAxis(models) }
    val sAxis = remember(models) { scoreAxis(models).let { if (full) it.copy(hi = it.hi + 2) else it } }
    val ticksX = remember(pAxis) { priceTicks(pAxis) }
    val ticksY = remember(sAxis) { generateSequence(45.0) { it + 5 }.takeWhile { it <= 75.0 }.toList() }
    val ink3 = LocalExtraColors.current.ink3
    val axisLine = LocalExtraColors.current.line
    val gridLine = LocalExtraColors.current.hairline
    val dash = remember { PathEffect.dashPathEffect(floatArrayOf(8f, 8f)) }
    val greenText = if (ThemeState.dark) Color(0xFF7BD39A) else Color(0xFF2E6B47)
    val zone = if (ThemeState.dark) Success.copy(alpha = 0.14f) else ZoneGreen

    fun fx(price: Double): Float = if (full) logFraction(price, pAxis.lo, pAxis.hi) else 0.03f + 0.92f * logFraction(price, pAxis.lo, pAxis.hi)
    fun fy(score: Double): Float = linearFraction(score, sAxis.lo, sAxis.hi)

    Column(modifier) {
        Row(Modifier.weight(1f).fillMaxWidth()) {
            Box(Modifier.width(if (full) 30.dp else 26.dp).fillMaxHeight()) {
                BoxWithConstraints(Modifier.fillMaxSize()) {
                    val labels = if (full) ticksY else listOf(sAxis.hi, (sAxis.lo + sAxis.hi) / 2, sAxis.lo)
                    labels.forEach { v ->
                        Text(formatScore(v), Modifier.align(Alignment.TopEnd).offset(y = maxHeight * (1f - fy(v)) - 8.dp).padding(end = if (full) 6.dp else 4.dp), fontSize = 12.sp, color = ink3)
                    }
                }
            }
            BoxWithConstraints(Modifier.weight(1f).fillMaxHeight()) {
                val w = maxWidth
                val h = maxHeight
                Canvas(Modifier.fillMaxSize()) {
                    val zoneW = if (full) size.width * fx(1.0) else size.width * 0.45f
                    val zoneH = if (full) size.height * (1f - fy(65.0)) else size.height * 0.45f
                    drawRoundRect(zone, topLeft = Offset.Zero, size = Size(zoneW, zoneH), cornerRadius = CornerRadius(8.dp.toPx()))
                    drawRect(zone, topLeft = Offset.Zero, size = Size(zoneW - 8.dp.toPx(), zoneH))
                    drawRect(zone, topLeft = Offset.Zero, size = Size(zoneW, zoneH - 8.dp.toPx()))
                    if (full) {
                        ticksY.forEach { v -> val y = size.height * (1f - fy(v)); drawLine(gridLine, Offset(0f, y), Offset(size.width, y), 1f, pathEffect = dash) }
                        ticksX.forEach { p -> val x = size.width * fx(p); drawLine(gridLine, Offset(x, 0f), Offset(x, size.height), 1f, pathEffect = dash) }
                    } else {
                        drawLine(axisLine, Offset(0f, size.height / 2), Offset(size.width, size.height / 2), 1f, pathEffect = dash)
                        drawLine(axisLine, Offset(size.width / 2, 0f), Offset(size.width / 2, size.height), 1f, pathEffect = dash)
                    }
                    drawLine(axisLine, Offset(0f, 0f), Offset(0f, size.height), 1f)
                    drawLine(axisLine, Offset(0f, size.height), Offset(size.width, size.height), 1f)
                }
                Text("高智能 · 低价格", Modifier.offset(if (full) 8.dp else 6.dp, if (full) 6.dp else 4.dp), fontSize = 12.sp, color = greenText)
                models.forEach { m ->
                    val on = filter.accepts(m)
                    val isSel = m.key == selected
                    val size = if (!full) 10.dp else if (isSel) 26.dp else 20.dp
                    val vendor = vendors.firstOrNull { it.key == m.vendor }
                    val color = vendorColor(vendor)
                    val x = w * fx(m.price)
                    val y = h * (1f - fy(m.iq))
                    Box(
                        Modifier.offset(x - size / 2, y - size / 2).size(size).alpha(if (on) 1f else 0.18f)
                            .let { if (isSel) it.border(3.dp, color.copy(alpha = 0.33f), CircleShape) else it }
                            .clip(CircleShape).background(color).border(if (full) 2.dp else 1.5.dp, MaterialTheme.colorScheme.surface, CircleShape)
                            .let { if (onSelect != null && on) it.clickable { onSelect(m.key) } else it },
                        contentAlignment = Alignment.Center,
                    ) {
                        val logo = if (full) vendor?.let { BrandLogos.find(it.name) } else null
                        if (logo != null) XIcon(logo.icon, tint = Color.White, size = if (isSel) 14.dp else 11.dp)
                    }
                    Text(
                        if (full) m.name else shortLabel(m.name),
                        Modifier.offset(x + (if (full) 13.dp else 7.dp), y - (if (full) 20.dp else 18.dp)).alpha(if (on) 1f else 0.18f),
                        fontSize = 12.sp, fontWeight = if (isSel) FontWeight.Bold else FontWeight.Normal, color = MaterialTheme.colorScheme.onSurface, maxLines = 1,
                    )
                }
            }
        }
        // 横轴刻度
        BoxWithConstraints(Modifier.padding(start = if (full) 30.dp else 26.dp).fillMaxWidth().height(if (full) 20.dp else 22.dp)) {
            val labels = if (full) ticksX else listOf(pAxis.lo, 1.0, pAxis.hi).filter { it in pAxis.lo..pAxis.hi }.distinct()
            labels.forEach { p ->
                val x = maxWidth * fx(p)
                Text(formatTick(p), Modifier.offset(x - 14.dp, 4.dp), fontSize = 12.sp, color = ink3, maxLines = 1)
            }
        }
    }
}

/** 横屏全屏的「智能 vs 价格」：左边大图，右边是选中模型的详情和厂商图例。进入时锁横屏，离开恢复。 */
@Composable
fun ScatterScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    LiveRankingsScatterScreen(env, back)
}
