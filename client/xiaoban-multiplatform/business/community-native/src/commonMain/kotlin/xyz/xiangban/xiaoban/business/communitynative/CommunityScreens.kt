package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.ForumCategories
import xyz.xiangban.xiaoban.common.community.ForumSection
import xyz.xiangban.xiaoban.common.community.Load

private enum class CommunitySection(val label: String) { Home("首页"), Discuss("讨论"), Resources("资源") }

/**
 * 社区：顶部是 首页 / 讨论 / 资源，首页默认展示创意。
 * 内容可以滚到底部的悬浮导航下面，所以各列表自己在末尾留出空白。
 */
@Composable
fun CommunityScreen(env: CommunityEnv, open: (Route) -> Unit, onPostDestination: (Route.NewPost) -> Unit = {}) {
    var section by rememberSaveable { mutableStateOf(CommunitySection.Home.name) }
    var homeSub by rememberSaveable { mutableStateOf(HomeSub.Ideas.key) }
    // 旧版会把热榜存为独立的首页标签。
    val legacyHotSelected = homeSub == "hot"
    var hotRankingOpenCount by rememberSaveable { mutableIntStateOf(if (legacyHotSelected) 1 else 0) }
    var discussSub by rememberSaveable { mutableStateOf(AllDiscussions) }
    var resourceSub by rememberSaveable { mutableStateOf(SoftwareResourceTab) }
    val current = CommunitySection.entries.firstOrNull { it.name == section } ?: CommunitySection.Home
    val currentHome = when {
        section == "Ideas" -> HomeSub.Ideas
        legacyHotSelected -> HomeSub.OpenSource
        else -> HomeSub.byKey(homeSub)
    }
    val loader = rememberLoader(env, "forum-categories") { ForumCategories(env.forum.categories()) }
    val categoryState by loader.state.collectAsState()
    val catalog = (categoryState as? Load.Ready)?.value
    val session by env.session.state.collectAsState()
    val notificationRevision by env.social.notificationRevision.collectAsState()
    val unread = if (session.loggedIn) {
        val notificationLoader = rememberLoader(env, listOf("community-notifications", session.accountId, session.generation, notificationRevision)) {
            env.forum.unreadCount()
        }
        val state by notificationLoader.state.collectAsState()
        (state as? Load.Ready)?.value ?: 0
    } else 0
    val resourceBoards = catalog?.boardsIn(ForumSection.Resources).orEmpty()
    val resourceTab = resourceBoards.firstOrNull { it.slug == resourceSub }?.slug ?: SoftwareResourceTab
    SideEffect {
        onPostDestination(when (current) {
            CommunitySection.Home -> if (currentHome == HomeSub.Ideas) Route.NewPost(ForumSection.Ideas) else Route.NewPost()
            CommunitySection.Discuss -> Route.NewPost(ForumSection.Discuss, catalog?.discussionTab(discussSub)?.category?.slug)
            CommunitySection.Resources -> Route.NewPost(ForumSection.Resources, catalog?.find(resourceTab)?.slug)
        })
    }

    fun openHotRanking() {
        hotRankingOpenCount++
        homeSub = HomeSub.OpenSource.key
    }

    LaunchedEffect(Unit) {
        if (section == "Ideas") {
            section = CommunitySection.Home.name
            homeSub = HomeSub.Ideas.key
        }
        discussSub = when (discussSub) {
            "create" -> {
                if (current == CommunitySection.Discuss) {
                    section = CommunitySection.Home.name
                    homeSub = HomeSub.Ideas.key
                }
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

    val background = if (current == CommunitySection.Home && currentHome == HomeSub.Rank) MaterialTheme.colorScheme.background
        else if (ThemeState.dark) Color(0xFF0F1219) else Color(0xFFF2F5FA)
    Box(Modifier.fillMaxSize().background(background)) {
        Column(Modifier.fillMaxSize()) {
            CommunityHeader(
                current = current,
                unread = unread,
                onSelect = { section = it.name },
                onBrand = { section = CommunitySection.Home.name; homeSub = HomeSub.Ideas.key },
                onSearch = { open(Route.Search) },
                onNotifications = { open(if (session.loggedIn) Route.Notifications else Route.Login) },
            )
            when (current) {
                CommunitySection.Home -> {
                    CommunityTabRow(HomeSub.entries.map { it.key to it.label }, currentHome.key) { homeSub = it }
                    Hairline()
                }
                CommunitySection.Discuss -> Unit
                CommunitySection.Resources -> {
                    CommunityTabRow(listOf(SoftwareResourceTab to "开源软件") + resourceBoards.map { it.slug to it.name }, resourceTab) { resourceSub = it }
                    Hairline()
                }
            }
            Box(Modifier.weight(1f).fillMaxWidth()) {
                HomeSub.entries.forEach { page ->
                    key(page.key) {
                        RetainedTabPage(selected = current == CommunitySection.Home && page == currentHome) {
                            when (page) {
                                HomeSub.Ideas -> IdeasFeed(env, categoryState, loader::reload, open)
                                HomeSub.News -> NewsTab(env, open, ::openHotRanking)
                                HomeSub.OpenSource -> OpenSourceTab(env, open, hotRankingOpenCount)
                                HomeSub.Rank -> RankTab(env, open)
                                HomeSub.Nav -> NavTab(env)
                            }
                        }
                    }
                }
                when (current) {
                    CommunitySection.Home -> Unit
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
private fun CommunityHeader(
    current: CommunitySection,
    unread: Int,
    onSelect: (CommunitySection) -> Unit,
    onBrand: () -> Unit,
    onSearch: () -> Unit,
    onNotifications: () -> Unit,
) {
    val surface = communityBarSurface
    Row(
        Modifier.fillMaxWidth().background(surface).statusBarsPadding().height(46.dp).padding(horizontal = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.width(72.dp), contentAlignment = Alignment.CenterStart) {
            Box(Modifier.size(34.dp).clip(CircleShape).clickable(role = Role.Button, onClick = onBrand),
                contentAlignment = Alignment.Center) {
                CommunityMark()
            }
        }
        BoxWithConstraints(Modifier.weight(1f)) {
            val spacing = if (maxWidth < 152.dp) 6.dp else 18.dp
            Row(Modifier.fillMaxWidth().selectableGroup(),
                horizontalArrangement = Arrangement.spacedBy(spacing, Alignment.CenterHorizontally),
                verticalAlignment = Alignment.CenterVertically) {
                CommunitySection.entries.forEach { section ->
                    val active = section == current
                    Box(Modifier.height(46.dp).selectable(active, role = Role.Tab, onClick = { onSelect(section) })
                        .padding(horizontal = 2.dp), contentAlignment = Alignment.Center) {
                        CommunityNavLabel(section.label, active, feedText(15, 22.5))
                    }
                }
            }
        }
        Row(Modifier.width(72.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            Box(Modifier.size(34.dp).clip(CircleShape).clickable(role = Role.Button, onClick = onSearch),
                contentAlignment = Alignment.Center) {
                XIcon(XIcons.NavSearch, tint = feedInk, size = 17.dp, description = "搜索社区")
            }
            Box(Modifier.size(34.dp).clip(CircleShape).clickable(role = Role.Button, onClick = onNotifications),
                contentAlignment = Alignment.Center) {
                XIcon(XIcons.NavBell, tint = feedInk, size = 17.dp,
                    description = if (unread > 0) "通知，${unread}条未读" else "通知")
                if (unread > 0) Box(Modifier.align(Alignment.TopEnd).padding(3.5.dp).size(9.dp)
                    .background(surface, CircleShape).padding(1.5.dp).background(Brand, CircleShape))
            }
        }
    }
}

@Composable
private fun CommunityMark() {
    val brand = Brand
    val mark = remember(brand) {
        ImageVector.Builder("CommunityMark", 25.dp, 25.dp, 64f, 64f)
            .addPath(
                pathData = addPathNodes("M10 8h44a4 4 0 0 1 4 4v13a7 7 0 0 0 0 14v13a4 4 0 0 1-4 4H10a4 4 0 0 1-4-4V39a7 7 0 0 0 0-14V12a4 4 0 0 1 4-4z"),
                fill = SolidColor(brand),
            )
            .addPath(
                pathData = addPathNodes("M21 33l8 8 15-17"),
                stroke = SolidColor(Color.White), strokeLineWidth = 6f,
                strokeLineCap = StrokeCap.Round, strokeLineJoin = StrokeJoin.Round,
            )
            .build()
    }
    Icon(mark, contentDescription = "超级小伴首页", modifier = Modifier.size(25.dp), tint = Color.Unspecified)
}
