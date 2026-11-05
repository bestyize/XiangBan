package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.offset
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
import androidx.compose.runtime.LaunchedEffect
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
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.HomeBanner
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.Metric
import xyz.xiangban.xiaoban.common.community.ModelBoard
import xyz.xiangban.xiaoban.common.community.NewsItem
import xyz.xiangban.xiaoban.common.community.Pack
import xyz.xiangban.xiaoban.common.community.QuotaInfo
import xyz.xiangban.xiaoban.common.community.RepoIssue
import xyz.xiangban.xiaoban.common.community.RepoRank
import xyz.xiangban.xiaoban.common.community.shownDescription
import xyz.xiangban.xiaoban.common.community.repoObservationLabel
import xyz.xiangban.xiaoban.common.community.trackerOfficialSource
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.clockText
import xyz.xiangban.xiaoban.common.community.compactCount
import xyz.xiangban.xiaoban.common.community.countdownText
import xyz.xiangban.xiaoban.common.community.dayLabel
import xyz.xiangban.xiaoban.common.community.formatGain
import xyz.xiangban.xiaoban.common.community.formatScore
import xyz.xiangban.xiaoban.common.community.formatStars
import xyz.xiangban.xiaoban.common.community.kindLabel
import xyz.xiangban.xiaoban.common.community.parseIsoMillis
import xyz.xiangban.xiaoban.common.community.rankModels

/** 社区首页栏目。 */
internal enum class HomeSub(val key: String, val label: String) {
    Recommend("recommend", "推荐"), OpenSource("open-source", "精选"), Hot("hot", "开源热榜"), Rank("rank", "排行榜"), Track("track", "追踪"), Nav("nav", "导航");

    companion object {
        fun byKey(key: String) = entries.firstOrNull { it.key == key } ?: Recommend
    }
}

/** 白底卡片之间留 8dp，两边留 12dp；底部让出悬浮导航的高度。 */
private val ListPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 24.dp + NavInset)
private val Gap = 8.dp

/** 「2026-10-31」→「10 月 31 日」。 */
internal fun dayText(date: String?): String {
    val parts = date?.split('-') ?: return ""
    val m = parts.getOrNull(1)?.toIntOrNull() ?: return ""
    val d = parts.getOrNull(2)?.toIntOrNull() ?: return ""
    return "$m 月 $d 日"
}

@Composable
private fun CardTitleRow(title: String, icon: androidx.compose.ui.graphics.vector.ImageVector? = null) {
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        if (icon != null) XIcon(icon, size = 18.dp)
        Text(title, style = XType.Section, fontWeight = FontWeight.SemiBold)
    }
}

/** 榜单行里的小图标：热榜仓库用，认不出的用仓库名首字。 */
private fun repoGlyph(key: String) = when (key) {
    "term" -> XIcons.Terminal; "code" -> XIcons.Code; "doc" -> XIcons.File; "mic" -> XIcons.Mic; "bolt" -> XIcons.Bolt; else -> null
}

@Composable
internal fun RepoTile(r: RepoRank, size: Dp, corner: Dp) {
    val glyph = repoGlyph(r.icon)
    val tint = if (ThemeState.dark) hexColor(r.tint, Color(0xFF3A3A3C)).copy(alpha = 0.3f) else hexColor(r.tint, Color(0xFFF2F2F2))
    if (glyph != null) Box(Modifier.size(size).clip(RoundedCornerShape(corner)).background(tint), contentAlignment = Alignment.Center) { XIcon(glyph, size = size * 0.55f) }
    else LetterIcon(r.fullName.substringAfter('/'), hexColor(r.languageColor, DataBlue), tint, size = size)
}

@Composable
internal fun LangLine(r: RepoRank) {
    if (r.language.isEmpty()) return
    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        Dot(hexColor(r.languageColor), 8.dp)
        Text(r.language, style = XType.Caption, color = LocalExtraColors.current.ink3)
    }
}

// ───────────── 推荐 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun RecommendTab(env: CommunityEnv, open: (Route) -> Unit, switchTo: (CommunitySection, String) -> Unit) {
    val loader = rememberLoader(env, "home") { env.forum.home() }
    val state by loader.state.collectAsState()
    val frontierLoader = rememberLoader(env, "frontier") { env.forum.frontier() }
    val frontierState by frontierLoader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = { loader.reload(); frontierLoader.reload() }, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { home ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                if (home.banners.isNotEmpty()) item { BannerCarousel(home.banners) { openLink(it.link, open) } }
                item(key = "frontier") { FrontierSection(env, frontierState, frontierLoader::reload, open) }
                item {
                    Column(Modifier.padding(start = 2.dp, end = 2.dp, top = 10.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                XIcon(XIcons.Message, size = 18.dp)
                                Text("AI 日报", style = XType.Section, fontWeight = FontWeight.SemiBold)
                                val note = listOf(dayText(home.news.date), "更新").filter { it.isNotEmpty() }.joinToString(" · ")
                                if (home.news.total > 0) Text(note, style = XType.Caption, color = LocalExtraColors.current.ink3)
                            }
                            Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable { open(Route.Daily) }.padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                                Text("往期", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                                XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                            }
                        }
                    }
                }
                if (home.news.items.isEmpty()) item { WhiteCard { Text("今天还没有日报", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
                items(home.news.items, key = { "n-${it.key}" }) { NewsRow(it) { open(Route.News(it.key)) } }
                home.repos?.takeIf { it.ranks.isNotEmpty() }?.let { issue ->
                    item {
                        WhiteCard(padding = 0.dp) {
                            Column(Modifier.padding(start = 14.dp, end = 14.dp, top = 14.dp, bottom = 4.dp)) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                        XIcon(XIcons.Star, size = 18.dp)
                                        Text(if (issue.isBaseline) "开源项目初始快照" else if (issue.isTrending) "GitHub 开源热榜" else if (issue.source == "GitHub") "开源观测榜" else "本周开源热榜", style = XType.Section, fontWeight = FontWeight.SemiBold)
                                    }
                                    Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable { switchTo(CommunitySection.Home, HomeSub.Hot.key) }.padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                                        Text("完整榜单", style = XType.Secondary, color = LocalExtraColors.current.ink3)
                                        XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                                    }
                                }
                                issue.ranks.take(3).forEachIndexed { i, r ->
                                    if (i > 0) Hairline()
                                    RepoMiniRow(r) { openRepo(env, r, open) }
                                }
                            }
                        }
                    }
                }
                if (home.sample) item { FootNote("日报、头图和热榜里带「示例」字样的内容为示例数据。") }
            }
        }
    }
}

/** 社区运营推荐：与网页相同的浅色卡片，标题与署名直接在卡片内。 */
@Composable
private fun BannerCarousel(banners: List<HomeBanner>, onClick: (HomeBanner) -> Unit) {
    val pager = rememberPagerState { banners.size }
    HorizontalPager(pager, Modifier.fillMaxWidth().height(136.dp), pageSpacing = 8.dp) { page ->
        val banner = banners[page]
        Column(Modifier.fillMaxSize().outlinedCard(16.dp).clickable { onClick(banner) }.padding(16.dp), verticalArrangement = Arrangement.SpaceBetween) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(banner.tag, Modifier.weight(1f), style = XType.Caption, color = Brand)
                XIcon(XIcons.Sparkle, tint = Brand, size = 18.dp)
            }
            Text(banner.title, style = XType.Section, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                if (banners.size > 1) Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    repeat(banners.size) { index -> Box(Modifier.size(if (index == pager.currentPage) 12.dp else 4.dp, 4.dp).clip(RoundedCornerShape(2.dp)).background(if (index == pager.currentPage) Brand else LocalExtraColors.current.line)) }
                }
                Text(banner.byline, Modifier.weight(1f), style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 1, textAlign = TextAlign.End, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

/** 日报为阅读卡，前沿往期为封面卡；没有图片时只展示真实文字。 */
@Composable
internal fun NewsRow(item: NewsItem, visual: Boolean = false, onClick: () -> Unit) {
    val content: @Composable () -> Unit = {
        Column(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(item.title, style = if (visual) XType.Section else XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 3, overflow = TextOverflow.Ellipsis)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
                BrandLogos.find(item.source)?.let { XIcon(it.icon, tint = it.color, size = 14.dp) }
                val time = agoText(item.sourcePublishedAt ?: item.publishedAt)
                Text(listOf(item.source, if (item.timeBasis == "collected") "采集于 $time" else time, if (item.sample) "示例" else "").filter { it.isNotBlank() }.joinToString(" · "),
                    style = XType.Caption, color = LocalExtraColors.current.ink3, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
        }
    }
    if (visual) Column(Modifier.fillMaxWidth().outlinedCard(12.dp).clickable(onClick = onClick)) {
        if (item.coverUrl.isNotBlank()) Cover(item.coverUrl, item.key.hashCode().toLong(), Modifier.fillMaxWidth().aspectRatio(16f / 9f))
        Box(Modifier.padding(16.dp)) { content() }
    } else Row(Modifier.fillMaxWidth().outlinedCard(16.dp).clickable(onClick = onClick).padding(16.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Box(Modifier.weight(1f)) { content() }
        if (item.coverUrl.isNotBlank()) Cover(item.coverUrl, item.key.hashCode().toLong(), Modifier.width(88.dp).aspectRatio(16f / 9f))
    }
}

/** 推荐页热榜前三：名次、36dp 图标块、仓库名和语言、7 根小柱、本周新增。 */
@Composable
private fun RepoMiniRow(r: RepoRank, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Text("${r.rank}", Modifier.width(16.dp), style = XType.Section, fontWeight = FontWeight.Bold)
        RepoTile(r, 36.dp, 8.dp)
        Column(Modifier.weight(1f)) {
            Text(r.fullName, style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            LangLine(r)
        }
        if (r.starGain != null && r.spark.size >= 2) SparkBars(r.spark, height = 24.dp)
        Text(r.starGain?.let(::formatGain) ?: "${formatStars(r.starTotal)}★", Modifier.width(52.dp), style = XType.Body, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.End, maxLines = 1)
    }
}

// ───────────── 开源热榜 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun HotTab(env: CommunityEnv, open: (Route) -> Unit) {
    var issueKey by rememberSaveable { mutableStateOf<String?>(null) }
    var picking by remember { mutableStateOf(false) }
    val loader = rememberLoader(env, issueKey ?: "latest") { env.forum.repoIssue(issueKey) }
    val state by loader.state.collectAsState()
    Box(Modifier.fillMaxSize()) {
        PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
            LoadBox(state, loader::reload) { issue ->
                LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                    item { PeriodCard(issue) { picking = true } }
                    if (issue.ranks.isEmpty()) item { WhiteCard { Text("这一期还没有榜单", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
                    items(issue.ranks, key = { it.fullName }) { r -> RepoCard(env, r, open) }
                }
            }
        }
        // 弹层要盖住底部悬浮导航，所以放在独立的全屏窗口里，而不是页面内容里。
        if (picking) androidx.compose.ui.window.Dialog(onDismissRequest = { picking = false }, properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false)) {
            IssuePicker(env, issueKey, { issueKey = it; picking = false }, { picking = false })
        }
    }
}

@Composable
private fun PeriodCard(issue: RepoIssue, onPast: () -> Unit) {
    var details by rememberSaveable(issue.key) { mutableStateOf(false) }
    WhiteCard(padding = 14.dp) {
        SectionHeader(if (issue.isBaseline) "开源项目快照" else if (issue.isTrending) "GitHub 开源热榜" else "开源观测榜", action = "往期", onAction = onPast,
            note = listOf(issue.weekStart, issue.weekEnd).filter { it.isNotEmpty() }.distinct().joinToString(" – "))
        Text(if (issue.isBaseline) "${issue.ranks.size} 个项目 · 按总 Star 排序 · 增量待积累"
            else if (issue.isTrending) "${issue.ranks.size} 个项目 · 近 7 天新增 Star 合计 ${issue.totalGain?.let(::formatGain) ?: "暂无数据"}"
            else "${issue.ranks.size} 个项目 · ${issue.observedDays?.let { "$it 日观测" } ?: "本期观测"} · 净增 ${issue.totalGain?.let(::formatGain) ?: "暂无数据"}",
            Modifier.padding(top = 8.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth().clickable { details = !details }.padding(top = 10.dp, bottom = 2.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(issue.source.ifBlank { "数据来源" } + " · " + if (details) "收起数据说明" else "口径与更新时间", Modifier.weight(1f), style = XType.Caption, color = LocalExtraColors.current.ink3)
            XIcon(if (details) XIcons.ChevronUp else XIcons.ChevronDown, size = 14.dp, tint = LocalExtraColors.current.ink3)
        }
        if (details) {
            repoObservationLabel(issue.asOf)?.let { Text("整理于 $it", Modifier.padding(top = 8.dp), style = XType.Caption, color = LocalExtraColors.current.ink3) }
            Text((if (issue.isBaseline) "当前为首次观测，至少两次快照后计算净增量。"
                else if (issue.isTrending) "取自 GitHub Trending 周榜，按 GitHub 页面顺序排列；新增 Star 为 GitHub 公布的近 7 天数据。"
                else if (issue.complete) "已取得连续 7 日边界快照。" else "观测不足 7 日或存在缺采，未补造缺失数据。") + if (issue.sample) " · 示例数据" else "",
                Modifier.padding(top = 4.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
            if (issue.dailyGain.size == 7 && issue.source != "GitHub") DayBars(issue.dailyGain, listOf("一", "二", "三", "四", "五", "六", "日"))
        }
    }
}

/** 热榜卡：名次、仓库、简介、语言和实际星数；收藏操作留在详情页。 */
@Composable
private fun RepoCard(env: CommunityEnv, r: RepoRank, open: (Route) -> Unit) {
    WhiteCard(Modifier.clickable { openRepo(env, r, open) }, padding = 14.dp) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(r.rank.toString().padStart(2, '0'), style = XType.Secondary, fontWeight = FontWeight.Medium, color = if (r.rank <= 3) Brand else LocalExtraColors.current.ink3)
            RepoTile(r, 32.dp, 8.dp)
            Text(r.fullName, Modifier.weight(1f), style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
        }
        if (r.shownDescription.isNotBlank()) Text(r.shownDescription, Modifier.padding(top = 8.dp), fontSize = 13.sp, lineHeight = 20.sp, maxLines = 2, overflow = TextOverflow.Ellipsis, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth().padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            LangLine(r)
            Text("☆ ${formatStars(r.starTotal)}", style = XType.Caption, color = LocalExtraColors.current.ink3)
            Spacer(Modifier.weight(1f))
            r.starGain?.let { Text("${formatGain(it)} Star", style = XType.Caption, fontWeight = FontWeight.Medium) }
            if (r.starGain != null && r.spark.size >= 2) SparkBars(r.spark, height = 20.dp)
        }
        if (r.suspectedGrowth) Text("增长异常 · 编辑复核后收录", Modifier.padding(top = 6.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
        if (r.starGain != null && r.observedDays != null && !r.complete) Text("实际观测 ${r.observedDays} 天", Modifier.padding(top = 6.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun IssuePicker(env: CommunityEnv, current: String?, onPick: (String?) -> Unit, onDismiss: () -> Unit) {
    val loader = rememberLoader(env, "repo-issues") { env.forum.repoIssues() }
    val state by loader.state.collectAsState()
    BottomSheet(onDismiss) {
        Text("往期热榜", style = XType.Title, fontWeight = FontWeight.SemiBold)
        Spacer(Modifier.height(8.dp))
        LoadBox(state, loader::reload) { list ->
            if (list.items.isEmpty()) Text("还没有往期", Modifier.padding(vertical = 16.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
            list.items.take(12).forEachIndexed { i, ref ->
                RadioRow(selected = (current ?: list.items.first().key) == ref.key, title = if (ref.key.startsWith("github-rolling-")) "${ref.weekEnd} 观测榜" else "第 ${ref.issueNo} 期", desc = listOf(ref.weekStart, ref.weekEnd).filter { it.isNotEmpty() }.joinToString(" – ").ifEmpty { null }, onClick = { onPick(if (i == 0) null else ref.key) })
            }
        }
        GhostPillButton("取消", onDismiss, Modifier.padding(top = 8.dp).fillMaxWidth())
    }
}

// ───────────── 排行榜 ─────────────

@Composable
internal fun RankTab(env: CommunityEnv, open: (Route) -> Unit) {
    LiveRankingsContent(env, open, preview = true)
}

// ───────────── 追踪 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun TrackTab(env: CommunityEnv) {
    var section by rememberSaveable { mutableStateOf("launches") }
    val loader = rememberLoader(env, "tracker") { env.forum.tracker() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { tracker ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                item { PageChipRow(listOf("launches" to "发布记录", "promos" to "优惠与试用"), section, { section = it }) }
                tracker.quota?.let { q -> item { QuotaCard(env, q) } }
                if (section == "launches") item {
                    WhiteCard(padding = 14.dp) {
                        CardTitleRow("发布记录", XIcons.Bolt)
                        if (tracker.launches.isEmpty()) Text("暂无已核实的发布记录", Modifier.padding(vertical = 12.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Column(Modifier.padding(top = 10.dp)) {
                            tracker.launches.forEachIndexed { i, l ->
                                val source = trackerOfficialSource(l.sourceUrl)
                                Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                    Column(Modifier.width(12.dp).fillMaxHeight(), horizontalAlignment = Alignment.CenterHorizontally) {
                                        Dot(if (i == 0) DataBlue else DataBlueSoft, 10.dp, Modifier.padding(top = 5.dp))
                                        if (i < tracker.launches.lastIndex) Box(Modifier.padding(top = 2.dp).width(2.dp).weight(1f).background(LocalExtraColors.current.hairline))
                                    }
                                    Column(Modifier.weight(1f).padding(bottom = 14.dp)) {
                                        Text(l.publishedAt.take(10).ifBlank { "日期未提供" }, style = XType.Caption, color = LocalExtraColors.current.ink3)
                                        Text(l.title, Modifier.padding(top = 2.dp), style = XType.Body, fontWeight = FontWeight.SemiBold)
                                        if (l.summary.isNotBlank()) DisclosureDescription("发布说明", l.summary)
                                        if (source != null) Row(Modifier.clickable { env.platform?.openUrl(source) }.padding(vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                            Text("官方原文", style = XType.Caption, color = LocalExtraColors.current.brandInk)
                                            XIcon(XIcons.ExternalLink, size = 13.dp, tint = LocalExtraColors.current.brandInk)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if (section == "promos") item {
                    WhiteCard(padding = 14.dp) {
                        CardTitleRow("优惠与试用", XIcons.Clock)
                        if (tracker.promos.isEmpty()) Text("暂无已核实的优惠与试用", Modifier.padding(vertical = 12.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        tracker.promos.forEachIndexed { i, p ->
                            if (i > 0) Hairline(Modifier.padding(top = 10.dp))
                            Column(Modifier.padding(top = 10.dp)) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text(p.text, Modifier.weight(1f), style = XType.Body, fontWeight = FontWeight.Medium)
                                    if (!p.endsAt.isNullOrBlank()) p.daysLeft?.let { Text("剩 $it 天", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                                }
                                if (p.endsAt.isNullOrBlank()) Text("长期提供，以官方条件为准", Modifier.padding(top = 6.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                                else p.pct?.let { HBar(it / 100f, DataBlue, Modifier.padding(top = 8.dp), height = 4.dp, track = LocalExtraColors.current.fill) }
                                if (p.eligibility.isNotBlank()) Text(p.eligibility, Modifier.padding(top = 6.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                trackerOfficialSource(p.sourceUrl)?.let { source -> LinkText("查看官方条件与申请入口 ›", { env.platform?.openUrl(source) }) }
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
    WhiteCard(padding = 14.dp) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                XIcon(XIcons.Refresh, size = 18.dp)
                Text("Codex 额度重置", style = XType.Section, fontWeight = FontWeight.SemiBold)
            }
            if (q.updatedAt.isNotEmpty()) Text("${agoText(q.updatedAt)}更新", style = XType.Caption, color = LocalExtraColors.current.ink3)
        }
        Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            Box(contentAlignment = Alignment.Center) {
                ProgressRing(q.progressPct, DataBlue, size = 96.dp, stroke = 8.dp)
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("${q.progressPct}%", style = XType.Headline, fontWeight = FontWeight.Bold)
                    Text("本轮已过", style = XType.Caption, color = LocalExtraColors.current.ink3)
                }
            }
            Column(Modifier.weight(1f)) {
                Text("距离下次重置", style = XType.Caption, color = LocalExtraColors.current.ink3)
                Text(countdownText(q.nextResetAt), Modifier.padding(top = 2.dp), style = XType.Headline, fontWeight = FontWeight.Bold)
                Text("预计${clockText(q.nextResetAt)}", Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                val at = parseIsoMillis(q.nextResetAt)
                if (env.reminders != null && at != null) {
                    if (remind) Row(
                        Modifier.padding(top = 10.dp).height(34.dp).clip(RoundedCornerShape(17.dp)).background(LocalExtraColors.current.fill).clickable { setRemind(false) }.padding(horizontal = 14.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        XIcon(XIcons.Check, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 16.dp)
                        Text("已开启提醒", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    } else Row(
                        Modifier.padding(top = 10.dp).height(34.dp).clip(RoundedCornerShape(17.dp)).background(Brand).clickable {
                            scope.launch {
                                val ok = env.reminders.schedule("Codex 额度已重置", at)
                                if (ok) { setRemind(true); note = "重置时会通知你" } else note = "没能设置提醒，请允许小伴发送通知"
                            }
                        }.padding(horizontal = 14.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        XIcon(XIcons.Bell, tint = Color.White, size = 16.dp)
                        Text("提醒我", style = XType.Body, fontWeight = FontWeight.SemiBold, color = Color.White)
                    }
                }
            }
        }
        note?.let { Text(it, Modifier.padding(top = 8.dp), style = XType.Caption, color = LocalExtraColors.current.ink3) }
        if (q.recentIntervals.isNotEmpty()) {
            Hairline(Modifier.padding(top = 14.dp))
            Text("最近 ${q.recentIntervals.size} 次重置间隔（小时）", Modifier.padding(top = 12.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
            IntervalBars(q.recentIntervals, Modifier.padding(top = 8.dp))
        }
    }
}

// ───────────── 往期日报 ─────────────

@Composable
fun DailyScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit, frontier: Boolean = false) {
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
                val next = if (frontier) env.forum.frontier(page + 1) else env.forum.dailyNews(page + 1)
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
    LaunchedEffect(retry) { load() }

    Column(Modifier.fillMaxSize()) {
        ScreenBar(if (frontier) "小伴前沿" else "AI 日报", onBack = back)
        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(Gap)) {
            items(items, key = { it.key }) { n -> NewsRow(n, visual = frontier) { open(if (frontier) Route.FrontierArticle(n.key) else Route.News(n.key)) } }
            item {
                when {
                    error != null -> Text("加载失败，点这里重试", Modifier.fillMaxWidth().clickable { error = null; retry += 1 }.padding(16.dp), color = Danger, style = XType.Body)
                    more -> { LaunchedEffect(items.size) { load() }; Spinner(Modifier.fillMaxWidth().height(48.dp)) }
                    items.isEmpty() -> CenterMessage(if (frontier) "还没有已发布前沿" else "还没有日报", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp))
                }
            }
        }
    }
}
