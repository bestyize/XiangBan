package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.horizontalScroll
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
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import xyz.xiangban.xiaoban.common.community.ChatSummary
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.CloudChatState
import kotlin.time.Clock

/** 首页用的几种点缀色，浅色和深色各一套；只用在图标和极浅的底上。 */
private class Tint(private val light: Long, private val dark: Long) {
    val color: Color get() = Color(if (ThemeState.dark) dark else light)
}

private val Ochre = Tint(0xFFB06A34L, 0xFFE0A274L)
private val Pine = Tint(0xFF4C7D5FL, 0xFF8CC3A0L)
private val Plum = Tint(0xFF7A5A8CL, 0xFFBFA2D0L)
private val Indigo = Tint(0xFF3D6B8CL, 0xFF8DB4D2L)

private class HomeFeature(val icon: ImageVector, val title: String, val desc: String, val tint: Tint, val onClick: () -> Unit)

private class HomeStarter(val icon: ImageVector, val text: String, val tint: Tint)

private val HomeStarters = listOf(
    HomeStarter(XIcons.Target, StarterPrompts[0], Ochre),
    HomeStarter(XIcons.Book, StarterPrompts[1], Indigo),
    HomeStarter(XIcons.Pen, StarterPrompts[2], Pine),
    HomeStarter(XIcons.Sparkles, "给我讲一个三分钟能读完的小故事", Plum),
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
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()

    LaunchedEffect(session.loggedIn) {
        if (session.loggedIn) {
            chat.loadMeta()
            env.remoteChat.refreshPeers()
        } else {
            chat.reset()
        }
    }
    val clock = remember(env.utcOffsetMinutes) { localClock(Clock.System.now().toEpochMilliseconds(), env.utcOffsetMinutes) }
    val (hello, line) = greeting(clock.hour)
    fun signedIn(action: () -> Unit) { if (session.loggedIn) action() else open(Route.Login) }

    val peers = remote.peers
    val computerDesc = when {
        !session.loggedIn -> "登录后可用"
        peers.isEmpty() -> "还没有连接电脑"
        peers.any { it.online } -> peers.first { it.online }.name + " 在线"
        else -> "电脑都不在线"
    }
    // 新功能加在这里，首页会按两列排开。
    val features = listOf(
        HomeFeature(XIcons.Image, "创作图片", "描述画面，生成插画和壁纸", Ochre) { signedIn(onCreateImage) },
        HomeFeature(XIcons.Gamepad, "创作游戏", "说出想法，做成能玩的小游戏", Pine) { onCreateWork("游戏") },
        HomeFeature(XIcons.Sparkles, "桌面装扮", "壁纸、小组件、特效和伙伴", Plum) { onCreateWork("壁纸") },
        HomeFeature(XIcons.Monitor, "遥控电脑", computerDesc, Indigo) { open(Route.Devices) },
    )

    Box(Modifier.fillMaxSize()) {
        HomeBackdrop(Modifier.fillMaxWidth().height(320.dp))
        Column(Modifier.fillMaxSize()) {
            Row(Modifier.fillMaxWidth().statusBarsPadding().height(52.dp).padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                IconTap(XIcons.Menu, "对话列表", onOpenDrawer)
                Text("小伴AI", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.Center)
                IconTap(XIcons.NewChat, "新对话", { signedIn { chat.clearConversation(); open(Route.AiChat) } }, size = 22.dp)
            }
            Column(Modifier.weight(1f).fillMaxWidth().verticalScroll(rememberScrollState()).padding(start = 16.dp, end = 16.dp, top = 16.dp)) {
                Text("${clock.month}月${clock.day}日 · ${Weekdays[clock.weekday]}", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Text(hello, Modifier.padding(top = 6.dp), style = XType.Headline, fontWeight = FontWeight.SemiBold)
                Text(line, Modifier.padding(top = 4.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)

                HomeComposerCard(cloud, session.loggedIn, Modifier.padding(top = 20.dp)) { signedIn { open(Route.AiChat) } }

                SectionTitle("用小伴创作", Modifier.padding(top = 28.dp))
                features.chunked(2).forEach { pair ->
                    Row(Modifier.padding(top = 10.dp).fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        pair.forEach { FeatureCard(it, Modifier.weight(1f)) }
                        if (pair.size == 1) Spacer(Modifier.weight(1f))
                    }
                }

                SectionTitle("试试这样问我", Modifier.padding(top = 28.dp))
                Row(
                    Modifier.padding(top = 10.dp).fillMaxWidth().horizontalScroll(rememberScrollState()),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    HomeStarters.forEach { starter ->
                        StarterCard(starter) { signedIn { chat.clearConversation(); chat.send(starter.text); open(Route.AiChat) } }
                    }
                }

                val recent = cloud.conversations.take(3)
                if (session.loggedIn && recent.isNotEmpty()) {
                    SectionTitle("继续对话", Modifier.padding(top = 28.dp))
                    Column(Modifier.padding(top = 10.dp).fillMaxWidth().outlinedCard(16.dp)) {
                        recent.forEachIndexed { i, item ->
                            if (i > 0) Hairline(Modifier.padding(start = 60.dp))
                            RecentRow(item, first = i == 0) { chat.openConversation(item.id); open(Route.AiChat) }
                        }
                    }
                    if (!env.chatsKept) Text("对话只保存在这次打开应用期间。", Modifier.padding(top = 6.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                Spacer(Modifier.height(NavInset + 12.dp))
            }
        }
    }
}

/** 顶部的淡色晕染和两道远山，往下渐隐到页面底色。 */
@Composable
private fun HomeBackdrop(modifier: Modifier) {
    val brand = Brand
    val soft = LocalExtraColors.current.brandSoft
    val page = MaterialTheme.colorScheme.background
    Canvas(modifier) {
        val w = size.width
        val h = size.height
        drawRect(Brush.verticalGradient(listOf(soft.copy(alpha = 0.9f), page), startY = 0f, endY = h))
        val far = Path().apply {
            moveTo(0f, h * 0.62f)
            cubicTo(w * 0.16f, h * 0.48f, w * 0.30f, h * 0.58f, w * 0.46f, h * 0.50f)
            cubicTo(w * 0.62f, h * 0.42f, w * 0.80f, h * 0.60f, w, h * 0.46f)
            lineTo(w, h); lineTo(0f, h); close()
        }
        val near = Path().apply {
            moveTo(0f, h * 0.78f)
            cubicTo(w * 0.22f, h * 0.64f, w * 0.38f, h * 0.80f, w * 0.58f, h * 0.70f)
            cubicTo(w * 0.74f, h * 0.62f, w * 0.88f, h * 0.76f, w, h * 0.70f)
            lineTo(w, h); lineTo(0f, h); close()
        }
        drawPath(far, brand.copy(alpha = 0.05f))
        drawPath(near, brand.copy(alpha = 0.07f))
        drawRect(Brush.verticalGradient(listOf(page.copy(alpha = 0f), page), startY = h * 0.6f, endY = h), topLeft = Offset(0f, h * 0.6f))
    }
}

@Composable
private fun SectionTitle(text: String, modifier: Modifier = Modifier) {
    Text(text, modifier, style = XType.Section, fontWeight = FontWeight.SemiBold)
}

@Composable
private fun HomeComposerCard(cloud: CloudChatState, loggedIn: Boolean, modifier: Modifier, onClick: () -> Unit) {
    val shape = RoundedCornerShape(22.dp)
    Column(
        modifier.fillMaxWidth()
            .shadow(14.dp, shape, ambientColor = Brand.copy(alpha = 0.25f), spotColor = Brand.copy(alpha = 0.25f))
            .clip(shape).background(MaterialTheme.colorScheme.surface)
            .border(1.dp, Brand.copy(alpha = 0.28f), shape)
            .clickable(onClick = onClick).padding(start = 18.dp, end = 14.dp, top = 18.dp, bottom = 14.dp),
    ) {
        Text("问点什么，或描述想要的内容…", style = XType.Body, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Row(Modifier.fillMaxWidth().padding(top = 18.dp), verticalAlignment = Alignment.CenterVertically) {
            ChipPill(cloud.modelName(), trailingIcon = XIcons.ChevronDown)
            Text(
                if (loggedIn) quotaText(cloud.balance) else "登录后可用",
                Modifier.weight(1f).padding(start = 10.dp), style = XType.Caption,
                color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis,
            )
            Box(Modifier.size(40.dp).clip(CircleShape).background(Brand), contentAlignment = Alignment.Center) {
                XIcon(XIcons.ArrowUp, tint = Color.White, size = 20.dp)
            }
        }
    }
}

@Composable
private fun FeatureCard(feature: HomeFeature, modifier: Modifier) {
    val tint = feature.tint.color
    val shape = RoundedCornerShape(18.dp)
    Column(
        modifier.clip(shape).background(MaterialTheme.colorScheme.surface)
            .background(tint.copy(alpha = if (ThemeState.dark) 0.12f else 0.07f))
            .border(1.dp, tint.copy(alpha = 0.16f), shape)
            .clickable(onClick = feature.onClick).padding(14.dp),
    ) {
        Box(Modifier.size(40.dp).clip(RoundedCornerShape(12.dp)).background(tint.copy(alpha = 0.16f)), contentAlignment = Alignment.Center) {
            XIcon(feature.icon, tint = tint, size = 22.dp)
        }
        Text(feature.title, Modifier.padding(top = 12.dp), style = XType.Body, fontWeight = FontWeight.SemiBold, maxLines = 1)
        Text(
            feature.desc, Modifier.padding(top = 2.dp).heightIn(min = 32.dp), style = XType.Caption,
            color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun StarterCard(starter: HomeStarter, onClick: () -> Unit) {
    val tint = starter.tint.color
    Column(
        Modifier.width(156.dp).outlinedCard(16.dp).clickable(onClick = onClick).padding(14.dp),
    ) {
        XIcon(starter.icon, tint = tint, size = 20.dp)
        Text(starter.text, Modifier.padding(top = 10.dp).heightIn(min = 44.dp), style = XType.Secondary, maxLines = 2, overflow = TextOverflow.Ellipsis)
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
                                relativeTime(item.updatedAtMs).takeIf { it.isNotEmpty() }?.let {
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
