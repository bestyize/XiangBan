package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
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

/** 白底卡片之间留 4dp，两边留 12dp；底部让出悬浮导航的高度。 */
private val ListPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 4.dp, bottom = 24.dp + NavInset)
private val Gap = 4.dp

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
private fun RepoTile(r: RepoRank, size: Dp, corner: Dp) {
    val glyph = repoGlyph(r.icon)
    val tint = if (ThemeState.dark) hexColor(r.tint, Color(0xFF3A3A3C)).copy(alpha = 0.3f) else hexColor(r.tint, Color(0xFFF2F2F2))
    if (glyph != null) Box(Modifier.size(size).clip(RoundedCornerShape(corner)).background(tint), contentAlignment = Alignment.Center) { XIcon(glyph, size = size * 0.55f) }
    else LetterIcon(r.fullName.substringAfter('/'), hexColor(r.languageColor, DataBlue), tint, size = size)
}

@Composable
private fun LangLine(r: RepoRank) {
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
                                val note = listOf(dayText(home.news.date), "${home.news.total} 条").filter { it.isNotEmpty() }.joinToString(" · ")
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
                items(home.news.items, key = { "n-${it.key}" }) { NewsRow(it) { env.platform?.openUrl(it.url) } }
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
                                        Text("本周开源热榜", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                                    }
                                    Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable { switchTo(CommunitySection.Home, HomeSub.Hot.key) }.padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                                        Text("完整榜单", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                                        XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                                    }
                                }
                                issue.ranks.take(3).forEachIndexed { i, r ->
                                    if (i > 0) Hairline()
                                    RepoMiniRow(r) { env.platform?.openUrl("https://github.com/${r.fullName}") }
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

/** 日报一行：白底圆角卡，左边 96×72 缩略图，右边两行标题，底下来源小圆点和「来源 · 时间」。 */
@Composable
internal fun NewsRow(item: NewsItem, onClick: () -> Unit) {
    val accent = hexColor(item.accent, Color(0xFF9FBFCF))
    Row(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        if (item.coverUrl.isNotEmpty()) Cover(item.coverUrl, item.key.hashCode().toLong() and 0x7fffffff, Modifier.size(width = 96.dp, height = 72.dp))
        else Box(Modifier.size(width = 96.dp, height = 72.dp).clip(RoundedCornerShape(8.dp)).background(Brush.linearGradient(listOf(lerp(accent, Color.Black, 0.35f), accent)))) {
            Canvas(Modifier.fillMaxSize()) { drawCircle(Brush.radialGradient(listOf(Color.White.copy(alpha = 0.5f), Color.Transparent), center = Offset(size.width * 0.5f, size.height * 0.3f), radius = 40.dp.toPx()), radius = 40.dp.toPx(), center = Offset(size.width * 0.5f, size.height * 0.3f)) }
        }
        Column(Modifier.weight(1f).height(72.dp), verticalArrangement = Arrangement.SpaceBetween) {
            Text(item.title, fontSize = 15.sp, fontWeight = FontWeight.Medium, lineHeight = 22.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Dot(accent, 16.dp)
                Text(listOf(item.source, agoText(item.publishedAt)).filter { it.isNotEmpty() }.joinToString(" · "), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
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
            Text(pack.author?.nickname ?: "匿名", Modifier.weight(1f), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            XIcon(XIcons.Heart, tint = LocalExtraColors.current.ink3, size = 12.dp)
            Text(compactCount(pack.applyCount.toInt()), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
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
        SparkBars(r.spark, height = 24.dp)
        Text(formatGain(r.starGain), Modifier.width(52.dp), fontSize = 14.sp, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.End, maxLines = 1)
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
            LoadBox(state, loader::reload) { issue ->
                LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                    item { PeriodCard(issue) { picking = true } }
                    if (issue.ranks.isEmpty()) item { WhiteCard { Text("这一期还没有榜单", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
                    items(issue.ranks, key = { it.fullName }) { r -> RepoCard(env, r) }
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
    WhiteCard(padding = 14.dp) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("第 ${issue.issueNo} 期", Modifier.clip(RoundedCornerShape(10.dp)).background(Brand).padding(horizontal = 10.dp, vertical = 2.dp), color = Color.White, fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                Text(listOf(issue.weekStart, issue.weekEnd).filter { it.isNotEmpty() }.joinToString(" – "), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable(onClick = onPast).padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                Text("往期", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
            }
        }
        Row(Modifier.padding(top = 14.dp).fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.Bottom) {
            Column {
                Text("前 ${issue.ranks.size.coerceAtLeast(1)} 名本周新增 star", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                Text(formatGain(issue.totalGain).removePrefix("+"), Modifier.padding(top = 2.dp), fontSize = 28.sp, fontWeight = FontWeight.Bold)
                if (issue.wowPct != 0.0) Text(if (issue.wowPct > 0) "比上周多 ${formatScore(issue.wowPct)}%" else "比上周少 ${formatScore(-issue.wowPct)}%", Modifier.padding(top = 2.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            if (issue.dailyGain.size == 7) DayBars(issue.dailyGain, listOf("一", "二", "三", "四", "五", "六", "日"))
        }
        Text("按 7 日新增 star 排序，只收 AI 相关项目" + if (issue.sample) " · 示例数据" else "", Modifier.padding(top = 10.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

/** 热榜卡：右上角贴边的名次角标，左边图标块、仓库名、简介、语言和星数，右边小柱、新增、收藏圆钮。 */
@Composable
private fun RepoCard(env: CommunityEnv, r: RepoRank) {
    val (fav, setFav) = rememberPrefFlag(env, "fav_repo:${r.fullName}")
    Box(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable { env.platform?.openUrl("https://github.com/${r.fullName}") }) {
        Box(
            Modifier.align(Alignment.TopEnd).defaultMinSize(minWidth = 36.dp).height(22.dp)
                .clip(RoundedCornerShape(topStart = 0.dp, topEnd = 8.dp, bottomEnd = 0.dp, bottomStart = 8.dp))
                .background(if (r.rank <= 3) Brand else LocalExtraColors.current.fill).padding(horizontal = 8.dp),
            contentAlignment = Alignment.Center,
        ) {
            Text("No.${r.rank}", fontSize = 12.sp, fontWeight = FontWeight.Bold, color = if (r.rank <= 3) Color.White else MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1)
        }
        Row(Modifier.padding(14.dp), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            RepoTile(r, 40.dp, 10.dp)
            Column(Modifier.weight(1f)) {
                Text(r.fullName, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(end = 6.dp))
                if (r.description.isNotEmpty()) Text(r.description, Modifier.padding(top = 4.dp), fontSize = 14.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
                Row(Modifier.padding(top = 6.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    LangLine(r)
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                        XIcon(XIcons.Star, tint = LocalExtraColors.current.ink3, size = 14.dp)
                        Text(formatStars(r.starTotal), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    }
                }
            }
            Column(Modifier.padding(top = 22.dp), horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(6.dp)) {
                SparkBars(r.spark, height = 22.dp)
                Text(formatGain(r.starGain), fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Box(Modifier.size(32.dp).clip(CircleShape).background(if (fav) Brand.copy(alpha = 0.14f) else LocalExtraColors.current.fill).clickable { setFav(!fav) }, contentAlignment = Alignment.Center) {
                    XIcon(XIcons.Bookmark, tint = if (fav) Brand else MaterialTheme.colorScheme.onSurfaceVariant, size = 18.dp, description = if (fav) "取消收藏" else "收藏")
                }
            }
        }
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
                RadioRow(selected = (current ?: list.items.first().key) == ref.key, title = "第 ${ref.issueNo} 期", desc = listOf(ref.weekStart, ref.weekEnd).filter { it.isNotEmpty() }.joinToString(" – ").ifEmpty { null }, onClick = { onPick(if (i == 0) null else ref.key) })
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
    val loader = rememberLoader(env, "tracker") { env.forum.tracker() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { tracker ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = ListPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                tracker.quota?.let { q -> item { QuotaCard(env, q) } }
                item {
                    WhiteCard(padding = 14.dp) {
                        CardTitleRow("模型上新", XIcons.Bolt)
                        if (tracker.launches.isEmpty()) Text("最近没有新模型", Modifier.padding(vertical = 12.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Column(Modifier.padding(top = 10.dp)) {
                            tracker.launches.forEachIndexed { i, l ->
                                Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                    Column(Modifier.width(12.dp).fillMaxHeight(), horizontalAlignment = Alignment.CenterHorizontally) {
                                        Dot(if (i == 0) DataBlue else DataBlueSoft, 10.dp, Modifier.padding(top = 5.dp))
                                        if (i < tracker.launches.lastIndex) Box(Modifier.padding(top = 2.dp).width(2.dp).weight(1f).background(LocalExtraColors.current.hairline))
                                    }
                                    Column(Modifier.weight(1f).padding(bottom = 14.dp)) {
                                        Text(dayLabel(l.publishedAt), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                        Text(l.title, Modifier.padding(top = 2.dp), fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                        if (l.tags.isNotEmpty()) Row(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) { l.tags.forEach { GrayTag(it) } }
                                    }
                                }
                            }
                        }
                    }
                }
                item {
                    WhiteCard(padding = 14.dp) {
                        CardTitleRow("限时免费", XIcons.Clock)
                        if (tracker.promos.isEmpty()) Text("现在没有限时免费的活动", Modifier.padding(vertical = 12.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        tracker.promos.forEachIndexed { i, p ->
                            if (i > 0) Hairline(Modifier.padding(top = 10.dp))
                            Column(Modifier.padding(top = 10.dp)) {
                                Row(verticalAlignment = Alignment.CenterVertically) {
                                    Text(p.text, Modifier.weight(1f), fontSize = 14.sp, fontWeight = FontWeight.Medium)
                                    Text("剩 ${p.daysLeft} 天", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                HBar(p.pct / 100f, DataBlue, Modifier.padding(top = 8.dp), height = 4.dp, track = LocalExtraColors.current.fill)
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
                        Column(Modifier.padding(start = 2.dp, end = 2.dp, top = 10.dp)) { CardTitleRow(group.name, XIcons.Globe) }
                    }
                    items(group.items, key = { "l-${it.key}" }) { link ->
                        val logo = BrandLogos.find(link.name) ?: BrandLogos.find(link.vendor)
                        Row(
                            Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable { env.platform?.openUrl(link.url) }.padding(horizontal = 14.dp, vertical = 10.dp),
                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            BrandTile(logo, link.name, size = 40.dp, corner = 10.dp, dashed = logo == null)
                            Column(Modifier.weight(1f)) {
                                Text(link.name, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                Text(link.vendor, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            }
                            XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
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
    LaunchedEffect(retry) { load() }

    Column(Modifier.fillMaxSize()) {
        ScreenBar("AI 日报", onBack = back)
        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(start = 12.dp, end = 12.dp, top = 8.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(Gap)) {
            items(items, key = { it.key }) { n -> NewsRow(n) { env.platform?.openUrl(n.url) } }
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
