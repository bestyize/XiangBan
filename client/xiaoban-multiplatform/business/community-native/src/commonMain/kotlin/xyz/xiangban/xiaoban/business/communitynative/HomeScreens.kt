package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.pager.HorizontalPager
import androidx.compose.foundation.pager.rememberPagerState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.pulltorefresh.PullToRefreshBox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.HomeBanner
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Metric
import xyz.xiangban.xiaoban.common.community.NewsItem
import xyz.xiangban.xiaoban.common.community.QuotaInfo
import xyz.xiangban.xiaoban.common.community.RepoIssue
import xyz.xiangban.xiaoban.common.community.RepoRank
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.clockText
import xyz.xiangban.xiaoban.common.community.countdownText
import xyz.xiangban.xiaoban.common.community.formatGain
import xyz.xiangban.xiaoban.common.community.formatScore
import xyz.xiangban.xiaoban.common.community.formatStars
import xyz.xiangban.xiaoban.common.community.parseIsoMillis
import xyz.xiangban.xiaoban.common.community.rankModels

/** 社区首页的五个二级栏目。 */
internal enum class HomeSub(val key: String, val label: String) {
    Recommend("recommend", "推荐"), Hot("hot", "开源热榜"), Rank("rank", "排行榜"), Track("track", "追踪"), Nav("nav", "导航");

    companion object {
        fun byKey(key: String) = entries.firstOrNull { it.key == key } ?: Recommend
    }
}

private val ListPadding = PaddingValues(start = 8.dp, end = 8.dp, top = 8.dp, bottom = 24.dp + NavInset)

/** 「2026-10-31」→「10 月 31 日」。 */
internal fun dayText(date: String?): String {
    val parts = date?.split('-') ?: return ""
    val m = parts.getOrNull(1)?.toIntOrNull() ?: return ""
    val d = parts.getOrNull(2)?.toIntOrNull() ?: return ""
    return "$m 月 $d 日"
}

@Composable
private fun <T> TabLoad(state: Load<T>, retry: () -> Unit, content: @Composable (T) -> Unit) = LoadBox(state, retry, content)

// ───────────── 推荐 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun RecommendTab(env: CommunityEnv, open: (Route) -> Unit, switchTo: (CommunitySection, String) -> Unit) {
    val loader = rememberLoader(env, "home") { env.forum.home() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        TabLoad(state, loader::reload) { home ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                if (home.banners.isNotEmpty()) item { BannerCarousel(home.banners) { openLink(it.link, env, open) } }
                item {
                    WhiteCard {
                        SectionHeader("AI 日报", note = listOf(dayText(home.news.date), "${home.news.total} 条").filter { it.isNotEmpty() && home.news.total > 0 }.joinToString(" · "), action = "往期", onAction = { open(Route.Daily) })
                        if (home.news.items.isEmpty()) Text("今天还没有日报", Modifier.padding(vertical = 16.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        home.news.items.forEach { NewsRow(it) { env.platform?.openUrl(it.url) } }
                    }
                }
                if (home.featured.isNotEmpty()) item {
                    WhiteCard(padding = 0.dp) {
                        SectionHeader("精选创作", Modifier.padding(16.dp), action = "更多", onAction = { switchTo(CommunitySection.Discuss, "create") })
                        LazyRow(contentPadding = PaddingValues(start = 16.dp, end = 16.dp, bottom = 16.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            items(home.featured, key = { it.id }) { pack -> Box(Modifier.width(140.dp)) { VisualCard(pack) { open(Route.Pack(pack.id)) } } }
                        }
                    }
                }
                home.repos?.takeIf { it.ranks.isNotEmpty() }?.let { issue ->
                    item {
                        WhiteCard {
                            SectionHeader("本周开源热榜", note = if (issue.issueNo > 0) "第 ${issue.issueNo} 期" else null, action = "完整榜单", onAction = { switchTo(CommunitySection.Home, HomeSub.Hot.key) })
                            issue.ranks.take(3).forEach { RepoMiniRow(it) { env.platform?.openUrl("https://github.com/${it.fullName}") } }
                        }
                    }
                }
                if (home.sample) item { FootNote("日报、头图和热榜里带「示例」字样的内容为示例数据。") }
            }
        }
    }
}

@Composable
private fun BannerCarousel(banners: List<HomeBanner>, onClick: (HomeBanner) -> Unit) {
    val pager = rememberPagerState { banners.size }
    Box(Modifier.fillMaxWidth().height(176.dp).clip(RoundedCornerShape(8.dp))) {
        HorizontalPager(pager, Modifier.fillMaxSize()) { page ->
            val b = banners[page]
            Column(
                Modifier.fillMaxSize().background(Brush.linearGradient(listOf(Color(0xFF1B2540), Color(0xFF34497A)))).clickable { onClick(b) }.padding(start = 16.dp, end = 16.dp, top = 16.dp, bottom = 28.dp),
                verticalArrangement = Arrangement.SpaceBetween,
            ) {
                if (b.tag.isNotEmpty()) Text(b.tag, Modifier.clip(RoundedCornerShape(6.dp)).background(Color.White).padding(horizontal = 8.dp, vertical = 2.dp), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = Brand) else Spacer(Modifier)
                Column {
                    Text(b.title, fontSize = 19.sp, fontWeight = FontWeight.Bold, lineHeight = 26.sp, color = Color.White, maxLines = 3, overflow = TextOverflow.Ellipsis)
                    if (b.byline.isNotEmpty()) Text(b.byline, Modifier.padding(top = 6.dp), fontSize = 12.sp, color = Color.White.copy(alpha = 0.72f))
                }
            }
        }
        if (banners.size > 1) {
            Row(Modifier.align(Alignment.BottomCenter).padding(bottom = 10.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                repeat(banners.size) { i -> Dot(if (i == pager.currentPage) Color.White else Color.White.copy(alpha = 0.4f), 6.dp) }
            }
        }
    }
}

@Composable
internal fun NewsRow(item: NewsItem, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Cover(item.coverUrl.ifEmpty { null }, item.key.hashCode().toLong() and 0x7fffffff, Modifier.size(width = 96.dp, height = 72.dp))
        Column(Modifier.weight(1f).height(72.dp), verticalArrangement = Arrangement.SpaceBetween) {
            Text(item.title, fontSize = 15.sp, fontWeight = FontWeight.Medium, lineHeight = 21.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                Dot(Brand, 6.dp)
                Text(listOf(item.source, agoText(item.publishedAt)).filter { it.isNotEmpty() }.joinToString(" · "), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

@Composable
private fun RepoMiniRow(r: RepoRank, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Text("${r.rank}", Modifier.width(18.dp), fontSize = 16.sp, fontWeight = FontWeight.Bold, color = if (r.rank <= 3) Brand else MaterialTheme.colorScheme.onSurfaceVariant)
        RepoIcon(r, 36.dp)
        Column(Modifier.weight(1f)) {
            Text(r.fullName, fontSize = 14.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
            LanguageLine(r)
        }
        Sparkline(r.spark, DataBlue, Modifier.size(width = 52.dp, height = 22.dp))
        Text(formatGain(r.starGain), Modifier.width(56.dp), fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = DataBlue, maxLines = 1)
    }
}

@Composable
private fun RepoIcon(r: RepoRank, size: androidx.compose.ui.unit.Dp) {
    val brand = hexColor(r.languageColor, DataBlue)
    LetterIcon(r.fullName.substringAfter('/'), brand, tintColor(brand, ""), size = size)
}

@Composable
private fun LanguageLine(r: RepoRank) {
    if (r.language.isEmpty()) return
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
        Dot(hexColor(r.languageColor), 8.dp)
        Text(r.language, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

// ───────────── 开源热榜 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun HotTab(env: CommunityEnv) {
    var issueKey by rememberSaveable { mutableStateOf<String?>(null) }
    var picking by remember { mutableStateOf(false) }
    val loader = rememberLoader(env, issueKey ?: "latest") { env.forum.repoIssue(issueKey) }
    val state by loader.state.collectAsState()
    Box(Modifier.fillMaxSize()) {
        PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
            TabLoad(state, loader::reload) { issue ->
                LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    item { PeriodCard(issue) { picking = true } }
                    if (issue.ranks.isEmpty()) item { WhiteCard { Text("这一期还没有榜单", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
                    items(issue.ranks, key = { it.fullName }) { r -> RepoCard(env, r) }
                    item { FootNote("按 7 日新增 star 排序，只收 AI 相关项目。" + if (issue.sample) "当前为示例数据。" else "") }
                }
            }
        }
        OverlayTransition(picking) { panel -> IssuePicker(env, issueKey, { issueKey = it; picking = false }, { picking = false }, panel) }
    }
}

@Composable
private fun PeriodCard(issue: RepoIssue, onPast: () -> Unit) {
    WhiteCard {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("第 ${issue.issueNo} 期", Modifier.clip(RoundedCornerShape(10.dp)).background(Brand).padding(horizontal = 10.dp, vertical = 2.dp), color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
            Text(listOf(issue.weekStart, issue.weekEnd).filter { it.isNotEmpty() }.joinToString(" – "), Modifier.weight(1f), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable(onClick = onPast).padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                Text("往期", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 14.dp)
            }
        }
        Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.Bottom, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("前 ${issue.ranks.size.coerceAtLeast(1)} 名本周新增 star", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(bottom = 3.dp))
            Text(formatGain(issue.totalGain).removePrefix("+"), fontSize = 24.sp, fontWeight = FontWeight.Bold, color = Brand)
        }
        if (issue.wowPct != 0.0) Text(if (issue.wowPct > 0) "比上周多 ${formatScore(issue.wowPct)}%" else "比上周少 ${formatScore(-issue.wowPct)}%", Modifier.padding(top = 2.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        if (issue.dailyGain.size == 7) {
            MiniBars(issue.dailyGain.map { it.toDouble() }, Modifier.padding(top = 12.dp).fillMaxWidth().height(56.dp))
            Row(Modifier.padding(top = 4.dp).fillMaxWidth()) {
                listOf("一", "二", "三", "四", "五", "六", "日").forEach { Text(it, Modifier.weight(1f), fontSize = 11.sp, color = LocalExtraColors.current.ink3, textAlign = androidx.compose.ui.text.style.TextAlign.Center) }
            }
        }
    }
}

@Composable
private fun RepoCard(env: CommunityEnv, r: RepoRank) {
    val (fav, setFav) = rememberPrefFlag(env, "fav_repo:${r.fullName}")
    WhiteCard(Modifier.clickable { env.platform?.openUrl("https://github.com/${r.fullName}") }) {
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            RepoIcon(r, 44.dp)
            Column(Modifier.weight(1f)) {
                Text(r.fullName, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (r.description.isNotEmpty()) Text(r.description, Modifier.padding(top = 2.dp), fontSize = 13.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
            Text("No.${r.rank}", Modifier.clip(RoundedCornerShape(6.dp)).background(if (r.rank <= 3) Brand else LocalExtraColors.current.fill).padding(horizontal = 7.dp, vertical = 2.dp), fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = if (r.rank <= 3) Color.White else MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            LanguageLine(r)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
                XIcon(XIcons.Star, tint = LocalExtraColors.current.ink3, size = 14.dp)
                Text(formatStars(r.starTotal), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
            Spacer(Modifier.weight(1f))
            Sparkline(r.spark, DataBlue, Modifier.size(width = 64.dp, height = 24.dp))
            Text(formatGain(r.starGain), fontSize = 14.sp, fontWeight = FontWeight.Bold, color = DataBlue)
            Box(Modifier.size(32.dp).clip(CircleShape).background(if (fav) Brand.copy(alpha = 0.14f) else LocalExtraColors.current.fill).clickable { setFav(!fav) }, contentAlignment = Alignment.Center) {
                XIcon(XIcons.Bookmark, tint = if (fav) Brand else MaterialTheme.colorScheme.onSurfaceVariant, size = 18.dp, description = if (fav) "取消收藏" else "收藏")
            }
        }
    }
}

@Composable
private fun IssuePicker(env: CommunityEnv, current: String?, onPick: (String?) -> Unit, onDismiss: () -> Unit, panel: Modifier) {
    val loader = rememberLoader(env, "repo-issues") { env.forum.repoIssues() }
    val state by loader.state.collectAsState()
    BottomSheet(onDismiss, panel) {
        Text("往期热榜", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
        Spacer(Modifier.height(8.dp))
        TabLoad(state, loader::reload) { list ->
            if (list.items.isEmpty()) Text("还没有往期", Modifier.padding(vertical = 16.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
            list.items.take(12).forEachIndexed { i, ref ->
                RadioRow(selected = (current ?: list.items.first().key) == ref.key, title = "第 ${ref.issueNo} 期", desc = listOf(ref.weekStart, ref.weekEnd).filter { it.isNotEmpty() }.joinToString(" – ").ifEmpty { null }, onClick = { onPick(if (i == 0) null else ref.key) })
            }
        }
        GhostPillButton("取消", onDismiss, Modifier.padding(top = 8.dp).fillMaxWidth())
    }
}

// ───────────── 排行榜 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun RankTab(env: CommunityEnv, open: (Route) -> Unit) {
    val loader = rememberLoader(env, "models") { env.forum.modelBoard() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        TabLoad(state, loader::reload) { board ->
            val iqRows = remember(board) { rankModels(board.models, Metric.Iq).take(10) }
            val speedRows = remember(board) { rankModels(board.models, Metric.Speed).take(5) }
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                item {
                    WhiteCard {
                        SectionHeader("AA 智能指数", note = "综合评测 · 越高越好", action = "全部排行", onAction = { open(Route.Leaderboard) })
                        if (iqRows.isEmpty()) Text("还没有模型数据", Modifier.padding(vertical = 16.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        iqRows.forEach { RankBarRow(it, board.vendor(it.model.vendor), formatScore(it.value)) }
                    }
                }
                item {
                    WhiteCard {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Column(Modifier.weight(1f)) {
                                Text("智能 vs 价格", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                Text("左上角最划算", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                            IconTap(XIcons.Fullscreen, "横屏全屏", { open(Route.Scatter) })
                        }
                        ScatterPlot(board.models, board.vendors, Modifier.padding(top = 8.dp).fillMaxWidth().height(220.dp), dot = 12.dp)
                        Text("横轴：价格（美元/百万 token，对数刻度）", Modifier.padding(top = 6.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                    }
                }
                item {
                    WhiteCard {
                        SectionHeader("输出速度 Top 5", note = "token/秒")
                        speedRows.forEach { RankBarRow(it, board.vendor(it.model.vendor), "${it.value.toInt()}") }
                    }
                }
                item { FootNote(sourceNote(board)) }
            }
        }
    }
}

// ───────────── 追踪 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun TrackTab(env: CommunityEnv) {
    val loader = rememberLoader(env, "tracker") { env.forum.tracker() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        TabLoad(state, loader::reload) { tracker ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                tracker.quota?.let { q -> item { QuotaCard(env, q) } }
                item {
                    WhiteCard {
                        SectionHeader("模型上新")
                        if (tracker.launches.isEmpty()) Text("最近没有新模型", Modifier.padding(vertical = 16.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        tracker.launches.forEachIndexed { i, l ->
                            Row(Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                    Dot(if (i == 0) Brand else LocalExtraColors.current.line, 10.dp, Modifier.padding(top = 4.dp))
                                    if (i < tracker.launches.lastIndex) Box(Modifier.padding(top = 4.dp).width(1.dp).height(44.dp).background(LocalExtraColors.current.hairline))
                                }
                                Column(Modifier.weight(1f)) {
                                    Text(agoText(l.publishedAt), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                    Text(l.title, Modifier.padding(top = 2.dp), fontSize = 15.sp, fontWeight = FontWeight.Medium, lineHeight = 21.sp)
                                    if (l.tags.isNotEmpty()) Row(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) { l.tags.forEach { GrayTag(it) } }
                                }
                            }
                        }
                    }
                }
                item {
                    WhiteCard {
                        SectionHeader("限时免费")
                        if (tracker.promos.isEmpty()) Text("现在没有限时免费的活动", Modifier.padding(vertical = 16.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        tracker.promos.forEach { p ->
                            Column(Modifier.padding(top = 14.dp)) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text(p.text, Modifier.weight(1f), fontSize = 15.sp, fontWeight = FontWeight.Medium, lineHeight = 21.sp)
                                    Text("剩 ${p.daysLeft} 天", fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = Brand)
                                }
                                HBar(p.pct / 100f, Brand, Modifier.padding(top = 8.dp), height = 6.dp)
                            }
                        }
                    }
                }
                if (tracker.sample) item { FootNote("追踪页里带「示例」字样的内容为示例数据。") }
            }
        }
    }
}

@Composable
private fun QuotaCard(env: CommunityEnv, q: QuotaInfo) {
    val scope = rememberCoroutineScope()
    val (remind, setRemind) = rememberPrefFlag(env, "quota_remind")
    var note by remember { mutableStateOf<String?>(null) }
    WhiteCard {
        SectionHeader("Codex 额度重置", note = if (q.updatedAt.isNotEmpty()) "${agoText(q.updatedAt)}更新" else null)
        Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            Box(contentAlignment = Alignment.Center) {
                ProgressRing(q.progressPct, Brand)
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("${q.progressPct}%", fontSize = 18.sp, fontWeight = FontWeight.Bold)
                    Text("本轮已过", fontSize = 10.sp, color = LocalExtraColors.current.ink3)
                }
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text("距离下次重置", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Text(countdownText(q.nextResetAt), fontSize = 20.sp, fontWeight = FontWeight.Bold)
                Text("预计${clockText(q.nextResetAt)}", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                val at = parseIsoMillis(q.nextResetAt)
                if (env.reminders != null && at != null) {
                    if (remind) GhostPillButton("已开启提醒", { setRemind(false) }, height = 32.dp, fontSize = 13)
                    else PillButton("提醒我", {
                        scope.launch {
                            val ok = env.reminders.schedule("Codex 额度已重置", at) == true
                            if (ok) { setRemind(true); note = "重置时会通知你" } else note = "没能设置提醒，请允许小伴发送通知"
                        }
                    }, height = 32.dp)
                }
            }
        }
        note?.let { Text(it, Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3) }
        if (q.recentIntervals.isNotEmpty()) {
            Text("最近 ${q.recentIntervals.size} 次重置间隔（小时）", Modifier.padding(top = 14.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            MiniBars(q.recentIntervals, Modifier.padding(top = 8.dp).fillMaxWidth().height(48.dp))
        }
    }
}

// ───────────── 导航 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun NavTab(env: CommunityEnv) {
    val loader = rememberLoader(env, "nav") { env.forum.navLinks() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        TabLoad(state, loader::reload) { groups ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                if (groups.groups.isEmpty()) item { CenterMessage("还没有导航", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                groups.groups.forEach { group ->
                    item(key = group.name) {
                        WhiteCard(padding = 0.dp) {
                            Text(group.name, Modifier.padding(start = 16.dp, top = 14.dp, bottom = 4.dp), fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            group.items.forEachIndexed { i, link ->
                                if (i > 0) Hairline(Modifier.padding(start = 68.dp))
                                Row(Modifier.fillMaxWidth().clickable { env.platform?.openUrl(link.url) }.padding(horizontal = 16.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                    val brand = hexColor(link.brandColor)
                                    LetterIcon(link.name, brand, tintColor(brand, link.tint), size = 40.dp)
                                    Column(Modifier.weight(1f)) {
                                        Text(link.name, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                        Text(link.vendor, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                    }
                                    XIcon(XIcons.ExternalLink, tint = LocalExtraColors.current.ink3, size = 16.dp)
                                }
                            }
                        }
                    }
                }
                item { FootNote("只放各家官方入口，点开会离开小伴。") }
            }
        }
    }
}

// ───────────── 往期日报 ─────────────

@Composable
fun DailyScreen(env: CommunityEnv, back: () -> Unit) {
    val scope = rememberCoroutineScope()
    var items by remember { mutableStateOf<List<NewsItem>>(emptyList()) }
    var page by remember { mutableStateOf(0) }
    var more by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }
    var retry by remember { mutableStateOf(0) }

    fun load() {
        if (busy || !more) return
        busy = true; error = null
        scope.launch {
            try {
                val next = env.forum.dailyNews(page + 1)
                items = items + next.items; page += 1; more = next.hasMore
            } catch (e: kotlinx.coroutines.CancellationException) {
                throw e
            } catch (e: Exception) {
                error = xyz.xiangban.xiaoban.common.community.messageOf(e)
            } finally {
                busy = false
            }
        }
    }
    androidx.compose.runtime.LaunchedEffect(retry) { load() }

    Column(Modifier.fillMaxSize()) {
        ScreenBar("AI 日报", onBack = back)
        LazyColumn(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surface), contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp)) {
            items(items, key = { it.key }) { n -> NewsRow(n) { env.platform?.openUrl(n.url) }; Hairline() }
            item {
                when {
                    error != null -> Text("加载失败，点这里重试", Modifier.fillMaxWidth().clickable { error = null; retry += 1 }.padding(16.dp), color = Danger, fontSize = 14.sp)
                    more -> { androidx.compose.runtime.LaunchedEffect(items.size) { load() }; Spinner(Modifier.fillMaxWidth().height(48.dp)) }
                    items.isEmpty() -> CenterMessage("还没有日报", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp))
                }
            }
        }
    }
}
