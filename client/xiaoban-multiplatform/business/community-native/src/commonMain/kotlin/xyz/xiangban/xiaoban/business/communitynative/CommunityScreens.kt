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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
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

internal enum class CommunitySection(val label: String) { Home("首页"), Discuss("讨论"), Resources("资源") }

/**
 * 社区：顶部是 首页 / 讨论 / 资源 三个栏目，每个栏目下面有自己的二级栏目。
 * 内容可以滚到底部的悬浮导航下面，所以各列表自己在末尾留出空白。
 */
@Composable
fun CommunityScreen(env: CommunityEnv, open: (Route) -> Unit) {
    var section by rememberSaveable { mutableStateOf(CommunitySection.Home.name) }
    var homeSub by rememberSaveable { mutableStateOf(HomeSub.Recommend.key) }
    var discussSub by rememberSaveable { mutableStateOf(DiscussSub.Create.key) }
    var resourceSub by rememberSaveable { mutableStateOf(ResourceSub.Tools.key) }
    var toast by remember { mutableStateOf<String?>(null) }
    val current = CommunitySection.valueOf(section)

    fun switchTo(target: CommunitySection, sub: String) {
        section = target.name
        when (target) {
            CommunitySection.Home -> homeSub = sub
            CommunitySection.Discuss -> discussSub = sub
            CommunitySection.Resources -> resourceSub = sub
        }
    }

    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            CommunityHeader(current, onSelect = { section = it.name }, onSearch = { open(Route.Search) })
            Hairline()
            when (current) {
                CommunitySection.Home -> TabStrip(HomeSub.entries.map { it.key to it.label }, homeSub) { homeSub = it }
                CommunitySection.Discuss -> TabStrip(DiscussSub.entries.map { it.key to it.label }, discussSub) { discussSub = it }
                CommunitySection.Resources -> TabStrip(ResourceSub.entries.map { it.key to it.label }, resourceSub) { resourceSub = it }
            }
            Box(Modifier.weight(1f).fillMaxWidth()) {
                when (current) {
                    CommunitySection.Home -> when (HomeSub.byKey(homeSub)) {
                        HomeSub.Recommend -> RecommendTab(env, open, ::switchTo)
                        HomeSub.Hot -> HotTab(env)
                        HomeSub.Rank -> RankTab(env, open)
                        HomeSub.Track -> TrackTab(env)
                        HomeSub.Nav -> NavTab(env)
                    }
                    CommunitySection.Discuss -> DiscussTab(env, DiscussSub.byKey(discussSub), open) { toast = it }
                    CommunitySection.Resources -> ResourceTab(env, ResourceSub.byKey(resourceSub), open) { toast = it }
                }
            }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
private fun CommunityHeader(current: CommunitySection, onSelect: (CommunitySection) -> Unit, onSearch: () -> Unit) {
    Column(Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.surface).statusBarsPadding()) {
        Row(Modifier.fillMaxWidth().height(52.dp).padding(horizontal = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.width(72.dp)) {
                Box(Modifier.size(28.dp).clip(RoundedCornerShape(8.dp)).background(Brand), contentAlignment = Alignment.Center) {
                    XIcon(XIcons.SparkleSmall, tint = Color.White, size = 18.dp)
                }
            }
            Row(Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(28.dp, Alignment.CenterHorizontally)) {
                CommunitySection.entries.forEach { s ->
                    val selected = s == current
                    Column(Modifier.clickable { onSelect(s) }.height(52.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
                        Text(
                            s.label, fontSize = 16.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                            color = if (selected) Brand else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        Box(Modifier.padding(top = 4.dp).size(width = 18.dp, height = 3.dp).clip(RoundedCornerShape(2.dp)).background(if (selected) Brand else Color.Transparent))
                    }
                }
            }
            Box(Modifier.width(72.dp), contentAlignment = Alignment.CenterEnd) { IconTap(XIcons.Search, "搜索", onSearch) }
        }
    }
}

/** 游戏：小游戏的玩法和试玩还没有接入手机端，先把入口位置占住，并说清楚。 */
@Composable
fun GamesScreen() {
    Column(Modifier.fillMaxSize()) {
        HomeBar("游戏")
        Hairline()
        Box(Modifier.weight(1f).fillMaxWidth().padding(bottom = NavInset), contentAlignment = Alignment.Center) {
            EmptyState(XIcons.Gamepad, "游戏还在路上", "社区里的小游戏很快就能在手机上试玩。现在可以先去社区的“首页 → 游戏”里看看别人做的游戏。")
        }
    }
}
