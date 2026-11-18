package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.text.font.FontFamily
import xyz.xiangban.xiaoban.common.community.ChatSummary
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CloudChatState
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import xiaoban.business.community_native.generated.resources.Res
import xiaoban.business.community_native.generated.resources.home_companion
import xiaoban.business.community_native.generated.resources.home_tool_computer
import xiaoban.business.community_native.generated.resources.home_tool_decor
import xiaoban.business.community_native.generated.resources.home_tool_game
import xiaoban.business.community_native.generated.resources.home_tool_image
import xyz.xiangban.xiaoban.common.community.isCustomModelTier
import kotlin.time.Clock

/** 首页用的几种点缀色，浅色和深色各一套；只用在图标和极浅的底上。 */
private class Tint(private val light: Long, private val dark: Long) {
    val color: Color get() = Color(if (ThemeState.dark) dark else light)
}

private val Ochre = Tint(0xFFB06A34L, 0xFFE0A274L)
private val Pine = Tint(0xFF4C7D5FL, 0xFF8CC3A0L)
private val Plum = Tint(0xFF7A5A8CL, 0xFFBFA2D0L)
private val Indigo = Tint(0xFF3D6B8CL, 0xFF8DB4D2L)

private class HomeFeature(val icon: DrawableResource, val title: String, val status: String? = null, val onClick: () -> Unit)

private class HomeStarter(val icon: ImageVector, val label: String, val text: String, val tint: Tint)

private val HomeStarters = listOf(
    HomeStarter(XIcons.Target, "理清思路", StarterPrompts[0], Ochre),
    HomeStarter(XIcons.Book, "理解概念", StarterPrompts[1], Indigo),
    HomeStarter(XIcons.Pen, "润色文字", StarterPrompts[2], Pine),
    HomeStarter(XIcons.Sparkles, "听个故事", "给我讲一个三分钟能读完的小故事", Plum),
)

private class LocalClock(val month: Int, val day: Int, val weekday: Int, val hour: Int)

private fun localClock(epochMs: Long, offsetMinutes: Int): LocalClock {
    val minutes = epochMs.floorDiv(60_000L) + offsetMinutes
    val days = minutes.floorDiv(1440L)
    val hour = (minutes.mod(1440L) / 60).toInt()
    // Civil-from-days (H. Hinnant); 1970-01-01 is a Thursday.
    val z = days + 719_468
    val era = z.floorDiv(146_097L)
    val doe = z - era * 146_097
    val yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365
    val doy = doe - (365 * yoe + yoe / 4 - yoe / 100)
    val mp = (5 * doy + 2) / 153
    val day = (doy - (153 * mp + 2) / 5 + 1).toInt()
    val month = (if (mp < 10) mp + 3 else mp - 9).toInt()
    return LocalClock(month, day, (days + 3).mod(7L).toInt(), hour)
}

private val Weekdays = listOf("星期一", "星期二", "星期三", "星期四", "星期五", "星期六", "星期日")

private fun greeting(hour: Int): Pair<String, String> = when (hour) {
    in 5..8 -> "早上好" to "新的一天，从一件小事开始"
    in 9..11 -> "上午好" to "专心做事，别的我帮你记着"
    in 12..13 -> "中午好" to "吃过饭了吗？歇一会儿也好"
    in 14..17 -> "下午好" to "有点累的话，换件事做做"
    in 18..22 -> "晚上好" to "今天辛苦了，想聊点什么？"
    else -> "夜深了" to "早点休息，想说的话我都在"
}

// ───────────── 小伴AI 首页 ─────────────

/** 小伴AI 首页：问候、输入卡片（点一下进入对话页）、创作推荐、试试这样问我、继续对话。 */
@OptIn(kotlin.time.ExperimentalTime::class)
@Composable
fun AiHomeScreen(
    env: CommunityEnv, chat: CloudChatController, open: (Route) -> Unit, onOpenDrawer: () -> Unit,
    onCreateImage: () -> Unit, onCreateWork: (kind: String) -> Unit,
) {
    val homeScroll = rememberScrollState()
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()

    LaunchedEffect(session.loggedIn) {
        if (session.loggedIn) {
            chat.loadMeta()
            env.remoteChat.refreshPeers()
        }
    }
    val clock = remember(env.utcOffsetMinutes) { localClock(Clock.System.now().toEpochMilliseconds(), env.utcOffsetMinutes) }
    val (hello, line) = greeting(clock.hour)
    fun signedIn(action: () -> Unit) { if (session.loggedIn) action() else open(Route.Login) }
    fun chatReady(action: () -> Unit) { if (session.loggedIn && cloud.officialAiEnabled || cloud.customProviders.any { it.available }) action() else open(Route.Models) }

    val peers = remote.peers
    val computerDesc = when {
        !session.loggedIn -> "登录后可用"
        peers.isEmpty() -> "尚未连接"
        peers.any { it.online } -> peers.first { it.online }.name + " 在线"
        else -> "电脑离线"
    }
    val features = buildList {
        if (cloud.officialAiEnabled) add(HomeFeature(Res.drawable.home_tool_image, "创作图片") { signedIn(onCreateImage) })
        add(HomeFeature(Res.drawable.home_tool_game, "创作游戏") { onCreateWork("游戏") })
        add(HomeFeature(Res.drawable.home_tool_decor, "桌面装扮") { onCreateWork("壁纸") })
        add(HomeFeature(Res.drawable.home_tool_computer, "遥控电脑", computerDesc) { open(Route.Devices) })
    }

    val paper = if (ThemeState.dark) Color(0xFF211F20) else Color(0xFFFAF7F1)
    Column(Modifier.fillMaxSize().background(paper).navigationBarsPadding().padding(bottom = 56.dp)) {
        Row(Modifier.fillMaxWidth().statusBarsPadding().height(48.dp).padding(horizontal = 6.dp), verticalAlignment = Alignment.CenterVertically) {
            IconTap(XIcons.Menu, "对话列表", onOpenDrawer)
            Text("小伴 AI", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.Bold)
            IconTap(XIcons.NewChat, "新对话", { chatReady { chat.clearConversation(); open(Route.AiChat) } }, size = 21.dp)
        }
        Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(homeScroll).padding(horizontal = 20.dp)) {
            Row(
                Modifier.fillMaxWidth().heightIn(min = 126.dp).padding(vertical = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Column(Modifier.weight(1f)) {
                    Text("${clock.month}月${clock.day}日 · ${Weekdays[clock.weekday]}", style = XType.Caption, color = LocalExtraColors.current.brandInk)
                    Text(hello, Modifier.padding(vertical = 3.dp), style = XType.Headline.copy(fontSize = 32.sp, lineHeight = 40.sp),
                        fontFamily = FontFamily.Serif, fontWeight = FontWeight.Bold)
                    Text(line, style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                Image(painterResource(Res.drawable.home_companion), null, Modifier.fillMaxWidth(0.38f).height(102.dp), contentScale = ContentScale.Fit)
            }
            SectionTitle("快捷工具")
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                features.forEach { FeatureShortcut(it, Modifier.weight(1f)) }
            }

            val recent = cloud.conversations.take(3)
            if ((session.loggedIn || cloud.customModelName != null) && recent.isNotEmpty()) {
                SectionTitle("继续对话")
                Column(Modifier.fillMaxWidth().outlinedCard(16.dp)) {
                    recent.forEachIndexed { i, item ->
                        if (i > 0) Hairline(Modifier.padding(start = 60.dp))
                        RecentRow(item, first = i == 0) { chat.openConversation(item.id); open(Route.AiChat) }
                    }
                }
                if (!env.chatsKept) Text("对话只保存在这次打开应用期间。", Modifier.padding(top = 6.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            SectionTitle("从这里聊起")
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                HomeStarters.chunked(2).forEach { pair ->
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        pair.forEach { starter ->
                            StarterNote(starter, Modifier.weight(1f)) {
                                chatReady { chat.clearConversation(); chat.send(starter.text); open(Route.AiChat) }
                            }
                        }
                    }
                }
            }
            Spacer(Modifier.height(14.dp))
        }
        HomeChatEntry(cloud, session.loggedIn || cloud.customModelName != null) { chatReady { open(Route.AiChat) } }
    }
}

@Composable
private fun SectionTitle(text: String) {
    Row(Modifier.fillMaxWidth().padding(top = 12.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        Text(text, style = XType.Section, fontWeight = FontWeight.Bold)
        Spacer(Modifier.weight(1f).height(1.dp).background(LocalExtraColors.current.line))
    }
}

@Composable
private fun StarterNote(starter: HomeStarter, modifier: Modifier, onClick: () -> Unit) {
    val ink = starter.tint.color
    Column(
        modifier.clip(RoundedCornerShape(12.dp)).background(ink.copy(alpha = if (ThemeState.dark) 0.12f else 0.075f))
            .clickable(onClick = onClick).heightIn(min = 96.dp).padding(12.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(5.dp)) {
            XIcon(starter.icon, tint = ink, size = 15.dp)
            Text(starter.label, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        Text(starter.text, Modifier.padding(top = 7.dp), style = XType.Body, fontWeight = FontWeight.Medium)
    }
}

@Composable
private fun HomeChatEntry(cloud: CloudChatState, loggedIn: Boolean, onClick: () -> Unit) {
    val modelStatus = when {
        isCustomModelTier(cloud.selectedTier) -> "自己的模型服务"
        !cloud.officialAiEnabled -> "添加或选择模型服务"
        loggedIn -> quotaText(cloud.balance)
        else -> "登录或添加模型服务"
    }
    val background = MaterialTheme.colorScheme.onSurface
    val foreground = MaterialTheme.colorScheme.surface
    Row(
        Modifier.padding(horizontal = 16.dp, vertical = 8.dp).fillMaxWidth().clip(RoundedCornerShape(18.dp))
            .background(background).clickable(onClick = onClick).heightIn(min = 62.dp).padding(horizontal = 16.dp, vertical = 9.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            Text("和小伴聊聊", style = XType.Section, fontWeight = FontWeight.SemiBold, color = foreground)
            Text("${cloud.modelName()} · $modelStatus", style = XType.Caption, color = foreground.copy(alpha = 0.78f))
        }
        Box(Modifier.size(34.dp).clip(CircleShape).background(foreground), contentAlignment = Alignment.Center) {
            XIcon(XIcons.ArrowRight, tint = background, size = 18.dp)
        }
    }
}

@Composable
private fun FeatureShortcut(feature: HomeFeature, modifier: Modifier) {
    Column(
        modifier.clip(RoundedCornerShape(12.dp)).clickable(onClick = feature.onClick).padding(vertical = 4.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Image(
            painterResource(feature.icon), contentDescription = null,
            modifier = Modifier.size(50.dp),
        )
        Text(feature.title, Modifier.padding(top = 6.dp), style = XType.Secondary, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.Center)
        feature.status?.let {
            Text(it, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center, maxLines = 2, overflow = TextOverflow.Ellipsis)
        }
    }
}

@Composable
private fun RecentRow(item: ChatSummary, first: Boolean, onClick: () -> Unit) {
    val extra = LocalExtraColors.current
    Row(
        Modifier.fillMaxWidth().clickable(onClick = onClick).padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Box(
            Modifier.size(34.dp).clip(RoundedCornerShape(10.dp)).background(if (first) extra.brandSoft else extra.fill),
            contentAlignment = Alignment.Center,
        ) {
            XIcon(XIcons.Message, tint = if (first) extra.brandInk else MaterialTheme.colorScheme.onSurfaceVariant, size = 18.dp)
        }
        Column(Modifier.weight(1f)) {
            Text(item.title, style = XType.Body, fontWeight = if (first) FontWeight.SemiBold else FontWeight.Normal, maxLines = 1, overflow = TextOverflow.Ellipsis)
            Text(relativeTime(item.updatedAtMs), Modifier.padding(top = 2.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        XIcon(XIcons.ChevronRight, tint = extra.ink3, size = 16.dp)
    }
}

// ───────────── 对话列表侧栏 ─────────────

/** 左侧对话列表：新对话、按工作空间分组的对话（正在看的那个用主题色浅底，长按或“…”打开操作）、模型与点数、设置。 */
@Composable
internal fun AiDrawer(
    cloud: CloudChatState, onClose: () -> Unit, onNew: () -> Unit, onOpen: (String) -> Unit, onDelete: (ChatSummary) -> Unit,
    onModels: () -> Unit, onSettings: () -> Unit, panelModifier: Modifier,
    onCreateWorkspace: () -> Unit, onEditWorkspace: (String) -> Unit,
) {
    var collapsed by remember { mutableStateOf(setOf<String?>()) }
    val extra = LocalExtraColors.current
    Column(
        panelModifier.fillMaxSize()
            .clip(RoundedCornerShape(topEnd = 24.dp, bottomEnd = 24.dp))
            .background(MaterialTheme.colorScheme.surface).statusBarsPadding().navigationBarsPadding(),
    ) {
        Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 8.dp, top = 8.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("对话", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
            IconTap(XIcons.Close, "收起侧栏", onClose, size = 20.dp)
        }
        Row(
            Modifier.padding(horizontal = 12.dp).fillMaxWidth().height(46.dp).clip(RoundedCornerShape(14.dp))
                .background(extra.brandSoft).clickable(onClick = onNew).padding(horizontal = 14.dp),
            verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            XIcon(XIcons.NewChat, tint = extra.brandInk, size = 20.dp)
            Text("开启新对话", style = XType.Body, fontWeight = FontWeight.SemiBold, color = extra.brandInk)
        }
        Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 8.dp, top = 16.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("工作空间", Modifier.weight(1f), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
            IconTap(XIcons.Plus, "新建工作空间", onCreateWorkspace, size = 18.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        LazyColumn(Modifier.weight(1f).fillMaxWidth(), contentPadding = PaddingValues(start = 8.dp, end = 8.dp, bottom = 8.dp)) {
            (cloud.workspaces.map { it as String? } + listOf<String?>(null)).forEach { group ->
                val chats = cloud.conversations.filter { it.workspace == group }
                item(key = "group:${group ?: ""}") {
                    Row(
                        Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp))
                            .clickable { collapsed = if (group in collapsed) collapsed - group else collapsed + group }
                            .padding(start = 12.dp, end = if (group == null) 12.dp else 0.dp).height(40.dp),
                        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        XIcon(if (group in collapsed) XIcons.ChevronRight else XIcons.ChevronDown, size = 14.dp, tint = extra.ink3)
                        Text(group ?: "未分组", Modifier.weight(1f), style = XType.Secondary, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text(chats.size.toString(), style = XType.Caption, color = extra.ink3)
                        if (group != null) IconTap(XIcons.More, "管理工作空间 $group", { onEditWorkspace(group) }, size = 16.dp, tint = extra.ink3)
                    }
                }
                if (group !in collapsed) {
                    if (chats.isEmpty()) item(key = "empty:${group ?: ""}") {
                        Text(
                            if (group == null) "新的对话会出现在这里" else "在对话菜单里移入聊天",
                            Modifier.padding(start = 34.dp, top = 4.dp, bottom = 12.dp), style = XType.Caption, color = extra.ink3,
                        )
                    }
                    items(chats, key = { it.id }) { item ->
                        val current = item.id == cloud.conversationId
                        Row(
                            Modifier.padding(vertical = 1.dp).fillMaxWidth().clip(RoundedCornerShape(12.dp))
                                .background(if (current) extra.brandSoft else Color.Transparent)
                                .combinedClickable(onLongClick = { onDelete(item) }) { onOpen(item.id) }
                                .padding(start = 34.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Column(Modifier.weight(1f).padding(vertical = 10.dp)) {
                                Text(
                                    item.title, style = XType.Body, maxLines = 1, overflow = TextOverflow.Ellipsis,
                                    color = if (current) extra.brandInk else MaterialTheme.colorScheme.onSurface,
                                    fontWeight = if (current) FontWeight.SemiBold else FontWeight.Normal,
                                )
                                (when (item.id) {
                                    in cloud.waitingSessions -> "等待你确认或回答"
                                    in cloud.runningSessions -> "任务进行中"
                                    in cloud.imageSessions -> "生图进行中"
                                    else -> relativeTime(item.updatedAtMs)
                                }).takeIf { it.isNotEmpty() }?.let {
                                    Text(it, Modifier.padding(top = 2.dp), style = XType.Caption, color = extra.ink3)
                                }
                            }
                            IconTap(XIcons.More, "对话操作", { onDelete(item) }, size = 16.dp, tint = extra.ink3)
                        }
                    }
                }
                item(key = "gap:${group ?: ""}") { Spacer(Modifier.height(6.dp)) }
            }
        }
        Hairline()
        Row(Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 6.dp)) {
            DrawerLink(XIcons.Star, "模型与点数", onModels, Modifier.weight(1f))
            DrawerLink(XIcons.Sliders, "设置", onSettings, Modifier.weight(1f))
        }
    }
}

@Composable
private fun DrawerLink(icon: ImageVector, text: String, onClick: () -> Unit, modifier: Modifier) {
    Row(
        modifier.height(44.dp).clip(RoundedCornerShape(12.dp)).clickable(onClick = onClick).padding(horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        XIcon(icon, size = 20.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(text, style = XType.Secondary)
    }
}
