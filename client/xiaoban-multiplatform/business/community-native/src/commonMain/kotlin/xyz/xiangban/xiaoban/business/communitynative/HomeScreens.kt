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

/** 社区首页的五个二级栏目。 */
internal enum class HomeSub(val key: String, val label: String) {
    Recommend("recommend", "推荐"), Hot("hot", "开源热榜"), Rank("rank", "排行榜"), Track("track", "追踪"), Nav("nav", "导航");

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
        Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
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
        Text(r.language, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

// ───────────── 推荐 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun RecommendTab(env: CommunityEnv, open: (Route) -> Unit, switchTo: (CommunitySection, String) -> Unit) {
    val loader = rememberLoader(env, "home") { env.forum.home() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { home ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                if (home.banners.isNotEmpty()) item { BannerCarousel(home.banners) { openLink(it.link, env, open) } }
                item {
                    Column(Modifier.padding(start = 2.dp, end = 2.dp, top = 10.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                XIcon(XIcons.Message, size = 18.dp)
                                Text("AI 日报", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                val note = listOf(dayText(home.news.date), "更新").filter { it.isNotEmpty() }.joinToString(" · ")
                                if (home.news.total > 0) Text(note, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            }
                            Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable { open(Route.Daily) }.padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                                Text("往期", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                                XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                            }
                        }
                    }
                }
                if (home.news.items.isEmpty()) item { WhiteCard { Text("今天还没有日报", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
                items(home.news.items, key = { "n-${it.key}" }) { NewsRow(it) { open(Route.News(it.key)) } }
                if (home.featured.isNotEmpty()) item {
                    WhiteCard(padding = 0.dp) {
                        Row(Modifier.padding(start = 14.dp, end = 14.dp, top = 14.dp), verticalAlignment = Alignment.CenterVertically) {
                            Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                XIcon(XIcons.Sparkle, size = 18.dp)
                                Text("精选创作", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                            }
                            Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable { switchTo(CommunitySection.Discuss, DiscussSub.Create.key) }.padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                                Text("更多", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                                XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                            }
                        }
                        LazyRow(contentPadding = PaddingValues(start = 14.dp, end = 14.dp, top = 12.dp, bottom = 14.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            items(home.featured, key = { it.id }) { pack -> PickCard(pack) { open(Route.Pack(pack.id)) } }
                        }
                    }
                }
                home.repos?.takeIf { it.ranks.isNotEmpty() }?.let { issue ->
                    item {
                        WhiteCard(padding = 0.dp) {
                            Column(Modifier.padding(start = 14.dp, end = 14.dp, top = 14.dp, bottom = 4.dp)) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                        XIcon(XIcons.Star, size = 18.dp)
                                        Text(if (issue.isBaseline) "开源项目初始快照" else if (issue.isTrending) "GitHub 开源热榜" else if (issue.source == "GitHub") "开源观测榜" else "本周开源热榜", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                    }
                                    Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable { switchTo(CommunitySection.Home, HomeSub.Hot.key) }.padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                                        Text("完整榜单", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
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

/** 头图：深蓝渐变，两团柔光和右上角的装饰柱，左下是标签、标题、署名，右下是分页点。 */
@Composable
private fun BannerCarousel(banners: List<HomeBanner>, onClick: (HomeBanner) -> Unit) {
    val pager = rememberPagerState { banners.size }
    Box(Modifier.fillMaxWidth().height(176.dp).clip(RoundedCornerShape(8.dp))) {
        HorizontalPager(pager, Modifier.fillMaxSize()) { page ->
            val b = banners[page]
            Box(Modifier.fillMaxSize().background(Brush.linearGradient(listOf(Color(0xFF1B2A41), Color(0xFF3E5C8A)))).clickable { onClick(b) }) {
                Canvas(Modifier.fillMaxSize()) {
                    drawCircle(Brush.radialGradient(listOf(Color(0xFFF4A6A0).copy(alpha = 0.7f), Color.Transparent), center = Offset(size.width + 60.dp.toPx(), 50.dp.toPx()), radius = 100.dp.toPx()), radius = 100.dp.toPx(), center = Offset(size.width + 60.dp.toPx(), 50.dp.toPx()))
                    drawCircle(Brush.radialGradient(listOf(Color(0xFF9FBFCF).copy(alpha = 0.55f), Color.Transparent), center = Offset(60.dp.toPx(), size.height + 30.dp.toPx()), radius = 90.dp.toPx()), radius = 90.dp.toPx(), center = Offset(60.dp.toPx(), size.height + 30.dp.toPx()))
                }
                Row(Modifier.align(Alignment.TopEnd).padding(end = 28.dp, top = 26.dp), horizontalArrangement = Arrangement.spacedBy(5.dp), verticalAlignment = Alignment.Bottom) {
                    listOf(18, 30, 24, 44, 36).forEach { h -> Box(Modifier.width(8.dp).height(h.dp).clip(RoundedCornerShape(2.dp)).background(Color.White.copy(alpha = 0.55f))) }
                }
                Column(Modifier.align(Alignment.BottomStart).padding(start = 16.dp, end = 16.dp, bottom = 14.dp)) {
                    if (b.tag.isNotEmpty()) Row(Modifier.clip(RoundedCornerShape(10.dp)).background(Color.White.copy(alpha = 0.2f)).padding(horizontal = 8.dp, vertical = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                        XIcon(XIcons.Sparkle, tint = Color.White, size = 12.dp)
                        Text(b.tag, fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = Color.White)
                    }
                    Text(b.title, Modifier.padding(top = 8.dp), fontSize = 18.sp, fontWeight = FontWeight.SemiBold, lineHeight = 25.sp, color = Color.White, maxLines = 2, overflow = TextOverflow.Ellipsis)
                    Row(Modifier.padding(top = 4.dp).fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Text(b.byline, Modifier.weight(1f), fontSize = 12.sp, color = Color.White.copy(alpha = 0.85f), maxLines = 1)
                        if (banners.size > 1) Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
                            repeat(banners.size) { i ->
                                val on = i == pager.currentPage
                                Box(Modifier.size(width = if (on) 14.dp else 4.dp, height = 4.dp).clip(RoundedCornerShape(2.dp)).background(Color.White.copy(alpha = if (on) 1f else 0.5f)))
                            }
                        }
                    }
                }
            }
        }
    }
}

/** 日报图文封面卡。没有封面时使用主题色生成占位。 */
@Composable
internal fun NewsRow(item: NewsItem, onClick: () -> Unit) {
    val accent = hexColor(item.accent, Color(0xFF9FBFCF))
    Column(Modifier.fillMaxWidth().padding(vertical = 4.dp).clip(RoundedCornerShape(16.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick)) {
        if (item.coverUrl.isNotEmpty()) Cover(item.coverUrl, item.key.hashCode().toLong() and 0x7fffffff, Modifier.fillMaxWidth().aspectRatio(16f / 9f))
        else Box(Modifier.fillMaxWidth().aspectRatio(16f / 9f).background(Brush.linearGradient(listOf(lerp(accent, Color.Black, 0.48f), accent))), contentAlignment = Alignment.BottomStart) {
            Text("AI  ·  DAILY", Modifier.padding(18.dp), color = Color.White.copy(alpha = .9f), fontWeight = FontWeight.Bold, letterSpacing = 2.sp, fontSize = 13.sp)
        }
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(item.title, fontSize = 18.sp, fontWeight = FontWeight.Bold, lineHeight = 26.sp, maxLines = 3, overflow = TextOverflow.Ellipsis)
            if (item.summary.isNotBlank()) Text(item.summary, fontSize = 13.sp, lineHeight = 20.sp, color = LocalExtraColors.current.ink3, maxLines = 2, overflow = TextOverflow.Ellipsis)
            val time = agoText(item.sourcePublishedAt ?: item.publishedAt)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                val logo = BrandLogos.find(item.source)
                if (logo != null) XIcon(logo.icon, tint = logo.color, size = 14.dp)
                Text(listOf(item.source, if (item.timeBasis == "collected") "采集于 $time" else time).filter { it.isNotEmpty() }.joinToString(" · "),
                    fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

/** 精选创作里的一张：宽 132、封面高 104，左上角黑底类型标签，下面是标题和「头像 作者 ♡ 数」。 */
@Composable
private fun PickCard(pack: Pack, onClick: () -> Unit) {
    Column(Modifier.width(132.dp).clickable(onClick = onClick)) {
        Box {
            Cover(pack.coverUrl, pack.id, Modifier.fillMaxWidth().height(104.dp))
            Text(kindLabel(pack.kind), Modifier.padding(6.dp).clip(RoundedCornerShape(6.dp)).background(Color.Black.copy(alpha = 0.45f)).padding(horizontal = 6.dp, vertical = 1.dp), fontSize = 12.sp, color = Color.White)
        }
        Text(pack.title, Modifier.padding(top = 6.dp), fontSize = 14.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
        Row(Modifier.padding(top = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            Avatar(pack.author?.nickname ?: "?", 14.dp)
            Text(pack.author?.nickname ?: "社区作品", Modifier.weight(1f), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (pack.applyCount > 0) Text("${compactCount(pack.applyCount.toInt())} 次应用", fontSize = 11.sp, color = LocalExtraColors.current.ink3)
        }
    }
}

/** 推荐页热榜前三：名次、36dp 图标块、仓库名和语言、7 根小柱、本周新增。 */
@Composable
private fun RepoMiniRow(r: RepoRank, onClick: () -> Unit) {
    Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Text("${r.rank}", Modifier.width(16.dp), fontSize = 16.sp, fontWeight = FontWeight.Bold)
        RepoTile(r, 36.dp, 8.dp)
        Column(Modifier.weight(1f)) {
            Text(r.fullName, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            LangLine(r)
        }
        if (r.starGain != null && r.spark.size >= 2) SparkBars(r.spark, height = 24.dp)
        Text(r.starGain?.let(::formatGain) ?: "${formatStars(r.starTotal)}★", Modifier.width(52.dp), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.End, maxLines = 1)
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
                    if (issue.ranks.isEmpty()) item { WhiteCard { Text("这一期还没有榜单", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
                    items(issue.ranks, key = { it.fullName }) { r -> RepoCard(env, r, open) }
                }
            }
        }
        // 弹层要盖住底部悬浮导航，所以放在独立的全屏窗口里，而不是页面内容里。
        if (picking) androidx.compose.ui.window.Dialog(onDismissRequest = { picking = false }, properties = androidx.compose.ui.window.DialogProperties(usePlatformDefaultWidth = false, decorFitsSystemWindows = false)) {
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
            Modifier.padding(top = 8.dp), fontSize = 13.sp, lineHeight = 20.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth().clickable { details = !details }.padding(top = 10.dp, bottom = 2.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(issue.source.ifBlank { "数据来源" } + " · " + if (details) "收起数据说明" else "口径与更新时间", Modifier.weight(1f), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
            XIcon(if (details) XIcons.ChevronUp else XIcons.ChevronDown, size = 14.dp, tint = LocalExtraColors.current.ink3)
        }
        if (details) {
            repoObservationLabel(issue.asOf)?.let { Text("整理于 $it", Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3) }
            Text((if (issue.isBaseline) "当前为首次观测，至少两次快照后计算净增量。"
                else if (issue.isTrending) "取自 GitHub Trending 周榜，按 GitHub 页面顺序排列；新增 Star 为 GitHub 公布的近 7 天数据。"
                else if (issue.complete) "已取得连续 7 日边界快照。" else "观测不足 7 日或存在缺采，未补造缺失数据。") + if (issue.sample) " · 示例数据" else "",
                Modifier.padding(top = 4.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
            if (issue.dailyGain.size == 7 && issue.source != "GitHub") DayBars(issue.dailyGain, listOf("一", "二", "三", "四", "五", "六", "日"))
        }
    }
}

/** 热榜卡：右上角贴边的名次角标，左边图标块、仓库名、简介、语言和星数，右边小柱、新增、收藏圆钮。 */
@Composable
private fun RepoCard(env: CommunityEnv, r: RepoRank, open: (Route) -> Unit) {
    val (fav, setFav) = rememberPrefFlag(env, "fav_repo:${r.fullName}")
    WhiteCard(Modifier.clickable { openRepo(env, r, open) }, padding = 14.dp) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(r.rank.toString().padStart(2, '0'), fontSize = 13.sp, fontWeight = FontWeight.Medium, color = if (r.rank <= 3) Brand else LocalExtraColors.current.ink3)
            RepoTile(r, 32.dp, 8.dp)
            Text(r.fullName, Modifier.weight(1f), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
            IconTap(XIcons.Bookmark, if (fav) "取消收藏 ${r.fullName}" else "收藏 ${r.fullName}", { setFav(!fav) }, size = 19.dp, tint = if (fav) Brand else LocalExtraColors.current.ink3)
        }
        if (r.shownDescription.isNotBlank()) Text(r.shownDescription, Modifier.padding(top = 8.dp), fontSize = 13.sp, lineHeight = 20.sp, maxLines = 2, overflow = TextOverflow.Ellipsis, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth().padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            LangLine(r)
            Text("☆ ${formatStars(r.starTotal)}", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            Spacer(Modifier.weight(1f))
            r.starGain?.let { Text("${formatGain(it)} Star", fontSize = 12.sp, fontWeight = FontWeight.Medium) }
            if (r.starGain != null && r.spark.size >= 2) SparkBars(r.spark, height = 20.dp)
        }
        if (r.suspectedGrowth) Text("增长异常 · 编辑复核后收录", Modifier.padding(top = 6.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
        if (r.starGain != null && r.observedDays != null && !r.complete) Text("实际观测 ${r.observedDays} 天", Modifier.padding(top = 6.dp), fontSize = 11.sp, color = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun IssuePicker(env: CommunityEnv, current: String?, onPick: (String?) -> Unit, onDismiss: () -> Unit) {
    val loader = rememberLoader(env, "repo-issues") { env.forum.repoIssues() }
    val state by loader.state.collectAsState()
    BottomSheet(onDismiss) {
        Text("往期热榜", fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
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
                        if (tracker.launches.isEmpty()) Text("暂无已核实的发布记录", Modifier.padding(vertical = 12.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Column(Modifier.padding(top = 10.dp)) {
                            tracker.launches.forEachIndexed { i, l ->
                                val source = trackerOfficialSource(l.sourceUrl)
                                Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                    Column(Modifier.width(12.dp).fillMaxHeight(), horizontalAlignment = Alignment.CenterHorizontally) {
                                        Dot(if (i == 0) DataBlue else DataBlueSoft, 10.dp, Modifier.padding(top = 5.dp))
                                        if (i < tracker.launches.lastIndex) Box(Modifier.padding(top = 2.dp).width(2.dp).weight(1f).background(LocalExtraColors.current.hairline))
                                    }
                                    Column(Modifier.weight(1f).padding(bottom = 14.dp)) {
                                        Text(l.publishedAt.take(10).ifBlank { "日期未提供" }, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                        Text(l.title, Modifier.padding(top = 2.dp), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                        if (l.summary.isNotBlank()) DisclosureDescription("发布说明", l.summary)
                                        if (source != null) Row(Modifier.clickable { env.platform?.openUrl(source) }.padding(vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                            Text("官方原文", fontSize = 12.sp, color = LocalExtraColors.current.brandInk)
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
                        if (tracker.promos.isEmpty()) Text("暂无已核实的优惠与试用", Modifier.padding(vertical = 12.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        tracker.promos.forEachIndexed { i, p ->
                            if (i > 0) Hairline(Modifier.padding(top = 10.dp))
                            Column(Modifier.padding(top = 10.dp)) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text(p.text, Modifier.weight(1f), fontSize = 14.sp, fontWeight = FontWeight.Medium)
                                    if (!p.endsAt.isNullOrBlank()) p.daysLeft?.let { Text("剩 $it 天", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                                }
                                if (p.endsAt.isNullOrBlank()) Text("长期提供，以官方条件为准", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                else p.pct?.let { HBar(it / 100f, DataBlue, Modifier.padding(top = 8.dp), height = 4.dp, track = LocalExtraColors.current.fill) }
                                if (p.eligibility.isNotBlank()) Text(p.eligibility, Modifier.padding(top = 6.dp), fontSize = 12.sp, lineHeight = 19.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
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
                Text("Codex 额度重置", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
            }
            if (q.updatedAt.isNotEmpty()) Text("${agoText(q.updatedAt)}更新", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
        }
        Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            Box(contentAlignment = Alignment.Center) {
                ProgressRing(q.progressPct, DataBlue, size = 96.dp, stroke = 8.dp)
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("${q.progressPct}%", fontSize = 20.sp, fontWeight = FontWeight.Bold)
                    Text("本轮已过", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                }
            }
            Column(Modifier.weight(1f)) {
                Text("距离下次重置", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                Text(countdownText(q.nextResetAt), Modifier.padding(top = 2.dp), fontSize = 24.sp, fontWeight = FontWeight.Bold)
                Text("预计${clockText(q.nextResetAt)}", Modifier.padding(top = 2.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                val at = parseIsoMillis(q.nextResetAt)
                if (env.reminders != null && at != null) {
                    if (remind) Row(
                        Modifier.padding(top = 10.dp).height(34.dp).clip(RoundedCornerShape(17.dp)).background(LocalExtraColors.current.fill).clickable { setRemind(false) }.padding(horizontal = 14.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        XIcon(XIcons.Check, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 16.dp)
                        Text("已开启提醒", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
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
                        Text("提醒我", fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = Color.White)
                    }
                }
            }
        }
        note?.let { Text(it, Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3) }
        if (q.recentIntervals.isNotEmpty()) {
            Hairline(Modifier.padding(top = 14.dp))
            Text("最近 ${q.recentIntervals.size} 次重置间隔（小时）", Modifier.padding(top = 12.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            IntervalBars(q.recentIntervals, Modifier.padding(top = 8.dp))
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
        LoadBox(state, loader::reload) { groups ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                if (groups.groups.isEmpty()) item { CenterMessage("还没有导航", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                groups.groups.forEach { group ->
                    item(key = "g-${group.name}") {
                        WhiteCard(padding = 14.dp) {
                            SectionHeader(group.name, note = "${group.items.size} 个官方入口")
                            group.items.chunked(2).forEach { row ->
                                Row(Modifier.fillMaxWidth().padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                    row.forEach { link ->
                                        Row(Modifier.weight(1f).clip(RoundedCornerShape(10.dp)).clickable { env.platform?.openUrl(link.url) }.padding(vertical = 8.dp),
                                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                            BrandTile(BrandLogos.find(link.name) ?: BrandLogos.find(link.vendor), link.name, size = 34.dp, corner = 9.dp)
                                            Column(Modifier.weight(1f)) {
                                                Text(link.name, fontSize = 14.sp, fontWeight = FontWeight.Medium, maxLines = 2, overflow = TextOverflow.Ellipsis)
                                            }
                                        }
                                    }
                                    if (row.size == 1) Spacer(Modifier.weight(1f))
                                }
                            }
                        }
                    }
                }
                item { Row(Modifier.padding(horizontal = 6.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    XIcon(XIcons.Info, tint = LocalExtraColors.current.ink3, size = 14.dp)
                    Text("只放各家官方入口，点开会离开小伴", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                } }
            }
        }
    }
}

// ───────────── 往期日报 ─────────────

@Composable
fun DailyScreen(env: CommunityEnv, back: () -> Unit, open: (Route) -> Unit) {
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
    LaunchedEffect(retry) { load() }

    Column(Modifier.fillMaxSize()) {
        ScreenBar("AI 日报", onBack = back)
        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(Gap)) {
            items(items, key = { it.key }) { n -> NewsRow(n) { open(Route.News(n.key)) } }
            item {
                when {
                    error != null -> Text("加载失败，点这里重试", Modifier.fillMaxWidth().clickable { error = null; retry += 1 }.padding(16.dp), color = Danger, fontSize = 14.sp)
                    more -> { LaunchedEffect(items.size) { load() }; Spinner(Modifier.fillMaxWidth().height(48.dp)) }
                    items.isEmpty() -> CenterMessage("还没有日报", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp))
                }
            }
        }
    }
}
