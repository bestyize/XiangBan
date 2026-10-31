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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
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
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.BuiltinMobileGames
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.MobileGame
import xyz.xiangban.xiaoban.common.community.MobileGameHistory

private data class LibraryGame(val game: MobileGame, val builtin: Boolean)

@Composable
fun GamesScreen(env: CommunityEnv, chat: CloudChatController, open: (Route) -> Unit, onOverlayChanged: (Boolean) -> Unit = {}) {
    val supported = env.platform?.gamesSupported == true
    var official by remember { mutableStateOf(emptyList<MobileGame>()) }
    var games by remember { mutableStateOf(emptyList<MobileGame>()) }
    var errors by remember { mutableStateOf(emptyList<String>()) }
    var loading by remember { mutableStateOf(true) }
    var message by remember { mutableStateOf<String?>(null) }
    var reload by remember { mutableStateOf(0) }
    var mine by rememberSaveable { mutableStateOf(!supported) }
    var genre by rememberSaveable { mutableStateOf("全部") }
    var detail by remember { mutableStateOf<LibraryGame?>(null) }
    var creating by remember { mutableStateOf(false) }
    var idea by rememberSaveable { mutableStateOf("") }
    var exporting by remember { mutableStateOf(false) }
    LaunchedEffect(detail != null, creating) { onOverlayChanged(detail != null || creating) }
    DisposableEffect(Unit) { onDispose { onOverlayChanged(false) } }
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    val history by (env.platform?.recentGames ?: remember { MutableStateFlow(emptyList<String>()) }).collectAsState()
    val context = LocalPlatformContext.current
    // Local bytes only: no network fetcher and no file URL is exposed to the games.
    val imageLoader = remember(context) { ImageLoader.Builder(context).components { add(SvgDecoder.Factory()) }.build() }
    DisposableEffect(imageLoader) { onDispose { imageLoader.shutdown() } }
    val covers = remember { mutableStateMapOf<String, ByteArray>() }
    fun create(prompt: String) {
        if (!session.loggedIn) { open(Route.Login); return }
        if (env.agent == null) { message = "当前安装包不支持本地游戏创作"; return }
        if (cloud.sending) { message = "先结束当前任务，再开始游戏创作"; return }
        creating = false
        detail = null
        chat.clearConversation()
        open(Route.AiChat)
        chat.send(prompt)
    }
    fun play(game: MobileGame) {
        if (env.platform?.openGame(game.id) == true) detail = null
        else message = "当前平台无法打开游戏"
    }
    LaunchedEffect(env.games, reload) {
        loading = true
        val failures = mutableListOf<String>()
        try {
            try { official = env.platform?.builtinGames().orEmpty() }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { failures += "官方游戏读取失败，请重试" }
            try { val scan = env.games?.scan(); games = scan?.games.orEmpty(); failures += scan?.errors.orEmpty() }
            catch (e: CancellationException) { throw e }
            catch (e: Exception) { failures += "我的游戏读取失败，请重试" }
            covers.clear()
            for (game in official) {
                try { env.platform?.builtinGameCover(game.id)?.let { covers[game.id] = it } }
                catch (e: CancellationException) { throw e }
                catch (_: Exception) { /* The title artwork remains usable when a cover is absent. */ }
            }
            for (game in games) {
                try { env.games?.readFile(game.id, "cover.svg")?.encodeToByteArray()?.takeIf { it.size <= 1024 * 1024 }?.let { covers[game.id] = it } }
                catch (e: CancellationException) { throw e }
                catch (_: Exception) { }
            }
        } finally { errors = failures; loading = false }
    }
    val all = official.map { LibraryGame(it, true) } + games.map { LibraryGame(it, false) }
    val visible = if (mine) all.filter { !it.builtin } else all.filter { it.builtin && (genre == "全部" || BuiltinMobileGames.genre(it.game.id) == genre) }
    val recent = MobileGameHistory.available(history, all.map { it.game }).firstOrNull()
    Box(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        LazyColumn(Modifier.fillMaxSize().statusBarsPadding(), contentPadding = PaddingValues(start = 22.dp, end = 22.dp, top = 8.dp, bottom = NavInset), verticalArrangement = Arrangement.spacedBy(20.dp)) {
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Row(Modifier.weight(1f), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(20.dp)) {
                        listOf(false to "发现游戏", true to "我的游戏").forEach { (value, label) ->
                            Column(Modifier.semantics { selected = mine == value }.clickable(role = Role.Tab) { mine = value }.padding(top = 4.dp, bottom = 4.dp), horizontalAlignment = Alignment.CenterHorizontally) {
                                Text(label, fontSize = 15.sp, fontWeight = if (mine == value) FontWeight.SemiBold else FontWeight.Normal, color = if (mine == value) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.onSurfaceVariant)
                                Box(Modifier.padding(top = 10.dp).width(18.dp).height(3.dp).background(if (mine == value) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.background, CircleShape))
                            }
                        }
                    }
                    IconTap(XIcons.Refresh, "刷新游戏", { if (!loading) reload++ }, size = 19.dp)
                    IconTap(XIcons.Sparkle, "让小伴创作游戏", { creating = true }, Modifier.background(MaterialTheme.colorScheme.primary.copy(alpha = .09f), CircleShape), tint = MaterialTheme.colorScheme.primary)
                }
            }
            if (!mine && recent != null) item {
                Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(18.dp)).background(MaterialTheme.colorScheme.surface).clickable { play(recent) }.padding(12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    GameCover(recent, covers[recent.id], imageLoader, Modifier.size(62.dp), false)
                    Column(Modifier.weight(1f)) {
                        Text("再来一局 ${recent.name}", fontSize = 14.sp, fontWeight = FontWeight.SemiBold, maxLines = 1, overflow = TextOverflow.Ellipsis)
                        Text("最近玩过", Modifier.padding(top = 6.dp), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    XIcon(XIcons.Play, Modifier.background(MaterialTheme.colorScheme.primary.copy(alpha = .09f), CircleShape).padding(11.dp), tint = MaterialTheme.colorScheme.primary, size = 16.dp, description = "继续游玩")
                }
            }
            item {
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text(if (mine) "我的小宇宙" else "挑一款，放松一下", Modifier.weight(1f), fontSize = 18.sp, fontWeight = FontWeight.SemiBold)
                    Text("${visible.size} 款游戏", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
                if (!mine) Row(Modifier.padding(top = 14.dp).horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    listOf("全部", "益智", "解压", "冒险").forEach { value ->
                        val active = genre == value
                        Text(value, Modifier.clip(RoundedCornerShape(6.dp)).background(if (active) MaterialTheme.colorScheme.onSurface else MaterialTheme.colorScheme.surfaceVariant).semantics { selected = active }.clickable(role = Role.Button) { genre = value }.padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 12.sp, color = if (active) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
            if (loading && visible.isEmpty()) item { Box(Modifier.fillMaxWidth().height(160.dp)) { Spinner() } }
            if (errors.isNotEmpty()) item {
                Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text(errors.joinToString("\n"), fontSize = 12.sp, color = MaterialTheme.colorScheme.error)
                    LinkText("重新读取", { reload++ })
                }
            }
            if (!loading && visible.isEmpty()) item {
                EmptyState(XIcons.Gamepad, if (mine) "想玩的，自己创造" else "暂时没有可用游戏", if (!supported) "当前平台暂不支持本地游戏，请使用 Android 版。" else if (mine) "告诉小伴一个点子，一起做成小游戏。" else "试试其他分类，或刷新游戏列表。")
            }
            items(visible.chunked(2), key = { row -> row.joinToString { it.game.id } }) { row ->
                Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                    row.forEach { entry ->
                        Column(Modifier.weight(1f)) {
                            Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).clickable(role = Role.Button) { detail = entry }.semantics { contentDescription = "查看${entry.game.name}" }) {
                                GameCover(entry.game, covers[entry.game.id], imageLoader, Modifier.fillMaxWidth().aspectRatio(1f), true)
                                Text(entry.game.name, Modifier.padding(top = 11.dp), fontSize = 14.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                Text((if (entry.builtin) BuiltinMobileGames.genre(entry.game.id) else "我的作品") + " · " + if (entry.game.orientation == "portrait") "竖屏" else "横屏", Modifier.padding(top = 5.dp), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                            if (!entry.builtin) Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                                Text("可游玩", Modifier.weight(1f), fontSize = 10.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                IconTap(XIcons.More, "管理${entry.game.name}", { detail = entry }, size = 18.dp)
                            }
                        }
                    }
                    if (row.size == 1) Spacer(Modifier.weight(1f))
                }
            }
            item {
                Row(Modifier.fillMaxWidth().clip(RoundedCornerShape(17.dp)).background(MaterialTheme.colorScheme.primary.copy(alpha = .08f)).clickable { creating = true }.padding(18.dp), verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f)) {
                        Text("有个好玩的点子？", fontSize = 14.sp, fontWeight = FontWeight.Medium, color = MaterialTheme.colorScheme.primary)
                        Text("和小伴一起，把它做出来", Modifier.padding(top = 6.dp), fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    XIcon(XIcons.Sparkle, tint = MaterialTheme.colorScheme.primary)
                }
            }
        }
        detail?.let { entry ->
            val game = entry.game
            BottomSheet({ if (!exporting) detail = null }) {
                Column(Modifier.fillMaxWidth().heightIn(max = 560.dp).verticalScroll(rememberScrollState())) {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                        GameCover(game, covers[game.id], imageLoader, Modifier.size(72.dp), false)
                        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            Text(game.name, fontSize = 18.sp, lineHeight = 24.sp, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
                            Text((if (entry.builtin) "小伴官方" else "我的作品") + " · " + if (game.orientation == "portrait") "竖屏游玩" else "横屏游玩", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        IconTap(XIcons.Close, "关闭游戏详情", { if (!exporting) detail = null }, Modifier.align(Alignment.Top), size = 18.dp)
                    }
                    Text(game.description, Modifier.fillMaxWidth().padding(vertical = 16.dp), fontSize = 14.sp, lineHeight = 22.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    if (game.save) Text("进度保存在这台手机上", Modifier.padding(bottom = 16.dp), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    PillButton("开始游玩", { play(game) }, Modifier.fillMaxWidth(), enabled = supported && !exporting, icon = XIcons.Play)
                    if (!entry.builtin) {
                        Row(Modifier.fillMaxWidth().padding(top = 10.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                            GhostPillButton("继续修改", { create("请先读取 games/${game.id} 中的游戏源码和试玩记录，问我想怎样修改《${game.name}》，然后在原游戏上迭代。") }, Modifier.weight(1f), enabled = !exporting, height = 44.dp, fontSize = 13)
                            GhostPillButton(if (exporting) "导出中…" else "导出游戏包", {
                                if (!exporting) scope.launch {
                                    exporting = true
                                    try { if (env.platform?.exportGame(game.id) != true) message = "当前平台无法导出游戏" }
                                    catch (e: CancellationException) { throw e }
                                    catch (e: Exception) { message = e.message ?: "导出失败" }
                                    finally { exporting = false }
                                }
                            }, Modifier.weight(1f), enabled = !exporting, height = 44.dp, fontSize = 13)
                        }
                    }
                }
            }
        }
        if (creating) BottomSheet({ creating = false }, panelModifier = Modifier.imePadding()) {
            Column(Modifier.fillMaxWidth().heightIn(max = 500.dp).verticalScroll(rememberScrollState())) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("想玩什么，你说了算", Modifier.weight(1f), fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
                    IconTap(XIcons.Close, "关闭游戏创作", { creating = false })
                }
                Box(Modifier.padding(top = 18.dp, bottom = 12.dp).fillMaxWidth().clip(RoundedCornerShape(14.dp)).background(MaterialTheme.colorScheme.background).padding(16.dp)) {
                    if (idea.isEmpty()) Text("比如：一只小猫在云朵间跳跃，收集星星…", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    BasicTextField(idea, { idea = it.take(2000) }, Modifier.fillMaxWidth().heightIn(min = 100.dp).semantics { contentDescription = "游戏想法" }, textStyle = TextStyle(color = MaterialTheme.colorScheme.onSurface, fontSize = 14.sp, lineHeight = 23.sp), cursorBrush = Brush.verticalGradient(listOf(MaterialTheme.colorScheme.primary, MaterialTheme.colorScheme.primary)))
                }
                Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    listOf("治愈系消除", "像素小冒险", "我的专属2048").forEach { prompt -> Text(prompt, Modifier.clip(RoundedCornerShape(9.dp)).background(MaterialTheme.colorScheme.surfaceVariant).clickable { idea = prompt }.padding(11.dp), fontSize = 12.sp) }
                }
                Spacer(Modifier.height(22.dp))
                PillButton("和小伴一起创作", { create("帮我设计并制作一款支持手机触控和电脑键鼠的离线小游戏。我的想法：${idea.trim()}。先给出简短玩法方案让我确认，再按小伴游戏格式制作。") }, Modifier.fillMaxWidth(), enabled = idea.isNotBlank())
            }
        }
        Toast(message) { message = null }
    }
}

@Composable
private fun GameCover(game: MobileGame, bytes: ByteArray?, loader: ImageLoader, modifier: Modifier, play: Boolean) {
    var failed by remember(bytes) { mutableStateOf(false) }
    Box(modifier.clip(RoundedCornerShape(17.dp)).background(MaterialTheme.colorScheme.surfaceVariant), contentAlignment = Alignment.Center) {
        if (bytes == null || failed) Column(Modifier.padding(14.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(12.dp)) {
            XIcon(XIcons.Gamepad, size = 32.dp, tint = MaterialTheme.colorScheme.primary)
            Text(game.name, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, maxLines = 2, overflow = TextOverflow.Ellipsis)
        } else AsyncImage(model = bytes, imageLoader = loader, contentDescription = null, modifier = Modifier.fillMaxSize(), contentScale = if (game.id == "jade-gomoku") ContentScale.Fit else ContentScale.Crop, onError = { failed = true })
        if (play) Box(Modifier.align(Alignment.BottomEnd).padding(10.dp).size(28.dp).background(MaterialTheme.colorScheme.surface.copy(alpha = .92f), CircleShape), contentAlignment = Alignment.Center) { XIcon(XIcons.Play, size = 13.dp) }
    }
}
