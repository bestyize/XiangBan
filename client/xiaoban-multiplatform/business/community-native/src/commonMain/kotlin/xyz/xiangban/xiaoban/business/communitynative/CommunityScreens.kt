package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

internal enum class CommunitySection(val label: String) { Home("首页"), Ideas("创意"), Discuss("讨论"), Resources("资源") }

/**
 * 社区：顶部是 首页 / 创意 / 讨论 / 资源 四个栏目，每个栏目下面有自己的二级栏目。
 * 内容可以滚到底部的悬浮导航下面，所以各列表自己在末尾留出空白。
 */
@Composable
fun CommunityScreen(env: CommunityEnv, open: (Route) -> Unit) {
    var section by rememberSaveable { mutableStateOf(CommunitySection.Home.name) }
    var homeSub by rememberSaveable { mutableStateOf(HomeSub.Recommend.key) }
    // 旧版会把热榜存为独立的首页标签。
    val legacyHotSelected = homeSub == "hot"
    var hotRankingOpenCount by rememberSaveable { mutableIntStateOf(if (legacyHotSelected) 1 else 0) }
    var discussSub by rememberSaveable { mutableStateOf(DiscussSub.All.key) }
    var resourceSub by rememberSaveable { mutableStateOf(ResourceSub.Software.key) }
    val current = CommunitySection.valueOf(section)
    val currentHome = if (legacyHotSelected) HomeSub.OpenSource else HomeSub.byKey(homeSub)

    fun openHotRanking() {
        hotRankingOpenCount++
        homeSub = HomeSub.OpenSource.key
    }

    LaunchedEffect(Unit) {
        if (discussSub == "create") {
            if (current == CommunitySection.Discuss) section = CommunitySection.Ideas.name
            discussSub = DiscussSub.All.key
        }
        if (discussSub == "casual-media") {
            if (current == CommunitySection.Discuss) section = CommunitySection.Resources.name
            resourceSub = ResourceSub.Media.key
            discussSub = DiscussSub.All.key
        }
        resourceSub = ResourceSub.byKey(resourceSub).key
    }

    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        Column(Modifier.fillMaxSize()) {
            CommunityHeader(current, onSelect = { section = it.name }, onSearch = { open(Route.Search) })
            when (current) {
                CommunitySection.Home -> TabStrip(HomeSub.entries.map { it.key to it.label }, currentHome.key, centered = true) { homeSub = it }
                CommunitySection.Ideas, CommunitySection.Discuss -> Unit
                CommunitySection.Resources -> TabStrip(ResourceSub.entries.map { it.key to it.label }, ResourceSub.byKey(resourceSub).key, centered = true) { resourceSub = it }
            }
            Box(Modifier.weight(1f).fillMaxWidth()) {
                HomeSub.entries.forEach { page ->
                    key(page.key) {
                        RetainedTabPage(selected = current == CommunitySection.Home && page == currentHome) {
                            when (page) {
                                HomeSub.Recommend -> RecommendTab(env, open, ::openHotRanking)
                                HomeSub.OpenSource -> OpenSourceTab(env, open, hotRankingOpenCount)
                                HomeSub.Rank -> RankTab(env, open)
                                HomeSub.Track -> TrackTab(env)
                                HomeSub.Nav -> NavTab(env)
                            }
                        }
                    }
                }
                when (current) {
                    CommunitySection.Home -> Unit
                    CommunitySection.Ideas -> IdeasTab(env, open)
                    CommunitySection.Discuss -> DiscussTab(env, DiscussSub.byKey(discussSub), { discussSub = it.key }, open)
                    CommunitySection.Resources -> ResourceTab(env, ResourceSub.byKey(resourceSub), open)
                }
            }
        }
    }
}

@Composable
private fun CommunityHeader(current: CommunitySection, onSelect: (CommunitySection) -> Unit, onSearch: () -> Unit) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(40.dp).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.width(44.dp)) {
                Box(Modifier.size(28.dp).clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.brandSoft), contentAlignment = Alignment.Center) {
                    XIcon(XIcons.SparkleSmall, tint = Brand, size = 18.dp)
                }
            }
            Row(Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(14.dp, Alignment.CenterHorizontally)) {
                CommunitySection.entries.forEach { s ->
                    val selected = s == current
                    Column(Modifier.clickable { onSelect(s) }.height(40.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
                        Text(
                            s.label, style = XType.Section, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                            color = if (selected) Brand else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Box(Modifier.padding(top = 2.dp).size(width = 18.dp, height = 3.dp).clip(RoundedCornerShape(2.dp)).background(if (selected) Brand else Color.Transparent))
                    }
                }
            }
            Box(Modifier.width(44.dp), contentAlignment = Alignment.CenterEnd) { IconTap(XIcons.Search, "搜索", onSearch) }
        }
    }
}
