package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.backhandler.BackHandler
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil3.ImageLoader
import coil3.compose.AsyncImage
import coil3.compose.LocalPlatformContext
import coil3.svg.SvgDecoder
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.MobileGame

private val CreationTypes = listOf("全部", "游戏", "壁纸", "小组件", "特效", "伙伴")
private data class CreationItem(val id: String, val name: String, val description: String, val kind: String,
    val game: MobileGame? = null, val owned: Boolean = false) {
    val key get() = (if (owned) "mine:" else "builtin:") + id
    val device get() = if (game != null) "手机可玩" else "电脑创作灵感"
}
// These are reference ideas, not installed mobile desktop resources. Actual creation runs on the selected desktop.
private val DesktopIdeas = listOf(
    CreationItem("soft-hills", "柔软山丘", "制作一张层层山丘随光线缓慢变化的动画壁纸，色调柔和，适合长时间工作。", "壁纸"),
    CreationItem("daylight-clock", "日光时钟", "制作一个简洁的桌面时钟小组件，显示真实时间与日期，配温暖的日光色调。", "小组件"),
    CreationItem("fireflies", "流萤之夜", "制作一个夜间桌面萤火特效，光点缓慢浮动，不影响点击和正常工作。", "特效"),
    CreationItem("sunflower", "向日葵小伴", "制作一位向日葵桌面伙伴，工作时安静陪伴，点击时轻轻摇摆回应。", "伙伴"),
)

@OptIn(androidx.compose.ui.ExperimentalComposeUiApi::class)
@Composable
fun GamesScreen(env: CommunityEnv, chat: CloudChatController, open: (Route) -> Unit, onOverlayChanged: (Boolean) -> Unit = {}, createRequest: String? = null, onCreateRequestHandled: () -> Unit = {}) {
    var official by remember { mutableStateOf(emptyList<MobileGame>()) }
    var games by remember { mutableStateOf(emptyList<MobileGame>()) }
    var errors by remember { mutableStateOf(emptyList<String>()) }
    var loading by remember { mutableStateOf(true) }
    var reload by remember { mutableStateOf(0) }
    var mine by rememberSaveable { mutableStateOf(false) }
    var category by rememberSaveable { mutableStateOf("全部") }
    var detail by remember { mutableStateOf<CreationItem?>(null) }
    // One sheet at a time; submenus retain the selected work without stacking focus locks.
    var panel by remember { mutableStateOf("actions") }
    var creating by remember { mutableStateOf(false) }
    var remixSource by remember { mutableStateOf<MobileGame?>(null) }
    var allGames by rememberSaveable { mutableStateOf(false) }
    var allTools by rememberSaveable { mutableStateOf(false) }
    var createKind by rememberSaveable { mutableStateOf("游戏") }
    var idea by rememberSaveable { mutableStateOf("") }
    var remotePrompt by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }
    var message by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    val remote by env.remoteChat.state.collectAsState()
    val context = LocalPlatformContext.current
    val imageLoader = remember(context) { ImageLoader.Builder(context).components { add(SvgDecoder.Factory()) }.build() }
    DisposableEffect(imageLoader) { onDispose { imageLoader.shutdown() } }
    val covers = remember { mutableStateMapOf<String, ByteArray>() }
    val overlay = detail != null || creating || remotePrompt != null || remixSource != null
    LaunchedEffect(overlay, allGames, allTools) { onOverlayChanged(overlay || allGames || allTools) }
    DisposableEffect(Unit) { onDispose { onOverlayChanged(false) } }
    fun close() { if (!busy) { detail = null; creating = false; remotePrompt = null; remixSource = null; panel = "actions" } }
    fun openOfficial(game: MobileGame) {
        if (env.platform?.openGame(game.id) != true) message = "当前平台无法打开作品"
    }
    fun showOfficialDetails(game: MobileGame) {
        detail = CreationItem(game.id, game.name, game.description, "游戏", game)
        panel = "actions"
    }
    BackHandler(enabled = overlay || allGames || allTools) {
        if (!overlay) { allGames = false; allTools = false }
        else if (!busy && detail != null && panel != "actions") panel = "actions" else close()
    }
    fun createLocal(prompt: String) {
        if (!session.loggedIn) { open(Route.Login); return }
        if (env.agent == null) { message = "当前安装包不支持手机本地创作，可连接电脑创作。"; return }
        if (cloud.sending) { message = "先结束当前任务，再开始新作品"; return }
        close()
        chat.clearConversation()
        open(Route.AiChat)
        chat.send(prompt)
    }
    fun createOnComputer(prompt: String) {
        if (!session.loggedIn) { open(Route.Login); return }
        creating = false; detail = null; remotePrompt = prompt
        scope.launch { env.remoteChat.refreshPeers() }
    }
    fun startCreation() {
        if (createKind == "游戏") createLocal("帮我设计并制作一款支持手机触控和电脑键鼠的离线小游戏。我的想法：${idea.trim()}。先给出简短玩法方案让我确认，再先用 app_read_format 读取小伴应用规范，按规范制作。")
        else createOnComputer("请在这台电脑上帮我创作小伴桌面的${createKind}。我的想法：${idea.trim()}。先读取对应的资源规范，给出简短方案让我确认，再制作并预览。")
    }
    fun prepare(item: CreationItem? = null) {
        detail = null
        createKind = item?.kind ?: category.takeUnless { it == "全部" } ?: "游戏"
        idea = item?.description.orEmpty()
        creating = true
    }
    LaunchedEffect(createRequest) {
        createRequest?.let { kind ->
            prepare()
            createKind = kind
            onCreateRequestHandled()
        }
    }
    fun runAction(action: suspend () -> Unit) {
        if (busy) return
        busy = true
        scope.launch {
            try { action() }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { message = e.message ?: "操作失败，请重试" }
            finally { busy = false }
        }
    }
    LaunchedEffect(env.games, reload) {
        loading = true
        val failures = mutableListOf<String>()
        try {
            try { official = env.platform?.builtinGames().orEmpty() }
            catch (e: CancellationException) { throw e }
            catch (_: Exception) { failures += "官方游戏读取失败，请重试" }
            try { val scan = env.games?.scan(); games = scan?.games.orEmpty(); failures += scan?.errors.orEmpty() }
            catch (e: CancellationException) { throw e }
            catch (_: Exception) { failures += "本机作品读取失败，请重试" }
            covers.clear()
            official.forEach { game ->
                try { env.platform?.builtinGameCover(game.id)?.let { covers["builtin:${game.id}"] = it } }
                catch (e: CancellationException) { throw e } catch (_: Exception) { }
            }
            games.forEach { game ->
                try { env.games?.readFile(game.id, "cover.svg")?.encodeToByteArray()?.takeIf { it.size <= 1024 * 1024 }?.let { covers["mine:${game.id}"] = it } }
                catch (e: CancellationException) { throw e } catch (_: Exception) { }
            }
            DesktopIdeas.forEach { item ->
                try { env.platform?.creationCover(item.id)?.let { covers[item.key] = it } }
                catch (e: CancellationException) { throw e } catch (_: Exception) { }
            }
        } finally { errors = failures; loading = false }
    }
    val myItems = games.map { CreationItem(it.id, it.name, it.description, if (it.kind == "app") "工具" else "游戏", it, true) }
    val visible = myItems.filter { category == "全部" || it.kind == category }
    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        Column(Modifier.fillMaxSize().statusBarsPadding()) {
            Row(Modifier.fillMaxWidth().padding(start = 16.dp, end = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                Row(Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(24.dp)) {
                    listOf(false to "广场", true to "我的作品").forEach { (value, label) ->
                        Column(Modifier.semantics { selected = mine == value }.clickable(role = Role.Tab) { mine = value; category = "全部" }.padding(vertical = 8.dp)) {
                            Text(label, style = XType.Title, fontWeight = if (mine == value) FontWeight.SemiBold else FontWeight.Normal, color = if (mine == value) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant)
                            Box(Modifier.padding(top = 8.dp).width(20.dp).height(3.dp).background(if (mine == value) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.background, CircleShape))
                        }
                    }
                }
                IconTap(XIcons.Search, "搜索应用", {}) // Search destination is reserved for a later version.
                if (mine) IconTap(XIcons.Plus, "新建作品", { prepare() })
            }
            if (!mine) AppPlazaScreen(env, official, games, covers, imageLoader, loading, errors, { reload++ },
                onOfficial = ::openOfficial, onOfficialDetails = ::showOfficialDetails,
                onAllGames = { allGames = true }, onAllTools = { allTools = true }, open = open)
            else LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 12.dp, bottom = NavInset), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                item {
                    Row(Modifier.fillMaxWidth().outlinedCard(16.dp).clickable { prepare() }.padding(16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        XIcon(XIcons.Sparkle, tint = MaterialTheme.colorScheme.primary, size = 24.dp)
                        Column(Modifier.weight(1f)) {
                            Text("你想做个什么？", style = XType.Body, fontWeight = FontWeight.SemiBold)
                            Text("和小伴一起，把想法变成作品", Modifier.padding(top = 4.dp), style = XType.Caption, color = LocalExtraColors.current.ink3)
                        }
                    }
                }
                item { Text("本机作品 · 电脑创作的装扮保存在对应电脑", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                item { CreationChips(CreationTypes, category) { category = it } }
                if (errors.isNotEmpty()) item {
                    Text(errors.joinToString("\n"), style = XType.Caption, color = MaterialTheme.colorScheme.error)
                    LinkText("重新读取", { reload++ })
                }
                if (loading && visible.isEmpty()) item { Spinner(Modifier.fillMaxWidth().height(120.dp)) }
                if (!loading && visible.isEmpty()) item {
                    EmptyState(XIcons.Sparkle, "还没有${if (category == "全部") "" else category}作品", if (category !in listOf("全部", "游戏")) "桌面装扮在电脑上创作与使用，手机可遥控电脑继续创作。" else "从一个小小的想法开始吧。")
                    AppActionButton("创作第一件作品", { prepare() }, Modifier.fillMaxWidth(), primary = true)
                }
                item { CreationSection("本机作品", "刷新", { if (!loading) reload++ }) }
                items(visible, key = { it.key }) { entry ->
                    Row(Modifier.fillMaxWidth().outlinedCard(16.dp).clickable(role = Role.Button) { detail = entry; panel = "actions" }.padding(16.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        CreationCover(entry, covers[entry.key], imageLoader, Modifier.size(68.dp))
                        Column(Modifier.weight(1f)) {
                            Text(entry.name, style = XType.Body, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Text("${entry.kind} · ${entry.device}", Modifier.padding(top = 4.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text("保存在这台手机上", Modifier.padding(top = 4.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        XIcon(XIcons.More, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 20.dp)
                    }
                }
            }
        }
        if (allTools && !mine) FeaturedToolsPage(env, official.filter { BuiltinMobileGames.isTool(it.id) }, back = { allTools = false },
            onDetails = ::showOfficialDetails, onOpen = ::openOfficial, open = open)
        if (allGames && !mine) FeaturedGamesPage(official.filterNot { BuiltinMobileGames.isTool(it.id) }, covers, imageLoader, back = { allGames = false },
            onDetails = ::showOfficialDetails, onOpen = ::openOfficial)
        detail?.let { entry ->
            if (!entry.owned && entry.game != null) OfficialAppSheet(env, entry.game, covers[entry.key], imageLoader,
                dismiss = { close() }, remix = { detail = null; remixSource = entry.game })
            else CreationSheet({ if (!busy && panel != "actions") panel = "actions" else close() }) {
                Column(Modifier.fillMaxWidth().heightIn(max = 590.dp).verticalScroll(rememberScrollState())) {
                    if (panel in listOf("actions", "preview")) Row(Modifier.padding(bottom = 21.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        CreationCover(entry, covers[entry.key], imageLoader, Modifier.size(52.dp), cornerRadius = 11)
                        Column(Modifier.weight(1f)) {
                            Text(entry.name, style = XType.Section, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                            Text("${entry.kind} · ${if (entry.owned) "我的作品" else if (entry.game != null) "小伴官方" else "创作灵感"}", Modifier.padding(top = 5.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        IconTap(XIcons.Close, "关闭作品操作", { close() }, Modifier.align(Alignment.Top), tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 17.dp)
                    } else Row(Modifier.padding(bottom = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text(when (panel) { "delete" -> "删除「${entry.name}」？"; "share" -> "分享到其他渠道"; else -> "发送到电脑" }, Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                        IconTap(XIcons.Close, "关闭作品操作", { close() }, tint = MaterialTheme.colorScheme.onSurfaceVariant, size = 17.dp)
                    }
                    when (panel) {
                        "actions" -> {
                            CreationActionGroup {
                                CreationAction(if (entry.game != null) XIcons.Play else XIcons.Eye, if (entry.game != null) "开始游玩" else "预览灵感", if (entry.game != null) "手机可玩" else "电脑装扮", !busy) {
                                    if (entry.game != null) {
                                        if (env.platform?.openGame(entry.id) == true) close() else message = "当前平台无法打开游戏"
                                    } else panel = "preview"
                                }
                                CreationAction(XIcons.Pen, if (entry.owned) "继续修改" else "做一个我的版本", enabled = !busy, divider = true) {
                                    if (entry.owned) createLocal("请先读取 games/${entry.id} 中的作品源码和试玩记录，问我想怎样修改《${entry.name}》，然后在原游戏上迭代。") else prepare(entry)
                                }
                                CreationAction(XIcons.Monitor, if (entry.game != null) "发送到电脑" else "在电脑上创作", enabled = !busy, divider = true) {
                                    if (entry.game == null) createOnComputer("请在电脑上参考以下想法制作小伴桌面${entry.kind}：${entry.description} 先读取规范并给出方案让我确认。") else panel = "computer"
                                }
                            }
                            Spacer(Modifier.height(11.dp))
                            CreationActionGroup {
                                CreationAction(XIcons.Globe, "分享到社区", enabled = !busy) {
                                    close()
                                    if (entry.owned) open(Route.PublishPack(localGameId = entry.id))
                                    else open(Route.CreationPost("分享：${entry.name}", "${entry.name}\n\n${entry.description}\n\n${if (entry.game != null) "来自小伴官方游戏，在小伴手机「应用」中游玩。" else "小伴桌面创作灵感，可连接电脑制作。"}"))
                                }
                                CreationAction(XIcons.Share, "分享到其他渠道", enabled = !busy, divider = true, arrow = true) { panel = "share" }
                            }
                            if (entry.owned) {
                                Spacer(Modifier.height(10.dp))
                                CreationActionGroup {
                                    CreationAction(XIcons.Trash, "删除作品", if (cloud.sending) "创作进行中" else "", enabled = !busy && !cloud.sending, destructive = true) { panel = "delete" }
                                }
                            }
                        }
                        "delete" -> {
                            Text("将删除这台手机上的作品文件、游戏进度和运行记录，无法撤销。已导出、分享或发布到社区的副本不受影响。", Modifier.padding(top = 2.dp, bottom = 18.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            if (entry.owned) CreationActionGroup {
                                CreationAction(XIcons.Trash, if (busy) "正在删除…" else "确认删除", enabled = !busy && !cloud.sending, destructive = true) {
                                    runAction {
                                        val store = env.games ?: error("本机作品不可用")
                                        store.delete(entry.id)
                                        games = games.filterNot { it.id == entry.id }
                                        covers.remove(entry.key)
                                        detail = null
                                        panel = "actions"
                                        env.platform?.forgetGame(entry.id)
                                        reload++
                                        message = "已删除《${entry.name}》"
                                    }
                                }
                            }
                        }
                        "preview" -> {
                            CreationCover(entry, covers[entry.key], imageLoader, Modifier.fillMaxWidth().height(220.dp))
                            Text(entry.description, Modifier.padding(vertical = 16.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Text("这是创作参考图。动画与互动效果需要在电脑上制作并预览。", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(Modifier.height(14.dp))
                            CreationActionGroup { CreationAction(XIcons.Sparkle, "做一个我的版本", enabled = !busy) { prepare(entry) } }
                        }
                        "computer" -> {
                            Text(if (entry.owned) "将作品包通过系统分享发送给电脑，或保存后自行传输。在电脑的小伴中导入即可使用；不会自动覆盖已有作品。" else "这款官方游戏已内置于小伴桌面。可打开电脑对话，让小伴找到它。", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(Modifier.height(16.dp))
                            CreationActionGroup {
                                if (entry.owned) {
                                    CreationAction(XIcons.Share, "分享作品包到电脑", enabled = !busy) { runAction { if (env.platform?.shareGamePackage(entry.id) != true) message = "当前平台不支持分享作品包" } }
                                    CreationAction(XIcons.Download, "保存作品包", enabled = !busy, divider = true) { runAction { if (env.platform?.exportGame(entry.id) != true) message = "当前平台不支持导出" } }
                                } else CreationAction(XIcons.Monitor, "选择电脑", enabled = !busy) { createOnComputer("请帮我在小伴桌面的官方游戏里找到《${entry.name}》（${entry.id}），告诉我怎么开始游玩。") }
                            }
                        }
                        "share" -> {
                            Text("选择分享形式，再选择接收应用", style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            Spacer(Modifier.height(14.dp))
                            CreationActionGroup {
                                CreationAction(XIcons.Image, "分享作品卡片", "图片", !busy) { runAction { if (env.platform?.shareCreationPreview(entry.name, entry.description, covers[entry.key]) != true) message = "当前平台不支持分享图片" } }
                                if (entry.owned) CreationAction(XIcons.Download, "分享作品包", "可导入使用", !busy, divider = true) { runAction { if (env.platform?.shareGamePackage(entry.id) != true) message = "当前平台不支持分享作品包" } }
                                CreationAction(XIcons.Share, "分享文字介绍", enabled = !busy, divider = true) { env.platform?.share(entry.name, entry.description) }
                            }
                        }
                    }
                    if (busy && panel != "delete") Text("正在准备作品…", Modifier.padding(top = 12.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text(if (panel in listOf("actions", "delete")) "取消" else "返回", Modifier.fillMaxWidth().clickable(enabled = !busy) { if (panel == "actions") close() else panel = "actions" }.padding(top = 24.dp, bottom = 13.dp), style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = androidx.compose.ui.text.style.TextAlign.Center)
                }
            }
        }
        if (creating) CreationSheet({ close() }, panelModifier = Modifier.imePadding()) {
            Column(Modifier.fillMaxWidth().heightIn(max = 550.dp).verticalScroll(rememberScrollState())) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("让想法开始发生", Modifier.weight(1f), style = XType.Title, fontWeight = FontWeight.SemiBold)
                    IconTap(XIcons.Close, "关闭创作", { close() }, size = 18.dp)
                }
                Text("告诉小伴你想做什么，一起把它变成作品。", Modifier.padding(top = 8.dp, bottom = 16.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                CreationChips(CreationTypes.drop(1), createKind) { createKind = it }
                Box(Modifier.padding(vertical = 14.dp).fillMaxWidth().clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.background).padding(14.dp)) {
                    if (idea.isEmpty()) Text(if (createKind == "游戏") "例如：做一个小猫跳跃收集星星的游戏" else "描述你想要的${createKind}，小伴会在电脑上制作", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    BasicTextField(idea, { idea = it.take(2000) }, Modifier.fillMaxWidth().heightIn(min = 110.dp).semantics { contentDescription = "创作想法" }, textStyle = XType.Body.copy(color = MaterialTheme.colorScheme.onSurface), cursorBrush = Brush.verticalGradient(listOf(MaterialTheme.colorScheme.primary, MaterialTheme.colorScheme.primary)))
                }
                val hints = if (createKind == "游戏") listOf("星星跳跃游戏", "像素小冒险", "治愈系消除") else DesktopIdeas.filter { it.kind == createKind }.map { it.description }
                Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(7.dp)) {
                    hints.forEach { hint -> Text(hint.take(14), Modifier.clip(RoundedCornerShape(7.dp)).background(MaterialTheme.colorScheme.surfaceVariant).clickable { idea = hint }.padding(9.dp), style = XType.Caption) }
                }
                Text(if (createKind == "游戏") "在手机上制作，进入对话后先确认方案。" else "选择已连接电脑后进入对话，作品保存在电脑上。", Modifier.padding(vertical = 16.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                AppActionButton(if (createKind == "游戏") "和小伴一起创作" else "选择电脑继续创作", { startCreation() }, Modifier.fillMaxWidth(), primary = true, enabled = idea.isNotBlank())
            }
        }
        remixSource?.let { source ->
            AppRemixSheet(env, chat, source.name, source.description, { remixSource = null }, open)
        }
        remotePrompt?.let { prompt ->
            CreationSheet({ close() }) {
                Text("选择电脑", style = XType.Title, fontWeight = FontWeight.SemiBold)
                Text("将想法带入电脑对话，由你确认发送。", Modifier.padding(vertical = 12.dp), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Column(Modifier.heightIn(max = 300.dp).verticalScroll(rememberScrollState())) {
                    remote.peers.forEach { peer ->
                        CreationAction(XIcons.Monitor, peer.name, if (peer.online) "在线" else "离线", enabled = peer.online && !remote.sending) {
                            env.remoteChat.selectPeer(peer.id)
                            env.remoteChat.newSession()
                            close()
                            open(Route.RemoteChat(peer.id, prompt))
                        }
                    }
                }
                if (remote.peers.isEmpty()) Text("还没有连接的电脑，请在电脑上登录同一小伴账号并开启手机遥控。", style = XType.Secondary, color = MaterialTheme.colorScheme.onSurfaceVariant)
                if (remote.sending) Text("电脑正在执行任务，请结束当前任务后再创建。", color = MaterialTheme.colorScheme.onSurfaceVariant, style = XType.Caption)
                Spacer(Modifier.height(12.dp))
                GhostPillButton("管理我的电脑", { close(); open(Route.Devices) }, Modifier.fillMaxWidth())
            }
        }
        Toast(message) { message = null }
    }
}

@Composable
private fun CreationSection(title: String, trailing: String, action: (() -> Unit)? = null) {
    Row(Modifier.fillMaxWidth().padding(top = 4.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(title, Modifier.weight(1f), style = XType.Body, fontWeight = FontWeight.SemiBold)
        Text(trailing, Modifier.then(if (action != null) Modifier.clickable { action() }.padding(6.dp) else Modifier), style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun CreationChips(values: List<String>, value: String, select: (String) -> Unit) {
    Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        values.forEach { item ->
            val active = value == item
            Text(item, Modifier.clip(CircleShape).heightIn(min = 34.dp).background(if (active) LocalExtraColors.current.brandSoft else LocalExtraColors.current.fill).semantics { selected = active }.clickable(role = Role.Button) { select(item) }.padding(horizontal = 9.dp, vertical = 5.dp), style = XType.Caption, color = if (active) LocalExtraColors.current.brandInk else MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

/** Creation sheets share the active mobile palette and standard bottom-sheet behavior. */
@OptIn(ExperimentalComposeUiApi::class)
@Composable
private fun CreationSheet(onDismiss: () -> Unit, panelModifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    BottomSheet(onDismiss, panelModifier, cornerRadius = 16.dp) {
        Box(Modifier.align(Alignment.CenterHorizontally).size(32.dp, 4.dp).clip(CircleShape).background(LocalExtraColors.current.line))
        Spacer(Modifier.height(16.dp))
        content()
    }
}

@Composable
private fun CreationActionGroup(content: @Composable ColumnScope.() -> Unit) {
    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(15.dp)).background(MaterialTheme.colorScheme.surface).padding(horizontal = 14.dp, vertical = 3.dp), content = content)
}

@Composable
private fun CreationAction(icon: ImageVector, title: String, trailing: String = "", enabled: Boolean = true, destructive: Boolean = false, divider: Boolean = false, arrow: Boolean = false, action: () -> Unit) {
    val line = MaterialTheme.colorScheme.outlineVariant
    Row(Modifier.fillMaxWidth().drawBehind { if (divider) drawLine(line, Offset.Zero, Offset(size.width, 0f), 1.dp.toPx()) }.heightIn(min = 51.dp).clickable(enabled = enabled, role = Role.Button, onClick = action).padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        XIcon(icon, size = 18.dp, tint = (if (destructive) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant).copy(alpha = if (enabled) 1f else .45f))
        Text(title, Modifier.weight(1f), style = XType.Secondary, color = (if (destructive) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface).copy(alpha = if (enabled) 1f else .45f))
        if (trailing.isNotEmpty()) Text(trailing, style = XType.Caption, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (arrow) XIcon(XIcons.ArrowRight, size = 18.dp, tint = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun CreationCover(item: CreationItem, bytes: ByteArray?, loader: ImageLoader, modifier: Modifier, cornerRadius: Int = 15) {
    var failed by remember(bytes) { mutableStateOf(false) }
    Box(modifier.clip(RoundedCornerShape(cornerRadius.dp)).background(LocalExtraColors.current.fill), contentAlignment = Alignment.Center) {
        if (bytes == null || failed) XIcon(if (item.game != null) XIcons.Gamepad else XIcons.Sparkle, size = 28.dp, tint = MaterialTheme.colorScheme.primary)
        else AsyncImage(model = bytes, imageLoader = loader, contentDescription = null, modifier = Modifier.fillMaxSize().then(if (item.kind == "伙伴") Modifier.padding(10.dp) else Modifier), contentScale = if (item.game != null || item.id == "sunflower") ContentScale.Fit else ContentScale.Crop, onError = { failed = true })
    }
}
