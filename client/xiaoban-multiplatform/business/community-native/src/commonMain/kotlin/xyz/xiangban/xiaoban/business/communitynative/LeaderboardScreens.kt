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
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
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
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.AxisRange
import xyz.xiangban.xiaoban.common.community.Metric
import xyz.xiangban.xiaoban.common.community.ModelBoard
import xyz.xiangban.xiaoban.common.community.ModelEntry
import xyz.xiangban.xiaoban.common.community.ModelFilter
import xyz.xiangban.xiaoban.common.community.ModelVendor
import xyz.xiangban.xiaoban.common.community.RankRow
import xyz.xiangban.xiaoban.common.community.TierDef
import xyz.xiangban.xiaoban.common.community.formatPrice
import xyz.xiangban.xiaoban.common.community.formatScore
import xyz.xiangban.xiaoban.common.community.formatTick
import xyz.xiangban.xiaoban.common.community.groupByTier
import xyz.xiangban.xiaoban.common.community.linearFraction
import xyz.xiangban.xiaoban.common.community.logFraction
import xyz.xiangban.xiaoban.common.community.priceAxis
import xyz.xiangban.xiaoban.common.community.priceTicks
import xyz.xiangban.xiaoban.common.community.rankModels
import xyz.xiangban.xiaoban.common.community.scoreAxis

// 模型天梯：榜单从上到下排，每行下面一条横向的柱子，从左往右长，分数越高柱子越长、越靠上。

private fun Metric.valueText(m: ModelEntry): String = when (this) {
    Metric.Price -> formatPrice(value(m))
    Metric.Speed -> "${value(m).toInt()}"
    else -> formatScore(value(m))
}

private fun Metric.help(): String = when (this) {
    Metric.Iq -> "综合推理、知识、数学、编程等多项评测的整体表现，满分 100。"
    Metric.Code -> "代码生成、修复类评测的得分，满分 100。"
    Metric.Math -> "数学与竞赛类题目的得分，满分 100。"
    Metric.Agent -> "多步骤调用工具、完成任务的得分，满分 100。"
    Metric.Speed -> "每秒输出多少个 token，越快等回答的时间越短。"
    Metric.Price -> "输入和输出按 3:1 混合计算的价格，美元/百万 token。"
}

private fun vendorColor(vendor: ModelVendor?): Color = hexColor(vendor?.brandColor.orEmpty())

/**
 * 横向柱状图的一行：名次、厂商图标、模型名和数值，下面一条按 [RankRow.fraction] 定长的柱子。
 * [detail] 不为空时点一下展开。
 */
@Composable
internal fun RankBarRow(
    row: RankRow,
    vendor: ModelVendor?,
    valueText: String,
    modifier: Modifier = Modifier,
    expanded: Boolean = false,
    onClick: (() -> Unit)? = null,
    detail: (@Composable () -> Unit)? = null,
) {
    val m = row.model
    Column(modifier.fillMaxWidth().let { if (onClick != null) it.clickable(onClick = onClick) else it }.padding(vertical = 10.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Text("${row.rank}", Modifier.width(22.dp), fontSize = 15.sp, fontWeight = FontWeight.Bold, color = if (row.rank <= 3) Brand else MaterialTheme.colorScheme.onSurfaceVariant)
            VendorIcon(m.name, vendor, size = 32.dp)
            Column(Modifier.weight(1f)) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(m.name, Modifier.weight(1f, fill = false), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    if (m.reasoning) GrayTag("推理")
                    if (m.open) GreenTag("开源")
                }
                Text(listOfNotNull(vendor?.name, m.context.takeIf { it.isNotEmpty() }?.let { "上下文 $it" }).joinToString(" · "), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text(valueText, fontSize = 18.sp, fontWeight = FontWeight.Bold)
        }
        HBar(row.fraction, vendorColor(vendor), Modifier.padding(top = 8.dp, start = 32.dp))
        if (expanded && detail != null) detail()
    }
}

@Composable
private fun StatBlock(label: String, value: String, modifier: Modifier = Modifier) {
    Column(modifier.clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(vertical = 8.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Text(value, fontSize = 15.sp, fontWeight = FontWeight.Bold)
        Text(label, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun RowDetail(m: ModelEntry) {
    Column(Modifier.padding(top = 10.dp, start = 32.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            StatBlock("智能", formatScore(m.iq), Modifier.weight(1f))
            StatBlock("价格", formatPrice(m.price), Modifier.weight(1f))
            StatBlock("速度 token/秒", "${m.speed.toInt()}", Modifier.weight(1f))
            StatBlock("延迟", "${formatScore(m.latency)}s", Modifier.weight(1f))
        }
        listOf("编程" to m.code, "数学" to m.math, "智能体" to m.agent).forEach { (label, v) ->
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(label, Modifier.width(48.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                HBar((v / 100.0).toFloat(), DataBlue, Modifier.weight(1f), height = 6.dp)
                Text(formatScore(v), Modifier.width(32.dp), fontSize = 12.sp, fontWeight = FontWeight.Medium)
            }
        }
    }
}

@Composable
private fun TierHeader(tier: TierDef, count: Int, modifier: Modifier = Modifier) {
    val (bg, fg) = when (tier.id) {
        "S" -> Brand to Color.White
        "A" -> (if (ThemeState.dark) Color(0xFF48484A) else Color(0xFF222222)) to Color.White
        "B" -> LocalExtraColors.current.line to MaterialTheme.colorScheme.onSurface
        else -> LocalExtraColors.current.fill to LocalExtraColors.current.ink3
    }
    Row(modifier.padding(top = 6.dp, bottom = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(tier.id, Modifier.clip(RoundedCornerShape(6.dp)).background(bg).padding(horizontal = 9.dp, vertical = 1.dp), color = fg, fontSize = 14.sp, fontWeight = FontWeight.Bold)
        Text(tier.label, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
        Text(if (tier.min != null) "${formatScore(tier.min!!)} 分以上 · $count 个" else "其余 · $count 个", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun VendorLegend(board: ModelBoard, modifier: Modifier = Modifier) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        board.vendors.forEach { v ->
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                Dot(hexColor(v.brandColor), 8.dp)
                Text(v.name, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

internal fun sourceNote(board: ModelBoard): String =
    "数据来源 ${board.source.ifEmpty { "Artificial Analysis" }}，每天同步一次，注明出处。" + if (board.sample) "当前为示例数据。" else ""

/** 模型天梯：按指标和筛选排榜，分数类按档位分组。 */
@Composable
fun LeaderboardScreen(env: CommunityEnv, back: () -> Unit) {
    val loader = rememberLoader(env, "models") { env.forum.modelBoard() }
    val state by loader.state.collectAsState()
    var metricName by rememberSaveable { mutableStateOf(Metric.Iq.name) }
    var filterName by rememberSaveable { mutableStateOf(ModelFilter.All.name) }
    var expanded by rememberSaveable { mutableStateOf<String?>(null) }
    var help by remember { mutableStateOf(false) }
    val metric = Metric.valueOf(metricName)
    val filter = ModelFilter.valueOf(filterName)

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            ScreenBar("模型天梯", onBack = back, trailing = {
                Text("指标说明", Modifier.clip(RoundedCornerShape(8.dp)).clickable { help = true }.padding(8.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            })
            TabStrip(Metric.entries.map { it.name to it.label }, metricName) { metricName = it; expanded = null }
            LoadBox(state, loader::reload) { board ->
                val rows = remember(board, metric, filter) { rankModels(board.models, metric, filter) }
                // 默认展开第一行。
                val open = expanded ?: rows.firstOrNull()?.model?.key
                LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(start = 8.dp, end = 8.dp, top = 8.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    item {
                        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            items(ModelFilter.entries) { f -> ChipPill(f.label, selected = f == filter, onClick = { filterName = f.name; expanded = null }) }
                        }
                    }
                    item {
                        WhiteCard {
                            Text("${metric.label} · ${metric.unit}，${if (metric.higherIsBetter) "越高越好" else "越低越好"}", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                            Text(metric.help(), Modifier.padding(top = 4.dp), fontSize = 13.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            VendorLegend(board, Modifier.padding(top = 10.dp))
                        }
                    }
                    if (rows.isEmpty()) {
                        item { WhiteCard { Text("没有符合筛选的模型", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
                    } else if (metric.tiered && board.tiers.isNotEmpty()) {
                        groupByTier(rows, board.tiers).forEach { group ->
                            item(key = "tier-${group.tier.id}") {
                                WhiteCard(padding = 12.dp) {
                                    TierHeader(group.tier, group.rows.size)
                                    group.rows.forEach { row -> BarRowWithDetail(row, board, metric, open, { expanded = if (open == row.model.key) "" else row.model.key }) }
                                }
                            }
                        }
                    } else {
                        item {
                            WhiteCard(padding = 12.dp) {
                                Text(if (metric == Metric.Price) "从便宜到贵" else "从快到慢", Modifier.padding(top = 6.dp, bottom = 2.dp), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                rows.forEach { row -> BarRowWithDetail(row, board, metric, open, { expanded = if (open == row.model.key) "" else row.model.key }) }
                            }
                        }
                    }
                    item { FootNote(sourceNote(board)) }
                }
            }
        }
        OverlayTransition(help, direction = "center") {
            CenterDialog({ help = false }) {
                Text("指标说明", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                Metric.entries.forEach { m ->
                    Text(m.label, Modifier.padding(top = 12.dp), fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                    Text(m.help(), Modifier.padding(top = 2.dp), fontSize = 13.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                GhostPillButton("知道了", { help = false }, Modifier.padding(top = 16.dp).fillMaxWidth())
            }
        }
    }
}

@Composable
private fun BarRowWithDetail(row: RankRow, board: ModelBoard, metric: Metric, openKey: String?, toggle: () -> Unit) {
    RankBarRow(row, board.vendor(row.model.vendor), metric.valueText(row.model), expanded = openKey == row.model.key, onClick = toggle, detail = { RowDetail(row.model) })
}

// ───────────── 智能 vs 价格 散点图 ─────────────

/**
 * 横轴是价格（对数刻度），纵轴是智能指数，圆点按厂商品牌色着色，左上角最划算。
 * [onSelect] 为空时只看不点；没通过 [filter] 的点变淡但不消失。
 */
@Composable
fun ScatterPlot(
    models: List<ModelEntry>,
    vendors: List<ModelVendor>,
    modifier: Modifier = Modifier,
    filter: ModelFilter = ModelFilter.All,
    selected: String? = null,
    onSelect: ((String) -> Unit)? = null,
    dot: Dp = 14.dp,
) {
    val sAxis = remember(models) { scoreAxis(models) }
    val pAxis = remember(models) { priceAxis(models) }
    val priceTicks = remember(pAxis) { priceTicks(pAxis) }
    val scoreTicks = remember(sAxis) { generateSequence(sAxis.lo) { it + 10 }.takeWhile { it <= sAxis.hi }.toList() }
    val grid = LocalExtraColors.current.hairline
    val zone = Success.copy(alpha = 0.10f)
    val dash = remember { PathEffect.dashPathEffect(floatArrayOf(12f, 10f)) }
    val ink3 = LocalExtraColors.current.ink3

    BoxWithConstraints(modifier) {
        val padL = 30.dp
        val padB = 20.dp
        val padT = 8.dp
        val padR = 10.dp
        val plotW = maxWidth - padL - padR
        val plotH = maxHeight - padB - padT
        fun xOf(price: Double) = logFraction(price, pAxis.lo, pAxis.hi)
        fun yOf(score: Double) = 1f - linearFraction(score, sAxis.lo, sAxis.hi)

        Canvas(Modifier.fillMaxSize()) {
            val left = padL.toPx()
            val top = padT.toPx()
            val w = plotW.toPx()
            val h = plotH.toPx()
            scoreTicks.forEach { s -> drawLine(grid, Offset(left, top + h * yOf(s)), Offset(left + w, top + h * yOf(s)), 1f) }
            priceTicks.forEach { p -> drawLine(grid, Offset(left + w * xOf(p), top), Offset(left + w * xOf(p), top + h), 1f) }
            // 左上角「高智能 · 低价格」：价格不超过 $1 且智能指数不低于 60。
            val zx = left + w * xOf(1.0)
            val zy = top + h * yOf(60.0)
            drawRect(zone, Offset(left, top), androidx.compose.ui.geometry.Size(zx - left, zy - top))
            drawLine(DataBlue.copy(alpha = 0.5f), Offset(left, zy), Offset(left + w, zy), 2f, pathEffect = dash)
            drawLine(DataBlue.copy(alpha = 0.5f), Offset(zx, top), Offset(zx, top + h), 2f, pathEffect = dash)
        }
        Text("高智能 · 低价格", Modifier.offset(padL + 6.dp, padT + 4.dp), fontSize = 10.sp, color = if (ThemeState.dark) Color(0xFF7BD39A) else Color(0xFF2E6B47))
        scoreTicks.forEach { s ->
            Text(formatScore(s), Modifier.offset(0.dp, padT + plotH * yOf(s) - 7.dp), fontSize = 10.sp, color = ink3)
        }
        priceTicks.forEach { p ->
            Text(formatTick(p), Modifier.offset(padL + plotW * xOf(p) - 14.dp, maxHeight - padB + 4.dp), fontSize = 10.sp, color = ink3)
        }
        models.forEach { m ->
            val on = filter.accepts(m)
            val isSel = m.key == selected
            val size = if (isSel) dot * 1.4f else dot
            val color = hexColor(vendors.firstOrNull { it.key == m.vendor }?.brandColor.orEmpty())
            Box(
                Modifier.offset(padL + plotW * xOf(m.price) - size / 2, padT + plotH * yOf(m.iq) - size / 2).size(size).alpha(if (on) 1f else 0.18f)
                    .clip(CircleShape).background(color).let { if (isSel) it.border(2.dp, MaterialTheme.colorScheme.onSurface, CircleShape) else it }
                    .let { if (onSelect != null && on) it.clickable { onSelect(m.key) } else it },
            )
        }
    }
}

/** 横屏全屏的「智能 vs 价格」：左边大图，右边是选中模型的详情。进入时锁横屏，离开恢复。 */
@Composable
fun ScatterScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
    DisposableEffect(env) {
        env.platform?.setLandscape(true)
        onDispose { env.platform?.setLandscape(false) }
    }
    val loader = rememberLoader(env, "models") { env.forum.modelBoard() }
    val state by loader.state.collectAsState()
    var filterName by rememberSaveable { mutableStateOf(ModelFilter.All.name) }
    var selected by rememberSaveable { mutableStateOf<String?>(null) }
    val filter = ModelFilter.valueOf(filterName)

    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        LoadBox(state, loader::reload) { board ->
            val current = board.models.firstOrNull { it.key == selected } ?: board.models.firstOrNull { filter.accepts(it) }
            Row(Modifier.fillMaxSize().padding(horizontal = 16.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                Column(Modifier.weight(0.68f).fillMaxHeight()) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable(onClick = back).padding(end = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                            XIcon(XIcons.ChevronLeft, size = 20.dp)
                            Text("退出全屏", fontSize = 13.sp)
                        }
                        Text("智能 vs 价格", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                        Text("纵轴 智能指数 · 横轴 价格（美元/百万 token，对数）", Modifier.weight(1f), fontSize = 11.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                    LazyRow(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        items(ModelFilter.entries) { f -> ChipPill(f.label, selected = f == filter, onClick = { filterName = f.name }) }
                    }
                    ScatterPlot(board.models, board.vendors, Modifier.weight(1f).fillMaxWidth().padding(top = 6.dp), filter, current?.key, onSelect = { selected = it }, dot = 16.dp)
                }
                Column(Modifier.weight(0.32f).fillMaxHeight(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    if (current != null) {
                        val vendor = board.vendor(current.vendor)
                        Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                VendorIcon(current.name, vendor, size = 28.dp)
                                Column {
                                    Text(current.name, fontSize = 14.sp, fontWeight = FontWeight.Bold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                    Text(listOfNotNull(vendor?.name, "推理".takeIf { current.reasoning }, "开源".takeIf { current.open }).joinToString(" · "), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                                }
                            }
                            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                StatBlock("智能", formatScore(current.iq), Modifier.weight(1f))
                                StatBlock("价格", formatPrice(current.price), Modifier.weight(1f))
                                StatBlock("速度", "${current.speed.toInt()}", Modifier.weight(1f))
                            }
                            listOf("编程" to current.code, "数学" to current.math, "智能体" to current.agent).forEach { (label, v) ->
                                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                    Text(label, Modifier.width(44.dp), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                    HBar((v / 100.0).toFloat(), DataBlue, Modifier.weight(1f), height = 5.dp)
                                    Text(formatScore(v), fontSize = 11.sp)
                                }
                            }
                        }
                        GhostPillButton("在天梯中查看", { open(Route.Leaderboard) }, Modifier.fillMaxWidth(), height = 36.dp, fontSize = 13)
                    } else {
                        Text("没有符合筛选的模型", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    VendorLegend(board)
                    Text("点圆点看详情${if (board.sample) " · 示例数据" else ""}", fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                }
            }
        }
    }
}
