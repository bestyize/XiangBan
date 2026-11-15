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
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.collectAsState
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
import xyz.xiangban.xiaoban.common.community.ForumCategories
import xyz.xiangban.xiaoban.common.community.ForumSection
import xyz.xiangban.xiaoban.common.community.Load

internal enum class CommunitySection(val label: String) { Home("首页"), Ideas("创意"), Discuss("讨论"), Resources("资源") }

/**
 * 社区：顶部是 首页 / 创意 / 讨论 / 资源 四个栏目，每个栏目下面有自己的二级栏目。
 * 内容可以滚到底部的悬浮导航下面，所以各列表自己在末尾留出空白。
 */
@Composable
fun CommunityScreen(env: CommunityEnv, open: (Route) -> Unit, onPostDestination: (Route.NewPost) -> Unit = {}) {
    var section by rememberSaveable { mutableStateOf(CommunitySection.Home.name) }
    var homeSub by rememberSaveable { mutableStateOf(HomeSub.Recommend.key) }
    // 旧版会把热榜存为独立的首页标签。
    val legacyHotSelected = homeSub == "hot"
    var hotRankingOpenCount by rememberSaveable { mutableIntStateOf(if (legacyHotSelected) 1 else 0) }
    var discussSub by rememberSaveable { mutableStateOf(AllDiscussions) }
    var resourceSub by rememberSaveable { mutableStateOf(SoftwareResourceTab) }
    val current = CommunitySection.valueOf(section)
    val currentHome = if (legacyHotSelected) HomeSub.OpenSource else HomeSub.byKey(homeSub)
    val loader = rememberLoader(env, "forum-categories") { ForumCategories(env.forum.categories()) }
    val categoryState by loader.state.collectAsState()
    val catalog = (categoryState as? Load.Ready)?.value
    val resourceBoards = catalog?.boardsIn(ForumSection.Resources).orEmpty()
    val resourceTab = resourceBoards.firstOrNull { it.slug == resourceSub }?.slug ?: SoftwareResourceTab
    SideEffect {
        onPostDestination(when (current) {
            CommunitySection.Home -> Route.NewPost()
            CommunitySection.Ideas -> Route.NewPost(ForumSection.Ideas)
            CommunitySection.Discuss -> Route.NewPost(ForumSection.Discuss, catalog?.discussionTab(discussSub)?.category?.slug)
            CommunitySection.Resources -> Route.NewPost(ForumSection.Resources, catalog?.find(resourceTab)?.slug)
        })
    }

    fun openHotRanking() {
        hotRankingOpenCount++
        homeSub = HomeSub.OpenSource.key
    }

    LaunchedEffect(Unit) {
        discussSub = when (discussSub) {
            "create" -> {
                if (current == CommunitySection.Discuss) section = CommunitySection.Ideas.name
                AllDiscussions
            }
            "casual-media" -> {
                if (current == CommunitySection.Discuss) section = CommunitySection.Resources.name
                resourceSub = "resource-movies"
                AllDiscussions
            }
            "design", "3d" -> "tag:$discussSub"
            else -> discussSub
        }
        resourceSub = when (resourceSub) {
            "media" -> "resource-movies"
            "help" -> "resource-help"
            "open", "resource-open" -> SoftwareResourceTab
            else -> resourceSub
        }
    }

    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        Column(Modifier.fillMaxSize()) {
            CommunityHeader(current, onSelect = { section = it.name }, onSearch = { open(Route.Search) })
            when (current) {
                CommunitySection.Home -> TabStrip(HomeSub.entries.map { it.key to it.label }, currentHome.key, centered = true) { homeSub = it }
                CommunitySection.Ideas, CommunitySection.Discuss -> Unit
                CommunitySection.Resources -> TabStrip(
                    listOf(SoftwareResourceTab to "开源软件") + resourceBoards.map { it.slug to it.name },
                    resourceTab, centered = true,
                ) { resourceSub = it }
            }
            Box(Modifier.weight(1f).fillMaxWidth()) {
                HomeSub.entries.forEach { page ->
                    key(page.key) {
                        RetainedTabPage(selected = current == CommunitySection.Home && page == currentHome) {
                            when (page) {
                                HomeSub.Recommend -> RecommendTab(env, open, ::openHotRanking)
                                HomeSub.OpenSource -> OpenSourceTab(env, open, hotRankingOpenCount)
                                HomeSub.Rank -> RankTab(env, open)
                                HomeSub.Nav -> NavTab(env)
                            }
                        }
                    }
                }
                when (current) {
                    CommunitySection.Home -> Unit
                    CommunitySection.Ideas -> IdeasFeed(env, categoryState, loader::reload, open)
                    CommunitySection.Discuss -> LoadBox(categoryState, loader::reload) {
                        DiscussTab(env, it, discussSub, { discussSub = it }, open)
                    }
                    CommunitySection.Resources -> Column(Modifier.fillMaxSize()) {
                        val failure = categoryState as? Load.Failed
                        if (failure != null) Banner(failure.message, warn = true, actionText = "重试", onAction = loader::reload)
                        ResourceTab(env, resourceTab, open)
                    }
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
