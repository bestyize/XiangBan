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

private val RankingsPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 112.dp)

/** 首页和模型天梯共享真实榜单入口，切换和筛选写入原有轻量偏好存储。 */
@Composable
internal fun LiveRankingsContent(env: CommunityEnv, open: ((Route) -> Unit)? = null, preview: Boolean = false) {
    var mode by rememberSaveable { mutableStateOf(env.prefs.get("community.rankings.mode") ?: "ability") }
    Column(Modifier.fillMaxSize()) {
        TabStrip(listOf("ability" to "能力榜", "usage" to "调用量榜"), mode) {
            mode = it; env.prefs.put("community.rankings.mode", it)
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

@Composable
private fun RankingSource(env: CommunityEnv, source: String, url: String, note: String) {
    Column(Modifier.fillMaxWidth().padding(vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("数据来源 $source", Modifier.clip(RoundedCornerShape(6.dp)).clickable(enabled = env.platform != null) {
            env.platform?.openUrl(url)
        }.padding(vertical = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.brandInk)
        Text(note, fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
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
            if (board.models.isEmpty()) {
                CenterMessage("暂无模型数据", "刷新", loader::reload)
            } else {
                val available = remember(board) { availableLiveMetrics(board) }
                val metric = available.firstOrNull { it.name == metricName } ?: available.first()
                val rows = remember(board, metric) { rankLiveModels(board.models, metric) }
                val vendors = remember(board) { board.vendors.associateBy { it.id } }
                val points = remember(board) { liveScatterPoints(board) }
                LazyColumn(Modifier.fillMaxSize(), contentPadding = RankingsPadding, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    item {
                        WhiteCard(padding = 14.dp) {
                            Text("Artificial Analysis 能力榜", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                            Text("智能指数 v${board.indexVersion} · 仅在相同口径内比较", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            Text("采集于 ${board.asOf}", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                        }
                    }
                    item {
                        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp), contentPadding = PaddingValues(vertical = 6.dp)) {
                            items(available) { option -> PageChip(option.label, metric == option) {
                                metricName = option.name; env.prefs.put("community.rankings.metric", option.name)
                            } }
                        }
                    }
                    item {
                        Text("${metric.label} · ${metric.unit} · ${if (metric.higherIsBetter) "越高越好" else "越低越好"}",
                            Modifier.padding(horizontal = 4.dp, vertical = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    }
                    items(if (preview) rows.take(10) else rows, key = { it.model.id }) { row ->
                        WhiteCard(padding = 12.dp) {
                            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                Text(row.rank?.toString() ?: "—", Modifier.width(24.dp), fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                                Column(Modifier.weight(1f)) {
                                    Text(row.model.name, fontSize = 14.sp, fontWeight = FontWeight.Medium)
                                    Text(vendors[row.model.vendorId]?.name ?: "厂商暂无数据", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                }
                                Text(row.value?.let { if (metric == LiveModelMetric.InputPrice || metric == LiveModelMetric.OutputPrice) formatLivePrice(it) else formatScore(it) } ?: "暂无数据",
                                    fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                            }
                            row.fraction?.let { HBar(it, Brand, Modifier.padding(top = 10.dp).fillMaxWidth(), height = 5.dp, track = LocalExtraColors.current.fill) }
                        }
                    }
                    if (preview && open != null) item {
                        GhostPillButton("查看全部 ${rows.size} 个模型", { open(Route.Leaderboard) }, Modifier.fillMaxWidth().padding(vertical = 4.dp))
                    }
                    item {
                        WhiteCard(padding = 14.dp) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Text("智能 vs 价格", Modifier.weight(1f), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                if (points.isNotEmpty() && open != null) IconTap(XIcons.Fullscreen, "全屏查看", { open(Route.Scatter) })
                            }
                            if (points.isEmpty()) Text("暂无同时具备智能指数和完整价格的模型", Modifier.padding(vertical = 16.dp), fontSize = 13.sp, color = LocalExtraColors.current.ink3)
                            else LiveIntelligenceScatter(points, Modifier.padding(top = 10.dp).fillMaxWidth().height(240.dp))
                            Text("横轴价格 = 输入 × 75% + 输出 × 25%（美元/百万 Token）；纵轴为当前版本智能指数。缺失测量或零价格不参与对数图。",
                                Modifier.padding(top = 8.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
                        }
                    }
                    item { RankingSource(env, board.source, board.sourceUrl, "暂无数据表示未测量，不计入排名。编程及智能体旧指数已停止发布。") }
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
            LazyRow(contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                item { PageChip("最新日榜", date == null) { date = null; env.prefs.put("community.rankings.date", "") } }
                items(dates, key = { it.date }) { ref -> PageChip(ref.date, date == ref.date) {
                    date = ref.date; env.prefs.put("community.rankings.date", ref.date)
                } }
            }
            when (datesState) {
                is Load.Failed -> Text("历史日期加载失败，点此重试", Modifier.clickable { datesLoader.reload() }.padding(horizontal = 16.dp, vertical = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.brandInk)
                Load.Loading -> Text("正在读取历史日期…", Modifier.padding(horizontal = 16.dp, vertical = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                else -> Unit
            }
            Box(Modifier.weight(1f)) {
                RankingLoadBox(state, ::reload, "这一天的调用量榜尚未发布\n可切换统计日或稍后刷新") { board ->
                    if (board.models.isEmpty()) CenterMessage("暂无模型用量数据", "刷新", ::reload)
                    else LazyColumn(Modifier.fillMaxSize(), contentPadding = RankingsPadding, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        item {
                            WhiteCard(padding = 14.dp) {
                                Text("调用量榜（Token 用量）", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                Text("统计日 ${board.date} · UTC 自然日", Modifier.padding(top = 6.dp), fontSize = 13.sp)
                                Text("来源更新于 ${board.asOf}", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                Text("全天总量 ${formatTokenCount(board.totalTokens)} Token", Modifier.padding(top = 8.dp), fontSize = 13.sp)
                                Text("输入 + 输出 Token，代表 OpenRouter 平台流量。不同提供商使用各自的分词器。", Modifier.padding(top = 4.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
                            }
                        }
                        items(board.models.sortedBy { it.rank }, key = { it.id }) { model ->
                            WhiteCard(padding = 12.dp) {
                                Row(verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                    Text(model.rank.toString(), Modifier.width(24.dp), fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                                    Column(Modifier.weight(1f)) {
                                        Text(model.name, fontSize = 14.sp, fontWeight = FontWeight.Medium)
                                        Text("${formatTokenCount(model.tokens)} Token", Modifier.padding(top = 4.dp), fontSize = 13.sp)
                                        Text("占比 ${shareText(model.sharePct)} · ${modelUsageMovement(model, board.previousDateAvailable)}", Modifier.padding(top = 4.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                    }
                                }
                                HBar((model.sharePct / 100.0).toFloat().coerceIn(0f, 1f), Brand, Modifier.padding(top = 8.dp).fillMaxWidth(), height = 5.dp, track = LocalExtraColors.current.fill)
                            }
                        }
                        item { FootNote("其余模型合计 ${formatTokenCount(board.otherTokens)} Token，仅计入总量和占比分母。排名变化对比前一完整 UTC 日。") }
                        item { RankingSource(env, "OpenRouter", board.sourceUrl, "Source: OpenRouter (openrouter.ai/rankings), as of ${board.asOf}.\n按平台输入 + 输出 Token 排名，不代表请求次数、用户数或全市场份额。数据版本 ${board.version}。") }
                    }
                }
            }
        }
    }
}

private fun shareText(value: Double): String {
    if (!value.isFinite() || value < 0.0) return "暂无数据"
    if (value > 0 && value < 0.000001) return "<0.000001%"
    val scaled = (value * 1_000_000 + 0.5).toLong()
    val decimals = (scaled % 1_000_000).toString().padStart(6, '0').trimEnd('0')
    return "${scaled / 1_000_000}${if (decimals.isEmpty()) "" else ".$decimals"}%"
}

/** 保留散点图的可选中交互，只画真实的两个坐标，不构造旧榜单的占位评分。 */
@Composable
private fun LiveIntelligenceScatter(points: List<LiveScatterPoint>, modifier: Modifier = Modifier) {
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
                Dot(if (point.id == selected) Brand else Brand.copy(alpha = 0.65f), if (point.id == selected) 14.dp else 9.dp,
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
                    LiveIntelligenceScatter(points, Modifier.weight(1f).fillMaxWidth())
                    Text("来源 ${board.source} · 采集于 ${board.asOf} · 缺失测量或零价格不参与图表",
                        Modifier.padding(bottom = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                }
            }
        }
    }
}
