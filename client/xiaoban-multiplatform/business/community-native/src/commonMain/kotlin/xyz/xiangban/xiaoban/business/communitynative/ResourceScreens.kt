package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import xyz.xiangban.xiaoban.common.community.DiscussController
import xyz.xiangban.xiaoban.common.community.HelpInfo
import xyz.xiangban.xiaoban.common.community.HelpStats
import xyz.xiangban.xiaoban.common.community.Load
import xyz.xiangban.xiaoban.common.community.ToolArtifact
import xyz.xiangban.xiaoban.common.community.ToolDetail
import xyz.xiangban.xiaoban.common.community.ToolSummary
import xyz.xiangban.xiaoban.common.community.Topic
import xyz.xiangban.xiaoban.common.community.agoText
import xyz.xiangban.xiaoban.common.community.formatSize
import xyz.xiangban.xiaoban.common.community.nowMillis
import xyz.xiangban.xiaoban.common.community.parseIsoMillis
import xyz.xiangban.xiaoban.common.community.remainText

/** 资源的五个二级栏目。 */
internal enum class ResourceSub(val key: String, val label: String) {
    Tools("tools", "AI 工具"), Docs("docs", "资料"), Media("media", "影音娱乐"), Help("help", "求助悬赏"), Open("open", "开源精选");

    companion object {
        fun byKey(key: String) = entries.firstOrNull { it.key == key } ?: Tools
    }
}

@Composable
internal fun ResourceTab(env: CommunityEnv, sub: ResourceSub, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    when (sub) {
        ResourceSub.Tools -> ToolsTab(env, open)
        ResourceSub.Docs -> DocsTab(env)
        ResourceSub.Media -> MediaTab(env, open, onMessage)
        ResourceSub.Help -> HelpTab(env, open)
        ResourceSub.Open -> OpenTab(env, open, onMessage)
    }
}

@Composable
private fun TabNote(text: String) {
    Text(text, Modifier.padding(horizontal = 8.dp, vertical = 2.dp), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
}

// ───────────── AI 工具 ─────────────

@Composable
private fun LicenseTag(license: String, text: String) {
    when (license) {
        "open" -> GreenTag(text)
        "closed" -> GrayTag(text)
        else -> AmberTag(text)
    }
}

@Composable
private fun ToolIcon(name: String, size: androidx.compose.ui.unit.Dp = 48.dp) {
    val brand = Brand
    LetterIcon(name, brand, brand.copy(alpha = 0.14f), size = size)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ToolsTab(env: CommunityEnv, open: (Route) -> Unit) {
    val loader = rememberLoader(env, "tools") { env.forum.tools() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { list ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = FeedPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                if (list.items.isEmpty()) item { CenterMessage("还没有收录的 AI 工具", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                items(list.items, key = { it.key }) { tool -> ToolRow(tool) { open(Route.Tool(tool.key)) } }
                if (list.items.any { it.sample }) item { FootNote("版本号和更新日期为示例数据。") }
            }
        }
    }
}

@Composable
private fun ToolRow(tool: ToolSummary, onClick: () -> Unit) {
    WhiteCard(Modifier.clickable(onClick = onClick)) {
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            ToolIcon(tool.name)
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(tool.name, Modifier.weight(1f, fill = false), fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    LicenseTag(tool.license, tool.licenseText)
                }
                Text(listOf(tool.developer, tool.typeLabel).filter { it.isNotEmpty() }.joinToString(" · "), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                val meta = when {
                    tool.license == "unknown" && tool.latestVersion == null -> "许可证核实中"
                    else -> listOfNotNull(tool.latestVersion, tool.latestDate?.takeIf { it.isNotEmpty() }?.let { "$it 更新" }).joinToString(" · ")
                }
                if (meta.isNotEmpty()) Text(meta, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
            XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 18.dp)
        }
    }
}

// ───────────── 资料 ─────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun DocsTab(env: CommunityEnv) {
    val loader = rememberLoader(env, "docs") { env.forum.docs() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { groups ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = FeedPadding, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                item { TabNote("官方文档的索引和导读，原文一律跳转官网。") }
                if (groups.groups.isEmpty()) item { CenterMessage("还没有收录的资料", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                items(groups.groups, key = { it.name }) { group ->
                    WhiteCard(padding = 0.dp) {
                        Row(Modifier.padding(start = 16.dp, end = 16.dp, top = 14.dp, bottom = 4.dp), verticalAlignment = Alignment.CenterVertically) {
                            Text(group.name, Modifier.weight(1f), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                            Text("官方原文", fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                        }
                        group.items.forEachIndexed { i, doc ->
                            if (i > 0) Hairline(Modifier.padding(start = 16.dp))
                            Row(Modifier.fillMaxWidth().clickable { env.platform?.openUrl(doc.url) }.padding(horizontal = 16.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                                Column(Modifier.weight(1f)) {
                                    Text(doc.title, fontSize = 15.sp, fontWeight = FontWeight.Medium, lineHeight = 21.sp)
                                    if (doc.subtitle.isNotEmpty()) Text(doc.subtitle, Modifier.padding(top = 2.dp), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                }
                                GrayTag(doc.labelText)
                                XIcon(XIcons.ExternalLink, tint = LocalExtraColors.current.ink3, size = 16.dp)
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────── 影音娱乐、开源精选 ─────────────

@Composable
private fun MediaTab(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { DiscussController(env.forum, scope, category = "share-movies,share-music", home = false) }
    TopicFeed(controller, "还没有影音分享", open, header = { TabNote("只收片单、影评、乐评和公版或授权内容，不提供盗版下载。") }) { TopicCard(env, it, open, onMessage) }
}

@Composable
private fun OpenTab(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { DiscussController(env.forum, scope, category = "share-software", tag = "开源", home = false) }
    TopicFeed(controller, "还没有开源精选", open, header = { TabNote("只收开源、免费软件和可免费使用的设计素材。") }) { TopicCard(env, it, open, onMessage) }
}

// ───────────── 求助悬赏 ─────────────

private fun HelpInfo.expiredByTime(): Boolean = status == "open" && (parseIsoMillis(expiresAt) ?: Long.MAX_VALUE) <= nowMillis()

/** 求助的状态标签和右边的剩余时间文字。 */
@Composable
internal fun HelpStatusTag(help: HelpInfo) {
    when {
        help.status == "solved" -> GreenTag("已采纳")
        help.status == "open" && !help.expiredByTime() -> BlueTag("进行中")
        help.status == "closed" -> GrayTag("已关闭")
        else -> GrayTag("已到期")
    }
}

internal fun helpRemain(help: HelpInfo): String = when {
    help.status == "solved" -> "已结束"
    help.status == "open" && !help.expiredByTime() -> remainText(help.expiresAt)
    help.bounty > 0 -> "积分已退回"
    else -> "已结束"
}

@Composable
private fun HelpTab(env: CommunityEnv, open: (Route) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { DiscussController(env.forum, scope, category = "share-help", home = false) }
    val stats = rememberLoader(env, "help-stats") { env.forum.helpStats() }
    val statsState by stats.state.collectAsState()
    TopicFeed(controller, "还没有人发求助", open, header = {
        WhiteCard {
            Row(verticalAlignment = Alignment.CenterVertically) {
                val s = (statsState as? Load.Ready<HelpStats>)?.value
                StatNumber(s?.open?.toString() ?: "–", "进行中")
                Spacer(Modifier.width(20.dp))
                StatNumber(s?.solvedThisWeek?.toString() ?: "–", "本周已解决")
                Spacer(Modifier.width(20.dp))
                StatNumber(s?.bountyPoints?.let(::groupDigits) ?: "–", "悬赏积分")
                Spacer(Modifier.weight(1f))
                PillButton("发求助", { open(Route.NewHelp) }, height = 40.dp, icon = XIcons.Plus)
            }
        }
    }) { HelpCard(env, it, open) }
}

private fun groupDigits(n: Long): String = n.toString().reversed().chunked(3).joinToString(",").reversed()

@Composable
private fun StatNumber(value: String, label: String) {
    Column {
        Text(value, fontSize = 22.sp, fontWeight = FontWeight.Bold)
        Text(label, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun HelpCard(env: CommunityEnv, topic: Topic, open: (Route) -> Unit) {
    val help = topic.help
    WhiteCard(Modifier.clickable { open(Route.Topic(topic.id)) }) {
        if (help != null) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                XIcon(XIcons.Coin, tint = Amber, size = 16.dp)
                Text(if (help.bounty > 0) "悬赏 ${help.bounty} 分" else "无悬赏", fontSize = 13.sp, fontWeight = FontWeight.SemiBold, color = Amber)
                HelpStatusTag(help)
                Spacer(Modifier.weight(1f))
                XIcon(XIcons.Clock, tint = LocalExtraColors.current.ink3, size = 14.dp)
                Text(helpRemain(help), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
        }
        val thumb = topic.images.firstOrNull()?.let { env.forum.imageUrl(it) }
        Row(Modifier.padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Column(Modifier.weight(1f)) {
                Text(topic.title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, lineHeight = 23.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
                if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 4.dp), fontSize = 14.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
            if (thumb != null) Box { Cover(thumb, topic.id, Modifier.size(88.dp)); Text("参考图", Modifier.align(Alignment.BottomStart).padding(4.dp).clip(RoundedCornerShape(4.dp)).background(androidx.compose.ui.graphics.Color.Black.copy(alpha = 0.55f)).padding(horizontal = 4.dp), fontSize = 10.sp, color = androidx.compose.ui.graphics.Color.White) }
        }
        Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Avatar(topic.author.nickname, 20.dp)
            Text(topic.author.nickname, Modifier.weight(1f), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Stat(XIcons.Message, "${topic.replyCount} 个回答")
        }
    }
}

// ───────────── 工具详情 ─────────────

/** 工具详情：基本信息、下载（小伴镜像或 GitHub）、更新记录、开发者和主页。 */
@Composable
fun ToolDetailScreen(env: CommunityEnv, key: String, back: () -> Unit) {
    val loader = rememberLoader(env, key) { env.forum.tool(key) }
    val state by loader.state.collectAsState()
    var toast by remember { mutableStateOf<String?>(null) }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            val detail = (state as? Load.Ready<ToolDetail>)?.value
            val shareUrl = detail?.homepageUrl.orEmpty()
            val trailing: (@Composable () -> Unit)? =
                if (detail != null && shareUrl.isNotEmpty()) ({ IconTap(XIcons.Share, "分享", { env.platform?.share(detail.name, shareUrl) }) }) else null
            ScreenBar("工具详情", onBack = back, trailing = trailing)
            LoadBox(state, loader::reload) { tool -> ToolDetailBody(env, tool) { toast = it } }
        }
        Toast(toast) { toast = null }
    }
}

@Composable
private fun ToolDetailBody(env: CommunityEnv, tool: ToolDetail, onMessage: (String) -> Unit) {
    var source by rememberSaveable { mutableStateOf(if (tool.hasMirror) "mirror" else "github") }
    var os by rememberSaveable { mutableStateOf(tool.artifacts.firstOrNull()?.os.orEmpty()) }
    val artifact = tool.artifacts.firstOrNull { it.os == os } ?: tool.artifacts.firstOrNull()
    fun urlOf(a: ToolArtifact) = (if (source == "mirror") a.mirrorUrl else a.githubUrl).ifEmpty { a.githubUrl.ifEmpty { a.mirrorUrl } }

    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(horizontal = 8.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        WhiteCard {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                ToolIcon(tool.name, 64.dp)
                Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text(tool.name, fontSize = 22.sp, fontWeight = FontWeight.Bold)
                    Text(listOf(tool.developer, tool.typeLabel).filter { it.isNotEmpty() }.joinToString(" · "), fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        LicenseTag(tool.license, tool.licenseText)
                        tool.platforms.forEach { GrayTag(it) }
                    }
                }
            }
            if (tool.description.isNotEmpty()) Text(tool.description, Modifier.padding(top = 12.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Row(Modifier.padding(top = 14.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                InfoTile(tool.latestVersion ?: "–", "最新版本", Modifier.weight(1f))
                InfoTile(tool.latestDate?.takeIf { it.isNotEmpty() } ?: "–", "更新日期", Modifier.weight(1f))
                InfoTile(artifact?.size?.takeIf { it > 0 }?.let(::formatSize) ?: "–", "安装包", Modifier.weight(1f))
            }
        }

        if (tool.artifacts.isNotEmpty() && artifact != null) {
            WhiteCard {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    XIcon(XIcons.Download, size = 20.dp)
                    Text("下载", Modifier.padding(start = 8.dp).weight(1f), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    if (tool.hasMirror) Row(Modifier.clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(2.dp)) {
                        SourceSegment("小伴镜像", source == "mirror") { source = "mirror" }
                        SourceSegment("GitHub", source == "github") { source = "github" }
                    }
                }
                val url = urlOf(artifact)
                PillButton("下载 ${artifact.osLabel} 版 · ${formatSize(artifact.size)}", { if (url.isNotEmpty()) env.platform?.openUrl(url) else onMessage("这个安装包还没有下载地址") }, Modifier.padding(top = 12.dp).fillMaxWidth(), icon = XIcons.Download)
                Text(if (source == "mirror" && tool.hasMirror) "从小伴镜像下载，国内更快；手机上下载后需要传到电脑再安装" else "从 GitHub Releases 下载；手机上下载后需要传到电脑再安装", Modifier.padding(top = 8.dp).fillMaxWidth(), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3)
                tool.artifacts.forEach { a ->
                    Hairline(Modifier.padding(top = 10.dp))
                    Row(Modifier.fillMaxWidth().clickable { os = a.os }.padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        Box(Modifier.size(40.dp).clip(RoundedCornerShape(10.dp)).background(if (a.os == artifact.os) Brand.copy(alpha = 0.14f) else LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
                            XIcon(XIcons.Monitor, tint = if (a.os == artifact.os) Brand else MaterialTheme.colorScheme.onSurface, size = 22.dp)
                        }
                        Column(Modifier.weight(1f)) {
                            Text(a.osLabel, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                            Text(listOf(a.desc, formatSize(a.size)).filter { it.isNotEmpty() }.joinToString(" · "), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                        }
                        IconTap(XIcons.Download, "下载 ${a.osLabel} 版", { val u = urlOf(a); if (u.isNotEmpty()) env.platform?.openUrl(u) else onMessage("这个安装包还没有下载地址") }, size = 20.dp)
                    }
                }
                if (artifact.sha256.isNotEmpty()) {
                    Row(Modifier.padding(top = 12.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(horizontal = 12.dp, vertical = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        XIcon(XIcons.Shield, tint = Success, size = 18.dp)
                        Column(Modifier.weight(1f)) {
                            Text("SHA-256 校验值 · ${artifact.osLabel}", fontSize = 11.sp, color = LocalExtraColors.current.ink3)
                            Text(artifact.sha256.take(8) + "…" + artifact.sha256.takeLast(6), fontSize = 13.sp)
                        }
                        GhostPillButton("复制", { env.platform?.copy(artifact.sha256); onMessage("校验值已复制") }, height = 30.dp, fontSize = 13, onPage = true)
                    }
                }
            }
        }

        if (tool.releases.isNotEmpty()) ReleaseLog(tool)

        WhiteCard(padding = 0.dp) {
            InfoRow("开发者", tool.developer, first = true)
            InfoRow("许可证", when (tool.license) { "open" -> "以仓库 LICENSE 为准"; "closed" -> "闭源"; else -> "待确认" })
            if (tool.homepageUrl.isNotEmpty()) InfoRow("项目主页", tool.homepageUrl.removePrefix("https://").removePrefix("http://"), onClick = { env.platform?.openUrl(tool.homepageUrl) })
            if (tool.discussionCount > 0) InfoRow("相关讨论", "${tool.discussionCount} 篇教程和讨论")
        }
        if (tool.sample) FootNote("版本号、日期、大小和更新内容为示例数据。")
    }
}

@Composable
private fun InfoTile(value: String, label: String, modifier: Modifier = Modifier) {
    Column(modifier.clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(vertical = 10.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Text(value, fontSize = 15.sp, fontWeight = FontWeight.Bold, maxLines = 1)
        Text(label, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun SourceSegment(text: String, selected: Boolean, onClick: () -> Unit) {
    Text(
        text,
        Modifier.clip(RoundedCornerShape(6.dp)).background(if (selected) MaterialTheme.colorScheme.surface else androidx.compose.ui.graphics.Color.Transparent).clickable(onClick = onClick).padding(horizontal = 12.dp, vertical = 5.dp),
        fontSize = 13.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal, color = if (selected) Brand else MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
private fun InfoRow(label: String, value: String, first: Boolean = false, onClick: (() -> Unit)? = null) {
    if (!first) Hairline(Modifier.padding(start = 16.dp))
    Row(Modifier.fillMaxWidth().let { if (onClick != null) it.clickable(onClick = onClick) else it }.padding(horizontal = 16.dp, vertical = 14.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(label, Modifier.width(72.dp), fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(value, Modifier.weight(1f), fontSize = 14.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        if (onClick != null) XIcon(XIcons.ExternalLink, tint = LocalExtraColors.current.ink3, size = 16.dp)
    }
}

@Composable
private fun ReleaseLog(tool: ToolDetail) {
    var all by remember { mutableStateOf(false) }
    var openVersion by rememberSaveable { mutableStateOf(tool.releases.first().version) }
    val shown = if (all) tool.releases else tool.releases.take(3)
    WhiteCard {
        Row(verticalAlignment = Alignment.CenterVertically) {
            XIcon(XIcons.Clock, size = 20.dp)
            Text("更新记录", Modifier.padding(start = 8.dp).weight(1f), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
            if (tool.releases.size > 3) Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable { all = !all }.padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(if (all) "收起" else "全部", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                XIcon(if (all) XIcons.ChevronUp else XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 14.dp)
            }
        }
        shown.forEachIndexed { i, r ->
            val expanded = openVersion == r.version
            Column(Modifier.fillMaxWidth().padding(top = 12.dp)) {
                Row(Modifier.fillMaxWidth().clickable { openVersion = if (expanded) "" else r.version }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Dot(if (i == 0) Brand else LocalExtraColors.current.line, 10.dp)
                    Text(r.version, fontSize = 15.sp, fontWeight = FontWeight.Bold)
                    if (i == 0) Tag("最新", Brand, Brand.copy(alpha = 0.12f))
                    Spacer(Modifier.weight(1f))
                    Text(r.date, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                    XIcon(if (expanded) XIcons.ChevronUp else XIcons.ChevronDown, tint = LocalExtraColors.current.ink3, size = 16.dp)
                }
                if (expanded) Column(Modifier.padding(start = 18.dp, top = 6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    r.items.forEach { item ->
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            when (item.kind) {
                                "fix" -> BlueTag("修复")
                                "improve" -> AmberTag("优化")
                                else -> GreenTag("新增")
                            }
                            Text(item.text, Modifier.weight(1f), fontSize = 14.sp, lineHeight = 20.sp)
                        }
                    }
                }
            }
        }
    }
}
