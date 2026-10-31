package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.*
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.log10
import kotlin.math.pow

private val RankingsPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 4.dp, bottom = 112.dp)

/** 保留首页的紧凑榜单、图表与速度概览；完整指标筛选在“全部排行”中。 */
@Composable
internal fun LiveRankingsContent(env: CommunityEnv, open: ((Route) -> Unit)? = null, preview: Boolean = false) {
    var mode by rememberSaveable { mutableStateOf(env.prefs.get("community.rankings.mode") ?: "ability") }
    Column(Modifier.fillMaxSize()) {
        Row(Modifier.padding(horizontal = 12.dp, vertical = 8.dp).clip(RoundedCornerShape(9.dp))
            .background(LocalExtraColors.current.fill).padding(3.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            listOf("ability" to "能力榜", "usage" to "调用量榜").forEach { (key, label) ->
                Text(label, Modifier.clip(RoundedCornerShape(7.dp))
                    .background(if (mode == key) MaterialTheme.colorScheme.surface else Color.Transparent)
                    .clickable { mode = key; env.prefs.put("community.rankings.mode", key) }
                    .padding(horizontal = 18.dp, vertical = 7.dp), fontSize = 13.sp,
                    color = if (mode == key) LocalExtraColors.current.brandInk else LocalExtraColors.current.ink3,
                    fontWeight = if (mode == key) FontWeight.SemiBold else FontWeight.Normal)
            }
        }
        Box(Modifier.weight(1f)) {
            if (mode == "usage") UsageRankings(env) else AbilityRankings(env, open, preview)
        }
    }
}

@Composable
private fun <T> RankingLoadBox(state: Load<T>, retry: () -> Unit, empty: String, content: @Composable (T) -> Unit) {
    when (state) {
        Load.Loading -> Spinner()
        is Load.Failed -> CenterMessage(if (state.notFound) empty else state.message, if (state.notFound) "刷新" else "重试", retry)
        is Load.Ready -> content(state.value)
    }
}

private fun rankingColor(name: String): Color {
    BrandLogos.find(name)?.let { return it.color }
    val n = name.lowercase()
    return when {
        "openai" in n -> Color(0xFF10A37F)
        "z-ai" in n -> Color(0xFF5366D3)
        "meta" in n || "tencent" in n -> Color(0xFF2F6BFF)
        "mistral" in n -> Color(0xFFEF9538)
        else -> DataBlue
    }
}

@Composable
private fun RankingVendor(name: String) {
    val logo = BrandLogos.find(name)
    if (logo != null) XIcon(logo.icon, tint = logo.color, size = 18.dp)
    else LetterIcon(name.ifBlank { "?" }, rankingColor(name), rankingColor(name).copy(alpha = 0.12f), size = 18.dp)
}

@Composable
private fun RankingSource(env: CommunityEnv, source: String, url: String, note: String) {
    Column(Modifier.fillMaxWidth().padding(horizontal = 2.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(3.dp)) {
        Text("数据来源 $source", Modifier.clip(RoundedCornerShape(6.dp)).clickable(enabled = env.platform != null) {
            env.platform?.openUrl(url)
        }.padding(vertical = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.brandInk)
        Text(note, fontSize = 11.sp, lineHeight = 16.sp, color = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun LiveModelBar(row: LiveModelRank, vendor: String, metric: LiveModelMetric) {
    var expanded by rememberSaveable(row.model.id) { mutableStateOf(false) }
    val value = row.value?.let {
        if (metric == LiveModelMetric.InputPrice || metric == LiveModelMetric.OutputPrice) formatLivePrice(it) else formatScore(it)
    } ?: "—"
    Column(Modifier.fillMaxWidth().clickable { expanded = !expanded }.padding(vertical = 3.dp)) {
        Row(Modifier.height(28.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
            Text(row.rank?.toString() ?: "—", Modifier.width(18.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
            RankingVendor(vendor)
            Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                HBar(row.fraction ?: 0f, rankingColor(vendor).copy(alpha = if (ThemeState.dark) 0.32f else 0.18f),
                    height = 26.dp, track = LocalExtraColors.current.fill)
                Text(row.model.name, Modifier.padding(horizontal = 7.dp), fontSize = 12.sp,
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text(value, Modifier.width(if (metric == LiveModelMetric.Speed) 46.dp else 40.dp), fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold, maxLines = 1, textAlign = androidx.compose.ui.text.style.TextAlign.End)
        }
        if (expanded) Text("${row.model.name}\n$vendor · ${metric.label} $value · 输入 ${formatLivePrice(row.model.pricing.input)} / 输出 ${formatLivePrice(row.model.pricing.output)}",
            Modifier.padding(start = 25.dp, top = 5.dp, bottom = 7.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
    }
}

@OptIn(androidx.compose.foundation.layout.ExperimentalLayoutApi::class)
@Composable
private fun LiveVendorLegend(vendors: List<LiveModelVendor>) {
    androidx.compose.foundation.layout.FlowRow(Modifier.padding(top = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp), verticalArrangement = Arrangement.spacedBy(5.dp)) {
        vendors.forEach { vendor ->
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                Dot(rankingColor(vendor.name), 6.dp)
                Text(vendor.name, fontSize = 11.sp, color = LocalExtraColors.current.ink3)
            }
        }
    }
}

@Composable
private fun LiveScatterCard(board: LiveModelBoard, vendors: Map<String, LiveModelVendor>, open: ((Route) -> Unit)?) {
    val points = remember(board) { liveScatterPoints(board) }
    WhiteCard(padding = 14.dp) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text("智能 vs 价格", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                Text("左上角：较高智能、较低价格", Modifier.padding(top = 2.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
            if (points.isNotEmpty() && open != null) IconTap(XIcons.Fullscreen, "全屏查看", { open(Route.Scatter) })
        }
        if (points.isEmpty()) Text("暂无同时具备智能指数和完整价格的模型", Modifier.padding(vertical = 16.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
        else LiveIntelligenceScatter(points, Modifier.padding(top = 8.dp).fillMaxWidth().height(220.dp), vendors)
        Text("加权价格：输入 × 75% + 输出 × 25%，美元/百万 Token", Modifier.padding(top = 6.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AbilityRankings(env: CommunityEnv, open: ((Route) -> Unit)?, preview: Boolean) {
    val loader = rememberLoader(env, "live-models-v2") { env.forum.liveModelBoard() }
    val state by loader.state.collectAsState()
    var metricName by rememberSaveable { mutableStateOf(env.prefs.get("community.rankings.metric") ?: LiveModelMetric.Intelligence.name) }
    PullToRefreshBox(state is Load.Loading, loader::reload, Modifier.fillMaxSize()) {
        RankingLoadBox(state, loader::reload, "能力榜尚未发布\n发布后可在这里查看真实评测和价格") { board ->
            if (board.models.isEmpty()) CenterMessage("暂无模型数据", "刷新", loader::reload)
            else {
                val available = remember(board) { availableLiveMetrics(board) }
                val metric = if (preview) LiveModelMetric.Intelligence else available.firstOrNull { it.name == metricName } ?: available.first()
                val rows = remember(board, metric) { rankLiveModels(board.models, metric) }
                val vendors = remember(board) { board.vendors.associateBy { it.id } }
                LazyColumn(Modifier.fillMaxSize(), contentPadding = RankingsPadding, verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    if (!preview) {
                        item {
                            LazyRow(horizontalArrangement = Arrangement.spacedBy(6.dp), contentPadding = PaddingValues(vertical = 4.dp)) {
                                items(available) { option -> PageChip(option.label, metric == option) {
                                    metricName = option.name; env.prefs.put("community.rankings.metric", option.name)
                                } }
                            }
                        }
                        item { Text("${metric.unit} · ${if (metric.higherIsBetter) "越高越好" else "越低越好"} · 缺失测量不参与排名",
                            Modifier.padding(horizontal = 2.dp, vertical = 6.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3) }
                    }
                    if (preview) item {
                        val top = rows.filter { it.value != null }.take(10)
                        WhiteCard(padding = 14.dp) {
                            Row(verticalAlignment = Alignment.Top) {
                                Column(Modifier.weight(1f)) {
                                    Text("AA 智能指数", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                    Text("指数 v${board.indexVersion} · ${rows.count { it.value != null }} 个已测模型", Modifier.padding(top = 3.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                                }
                                if (open != null) Row(Modifier.clip(RoundedCornerShape(6.dp)).clickable { open(Route.Leaderboard) }.padding(vertical = 3.dp), verticalAlignment = Alignment.CenterVertically) {
                                    Text("全部排行", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                    XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 14.dp)
                                }
                            }
                            Column(Modifier.padding(top = 10.dp)) {
                                top.forEach { row -> LiveModelBar(row, vendors[row.model.vendorId]?.name ?: "未知厂商", metric) }
                            }
                            if (top.isEmpty()) Text("暂无智能指数测量", Modifier.padding(vertical = 16.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                            LiveVendorLegend(top.mapNotNull { vendors[it.model.vendorId] }.distinctBy { it.id })
                            Text("${dayLabel(board.asOf)}更新 · 北京时间 · 点模型查看完整名称", Modifier.padding(top = 8.dp), fontSize = 10.sp, color = LocalExtraColors.current.ink3)
                        }
                    } else {
                        items(rows.chunked(25), key = { it.first().model.id }) { group ->
                            WhiteCard(padding = 12.dp) { group.forEach { row -> LiveModelBar(row, vendors[row.model.vendorId]?.name ?: "未知厂商", metric) } }
                        }
                    }
                    item { LiveScatterCard(board, vendors, open) }
                    if (preview) item {
                        val speed = remember(board) { rankLiveModels(board.models, LiveModelMetric.Speed).filter { it.value != null }.take(5) }
                        WhiteCard(padding = 14.dp) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Text("输出速度 Top 5", Modifier.weight(1f), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                Text("Token/秒", fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                            }
                            Column(Modifier.padding(top = 10.dp)) { speed.forEach { row -> LiveModelBar(row, vendors[row.model.vendorId]?.name ?: "未知厂商", LiveModelMetric.Speed) } }
                        }
                    }
                    item { RankingSource(env, board.source, board.sourceUrl,
                        "智能指数 v${board.indexVersion} · ${dayLabel(board.asOf)}更新（北京时间）。暂无数据表示未测量，不计入排名。") }
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun UsageRankings(env: CommunityEnv) {
    var date by rememberSaveable { mutableStateOf(env.prefs.get("community.rankings.date")?.takeIf { it.isNotEmpty() }) }
    val datesLoader = rememberLoader(env, "usage-dates") { env.forum.modelUsageDates() }
    val datesState by datesLoader.state.collectAsState()
    val loader = rememberLoader(env, "usage-${date ?: "latest"}") { env.forum.modelUsage(date) }
    val state by loader.state.collectAsState()
    fun reload() { loader.reload(); datesLoader.reload() }
    PullToRefreshBox(state is Load.Loading, ::reload, Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            val dates = (datesState as? Load.Ready)?.value?.items?.sortedByDescending { it.date }.orEmpty()
            LazyRow(contentPadding = PaddingValues(horizontal = 12.dp, vertical = 2.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                item { PageChip("最新日榜", date == null) { date = null; env.prefs.put("community.rankings.date", "") } }
                items(dates, key = { it.date }) { ref -> PageChip(ref.date.drop(5) + " UTC", date == ref.date) {
                    date = ref.date; env.prefs.put("community.rankings.date", ref.date)
                } }
            }
            if (datesState is Load.Failed) Text("日期加载失败，点此重试", Modifier.clickable { datesLoader.reload() }.padding(horizontal = 16.dp, vertical = 5.dp), fontSize = 12.sp, color = LocalExtraColors.current.brandInk)
            Box(Modifier.weight(1f)) {
                RankingLoadBox(state, ::reload, "这一天的调用量榜尚未发布\n可切换统计日或稍后刷新") { board ->
                    if (board.models.isEmpty()) CenterMessage("暂无模型用量数据", "刷新", ::reload)
                    else LazyColumn(Modifier.fillMaxSize(), contentPadding = RankingsPadding, verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        item {
                            WhiteCard(padding = 14.dp) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text("调用量 Top 50", Modifier.weight(1f), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                    Text("${board.date} · UTC", fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                                }
                                Text("OpenRouter · 输入与输出 Token 总量", Modifier.padding(top = 4.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                                Text("全天 ${formatTokenCount(board.totalTokens)} Token", Modifier.padding(top = 8.dp), fontSize = 13.sp, fontWeight = FontWeight.Medium)
                            }
                        }
                        items(board.models.sortedBy { it.rank }.chunked(10), key = { it.first().id }) { group ->
                            WhiteCard(padding = 12.dp) { group.forEach { model -> UsageModelBar(model, board.previousDateAvailable) } }
                        }
                        item { RankingSource(env, "OpenRouter", board.sourceUrl,
                            "Source: OpenRouter (openrouter.ai/rankings), as of ${board.asOf.take(10)}.\n其他模型 ${formatTokenCount(board.otherTokens)} Token；计入全天总量。排名变化与前一完整 UTC 日对比，不代表请求次数。") }
                    }
                }
            }
        }
    }
}

@Composable
private fun UsageModelBar(model: ModelUsageEntry, previousAvailable: Boolean) {
    var expanded by rememberSaveable(model.id) { mutableStateOf(false) }
    val vendor = model.id.substringBefore('/')
    Column(Modifier.fillMaxWidth().clickable { expanded = !expanded }.padding(vertical = 7.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(7.dp)) {
            Text(model.rank.toString(), Modifier.width(18.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
            RankingVendor(vendor)
            Text(model.name.substringAfter('/'), Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis, fontSize = 12.sp, fontWeight = FontWeight.Medium)
            Text(shareText(model.sharePct, digits = 2), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
        }
        Row(Modifier.padding(start = 25.dp, top = 5.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            HBar((model.sharePct / 100.0).toFloat().coerceIn(0f, 1f), rankingColor(vendor), Modifier.weight(1f), height = 5.dp, track = LocalExtraColors.current.fill)
            Text(modelUsageMovement(model, previousAvailable), Modifier.width(48.dp), fontSize = 10.sp, color = LocalExtraColors.current.ink3, textAlign = androidx.compose.ui.text.style.TextAlign.End)
        }
        if (expanded) Text("${model.name}\n${formatTokenCount(model.tokens)} Token · ${shareText(model.sharePct)}",
            Modifier.padding(start = 25.dp, top = 6.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
    }
}

private fun shareText(value: Double, digits: Int = 6): String {
    if (!value.isFinite() || value < 0.0) return "暂无数据"
    val scale = 10.0.pow(digits).toLong()
    if (value > 0 && value < 1.0 / scale) return "<0.${"0".repeat(digits - 1)}1%"
    val scaled = (value * scale + 0.5).toLong()
    val decimals = (scaled % scale).toString().padStart(digits, '0').trimEnd('0')
    return "${scaled / scale}${if (decimals.isEmpty()) "" else ".$decimals"}%"
}

/** 保留散点图的可选中交互，只画真实的两个坐标，不构造旧榜单的占位评分。 */
@Composable
private fun LiveIntelligenceScatter(points: List<LiveScatterPoint>, modifier: Modifier = Modifier, vendors: Map<String, LiveModelVendor> = emptyMap()) {
    var selected by rememberSaveable { mutableStateOf<String?>(null) }
    val current = points.firstOrNull { it.id == selected }
    val lo = points.minOf { it.intelligence }; val hi = points.maxOf { it.intelligence }
    val minScore = floor(lo / 5) * 5
    val maxScore = (ceil(hi / 5) * 5).coerceAtLeast(minScore + 5)
    val minPrice = 10.0.pow(floor(log10(points.minOf { it.weightedPrice })))
    val maxPrice = 10.0.pow(ceil(log10(points.maxOf { it.weightedPrice }))).coerceAtLeast(minPrice * 10)
    val ink = LocalExtraColors.current.ink3
    val line = LocalExtraColors.current.line
    Column(modifier) {
        BoxWithConstraints(Modifier.weight(1f).fillMaxWidth().padding(start = 32.dp, end = 12.dp, bottom = 20.dp, top = 16.dp)) {
            val width = maxWidth; val height = maxHeight
            Canvas(Modifier.fillMaxSize()) {
                drawLine(line, Offset.Zero, Offset(0f, size.height))
                drawLine(line, Offset(0f, size.height), Offset(size.width, size.height))
                drawLine(line, Offset(0f, size.height / 2), Offset(size.width, size.height / 2))
            }
            listOf(minScore, (minScore + maxScore) / 2, maxScore).forEach { score ->
                Text(formatScore(score), Modifier.offset(x = (-32).dp, y = height * (1 - linearFraction(score, minScore, maxScore)) - 7.dp), fontSize = 11.sp, color = ink)
            }
            listOf(minPrice, maxPrice).forEach { price ->
                Text(formatTick(price), Modifier.offset(x = width * logFraction(price, minPrice, maxPrice) - 12.dp, y = height + 4.dp), fontSize = 11.sp, color = ink)
            }
            points.forEach { point ->
                val x = width * logFraction(point.weightedPrice, minPrice, maxPrice)
                val y = height * (1 - linearFraction(point.intelligence, minScore, maxScore))
                val color = rankingColor(vendors[point.vendorId]?.name ?: point.vendorId)
                Dot(if (point.id == selected) color else color.copy(alpha = 0.7f), if (point.id == selected) 14.dp else 9.dp,
                    Modifier.offset(x - 5.dp, y - 5.dp).clickable { selected = point.id })
                if (points.size <= 14 || point.id == selected) Text(shortLabel(point.name), Modifier.offset(x + 6.dp, y - 18.dp), fontSize = 11.sp, maxLines = 1, color = ink)
            }
        }
        Text(current?.let { "${it.name} · 智能 ${formatScore(it.intelligence)} · 加权价格 ${formatLivePrice(it.weightedPrice)}" } ?: "点圆点查看模型 · 左上角为较高智能、较低价格",
            Modifier.padding(top = 6.dp), fontSize = 12.sp, color = ink)
    }
}

/** Route.Scatter 的横屏入口也读取同一份真实已发布快照。 */
@Composable
internal fun LiveRankingsScatterScreen(env: CommunityEnv, back: () -> Unit) {
    DisposableEffect(env) {
        env.platform?.setLandscape(true)
        onDispose { env.platform?.setLandscape(false) }
    }
    val loader = rememberLoader(env, "live-scatter-v2") { env.forum.liveModelBoard() }
    val state by loader.state.collectAsState()
    Column(Modifier.fillMaxSize()) {
        ScreenBar("智能 vs 价格", onBack = back)
        Box(Modifier.weight(1f)) {
            RankingLoadBox(state, loader::reload, "能力榜尚未发布，暂无法绘制图表") { board ->
                val points = remember(board) { liveScatterPoints(board) }
                if (points.isEmpty()) CenterMessage("暂无同时具备智能指数和完整价格的模型", "刷新", loader::reload)
                else Column(Modifier.fillMaxSize().padding(horizontal = 16.dp)) {
                    Text("AA 智能指数 v${board.indexVersion} · 输入:输出 = 3:1 的加权价格（美元/百万 Token，对数轴）",
                        fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    LiveIntelligenceScatter(points, Modifier.weight(1f).fillMaxWidth(), board.vendors.associateBy { it.id })
                    Text("来源 ${board.source} · ${dayLabel(board.asOf)}更新（北京时间） · 缺失测量或零价格不参与图表",
                        Modifier.padding(bottom = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                }
            }
        }
    }
}
