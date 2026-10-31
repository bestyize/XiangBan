package xyz.xiangban.xiaoban.business.communitynative

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import xyz.xiangban.xiaoban.common.community.CloudChatController
import xyz.xiangban.xiaoban.common.community.MobileGame

@Composable
fun GamesScreen(env: CommunityEnv, chat: CloudChatController, open: (Route) -> Unit) {
    var games by remember { mutableStateOf(emptyList<MobileGame>()) }
    var errors by remember { mutableStateOf(emptyList<String>()) }
    var loading by remember { mutableStateOf(false) }
    var message by remember { mutableStateOf<String?>(null) }
    var reload by remember { mutableStateOf(0) }
    val scope = rememberCoroutineScope()
    val session by env.session.state.collectAsState()
    val cloud by chat.state.collectAsState()
    fun create(prompt: String) {
        if (!session.loggedIn) { open(Route.Login); return }
        if (env.agent == null) { message = "当前安装包未包含手机 Agent 内核，请安装包含内核的 Android 版本"; return }
        if (cloud.sending) { message = "先结束当前任务，再开始游戏创作"; return }
        chat.clearConversation()
        open(Route.AiChat)
        chat.send(prompt)
    }
    LaunchedEffect(env.games, reload) {
        loading = true
        try {
            val scan = env.games?.scan()
            games = scan?.games.orEmpty()
            errors = scan?.errors.orEmpty()
        }
        catch (e: CancellationException) { throw e }
        catch (e: Exception) { message = e.message ?: "读取游戏失败" }
        finally { loading = false }
    }
    Box(Modifier.fillMaxSize()) {
        Column(Modifier.fillMaxSize()) {
            HomeBar("我的游戏")
            Hairline()
            LazyColumn(Modifier.weight(1f).fillMaxWidth(), contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 16.dp, bottom = NavInset), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                item {
                    Text("在手机上创作，导出后也能在小伴桌面游玩。", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Row(Modifier.padding(top = 12.dp), verticalAlignment = Alignment.CenterVertically) {
                        PillButton("用 AI 创作游戏", { create("帮我设计并制作一款支持手机触控和电脑键鼠的离线小游戏。先给出简短玩法方案让我确认，再按小伴游戏格式制作。") }, Modifier.weight(1f))
                        IconTap(XIcons.Refresh, "刷新游戏", { reload++ })
                    }
                }
                if (games.isEmpty()) item {
                    EmptyState(XIcons.Gamepad, if (loading) "正在读取游戏…" else "还没有自己的游戏", if (env.workspace == null) "当前平台尚未接入本地创作工作区。" else "告诉小伴你想玩什么，制作完成后会出现在这里。")
                }
                if (errors.isNotEmpty()) item {
                    Text("部分游戏需要修复：\n" + errors.joinToString("\n"), fontSize = 13.sp, color = MaterialTheme.colorScheme.error)
                }
                items(games, key = { it.id }) { game ->
                    Column(Modifier.fillMaxWidth().clip(RoundedCornerShape(16.dp)).background(MaterialTheme.colorScheme.surface).padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(game.name, fontSize = 17.sp, fontWeight = FontWeight.SemiBold)
                        Text(game.description, fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            PillButton("试玩", { if (env.platform?.openGame(game.id) != true) message = "当前平台无法打开游戏" }, Modifier.weight(1f))
                            GhostPillButton("导出 ZIP", {
                                scope.launch {
                                    try { if (env.platform?.exportGame(game.id) != true) message = "当前平台无法导出游戏" }
                                    catch (e: CancellationException) { throw e }
                                    catch (e: Exception) { message = e.message ?: "导出失败" }
                                }
                            }, Modifier.weight(1f))
                        }
                        LinkText("让小伴继续修改 ›", { create("请先读取 games/${game.id} 中的游戏源码和试玩记录，问我想怎样修改《${game.name}》，然后在原游戏上迭代。") })
                    }
                }
            }
        }
        Toast(message) { message = null }
    }
}
