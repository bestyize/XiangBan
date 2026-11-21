package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
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
import androidx.compose.ui.text.font.FontFamily
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
import xyz.xiangban.xiaoban.common.community.RepoIssue
import xyz.xiangban.xiaoban.common.community.RepoRank
import xyz.xiangban.xiaoban.common.community.shownDescription
import xyz.xiangban.xiaoban.common.community.repoObservationLabel
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.compactCount
import xyz.xiangban.xiaoban.common.community.dayLabel
import xyz.xiangban.xiaoban.common.community.formatGain
import xyz.xiangban.xiaoban.common.community.formatScore
import xyz.xiangban.xiaoban.common.community.formatStars
import xyz.xiangban.xiaoban.common.community.kindLabel
import xyz.xiangban.xiaoban.common.community.rankModels

/** 社区首页栏目。 */
internal enum class HomeSub(val key: String, val label: String) {
    Ideas("ideas", "创意"), News("recommend", "资讯"), OpenSource("open-source", "精选"), Rank("rank", "排行榜"), Nav("nav", "导航");

    companion object {
        fun byKey(key: String) = entries.firstOrNull { it.key == key } ?: Ideas
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

// ───────────── 资讯 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun NewsTab(env: CommunityEnv, open: (Route) -> Unit, openHotRanking: () -> Unit) {
    val loader = rememberLoader(env, "home") { env.forum.home() }
    val state by loader.state.collectAsState()
    val frontierLoader = rememberLoader(env, "frontier") { env.forum.frontier() }
    val frontierState by frontierLoader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = { loader.reload(); frontierLoader.reload() }, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { home ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = CommunityFeedPadding, verticalArrangement = Arrangement.spacedBy(14.dp)) {
                item(key = "frontier") { FrontierSection(env, frontierState, frontierLoader::reload, open) }
                item(key = "daily") {
                    Column(Modifier.fillMaxWidth().communityFeedCard(18.dp).padding(horizontal = 18.dp, vertical = 16.dp)) {
                        Row(Modifier.fillMaxWidth().padding(bottom = 5.dp), verticalAlignment = Alignment.CenterVertically) {
                            Text("AI 日报", Modifier.weight(1f), style = feedText(16), color = feedInk, fontWeight = FontWeight.Bold)
                            if (home.news.total > 0) {
                                Text("${dayText(home.news.date)} 更新", style = feedText(12), color = feedMuted, maxLines = 1)
                                Spacer(Modifier.width(8.dp))
                            }
                            LinkText("往期", { open(Route.Daily) })
                        }
                        if (home.news.items.isEmpty()) Text("今天还没有日报", style = feedText(13), color = feedSecondary)
                        home.news.items.forEachIndexed { index, item ->
                            if (index > 0) Hairline()
                            DailyPreviewRow(env, item, last = index == home.news.items.lastIndex) { open(Route.News(item.key)) }
                        }
                    }
                }
                home.repos?.takeIf { it.ranks.isNotEmpty() }?.let { issue ->
                    item(key = "repositories") {
                        Column(Modifier.fillMaxWidth().communityFeedCard(18.dp).padding(horizontal = 18.dp, vertical = 16.dp)) {
                            Row(Modifier.fillMaxWidth().padding(bottom = 5.dp), verticalAlignment = Alignment.CenterVertically) {
                                Text("开源热榜", Modifier.weight(1f), style = feedText(16), color = feedInk, fontWeight = FontWeight.Bold)
                                LinkText("更多精选", openHotRanking)
                            }
                            issue.ranks.take(3).forEachIndexed { i, r ->
                                if (i > 0) Hairline()
                                RepoMiniRow(r) { openRepo(env, r, open) }
                            }
                        }
                    }
                }
                if (home.banners.isNotEmpty()) item(key = "editorial") { BannerCarousel(home.banners) { openLink(it.link, open) } }
                if (home.sample) item { FootNote("日报、头图和热榜里带「示例」字样的内容为示例数据。") }
            }
        }
    }
}

@Composable
private fun DailyPreviewRow(env: CommunityEnv, item: NewsItem, last: Boolean, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(top = 12.dp, bottom = if (last) 4.dp else 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(14.dp)) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(item.title, style = feedText(15, 22.5), fontWeight = FontWeight(650), color = feedInk,
                maxLines = 2, overflow = TextOverflow.Ellipsis)
            val time = agoText(item.sourcePublishedAt ?: item.publishedAt)
            val meta = listOfNotNull(item.source.takeIf(String::isNotBlank), if (item.timeBasis == "collected") "采集于 $time" else time,
                "示例".takeIf { item.sample }).filter(String::isNotBlank).joinToString(" · ")
            if (meta.isNotBlank()) Text(meta, style = feedText(12), color = feedMuted, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        if (item.coverUrl.isNotBlank()) Cover(env.forum.imageUrl(item.coverUrl), item.key.hashCode().toLong(),
            Modifier.size(84.dp, 56.dp).clip(RoundedCornerShape(14.dp)))
    }
}

/** 社区运营推荐：与网页相同的浅色卡片，标题与署名直接在卡片内。 */
@Composable
private fun BannerCarousel(banners: List<HomeBanner>, onClick: (HomeBanner) -> Unit) {
    val pager = rememberPagerState { banners.size }
    HorizontalPager(pager, Modifier.fillMaxWidth().height(136.dp), pageSpacing = 8.dp) { page ->
        val banner = banners[page]
        Column(Modifier.fillMaxSize().communityFeedCard(18.dp).clickable { onClick(banner) }.padding(16.dp), verticalArrangement = Arrangement.SpaceBetween) {
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

@Composable
private fun RepoMiniRow(r: RepoRank, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 11.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        RepoRankBadge(r.rank)
        Column(Modifier.weight(1f)) {
            Text(r.fullName, style = feedText(13), color = feedInk, fontWeight = FontWeight(650),
                maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (r.shownDescription.isNotBlank()) Text(r.shownDescription, style = feedText(12), color = feedMuted,
                maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        Column(horizontalAlignment = Alignment.End) {
            Text(formatStars(r.starTotal), style = feedText(12.5, 17.5), color = feedInk, fontWeight = FontWeight(650))
            r.starGain?.let { Text(formatGain(it), style = feedText(11), color = Brand) }
        }
    }
}

@Composable
private fun RepoRankBadge(rank: Int) {
    val top = rank in 1..3
    Box(Modifier.size(22.dp).background(if (top) LocalExtraColors.current.brandSoft else feedFill,
        if (top) CircleShape else RoundedCornerShape(6.dp)), contentAlignment = Alignment.Center) {
        Text(rank.toString(), style = feedText(11.5, 16.1), color = if (top) Brand else feedSecondary, fontWeight = FontWeight.Bold)
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
                LazyColumn(Modifier.fillMaxSize(), contentPadding = CommunityFeedPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
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
    Column(Modifier.fillMaxWidth().communityFeedCard().clickable { openRepo(env, r, open) }.padding(horizontal = 12.dp, vertical = 11.dp),
        verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            RepoRankBadge(r.rank)
            RepoTile(r, 28.dp, 8.dp)
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(1.dp)) {
                Text(r.fullName.substringAfter('/'), style = feedText(14, 18.2), color = feedInk, fontWeight = FontWeight(650),
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(r.fullName, style = feedText(10.5, 14.7), color = feedMuted, fontFamily = FontFamily.Monospace,
                    maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        if (r.shownDescription.isNotBlank()) Text(r.shownDescription, style = feedText(12, 17.4), color = feedSecondary,
            maxLines = 2, overflow = TextOverflow.Ellipsis)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            if (r.language.isNotBlank()) FeedTag(r.language)
            Spacer(Modifier.weight(1f))
            r.starGain?.let { Text(formatGain(it), style = feedText(11), color = feedMuted) }
            Text("★ ${formatStars(r.starTotal)}", style = feedText(11), color = feedMuted, fontWeight = FontWeight.SemiBold)
        }
        if (r.suspectedGrowth) Text("增长异常 · 编辑复核后收录", style = feedText(11), color = feedMuted)
        if (r.starGain != null && r.observedDays != null && !r.complete) Text("实际观测 ${r.observedDays} 天", style = feedText(11), color = feedMuted)
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
