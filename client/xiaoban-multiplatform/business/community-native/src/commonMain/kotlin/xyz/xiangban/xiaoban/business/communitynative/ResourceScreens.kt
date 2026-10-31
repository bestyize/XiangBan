package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
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
import androidx.compose.ui.graphics.Color
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
import xyz.xiangban.xiaoban.common.community.compactCount
import xyz.xiangban.xiaoban.common.community.formatDownloadSize
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

private val Gap = 4.dp

/** 栏目顶部的灰色说明，前面带一个小 i。 */
@Composable
private fun TabNote(text: String) {
    Row(Modifier.fillMaxWidth().padding(horizontal = 2.dp, vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        XIcon(XIcons.Info, tint = LocalExtraColors.current.ink3, size = 14.dp, modifier = Modifier.padding(top = 2.dp))
        Text(text, fontSize = 12.sp, lineHeight = 19.sp, color = LocalExtraColors.current.ink3)
    }
}

// ───────────── AI 工具 ─────────────

/** 许可证小标签：闭源带锁、开源带代码符号、待确认带问号，灰底。 */
@Composable
private fun LicenseChip(license: String, text: String) {
    val icon = when (license) { "open" -> XIcons.Code; "closed" -> XIcons.Lock; else -> XIcons.Question }
    Row(Modifier.clip(RoundedCornerShape(4.dp)).background(LocalExtraColors.current.fill).padding(horizontal = 6.dp, vertical = 1.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
        XIcon(icon, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 12.dp)
        Text(text, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

/** 工具图标：Claude Code、OpenCode 用自己的像素图标，其余按名字认厂商标志；ChatGPT 官方图标待补，画虚线占位。 */
@Composable
private fun ToolIcon(name: String, developer: String, size: androidx.compose.ui.unit.Dp = 48.dp, corner: androidx.compose.ui.unit.Dp = 12.dp) {
    val logo = BrandLogos.tool(name) ?: BrandLogos.find(name) ?: BrandLogos.find(developer)
    val openai = name.contains("chatgpt", true) || developer.contains("openai", true)
    BrandTile(logo, name, size = size, corner = corner, dashed = logo == null && openai)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ToolsTab(env: CommunityEnv, open: (Route) -> Unit) {
    val loader = rememberLoader(env, "tools") { env.forum.tools() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { list ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = FeedPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                if (list.items.isEmpty()) item { CenterMessage("还没有收录的 AI 工具", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                items(list.items, key = { it.key }) { tool -> ToolRow(tool) { open(Route.Tool(tool.key)) } }
                if (list.items.any { it.sample }) item { FootNote("版本号和更新日期为示例数据。") }
            }
        }
    }
}

@Composable
private fun ToolRow(tool: ToolSummary, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        ToolIcon(tool.name, tool.developer)
        Column(Modifier.weight(1f)) {
            Text(tool.name, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(tool.developer, Modifier.padding(top = 1.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            Row(Modifier.padding(top = 4.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                LicenseChip(tool.license, tool.licenseText)
                val meta = when {
                    tool.license == "unknown" && tool.latestVersion == null -> "许可证核实中"
                    else -> listOfNotNull(tool.latestVersion, tool.latestDate?.takeIf { it.isNotEmpty() }?.let { "$it 更新" }).joinToString(" · ")
                }
                if (meta.isNotEmpty()) Text(meta, fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        XIcon(XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 18.dp)
    }
}

// ───────────── 资料 ─────────────

private val DocTints = listOf(Color(0xFFF1ECE4), Color(0xFFE6F4EA), Color(0xFFE8EEF8))

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun DocsTab(env: CommunityEnv) {
    val loader = rememberLoader(env, "docs") { env.forum.docs() }
    val state by loader.state.collectAsState()
    PullToRefreshBox(isRefreshing = state is Load.Loading, onRefresh = loader::reload, modifier = Modifier.fillMaxSize()) {
        LoadBox(state, loader::reload) { groups ->
            LazyColumn(Modifier.fillMaxSize(), contentPadding = FeedPadding, verticalArrangement = Arrangement.spacedBy(Gap)) {
                item { TabNote("官方文档的索引和导读，原文一律跳转官网。") }
                if (groups.groups.isEmpty()) item { CenterMessage("还没有收录的资料", modifier = Modifier.fillMaxWidth().padding(vertical = 48.dp)) }
                groups.groups.forEachIndexed { gi, group ->
                    item(key = "g-${group.name}") {
                        Row(Modifier.fillMaxWidth().padding(start = 2.dp, end = 2.dp, top = 8.dp, bottom = 2.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            val logo = BrandLogos.find(group.name)
                            Box(Modifier.size(28.dp).clip(RoundedCornerShape(6.dp)).background(if (ThemeState.dark) DocTints[gi % 3].copy(alpha = 0.2f) else DocTints[gi % 3]), contentAlignment = Alignment.Center) {
                                if (logo != null) XIcon(logo.icon, tint = logo.color, size = 16.dp) else XIcon(XIcons.Book, size = 16.dp)
                            }
                            Text(group.name, Modifier.weight(1f), fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                            Text("官方原文", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                            XIcon(XIcons.ExternalLink, tint = LocalExtraColors.current.ink3, size = 14.dp)
                        }
                    }
                    items(group.items, key = { "d-${it.key}" }) { doc ->
                        Row(
                            Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable { env.platform?.openUrl(doc.url) }.padding(horizontal = 14.dp, vertical = 12.dp),
                            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            XIcon(XIcons.File, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 20.dp)
                            Column(Modifier.weight(1f)) {
                                Text(doc.title, fontSize = 15.sp)
                                if (doc.subtitle.isNotEmpty()) Text(doc.subtitle, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            }
                            GrayTag(doc.labelText)
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
    TopicFeed(controller, "还没有影音分享", open, header = { TabNote("只收片单、影评、乐评和公版或授权内容，不提供盗版下载。") }) { topic ->
        // 72×96 的竖封面在左，右边是类型标签、标题、摘要，底下是作者。
        Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable { open(Route.Topic(topic.id)) }.padding(12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Cover(topic.images.firstOrNull()?.let { env.forum.imageUrl(it) }, topic.id, Modifier.size(width = 72.dp, height = 96.dp))
            Column(Modifier.weight(1f).heightIn(min = 96.dp)) {
                GrayTag(topic.tags.firstOrNull() ?: topic.category.name)
                Text(topic.title, Modifier.padding(top = 6.dp), fontSize = 15.sp, fontWeight = FontWeight.SemiBold, lineHeight = 21.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
                if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 4.dp), fontSize = 14.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
                Text(topic.author.nickname, Modifier.padding(top = 8.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1)
            }
        }
    }
}

@Composable
private fun OpenTab(env: CommunityEnv, open: (Route) -> Unit, onMessage: (String) -> Unit) {
    val scope = rememberCoroutineScope()
    val controller = remember(env) { DiscussController(env.forum, scope, category = "share-software", tag = "开源", home = false) }
    TopicFeed(controller, "还没有开源精选", open, header = { TabNote("只收开源、免费软件和可免费使用的设计素材。") }) { topic ->
        // 88×88 的缩略图在左，右边是标题、摘要，底下是来源标签、浏览数和作者。
        Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable { open(Route.Topic(topic.id)) }.padding(12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Cover(topic.images.firstOrNull()?.let { env.forum.imageUrl(it) }, topic.id, Modifier.size(88.dp))
            Column(Modifier.weight(1f).heightIn(min = 88.dp)) {
                Text(topic.title, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, lineHeight = 21.sp, maxLines = 2, overflow = TextOverflow.Ellipsis)
                if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 4.dp), fontSize = 14.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
                Row(Modifier.padding(top = 8.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    GrayTag(topic.tags.firstOrNull { it in setOf("开源", "免费", "免费可商用") } ?: "开源")
                    Stat(XIcons.Eye, compactCount(topic.viewCount))
                    Text(topic.author.nickname, fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1)
                }
            }
        }
    }
}

// ───────────── 求助悬赏 ─────────────

private fun HelpInfo.expiredByTime(): Boolean = status == "open" && (parseIsoMillis(expiresAt) ?: Long.MAX_VALUE) <= nowMillis()

/** 求助的状态标签：进行中蓝、已采纳绿、已到期灰。 */
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
        Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).padding(14.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            val s = (statsState as? Load.Ready<HelpStats>)?.value
            Row(Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                StatNumber(s?.open?.toString() ?: "–", "进行中")
                StatNumber(s?.solvedThisWeek?.toString() ?: "–", "本周已解决")
                StatNumber(s?.bountyPoints?.let(::groupDigits) ?: "–", "悬赏积分")
            }
            Row(
                Modifier.height(34.dp).clip(RoundedCornerShape(17.dp)).background(Brand).clickable { open(Route.NewHelp) }.padding(horizontal = 14.dp),
                verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                XIcon(XIcons.Plus, tint = Color.White, size = 16.dp)
                Text("发求助", fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = Color.White)
            }
        }
    }) { HelpCard(env, it, open) }
}

private fun groupDigits(n: Long): String = n.toString().reversed().chunked(3).joinToString(",").reversed()

@Composable
private fun StatNumber(value: String, label: String) {
    Column {
        Text(value, fontSize = 20.sp, fontWeight = FontWeight.Bold)
        Text(label, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

/** 求助卡：第一行 悬赏分、状态、剩余时间；正文左边可带 72dp 的参考图；底下是提问人和回答数。 */
@Composable
private fun HelpCard(env: CommunityEnv, topic: Topic, open: (Route) -> Unit) {
    val help = topic.help
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(MaterialTheme.colorScheme.surface).clickable { open(Route.Topic(topic.id)) }.padding(14.dp)) {
        if (help != null) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(3.dp)) {
                    XIcon(XIcons.Coin, tint = Amber, size = 14.dp)
                    Text(if (help.bounty > 0) "悬赏 ${help.bounty} 分" else "无悬赏", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = Amber)
                }
                HelpStatusTag(help)
                Spacer(Modifier.weight(1f))
                XIcon(XIcons.Clock, tint = LocalExtraColors.current.ink3, size = 14.dp)
                Text(helpRemain(help), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
            }
        }
        val thumb = topic.images.firstOrNull()?.let { env.forum.imageUrl(it) }
        Row(Modifier.padding(top = 8.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            if (thumb != null) Box(Modifier.size(72.dp)) {
                Cover(thumb, topic.id, Modifier.fillMaxSize())
                Text("参考图", Modifier.align(Alignment.BottomStart).padding(4.dp).clip(RoundedCornerShape(4.dp)).background(Color.Black.copy(alpha = 0.4f)).padding(horizontal = 4.dp), fontSize = 12.sp, color = Color.White)
            }
            Column(Modifier.weight(1f)) {
                Text(topic.title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, lineHeight = 23.sp, maxLines = 3, overflow = TextOverflow.Ellipsis)
                if (topic.excerpt.isNotBlank()) Text(topic.excerpt, Modifier.padding(top = 4.dp), fontSize = 14.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
            }
        }
        Row(Modifier.padding(top = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Avatar(topic.author.nickname, 18.dp)
            Text(topic.author.nickname, Modifier.weight(1f), fontSize = 12.sp, color = LocalExtraColors.current.ink3, maxLines = 1, overflow = TextOverflow.Ellipsis)
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

/** 「10 月 20 日」→「10-20」，其余原样。 */
private fun shortDate(date: String): String {
    val m = Regex("""(\d+)\s*月\s*(\d+)\s*日""").find(date) ?: return date
    return "${m.groupValues[1]}-${m.groupValues[2]}"
}

@Composable
private fun ToolDetailBody(env: CommunityEnv, tool: ToolDetail, onMessage: (String) -> Unit) {
    var source by rememberSaveable { mutableStateOf(if (tool.hasMirror) "mirror" else "github") }
    var os by rememberSaveable { mutableStateOf(tool.artifacts.firstOrNull()?.os.orEmpty()) }
    val artifact = tool.artifacts.firstOrNull { it.os == os } ?: tool.artifacts.firstOrNull()
    fun urlOf(a: ToolArtifact) = (if (source == "mirror") a.mirrorUrl else a.githubUrl).ifEmpty { a.githubUrl.ifEmpty { a.mirrorUrl } }

    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(start = 12.dp, end = 12.dp, top = 4.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(Gap)) {
        WhiteCard(padding = 0.dp) {
            Column(Modifier.padding(horizontal = 14.dp, vertical = 16.dp)) {
                Row(horizontalArrangement = Arrangement.spacedBy(14.dp), verticalAlignment = Alignment.CenterVertically) {
                    ToolIcon(tool.name, tool.developer, 64.dp, 16.dp)
                    Column(Modifier.weight(1f)) {
                        Text(tool.name, fontSize = 20.sp, fontWeight = FontWeight.SemiBold)
                        Text(listOf(tool.developer, tool.typeLabel).filter { it.isNotEmpty() }.joinToString(" · "), Modifier.padding(top = 2.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                        Row(Modifier.padding(top = 6.dp), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            LicenseChip(tool.license, tool.licenseText)
                            tool.platforms.forEach { GrayTag(it) }
                        }
                    }
                }
                if (tool.description.isNotEmpty()) Text(tool.description, Modifier.padding(top = 12.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Row(Modifier.padding(top = 14.dp), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    InfoTile(tool.latestVersion ?: "–", "最新版本", Modifier.weight(1f))
                    InfoTile(tool.latestDate?.takeIf { it.isNotEmpty() }?.let(::shortDate) ?: "–", "更新日期", Modifier.weight(1f))
                    InfoTile(artifact?.size?.takeIf { it > 0 }?.let(::formatDownloadSize) ?: "–", "安装包", Modifier.weight(1f))
                }
            }
        }

        if (tool.artifacts.isNotEmpty() && artifact != null) {
            WhiteCard(padding = 14.dp) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        XIcon(XIcons.Download, size = 18.dp)
                        Text("下载", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    }
                    if (tool.hasMirror) Row(Modifier.clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(2.dp)) {
                        SourceSegment("小伴镜像", source == "mirror") { source = "mirror" }
                        SourceSegment("GitHub", source == "github") { source = "github" }
                    }
                }
                val url = urlOf(artifact)
                Row(
                    Modifier.padding(top = 12.dp).fillMaxWidth().height(48.dp).clip(RoundedCornerShape(24.dp)).background(Brand).clickable { if (url.isNotEmpty()) env.platform?.openUrl(url) else onMessage("这个安装包还没有下载地址") },
                    horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically,
                ) {
                    XIcon(XIcons.Download, tint = Color.White, size = 20.dp)
                    Text("下载 ${artifact.osLabel} 版 · ${formatDownloadSize(artifact.size)}", fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = Color.White)
                }
                Text(
                    (if (source == "mirror" && tool.hasMirror) "从小伴镜像下载，国内更快" else "从 GitHub Releases 下载") + " · 手机上下载后需要传到电脑再安装",
                    Modifier.padding(top = 6.dp).fillMaxWidth(), fontSize = 12.sp, lineHeight = 18.sp, color = LocalExtraColors.current.ink3, textAlign = androidx.compose.ui.text.style.TextAlign.Center,
                )
                Column(Modifier.padding(top = 12.dp)) {
                    tool.artifacts.forEach { a ->
                        Hairline()
                        Row(Modifier.fillMaxWidth().clickable { os = a.os }.padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                            Box(Modifier.size(32.dp).clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
                                XIcon(XIcons.Monitor, size = 18.dp)
                            }
                            Column(Modifier.weight(1f)) {
                                Text(a.osLabel, fontSize = 14.sp, fontWeight = FontWeight.Medium)
                                Text(listOf(a.desc, formatDownloadSize(a.size)).filter { it.isNotEmpty() }.joinToString(" · "), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                            }
                            Box(Modifier.size(36.dp).clip(CircleShape).background(LocalExtraColors.current.fill).clickable { val u = urlOf(a); if (u.isNotEmpty()) env.platform?.openUrl(u) else onMessage("这个安装包还没有下载地址") }, contentAlignment = Alignment.Center) {
                                XIcon(XIcons.Download, size = 18.dp, description = "下载 ${a.osLabel} 版")
                            }
                        }
                    }
                }
                if (artifact.sha256.isNotEmpty()) {
                    Row(Modifier.padding(top = 4.dp).fillMaxWidth().clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(horizontal = 12.dp, vertical = 10.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        XIcon(XIcons.Shield, tint = Success, size = 18.dp)
                        Column(Modifier.weight(1f)) {
                            Text("SHA-256 校验值", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text(artifact.sha256.take(8) + "…" + artifact.sha256.takeLast(6), fontSize = 12.sp, fontFamily = FontFamily.Monospace, maxLines = 1)
                        }
                        Text("复制", Modifier.height(28.dp).clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.surface).clickable { env.platform?.copy(artifact.sha256); onMessage("校验值已复制") }.padding(horizontal = 10.dp, vertical = 5.dp), fontSize = 12.sp)
                    }
                }
            }
        }

        if (tool.releases.any { it.items.isNotEmpty() }) ReleaseLog(tool)

        WhiteCard(padding = 0.dp) {
            Column(Modifier.padding(horizontal = 14.dp, vertical = 4.dp)) {
                InfoRow("开发者", tool.developer, first = true)
                InfoRow("许可证", when (tool.license) { "open" -> "以仓库 LICENSE 为准"; "closed" -> "闭源"; else -> "待确认" })
                if (tool.homepageUrl.isNotEmpty()) InfoRow("项目主页", tool.homepageUrl.removePrefix("https://").removePrefix("http://"), onClick = { env.platform?.openUrl(tool.homepageUrl) })
                if (tool.discussionCount > 0) InfoRow("相关讨论", "${tool.discussionCount} 篇教程和讨论")
            }
        }
        if (tool.sample) FootNote("版本号、日期、大小和更新内容为示例数据。")
    }
}

@Composable
private fun InfoTile(value: String, label: String, modifier: Modifier = Modifier) {
    Column(modifier.clip(RoundedCornerShape(8.dp)).background(LocalExtraColors.current.fill).padding(vertical = 10.dp, horizontal = 4.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Text(value, fontSize = 16.sp, fontWeight = FontWeight.Bold, maxLines = 1)
        Text(label, Modifier.padding(top = 2.dp), fontSize = 12.sp, color = LocalExtraColors.current.ink3)
    }
}

@Composable
private fun SourceSegment(text: String, selected: Boolean, onClick: () -> Unit) {
    Text(
        text,
        Modifier.height(28.dp).clip(RoundedCornerShape(6.dp)).background(if (selected) MaterialTheme.colorScheme.surface else Color.Transparent).clickable(onClick = onClick).padding(horizontal = 10.dp, vertical = 5.dp),
        fontSize = 13.sp, fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal, color = if (selected) Brand else MaterialTheme.colorScheme.onSurfaceVariant,
    )
}

@Composable
private fun InfoRow(label: String, value: String, first: Boolean = false, onClick: (() -> Unit)? = null) {
    if (!first) Hairline()
    Row(Modifier.fillMaxWidth().let { if (onClick != null) it.clickable(onClick = onClick) else it }.padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        Text(label, Modifier.width(64.dp), fontSize = 14.sp, color = LocalExtraColors.current.ink3)
        Text(value, Modifier.weight(1f), fontSize = 14.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
        if (onClick != null) XIcon(XIcons.ExternalLink, tint = LocalExtraColors.current.ink3, size = 16.dp)
    }
}

/** 更新记录：时间线，最新一条默认展开；每条有 新增 / 优化 / 修复 标签。 */
@Composable
private fun ReleaseLog(tool: ToolDetail) {
    var all by remember { mutableStateOf(false) }
    var openVersion by rememberSaveable { mutableStateOf(tool.releases.first().version) }
    val shown = if (all) tool.releases else tool.releases.take(3)
    WhiteCard(padding = 0.dp) {
        Column(Modifier.padding(start = 14.dp, end = 14.dp, top = 14.dp, bottom = 6.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    XIcon(XIcons.Clock, size = 18.dp)
                    Text("更新记录", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                }
                if (tool.releases.size > 3) Row(Modifier.clip(RoundedCornerShape(8.dp)).clickable { all = !all }.padding(4.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(if (all) "收起" else "全部", fontSize = 14.sp, color = LocalExtraColors.current.ink3)
                    XIcon(if (all) XIcons.ChevronUp else XIcons.ChevronRight, tint = LocalExtraColors.current.ink3, size = 16.dp)
                }
            }
            Column(Modifier.padding(top = 12.dp)) {
                shown.forEachIndexed { i, r ->
                    val expanded = openVersion == r.version
                    Row(Modifier.fillMaxWidth().height(IntrinsicSize.Min), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        Column(Modifier.width(12.dp).fillMaxHeight(), horizontalAlignment = Alignment.CenterHorizontally) {
                            Dot(if (i == 0) Brand else (if (ThemeState.dark) Color(0xFF48546A) else Color(0xFFC9D6EA)), 10.dp, Modifier.padding(top = 6.dp))
                            if (i < shown.lastIndex) Box(Modifier.width(2.dp).weight(1f).background(LocalExtraColors.current.hairline))
                        }
                        Column(Modifier.weight(1f).padding(bottom = 12.dp)) {
                            Row(Modifier.fillMaxWidth().clickable { openVersion = if (expanded) "" else r.version }, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                Text(r.version, fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                                if (i == 0) Text("最新", Modifier.clip(RoundedCornerShape(4.dp)).background(LocalExtraColors.current.brandSoft).padding(horizontal = 6.dp), fontSize = 12.sp, color = LocalExtraColors.current.brandInk)
                                Spacer(Modifier.weight(1f))
                                Text(r.date, fontSize = 12.sp, color = LocalExtraColors.current.ink3)
                                XIcon(if (expanded) XIcons.ChevronUp else XIcons.ChevronDown, tint = LocalExtraColors.current.ink3, size = 16.dp)
                            }
                            if (expanded) Column(Modifier.padding(top = 8.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                                r.items.forEach { item ->
                                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                        when (item.kind) {
                                            "fix" -> BlueTag("修复")
                                            "improve" -> AmberTag("优化")
                                            else -> GreenTag("新增")
                                        }
                                        Text(item.text, Modifier.weight(1f), fontSize = 14.sp, lineHeight = 21.sp, color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.8f))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
