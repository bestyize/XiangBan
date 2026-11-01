package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.LocalTextStyle
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
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.*
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.pow

private val RankingsPadding = PaddingValues(start = 8.dp, end = 8.dp, top = 4.dp, bottom = 112.dp)

/** 保留首页的紧凑榜单、图表与速度概览；完整指标筛选在“全部排行”中。 */
@Composable
internal fun LiveRankingsContent(env: CommunityEnv, open: ((Route) -> Unit)? = null, preview: Boolean = false) {
    AbilityRankings(env, open, preview)
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
        "meta" in n -> Color(0xFF00A3D9)
        "tencent" in n -> Color(0xFF2F6BFF)
        "spacex" in n || n == "xai" -> Color(0xFF8B5CF6)
        "alibaba" in n -> Color(0xFFFF7A00)
        "xiaomi" in n -> Color(0xFFFF6900)
        "inclusion" in n -> Color(0xFF49B3F1)
        "stepfun" in n -> Color(0xFF00B8A9)
        "reflection" in n -> Color(0xFF155C2C)
        "nvidia" in n -> Color(0xFF76B900)
        "thinking" in n -> Color(0xFF6B7280)
        "mistral" in n -> Color(0xFFEF9538)
        else -> DataBlue
    }
}

@Composable
private fun RankingVendor(name: String) {
    val logo = BrandLogos.find(name)
    if (logo != null) XIcon(logo.icon, tint = if (ThemeState.dark && logo.color.luminance() < 0.18f) MaterialTheme.colorScheme.onSurface else logo.color, size = 18.dp)
    else LetterIcon(name.ifBlank { "?" }, rankingColor(name), rankingColor(name).copy(alpha = 0.12f), size = 18.dp)
}

@Composable
private fun RankingSource(env: CommunityEnv, source: String, url: String, note: String) {
    Column(Modifier.fillMaxWidth().padding(horizontal = 2.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(3.dp)) {
        Text("数据来源 $source", Modifier.clip(RoundedCornerShape(6.dp)).clickable(enabled = env.platform != null) {
            env.platform?.openUrl(url)
        }.padding(vertical = 4.dp), style = XType.Caption, color = LocalExtraColors.current.brandInk)
        Text(note, style = XType.Caption, color = LocalExtraColors.current.ink3)
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
            Text(row.rank?.toString() ?: "—", Modifier.width(18.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
            RankingVendor(vendor)
            Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                HBar(row.fraction ?: 0f, rankingColor(vendor).copy(alpha = if (ThemeState.dark) 0.32f else 0.18f),
                    height = 26.dp, track = LocalExtraColors.current.fill)
                Text(row.model.name, Modifier.padding(horizontal = 7.dp), style = XType.Caption,
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text(value, Modifier.width(if (metric == LiveModelMetric.Speed) 46.dp else 40.dp), style = XType.Caption,
                fontWeight = FontWeight.SemiBold, maxLines = 1, textAlign = androidx.compose.ui.text.style.TextAlign.End)
        }
        if (expanded) Text("${row.model.name}\n$vendor · ${metric.label} $value · 输入 ${formatLivePrice(row.model.pricing.input)} / 输出 ${formatLivePrice(row.model.pricing.output)}",
            Modifier.padding(start = 25.dp, top = 5.dp, bottom = 7.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
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
                Text(vendor.name, style = XType.Caption, color = LocalExtraColors.current.ink3)
            }
        }
    }
}

@Composable
private fun LiveScatterCard(board: LiveModelBoard, vendors: Map<String, LiveModelVendor>, open: ((Route) -> Unit)?) {
    val points = remember(board) { liveScatterPoints(board) }
    WhiteCard(padding = 14.dp, horizontalPadding = 8.dp) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text("智能 vs 价格", style = XType.Section, fontWeight = FontWeight.SemiBold)
                Text("左上角：较高智能、较低价格", Modifier.padding(top = 2.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
            }
            if (points.isNotEmpty() && open != null) IconTap(XIcons.Fullscreen, "全屏查看", { open(Route.Scatter) })
        }
        if (points.isEmpty()) Text("暂无同时具备智能指数和完整价格的模型", Modifier.padding(vertical = 16.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3)
        else LiveIntelligenceScatter(points, Modifier.padding(top = 8.dp).fillMaxWidth().height(370.dp), vendors)
        Text("加权价格：输入 × 75% + 输出 × 25%，美元/百万 Token", Modifier.padding(top = 6.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun AbilityRankings(env: CommunityEnv, open: ((Route) -> Unit)?, preview: Boolean) {
    val loader = rememberLoader(env, "live-models-v2") { env.forum.liveModelBoard() }
    val state by loader.state.collectAsState()
    val usageLoader = if (preview) rememberLoader(env, "usage-overview") { env.forum.modelUsage(null) } else null
    var metricName by rememberSaveable { mutableStateOf(env.prefs.get("community.rankings.metric") ?: LiveModelMetric.Intelligence.name) }
    PullToRefreshBox(state is Load.Loading, { loader.reload(); usageLoader?.reload() }, Modifier.fillMaxSize()) {
        RankingLoadBox(state, loader::reload, "能力榜尚未发布\n发布后可在这里查看真实评测和价格") { board ->
            if (board.models.isEmpty()) CenterMessage("暂无模型数据", "刷新", loader::reload)
            else {
                val available = remember(board) { availableLiveMetrics(board) }
                val metric = if (preview) LiveModelMetric.Intelligence else available.firstOrNull { it.name == metricName } ?: available.first()
                val rows = remember(board, metric) { rankLiveModels(board.models, metric) }
                val vendors = remember(board) { board.vendors.associateBy { it.id } }
                LazyColumn(Modifier.fillMaxSize(), contentPadding = RankingsPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    if (!preview) {
                        item {
                            LazyRow(horizontalArrangement = Arrangement.spacedBy(6.dp), contentPadding = PaddingValues(vertical = 4.dp)) {
                                items(available) { option -> PageChip(option.label, metric == option) {
                                    metricName = option.name; env.prefs.put("community.rankings.metric", option.name)
                                } }
                            }
                        }
                        item { Text("${metric.unit} · ${if (metric.higherIsBetter) "越高越好" else "越低越好"} · 缺失测量不参与排名",
                            Modifier.padding(horizontal = 2.dp, vertical = 6.dp), style = XType.Caption, color = LocalExtraColors.current.ink3) }
                    }
                    if (preview) item {
                        val top = rows.filter { it.value != null }.take(10)
                        WhiteCard(padding = 14.dp, horizontalPadding = 8.dp) {
                            Row(verticalAlignment = Alignment.Top) {
                                Column(Modifier.weight(1f)) {
                                    Text("AA 智能指数", style = XType.Section, fontWeight = FontWeight.SemiBold)
                                    Text("指数 v${board.indexVersion} · ${rows.count { it.value != null }} 个已测模型", Modifier.padding(top = 3.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                                }
                                if (open != null) Row(Modifier.clip(RoundedCornerShape(6.dp)).clickable { open(Route.Leaderboard) }.padding(vertical = 3.dp), verticalAlignment = Alignment.CenterVertically) {
                                    Text("全部排行", style = XType.Caption, color = LocalExtraColors.current.ink3)
                                    XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 14.dp)
                                }
                            }
                            Column(Modifier.padding(top = 10.dp)) {
                                top.forEach { row -> LiveModelBar(row, vendors[row.model.vendorId]?.name ?: "未知厂商", metric) }
                            }
                            if (top.isEmpty()) Text("暂无智能指数测量", Modifier.padding(vertical = 16.dp), style = XType.Secondary, color = LocalExtraColors.current.ink3)
                            LiveVendorLegend(top.mapNotNull { vendors[it.model.vendorId] }.distinctBy { it.id })
                            Text("${dayLabel(board.asOf)}更新 · 北京时间 · 点模型查看完整名称", Modifier.padding(top = 8.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                        }
                    } else {
                        items(rows.chunked(25), key = { it.first().model.id }) { group ->
                            WhiteCard(padding = 12.dp, horizontalPadding = 8.dp) { group.forEach { row -> LiveModelBar(row, vendors[row.model.vendorId]?.name ?: "未知厂商", metric) } }
                        }
                    }
                    item { LiveScatterCard(board, vendors, open) }
                    if (preview) item {
                        val speed = remember(board) { rankLiveModels(board.models, LiveModelMetric.Speed).filter { it.value != null }.take(5) }
                        WhiteCard(padding = 14.dp, horizontalPadding = 8.dp) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                Text("输出速度 Top 5", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
                                Text("Token/秒", style = XType.Caption, color = LocalExtraColors.current.ink3)
                            }
                            Column(Modifier.padding(top = 10.dp)) { speed.forEach { row -> LiveModelBar(row, vendors[row.model.vendorId]?.name ?: "未知厂商", LiveModelMetric.Speed) } }
                        }
                    }
                    if (preview && usageLoader != null) item {
                        val usageState by usageLoader.state.collectAsState()
                        UsagePreviewCard(usageState, usageLoader::reload) { open?.invoke(Route.UsageLeaderboard) }
                    }
                    item { RankingSource(env, board.source, board.sourceUrl,
                        "智能指数 v${board.indexVersion} · ${dayLabel(board.asOf)}更新（北京时间）。暂无数据表示未测量，不计入排名。") }
                }
            }
        }
    }
}

@Composable
private fun UsagePreviewCard(state: Load<ModelUsageBoard>, retry: () -> Unit, open: () -> Unit) {
    WhiteCard(Modifier.clickable(onClick = open), padding = 14.dp, horizontalPadding = 8.dp) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text("调用量 Top 5", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
            Text("全部排行", style = XType.Caption, color = LocalExtraColors.current.ink3)
            XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 14.dp)
        }
        when (state) {
            Load.Loading -> Text("正在加载 OpenRouter 日榜…", Modifier.padding(vertical = 16.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
            is Load.Failed -> Text("调用量暂时无法加载，点此重试", Modifier.clickable(onClick = retry).padding(vertical = 16.dp),
                style = XType.Caption, color = LocalExtraColors.current.brandInk)
            is Load.Ready -> {
                val top = state.value.models.sortedBy { it.rank }.take(5)
                Text("OpenRouter · ${state.value.date} UTC · 按 Token 用量", Modifier.padding(top = 3.dp, bottom = 10.dp),
                    style = XType.Caption, color = LocalExtraColors.current.ink3)
                if (top.isEmpty()) Text("暂无已发布的调用量排行", style = XType.Caption, color = LocalExtraColors.current.ink3)
                val maxShare = top.maxOfOrNull { it.sharePct }?.takeIf { it.isFinite() && it > 0 } ?: 1.0
                top.forEach { model ->
                    val vendor = model.id.substringBefore('/')
                    Row(Modifier.fillMaxWidth().height(34.dp), verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                        Text(model.rank.toString(), Modifier.width(18.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                        RankingVendor(vendor)
                        Box(Modifier.weight(1f), contentAlignment = Alignment.CenterStart) {
                            HBar((model.sharePct / maxShare).toFloat(), rankingColor(vendor).copy(alpha = if (ThemeState.dark) 0.32f else 0.18f),
                                height = 26.dp, track = LocalExtraColors.current.fill)
                            Text(model.name.substringAfter('/'), Modifier.padding(horizontal = 7.dp), style = XType.Caption,
                                maxLines = 1, overflow = TextOverflow.Ellipsis)
                        }
                        Text(shareText(model.sharePct, digits = 2), Modifier.width(46.dp), style = XType.Caption,
                            textAlign = androidx.compose.ui.text.style.TextAlign.End)
                    }
                }
            }
        }
    }
}

@Composable
internal fun LiveUsageRankingsScreen(env: CommunityEnv, back: () -> Unit) {
    Column(Modifier.fillMaxSize()) {
        ScreenBar("模型调用量榜", onBack = back)
        Box(Modifier.weight(1f)) { UsageRankings(env) }
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
            LazyRow(contentPadding = PaddingValues(horizontal = 8.dp, vertical = 2.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                item { PageChip("最新日榜", date == null) { date = null; env.prefs.put("community.rankings.date", "") } }
                items(dates, key = { it.date }) { ref -> PageChip(ref.date.drop(5) + " UTC", date == ref.date) {
                    date = ref.date; env.prefs.put("community.rankings.date", ref.date)
                } }
            }
            if (datesState is Load.Failed) Text("日期加载失败，点此重试", Modifier.clickable { datesLoader.reload() }.padding(horizontal = 16.dp, vertical = 5.dp), style = XType.Caption, color = LocalExtraColors.current.brandInk)
            Box(Modifier.weight(1f)) {
                RankingLoadBox(state, ::reload, "这一天的调用量榜尚未发布\n可切换统计日或稍后刷新") { board ->
                    if (board.models.isEmpty()) CenterMessage("暂无模型用量数据", "刷新", ::reload)
                    else LazyColumn(Modifier.fillMaxSize(), contentPadding = RankingsPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        item {
                            WhiteCard(padding = 14.dp, horizontalPadding = 8.dp) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text("调用量 Top 50", Modifier.weight(1f), style = XType.Section, fontWeight = FontWeight.SemiBold)
                                    Text("${board.date} · UTC", style = XType.Caption, color = LocalExtraColors.current.ink3)
                                }
                                Text("OpenRouter · 输入与输出 Token 总量", Modifier.padding(top = 4.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                                Text("全天 ${formatTokenCount(board.totalTokens)} Token", Modifier.padding(top = 8.dp), style = XType.Secondary, fontWeight = FontWeight.Medium)
                            }
                        }
                        items(board.models.sortedBy { it.rank }.chunked(10), key = { it.first().id }) { group ->
                            WhiteCard(padding = 12.dp, horizontalPadding = 8.dp) { group.forEach { model -> UsageModelBar(model, board.previousDateAvailable) } }
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
            Text(model.rank.toString(), Modifier.width(18.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
            RankingVendor(vendor)
            Text(model.name.substringAfter('/'), Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis, style = XType.Caption, fontWeight = FontWeight.Medium)
            Text(shareText(model.sharePct, digits = 2), style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        Row(Modifier.padding(start = 25.dp, top = 5.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            HBar((model.sharePct / 100.0).toFloat().coerceIn(0f, 1f), rankingColor(vendor), Modifier.weight(1f), height = 5.dp, track = LocalExtraColors.current.fill)
            Text(modelUsageMovement(model, previousAvailable), Modifier.width(48.dp), style = XType.Caption, color = LocalExtraColors.current.ink3, textAlign = androidx.compose.ui.text.style.TextAlign.End)
        }
        if (expanded) Text("${model.name}\n${formatTokenCount(model.tokens)} Token · ${shareText(model.sharePct)}",
            Modifier.padding(start = 25.dp, top = 6.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
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

/** 默认突出代表模型，横屏仅保留图表和必要操作，搜索列表按需展开。 */
@Composable
private fun LiveIntelligenceScatter(
    points: List<LiveScatterPoint>, modifier: Modifier = Modifier,
    vendors: Map<String, LiveModelVendor> = emptyMap(), full: Boolean = false,
    back: (() -> Unit)? = null, sourceNote: String = "",
) {
    val ordered = remember(points) { points.sortedByDescending { it.intelligence } }
    var selected by rememberSaveable { mutableStateOf<String?>(null) }
    var vendorId by rememberSaveable { mutableStateOf<String?>(null) }
    var query by rememberSaveable { mutableStateOf("") }
    var showList by rememberSaveable { mutableStateOf(false) }
    var showAll by rememberSaveable { mutableStateOf(false) }
    val activeVendorId = vendorId?.takeIf { id -> !full && ordered.any { it.vendorId == id } }
    val visible = ordered.filter { activeVendorId == null || it.vendorId == activeVendorId }
    val current = visible.firstOrNull { it.id == selected } ?: visible.first()
    val frontier = remember(visible) { liveScatterFrontier(visible) }
    val featured = remember(visible, showAll) {
        if (showAll || visible.size <= 28) visible else
            (frontier + visible.distinctBy { it.name.substringBefore(" (") }).distinctBy { it.id }.take(28)
    }
    val highlighted = (featured + current).distinctBy { it.id }
    val ink = LocalExtraColors.current.ink3
    val fill = LocalExtraColors.current.fill
    val surface = MaterialTheme.colorScheme.surface
    val vendorIds = remember(ordered) { ordered.map { it.vendorId }.distinct() }
    val fullControls: @Composable () -> Unit = {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
            ScatterFilter(if (showAll) "全部 ${visible.size}" else "精选 ${highlighted.size}/${visible.size}", showAll, null) { showAll = !showAll }
            Box(Modifier.size(30.dp).clip(RoundedCornerShape(5.dp)).background(if (showList) fill else Color.Transparent)
                .clickable { showList = !showList }.semantics { contentDescription = "搜索和选择模型" }, contentAlignment = Alignment.Center) {
                XIcon(XIcons.Search, size = 18.dp)
            }
            if (back != null) Box(Modifier.size(30.dp).clip(RoundedCornerShape(5.dp)).clickable(onClick = back)
                .semantics { contentDescription = "退出全屏" }, contentAlignment = Alignment.Center) { XIcon(XIcons.ChevronLeft, size = 20.dp) }
        }
    }
    val chart: @Composable (Modifier) -> Unit = { chartModifier ->
        Column(chartModifier) {
            if (!full) LazyRow(horizontalArrangement = Arrangement.spacedBy(6.dp), contentPadding = PaddingValues(vertical = 3.dp)) {
                item { ScatterFilter("全部厂商", activeVendorId == null, null) { vendorId = null } }
                items(vendorIds, key = { it }) { id ->
                    val name = vendors[id]?.name ?: id
                    ScatterFilter(name, activeVendorId == id, rankingColor(name)) { vendorId = if (activeVendorId == id) null else id }
                }
            }
            Row(Modifier.fillMaxWidth().padding(vertical = if (full) 0.dp else 3.dp), verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(if (full) sourceNote else "浅绿：智能高于中位数、价格低于中位数 · 虚线：帕累托前沿", Modifier.weight(1f),
                    style = XType.Caption, color = ink, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (full) fullControls()
            }
            ScatterCanvas(visible, highlighted, frontier, current, { selected = it.id }, vendors, full,
                Modifier.weight(1f).fillMaxWidth())
            if (!full) Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(6.dp)).background(fill).padding(horizontal = 8.dp, vertical = 5.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                RankingVendor(vendors[current.vendorId]?.name ?: current.vendorId)
                Column(Modifier.weight(1f)) {
                    Text(current.name, style = XType.Caption, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                    Text("智能 ${formatScore(current.intelligence)} · 加权价格 ${formatLivePrice(current.weightedPrice)}/百万 Token",
                        style = XType.Caption, color = ink)
                }
            }
            if (!full) {
                LazyRow(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    items(visible, key = { it.id }) { point ->
                        ScatterFilter(point.name, current.id == point.id, rankingColor(vendors[point.vendorId]?.name ?: point.vendorId)) { selected = point.id }
                    }
                }
                Text("点名称或圆点定位模型 · 重叠圆点可连续点选", Modifier.padding(top = 4.dp), style = XType.Caption, color = ink)
            }
        }
    }
    if (full) Column(modifier) {
        Row(Modifier.weight(1f).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            chart(Modifier.weight(1f).fillMaxSize())
            if (showList) Column(Modifier.width(216.dp).fillMaxSize().clip(RoundedCornerShape(8.dp)).background(fill).padding(8.dp)) {
                BasicTextField(query, { query = it }, Modifier.fillMaxWidth().clip(RoundedCornerShape(5.dp)).background(surface).padding(8.dp)
                    .semantics { contentDescription = "搜索图中模型" }, singleLine = true,
                    textStyle = XType.Caption.copy(color = MaterialTheme.colorScheme.onSurface),
                    decorationBox = { input -> Box { if (query.isEmpty()) Text("搜索图中模型", style = XType.Caption, color = ink); input() } })
                val matches = visible.filter { query.isBlank() || it.name.contains(query.trim(), ignoreCase = true) ||
                    (vendors[it.vendorId]?.name ?: it.vendorId).contains(query.trim(), ignoreCase = true) }
                Text("${matches.size} 个模型 · 点选后图中高亮", Modifier.padding(vertical = 5.dp), style = XType.Caption, color = ink)
                if (matches.isEmpty()) Text("没有匹配模型，可清空搜索或切换厂商", style = XType.Caption, color = ink)
                LazyColumn(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    items(matches, key = { it.id }) { point ->
                        val color = rankingColor(vendors[point.vendorId]?.name ?: point.vendorId)
                        Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(5.dp))
                            .background(if (current.id == point.id) color.copy(alpha = 0.12f) else Color.Transparent)
                            .clickable { selected = point.id }.padding(horizontal = 6.dp, vertical = 6.dp),
                            horizontalArrangement = Arrangement.spacedBy(7.dp), verticalAlignment = Alignment.CenterVertically) {
                            Dot(color, 7.dp)
                            Column(Modifier.weight(1f)) {
                                Text(point.name, style = XType.Caption, maxLines = 2, overflow = TextOverflow.Ellipsis,
                                    fontWeight = if (current.id == point.id) FontWeight.SemiBold else FontWeight.Normal)
                                Text("${formatScore(point.intelligence)} 分 · ${formatLivePrice(point.weightedPrice)}", style = XType.Caption, color = ink)
                            }
                        }
                    }
                }
            }
        }
    } else chart(modifier)
}

@Composable
private fun ScatterFilter(label: String, selected: Boolean, color: Color?, onClick: () -> Unit) {
    Row(Modifier.clip(RoundedCornerShape(5.dp))
        .background(if (selected) (color ?: LocalExtraColors.current.brandInk).copy(alpha = 0.12f) else LocalExtraColors.current.fill)
        .clickable(onClick = onClick).padding(horizontal = 8.dp, vertical = 5.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        if (color != null) Dot(color, 6.dp)
        Text(label, style = XType.Caption, maxLines = 1, color = if (selected) MaterialTheme.colorScheme.onSurface else LocalExtraColors.current.ink3,
            fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal)
    }
}

private data class ScatterName(val point: LiveScatterPoint, val text: String, val bounds: Rect)

@Composable
private fun ScatterCanvas(
    points: List<LiveScatterPoint>, featured: List<LiveScatterPoint>, frontier: List<LiveScatterPoint>,
    current: LiveScatterPoint, select: (LiveScatterPoint) -> Unit, vendors: Map<String, LiveModelVendor>, full: Boolean, modifier: Modifier,
) {
    val minScore = floor(points.minOf { it.intelligence } / 5) * 5
    val maxScore = (ceil(points.maxOf { it.intelligence } / 5) * 5).coerceAtLeast(minScore + 5)
    val priceRange = remember(points) { liveScatterPriceAxis(points) }
    val minPrice = priceRange.lo
    val maxPrice = priceRange.hi
    val medianScore = points.map { it.intelligence }.sorted().let { (it[(it.size - 1) / 2] + it[it.size / 2]) / 2 }
    val medianPrice = points.map { it.weightedPrice }.sorted().let { (it[(it.size - 1) / 2] + it[it.size / 2]) / 2 }
    val ink = LocalExtraColors.current.ink3
    val line = LocalExtraColors.current.line
    val surface = MaterialTheme.colorScheme.surface
    val foreground = MaterialTheme.colorScheme.onSurface
    val measurer = rememberTextMeasurer()
    val density = LocalDensity.current
    val labelSize = if (full) 9.sp else 10.sp
    val labelStyle = LocalTextStyle.current.copy(fontSize = labelSize, lineHeight = 12.sp, letterSpacing = 0.sp)
    val featuredIds = featured.map { it.id }.toSet() + current.id
    BoxWithConstraints(modifier.padding(start = 26.dp, end = 8.dp, bottom = 20.dp, top = 10.dp)) {
        val width = maxWidth.value; val height = maxHeight.value
        fun position(point: LiveScatterPoint) = Offset(width * logFraction(point.weightedPrice, minPrice, maxPrice),
            height * (1 - linearFraction(point.intelligence, minScore, maxScore)))
        val names = remember(points, featured, current.id, width, height, full, density) {
            val result = mutableListOf<ScatterName>()
            val nameHeight = 16f.coerceAtMost(height)
            val candidates = (listOf(current) + if (full) featured else featured.sortedByDescending { it.intelligence }
                .distinctBy { it.name.substringBefore(" (") }.take(3)).distinctBy { it.id }
            candidates.forEach { point ->
                val text = point.name.replace("Default Fallback", "fallback")
                val measured = measurer.measure(text, labelStyle.copy(fontWeight = if (point.id == current.id) FontWeight.SemiBold else FontWeight.Normal), maxLines = 1).size.width / density.density
                val nameWidth = (measured + 6).coerceAtMost(if (full) 220f else 150f).coerceAtMost(width)
                val pos = position(point)
                val options = buildList {
                    val steps = if (full) 8 else 3
                    for (step in 0..steps) {
                        val dy = step * 17f
                        for (y in if (step == 0) listOf(pos.y - 8) else listOf(pos.y - 8 - dy, pos.y - 8 + dy)) {
                            add(Offset(pos.x + 7, y)); add(Offset(pos.x - nameWidth - 7, y))
                        }
                    }
                }
                val bounds = options.map { o ->
                    val x = o.x.coerceIn(0f, (width - nameWidth).coerceAtLeast(0f))
                    val y = o.y.coerceIn(0f, (height - nameHeight).coerceAtLeast(0f))
                    Rect(x, y, x + nameWidth, y + nameHeight)
                }.firstOrNull { rect -> !rect.inflate(3f).contains(pos) && result.none { it.bounds.inflate(2f).overlaps(rect) } }
                if (bounds != null) result += ScatterName(point, text, bounds)
            }
            result
        }
        Canvas(Modifier.fillMaxSize().semantics { contentDescription = "智能与价格散点图，可从模型列表选择模型" }
            .pointerInput(points, current.id, width, height) {
                detectTapGestures { tap ->
                    val candidates = points.map { point ->
                        val p = position(point)
                        val delta = Offset(p.x.dp.toPx(), p.y.dp.toPx()) - tap
                        point to delta.getDistanceSquared()
                    }.filter { it.second <= 28.dp.toPx().let { r -> r * r } }.sortedBy { it.second }
                    val closest = candidates.firstOrNull()
                    if (closest != null) {
                        val overlap = candidates.takeWhile { it.second <= closest.second + 8.dp.toPx().let { r -> r * r } }.map { it.first }
                        val index = overlap.indexOfFirst { it.id == current.id }
                        select(overlap[if (index >= 0) (index + 1) % overlap.size else 0])
                    }
                }
            }) {
            drawRect(Color(0xFF6EDB83).copy(alpha = if (ThemeState.dark) 0.1f else 0.17f),
                size = androidx.compose.ui.geometry.Size(size.width * logFraction(medianPrice, minPrice, maxPrice),
                    size.height * (1 - linearFraction(medianScore, minScore, maxScore))))
            listOf(0f, 0.25f, 0.5f, 0.75f, 1f).forEach { t -> drawLine(line.copy(alpha = 0.5f), Offset(0f, size.height * t), Offset(size.width, size.height * t)) }
            drawLine(line, Offset.Zero, Offset(0f, size.height))
            fun center(point: LiveScatterPoint) = Offset(size.width * logFraction(point.weightedPrice, minPrice, maxPrice),
                size.height * (1 - linearFraction(point.intelligence, minScore, maxScore)))
            if (frontier.size > 1) {
                val path = Path()
                frontier.forEachIndexed { index, point ->
                    val pos = center(point)
                    if (index == 0) path.moveTo(pos.x, pos.y) else path.lineTo(pos.x, pos.y)
                }
                drawPath(path, foreground.copy(alpha = 0.6f), style = Stroke(1.dp.toPx(), pathEffect = PathEffect.dashPathEffect(floatArrayOf(2.dp.toPx(), 3.dp.toPx()))))
            }
            // 名称移位时画引导线，避免把邻近模型误认成同一个点。
            names.forEach { name ->
                val start = center(name.point)
                val b = name.bounds
                val end = Offset(start.x.coerceIn(b.left.dp.toPx(), b.right.dp.toPx()),
                    start.y.coerceIn(b.top.dp.toPx(), b.bottom.dp.toPx()))
                drawLine(ink.copy(alpha = 0.28f), start, end, strokeWidth = 0.5.dp.toPx())
            }
            fun drawPoint(point: LiveScatterPoint, active: Boolean) {
                val pos = center(point)
                val color = rankingColor(vendors[point.vendorId]?.name ?: point.vendorId)
                if (active) {
                    drawCircle(color.copy(alpha = 0.15f), 9.dp.toPx(), pos)
                    drawCircle(surface, 6.dp.toPx(), pos)
                }
                drawCircle(if (point.id in featuredIds) color else ink.copy(alpha = 0.28f), if (active) 4.dp.toPx() else 3.dp.toPx(), pos)
                if (active) drawCircle(color, 7.dp.toPx(), pos, style = Stroke(1.2.dp.toPx()))
            }
            points.filter { it.id != current.id }.forEach { drawPoint(it, false) }
            drawPoint(current, true)
        }
        listOf(0f, 0.25f, 0.5f, 0.75f, 1f).forEach { fraction ->
            Text(formatScore(minScore + (maxScore - minScore) * fraction),
                Modifier.offset(x = (-26).dp, y = (height * (1 - fraction) - 6).dp), style = XType.Caption, color = ink)
        }
        val ticks = liveScatterPriceTicks(priceRange).fold(mutableListOf<Double>()) { kept, price ->
            val previous = kept.lastOrNull()
            if (previous == null || width * (logFraction(price, minPrice, maxPrice) - logFraction(previous, minPrice, maxPrice)) >= 32f) kept += price
            kept
        }
        ticks.forEach { price ->
            Text(formatTick(price), Modifier.offset(x = (width * logFraction(price, minPrice, maxPrice) - 10).dp, y = (height + 3).dp),
                style = XType.Caption, color = ink)
        }
        names.forEach { name ->
            Text(name.text, Modifier.offset(name.bounds.left.dp, name.bounds.top.dp).width(name.bounds.width.dp)
                .clip(RoundedCornerShape(2.dp)).background(if (name.point.id == current.id) surface.copy(alpha = 0.9f) else Color.Transparent)
                .clickable { select(name.point) }.padding(horizontal = 3.dp, vertical = 2.dp),
                style = labelStyle, maxLines = 1, overflow = TextOverflow.Ellipsis,
                fontWeight = if (name.point.id == current.id) FontWeight.SemiBold else FontWeight.Normal,
                color = if (name.point.id == current.id) foreground else foreground.copy(alpha = 0.8f))
        }
    }
}

/** 横屏没有标题、厂商栏或底部详情，主要空间用于图表。 */
@Composable
internal fun LiveRankingsScatterScreen(env: CommunityEnv, back: () -> Unit) {
    DisposableEffect(env) {
        env.platform?.setLandscape(true)
        onDispose { env.platform?.setLandscape(false) }
    }
    val loader = rememberLoader(env, "live-scatter-v2") { env.forum.liveModelBoard() }
    val state by loader.state.collectAsState()
    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface)
        .windowInsetsPadding(WindowInsets.displayCutout.only(WindowInsetsSides.Horizontal)).padding(4.dp)) {
        RankingLoadBox(state, loader::reload, "能力榜尚未发布，暂无法绘制图表") { board ->
            val points = remember(board) { liveScatterPoints(board) }
            if (points.isEmpty()) CenterMessage("暂无同时具备智能指数和完整价格的模型", "刷新", loader::reload)
            else LiveIntelligenceScatter(points, Modifier.fillMaxSize(), board.vendors.associateBy { it.id }, full = true, back = back,
                sourceNote = "AA 智能指数 v${board.indexVersion} · ${dayLabel(board.asOf)} · 美元/百万 Token · 输入 75% + 输出 25% · 对数轴")
        }
        if ((state as? Load.Ready)?.value?.let { liveScatterPoints(it).isNotEmpty() } != true) {
            Box(Modifier.size(32.dp).clickable(onClick = back).semantics { contentDescription = "返回排行榜" }, contentAlignment = Alignment.Center) {
                XIcon(XIcons.ChevronLeft, size = 20.dp)
            }
        }
    }
}
